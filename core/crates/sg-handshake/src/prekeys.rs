//! Signed prekeys, post-quantum prekeys and one-time prekeys.
//!
//! * `SignedPreKey` (X25519) is signed over `"Sargoshi/SPK/v1" || public`.
//! * `PqSignedPreKey` (ML-KEM-768 encapsulation key) is signed over
//!   `"Sargoshi/PQSPK/v1" || encapsulation_key`.
//! * `OneTimePreKey` (X25519) is unsigned; a device keeps up to 100 and replenishes below 25.
//!
//! Signatures are deterministic (Ed25519), so they are never stored: they are recomputed from
//! the identity whenever a bundle is built.

use std::collections::VecDeque;

use sg_common::constants::{
    LABEL_PQSPK_SIG, LABEL_SPK_SIG, MAX_ONE_TIME_PREKEYS_PER_UPLOAD,
    MAX_REMEMBERED_INITIAL_MESSAGES, MAX_STORED_ONE_TIME_PREKEYS, MAX_STORED_SIGNED_PREKEYS,
    ONE_TIME_PREKEY_REPLENISH_BELOW,
};
use sg_common::{PROTOCOL_VERSION, check_version};
use sg_crypto::ed25519::{Signature, VerifyingKey};
use sg_crypto::kem::{MLKEM768_SEED_LEN, MlKem768PublicKey, MlKem768SecretKey};
use sg_crypto::x25519::{X25519PublicKey, X25519SecretKey};
use zeroize::Zeroizing;

use crate::codec::{Reader, Writer};
use crate::error::HandshakeError;
use crate::identity::{IdentityKeyPair, IdentityPublic};

/// A signed prekey's public half.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedPreKeyPublic {
    /// Identifier chosen by the device.
    pub id: u32,
    /// Creation time in seconds, supplied by the caller (the core has no clock).
    pub created_at: u64,
    /// The X25519 public key.
    pub public: X25519PublicKey,
    /// Signature over `"Sargoshi/SPK/v1" || public`.
    pub signature: Signature,
}

impl SignedPreKeyPublic {
    /// Verify the signature under an identity's signing key.
    ///
    /// # Errors
    /// [`HandshakeError::InvalidSignedPreKeySignature`] if it does not verify.
    pub fn verify(&self, signing: &VerifyingKey) -> Result<(), HandshakeError> {
        signing
            .verify_labeled(LABEL_SPK_SIG, self.public.as_bytes(), &self.signature)
            .map_err(|_| HandshakeError::InvalidSignedPreKeySignature)
    }
}

/// A post-quantum signed prekey's public half.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PqPreKeyPublic {
    /// Identifier chosen by the device.
    pub id: u32,
    /// The ML-KEM-768 encapsulation key.
    pub encapsulation_key: MlKem768PublicKey,
    /// Signature over `"Sargoshi/PQSPK/v1" || encapsulation_key`.
    pub signature: Signature,
}

impl PqPreKeyPublic {
    /// Verify the signature under an identity's signing key.
    ///
    /// # Errors
    /// [`HandshakeError::InvalidPqPreKeySignature`] if it does not verify.
    pub fn verify(&self, signing: &VerifyingKey) -> Result<(), HandshakeError> {
        signing
            .verify_labeled(
                LABEL_PQSPK_SIG,
                self.encapsulation_key.as_bytes(),
                &self.signature,
            )
            .map_err(|_| HandshakeError::InvalidPqPreKeySignature)
    }
}

/// A one-time prekey's public half. These are not signed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneTimePreKeyPublic {
    /// Identifier chosen by the device.
    pub id: u32,
    /// The X25519 public key.
    pub public: X25519PublicKey,
}

/// What a device uploads to the server: its current signed prekeys and a batch of one-time keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadBundle {
    /// The newest signed prekey.
    pub signed_prekey: SignedPreKeyPublic,
    /// The newest post-quantum prekey.
    pub pq_prekey: PqPreKeyPublic,
    /// One-time prekeys.
    pub one_time_prekeys: Vec<OneTimePreKeyPublic>,
}

/// What an initiator downloads for one device of the person it wants to talk to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreKeyBundle {
    /// The responder's identity (already verified).
    pub identity: IdentityPublic,
    /// The responder's signed prekey (signature already verified).
    pub signed_prekey: SignedPreKeyPublic,
    /// The responder's post-quantum prekey (signature already verified).
    pub pq_prekey: PqPreKeyPublic,
    /// One one-time prekey, if the server still had one.
    pub one_time_prekey: Option<OneTimePreKeyPublic>,
}

pub(crate) struct SignedPreKeySecret {
    pub(crate) id: u32,
    pub(crate) created_at: u64,
    pub(crate) secret: X25519SecretKey,
}

pub(crate) struct PqPreKeySecret {
    pub(crate) id: u32,
    pub(crate) created_at: u64,
    pub(crate) secret: MlKem768SecretKey,
}

pub(crate) struct OneTimePreKeySecret {
    pub(crate) id: u32,
    pub(crate) secret: X25519SecretKey,
}

/// All prekey secrets of one device, plus the bounded memory of accepted initial messages that
/// rejects replays. Methods that change state take `&mut self`; the FFI layer wraps them as
/// "blob in, blob out" so the core itself never stores anything.
pub struct PrekeySecrets {
    signed: Vec<SignedPreKeySecret>,
    pq: Vec<PqPreKeySecret>,
    one_time: Vec<OneTimePreKeySecret>,
    seen_ephemerals: VecDeque<[u8; 32]>,
}

impl PrekeySecrets {
    fn empty() -> Self {
        Self {
            signed: Vec::new(),
            pq: Vec::new(),
            one_time: Vec::new(),
            seen_ephemerals: VecDeque::new(),
        }
    }

    /// Create the first prekeys of a device and the bundle to upload.
    ///
    /// # Errors
    /// Fails if `opk_count` exceeds the per-upload limit, an identifier range overflows, or
    /// randomness is unavailable.
    pub fn generate(
        identity: &IdentityKeyPair,
        spk_id: u32,
        pq_id: u32,
        created_at: u64,
        opk_start_id: u32,
        opk_count: usize,
    ) -> Result<(Self, UploadBundle), HandshakeError> {
        let mut secrets = Self::empty();
        secrets.add_signed_pair(spk_id, pq_id, created_at)?;
        secrets.add_one_time(opk_start_id, opk_count)?;
        let bundle = secrets.upload_bundle(identity)?;
        Ok((secrets, bundle))
    }

    /// Build the upload bundle from the newest signed prekeys and every stored one-time prekey.
    ///
    /// # Errors
    /// [`HandshakeError::UnknownSignedPreKey`] or [`HandshakeError::UnknownPqPreKey`] if the
    /// store holds none.
    pub fn upload_bundle(
        &self,
        identity: &IdentityKeyPair,
    ) -> Result<UploadBundle, HandshakeError> {
        let spk = self
            .signed
            .iter()
            .max_by_key(|k| (k.created_at, k.id))
            .ok_or(HandshakeError::UnknownSignedPreKey)?;
        let pq = self
            .pq
            .iter()
            .max_by_key(|k| (k.created_at, k.id))
            .ok_or(HandshakeError::UnknownPqPreKey)?;
        let one_time_prekeys = self
            .one_time
            .iter()
            .map(|k| OneTimePreKeyPublic {
                id: k.id,
                public: k.secret.public_key(),
            })
            .collect();
        Ok(UploadBundle {
            signed_prekey: sign_spk(identity, spk.id, spk.created_at, spk.secret.public_key()),
            pq_prekey: sign_pq(identity, pq.id, pq.secret.public_key()?),
            one_time_prekeys,
        })
    }

    /// Add a new signed prekey and post-quantum prekey; older ones stay so that initial messages
    /// already in flight can still be accepted until they are pruned.
    ///
    /// # Errors
    /// Fails on a duplicate identifier, when a collection is full, or if randomness fails.
    pub fn rotate_signed_prekeys(
        &mut self,
        identity: &IdentityKeyPair,
        spk_id: u32,
        pq_id: u32,
        created_at: u64,
    ) -> Result<(SignedPreKeyPublic, PqPreKeyPublic), HandshakeError> {
        self.add_signed_pair(spk_id, pq_id, created_at)?;
        let spk = self
            .find_signed(spk_id)
            .ok_or(HandshakeError::UnknownSignedPreKey)?;
        let pq = self.find_pq(pq_id).ok_or(HandshakeError::UnknownPqPreKey)?;
        Ok((
            sign_spk(identity, spk.id, spk.created_at, spk.secret.public_key()),
            sign_pq(identity, pq.id, pq.secret.public_key()?),
        ))
    }

    /// Remove signed and post-quantum prekeys created before `cutoff`, always keeping the newest
    /// of each kind. Returns how many keys were removed.
    pub fn prune_before(&mut self, cutoff: u64) -> usize {
        let newest_signed = self.signed.iter().map(|k| (k.created_at, k.id)).max();
        let newest_pq = self.pq.iter().map(|k| (k.created_at, k.id)).max();
        let before = self.signed.len() + self.pq.len();
        self.signed
            .retain(|k| k.created_at >= cutoff || Some((k.created_at, k.id)) == newest_signed);
        self.pq
            .retain(|k| k.created_at >= cutoff || Some((k.created_at, k.id)) == newest_pq);
        before - (self.signed.len() + self.pq.len())
    }

    /// Generate more one-time prekeys and return their public halves for upload.
    ///
    /// # Errors
    /// Fails on a duplicate identifier, an overflowing range, when the store is full, or if
    /// randomness fails.
    pub fn replenish_one_time_prekeys(
        &mut self,
        start_id: u32,
        count: usize,
    ) -> Result<Vec<OneTimePreKeyPublic>, HandshakeError> {
        self.add_one_time(start_id, count)
    }

    /// How many one-time prekeys remain.
    #[must_use]
    pub fn one_time_remaining(&self) -> usize {
        self.one_time.len()
    }

    /// Whether fewer than 25 one-time prekeys remain.
    #[must_use]
    pub fn needs_replenishment(&self) -> bool {
        self.one_time.len() < ONE_TIME_PREKEY_REPLENISH_BELOW
    }

    fn add_signed_pair(
        &mut self,
        spk_id: u32,
        pq_id: u32,
        created_at: u64,
    ) -> Result<(), HandshakeError> {
        if self.signed.len() >= MAX_STORED_SIGNED_PREKEYS
            || self.pq.len() >= MAX_STORED_SIGNED_PREKEYS
        {
            return Err(HandshakeError::TooManyKeys);
        }
        if self.find_signed(spk_id).is_some() || self.find_pq(pq_id).is_some() {
            return Err(HandshakeError::DuplicateKeyId);
        }
        let spk = X25519SecretKey::generate()?;
        let (pq, _) = MlKem768SecretKey::generate()?;
        self.signed.push(SignedPreKeySecret {
            id: spk_id,
            created_at,
            secret: spk,
        });
        self.pq.push(PqPreKeySecret {
            id: pq_id,
            created_at,
            secret: pq,
        });
        Ok(())
    }

    fn add_one_time(
        &mut self,
        start_id: u32,
        count: usize,
    ) -> Result<Vec<OneTimePreKeyPublic>, HandshakeError> {
        if count > MAX_ONE_TIME_PREKEYS_PER_UPLOAD
            || self.one_time.len().saturating_add(count) > MAX_STORED_ONE_TIME_PREKEYS
        {
            return Err(HandshakeError::TooManyKeys);
        }
        let count_u32 = u32::try_from(count).map_err(|_| HandshakeError::TooManyKeys)?;
        if count_u32 > 0 && start_id.checked_add(count_u32 - 1).is_none() {
            return Err(HandshakeError::DuplicateKeyId);
        }
        if (0..count_u32).any(|i| self.find_one_time(start_id + i).is_some()) {
            return Err(HandshakeError::DuplicateKeyId);
        }
        let mut added = Vec::with_capacity(count);
        for i in 0..count_u32 {
            let secret = X25519SecretKey::generate()?;
            added.push(OneTimePreKeyPublic {
                id: start_id + i,
                public: secret.public_key(),
            });
            self.one_time.push(OneTimePreKeySecret {
                id: start_id + i,
                secret,
            });
        }
        Ok(added)
    }

    pub(crate) fn find_signed(&self, id: u32) -> Option<&SignedPreKeySecret> {
        self.signed.iter().find(|k| k.id == id)
    }

    pub(crate) fn find_pq(&self, id: u32) -> Option<&PqPreKeySecret> {
        self.pq.iter().find(|k| k.id == id)
    }

    pub(crate) fn find_one_time(&self, id: u32) -> Option<&OneTimePreKeySecret> {
        self.one_time.iter().find(|k| k.id == id)
    }

    pub(crate) fn has_seen(&self, ephemeral: &[u8; 32]) -> bool {
        self.seen_ephemerals.iter().any(|e| e == ephemeral)
    }

    /// Record a successfully accepted initial message: remember its ephemeral key and delete the
    /// one-time prekey it used.
    pub(crate) fn commit_accept(&mut self, ephemeral: [u8; 32], one_time_id: Option<u32>) {
        if let Some(id) = one_time_id {
            self.one_time.retain(|k| k.id != id);
        }
        if self.seen_ephemerals.len() >= MAX_REMEMBERED_INITIAL_MESSAGES {
            self.seen_ephemerals.pop_front();
        }
        self.seen_ephemerals.push_back(ephemeral);
    }

    /// Serialize all secrets. The blob is plaintext secret material; the app must store it only
    /// inside its encrypted database.
    ///
    /// Layout: `version || n_signed(u16) || {id u32, created_at u64, secret 32}* || n_pq(u16) ||
    /// {id u32, created_at u64, seed 64}* || n_one_time(u16) || {id u32, secret 32}* ||
    /// n_seen(u32) || {ephemeral 32}*`.
    ///
    /// # Errors
    /// [`HandshakeError::TooManyKeys`] if a count does not fit its field (cannot happen within
    /// the documented limits).
    pub fn to_bytes(&self) -> Result<Zeroizing<Vec<u8>>, HandshakeError> {
        let n_signed = u16::try_from(self.signed.len()).map_err(|_| HandshakeError::TooManyKeys)?;
        let n_pq = u16::try_from(self.pq.len()).map_err(|_| HandshakeError::TooManyKeys)?;
        let n_one_time =
            u16::try_from(self.one_time.len()).map_err(|_| HandshakeError::TooManyKeys)?;
        let n_seen =
            u32::try_from(self.seen_ephemerals.len()).map_err(|_| HandshakeError::TooManyKeys)?;
        let size = 1
            + 2
            + self.signed.len() * (4 + 8 + 32)
            + 2
            + self.pq.len() * (4 + 8 + MLKEM768_SEED_LEN)
            + 2
            + self.one_time.len() * (4 + 32)
            + 4
            + self.seen_ephemerals.len() * 32;
        let mut w = Writer::with_capacity(size);
        w.u8(PROTOCOL_VERSION);
        w.u16(n_signed);
        for k in &self.signed {
            w.u32(k.id);
            w.u64(k.created_at);
            w.put(k.secret.expose_bytes());
        }
        w.u16(n_pq);
        for k in &self.pq {
            w.u32(k.id);
            w.u64(k.created_at);
            w.put(k.secret.expose_seed());
        }
        w.u16(n_one_time);
        for k in &self.one_time {
            w.u32(k.id);
            w.put(k.secret.expose_bytes());
        }
        w.u32(n_seen);
        for e in &self.seen_ephemerals {
            w.put(e);
        }
        Ok(w.finish())
    }

    /// Restore secrets from [`Self::to_bytes`] output.
    ///
    /// # Errors
    /// [`HandshakeError::MalformedBlob`] for any structural problem, [`HandshakeError::TooManyKeys`]
    /// when a count exceeds its limit, [`HandshakeError::DuplicateKeyId`] for repeated identifiers,
    /// and a protocol error for an unknown version.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, HandshakeError> {
        let mut r = Reader::new(bytes);
        check_version(r.u8()?)?;
        let mut out = Self::empty();

        let n_signed = usize::from(r.u16()?);
        if n_signed > MAX_STORED_SIGNED_PREKEYS {
            return Err(HandshakeError::TooManyKeys);
        }
        for _ in 0..n_signed {
            let (id, created_at, secret) = (r.u32()?, r.u64()?, r.x25519_secret()?);
            out.signed.push(SignedPreKeySecret {
                id,
                created_at,
                secret,
            });
        }
        let n_pq = usize::from(r.u16()?);
        if n_pq > MAX_STORED_SIGNED_PREKEYS {
            return Err(HandshakeError::TooManyKeys);
        }
        for _ in 0..n_pq {
            let (id, created_at, seed) = (r.u32()?, r.u64()?, r.secret64()?);
            out.pq.push(PqPreKeySecret {
                id,
                created_at,
                secret: MlKem768SecretKey::from_seed(seed),
            });
        }
        let n_one_time = usize::from(r.u16()?);
        if n_one_time > MAX_STORED_ONE_TIME_PREKEYS {
            return Err(HandshakeError::TooManyKeys);
        }
        for _ in 0..n_one_time {
            let (id, secret) = (r.u32()?, r.x25519_secret()?);
            out.one_time.push(OneTimePreKeySecret { id, secret });
        }
        let n_seen = usize::try_from(r.u32()?).map_err(|_| HandshakeError::MalformedBlob)?;
        if n_seen > MAX_REMEMBERED_INITIAL_MESSAGES {
            return Err(HandshakeError::TooManyKeys);
        }
        for _ in 0..n_seen {
            out.seen_ephemerals.push_back(r.array32()?);
        }
        r.finish()?;
        out.check_unique_ids()?;
        Ok(out)
    }

    fn check_unique_ids(&self) -> Result<(), HandshakeError> {
        fn unique(mut ids: Vec<u32>) -> bool {
            ids.sort_unstable();
            ids.windows(2).all(|w| w.first() != w.get(1))
        }
        let ok = unique(self.signed.iter().map(|k| k.id).collect())
            && unique(self.pq.iter().map(|k| k.id).collect())
            && unique(self.one_time.iter().map(|k| k.id).collect());
        if ok {
            Ok(())
        } else {
            Err(HandshakeError::DuplicateKeyId)
        }
    }
}

impl core::fmt::Debug for PrekeySecrets {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "PrekeySecrets(redacted; {} signed, {} pq, {} one-time)",
            self.signed.len(),
            self.pq.len(),
            self.one_time.len()
        )
    }
}

fn sign_spk(
    identity: &IdentityKeyPair,
    id: u32,
    created_at: u64,
    public: X25519PublicKey,
) -> SignedPreKeyPublic {
    let signature = identity
        .signing()
        .sign_labeled(LABEL_SPK_SIG, public.as_bytes());
    SignedPreKeyPublic {
        id,
        created_at,
        public,
        signature,
    }
}

fn sign_pq(
    identity: &IdentityKeyPair,
    id: u32,
    encapsulation_key: MlKem768PublicKey,
) -> PqPreKeyPublic {
    let signature = identity
        .signing()
        .sign_labeled(LABEL_PQSPK_SIG, encapsulation_key.as_bytes());
    PqPreKeyPublic {
        id,
        encapsulation_key,
        signature,
    }
}

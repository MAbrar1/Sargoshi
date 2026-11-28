//! The Double Ratchet session (Signal's published Double Ratchet specification structure, with
//! the key derivations of Prompt B section 19.2). See docs/architecture/PROTOCOL.md section 8.
//!
//! Two properties drive the structure of this module:
//!
//! * **Transactional decryption.** `decrypt` works on copies and changes the session only after
//!   the message authenticated. A forged or corrupted message can never advance a chain, cache
//!   skipped keys or rotate a key pair.
//! * **Bounded work.** All skip limits are checked before any key is derived, so a header that
//!   claims a huge message number costs one comparison, not a long loop.

use core::fmt;

use sg_common::constants::{MAX_CACHED_SKIPPED_KEYS, MAX_SKIPPED_KEYS_PER_SESSION};
use sg_common::{Envelope, PROTOCOL_VERSION, RatchetHeader, check_version};
use sg_crypto::CryptoError;
use sg_crypto::SecretBytes;
use sg_crypto::aead::{open, seal};
use sg_crypto::codec::{Reader, Writer};
use sg_crypto::x25519::{X25519PublicKey, X25519SecretKey};
use sg_handshake::{InitialHeader, InitiatorHandshake, ResponderHandshake};
use zeroize::Zeroizing;

use crate::error::RatchetError;
use crate::hook::{NoPqRatchet, PqRatchetHook, RatchetDirection, RatchetStep};
use crate::kdf::{copy_secret, kdf_ck, kdf_rk, message_key};

/// Largest associated data a session accepts (the handshake produces 1,248 bytes).
pub const MAX_ASSOCIATED_DATA_BYTES: usize = 4_096;

struct SkippedKey {
    ratchet_public: [u8; 32],
    index: u32,
    seed: SecretBytes<32>,
}

/// One end of an encrypted conversation. Zeroized on drop; `Debug` is redacted.
pub struct Session {
    associated_data: Vec<u8>,
    root_key: SecretBytes<32>,
    dh_self: X25519SecretKey,
    dh_remote: Option<X25519PublicKey>,
    send_chain: Option<SecretBytes<32>>,
    sent: u32,
    recv_chain: Option<SecretBytes<32>>,
    received: u32,
    previous_chain_len: u32,
    skipped: Vec<SkippedKey>,
}

fn aad(associated_data: &[u8], header: &RatchetHeader) -> Vec<u8> {
    let mut out = Vec::with_capacity(associated_data.len() + sg_common::HEADER_LEN);
    out.extend_from_slice(associated_data);
    out.extend_from_slice(&header.to_bytes());
    out
}

/// Derive and cache the message keys of `chain` from `*index` up to (not including) `target`.
fn skip_to(
    chain: &mut Option<SecretBytes<32>>,
    index: &mut u32,
    target: u32,
    ratchet_public: [u8; 32],
    out: &mut Vec<SkippedKey>,
) -> Result<(), RatchetError> {
    while *index < target {
        let ck = chain.as_ref().ok_or(RatchetError::InvalidHeader)?;
        let (seed, next) = kdf_ck(ck)?;
        out.push(SkippedKey {
            ratchet_public,
            index: *index,
            seed,
        });
        *chain = Some(next);
        *index = index.checked_add(1).ok_or(RatchetError::CounterOverflow)?;
    }
    Ok(())
}

impl Session {
    /// Start a session as the initiator. Returns the session and the initial header that must
    /// reach the responder together with the first message.
    ///
    /// # Errors
    /// Fails if the associated data is too long, randomness is unavailable, or the remote
    /// ratchet key is a low-order point.
    pub fn from_initiator(
        handshake: InitiatorHandshake,
    ) -> Result<(Self, InitialHeader), RatchetError> {
        Self::from_initiator_with_hook(handshake, &NoPqRatchet)
    }

    /// [`Self::from_initiator`] with a post-quantum hook.
    ///
    /// # Errors
    /// Same as [`Self::from_initiator`].
    pub fn from_initiator_with_hook(
        handshake: InitiatorHandshake,
        hook: &dyn PqRatchetHook,
    ) -> Result<(Self, InitialHeader), RatchetError> {
        let InitiatorHandshake {
            initial_header,
            shared_key,
            associated_data,
            remote_ratchet_public,
            ..
        } = handshake;
        if associated_data.len() > MAX_ASSOCIATED_DATA_BYTES {
            return Err(CryptoError::InvalidLength.into());
        }
        let dh_self = X25519SecretKey::generate()?;
        let dh_output = dh_self.diffie_hellman(&remote_ratchet_public)?;
        let local_public = dh_self.public_key();
        let extra = hook.extra_root_secret(&RatchetStep {
            direction: RatchetDirection::Sending,
            local_ratchet_public: local_public.as_bytes(),
            remote_ratchet_public: remote_ratchet_public.as_bytes(),
        });
        let (root_key, send_chain) = kdf_rk(&shared_key, &dh_output, extra.as_ref())?;
        let session = Self {
            associated_data,
            root_key,
            dh_self,
            dh_remote: Some(remote_ratchet_public),
            send_chain: Some(send_chain),
            sent: 0,
            recv_chain: None,
            received: 0,
            previous_chain_len: 0,
            skipped: Vec::new(),
        };
        Ok((session, initial_header))
    }

    /// Start a session as the responder. The responder cannot send until it has received the
    /// initiator's first message.
    ///
    /// # Errors
    /// Fails if the associated data is too long.
    pub fn from_responder(handshake: ResponderHandshake) -> Result<Self, RatchetError> {
        let ResponderHandshake {
            shared_key,
            associated_data,
            ratchet_secret,
            ..
        } = handshake;
        if associated_data.len() > MAX_ASSOCIATED_DATA_BYTES {
            return Err(CryptoError::InvalidLength.into());
        }
        Ok(Self {
            associated_data,
            root_key: shared_key,
            dh_self: ratchet_secret,
            dh_remote: None,
            send_chain: None,
            sent: 0,
            recv_chain: None,
            received: 0,
            previous_chain_len: 0,
            skipped: Vec::new(),
        })
    }

    /// Whether this side can send (a responder can only after receiving a message).
    #[must_use]
    pub const fn can_send(&self) -> bool {
        self.send_chain.is_some()
    }

    /// Number of message keys currently cached for out-of-order delivery.
    #[must_use]
    pub fn skipped_key_count(&self) -> usize {
        self.skipped.len()
    }

    /// Drop the `count` oldest cached skipped keys (an explicit, caller-triggered eviction for a
    /// session whose cache filled with messages that will never arrive). Returns how many were
    /// removed. Messages protected by those keys can no longer be decrypted.
    pub fn discard_oldest_skipped_keys(&mut self, count: usize) -> usize {
        let n = count.min(self.skipped.len());
        self.skipped.drain(..n);
        n
    }

    /// The associated data bound into every message of this session.
    #[must_use]
    pub fn associated_data(&self) -> &[u8] {
        &self.associated_data
    }

    /// Encrypt one message and return the serialized envelope. The session is unchanged on error.
    ///
    /// # Errors
    /// [`RatchetError::NoSendChain`] before the first message was received (responder),
    /// [`RatchetError::CounterOverflow`] after 2^32 - 1 messages in one chain, and a primitive or
    /// size error (an envelope may not exceed 128 KiB).
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, RatchetError> {
        let chain = self.send_chain.as_ref().ok_or(RatchetError::NoSendChain)?;
        let next_count = self
            .sent
            .checked_add(1)
            .ok_or(RatchetError::CounterOverflow)?;
        let (seed, next_chain) = kdf_ck(chain)?;
        let header = RatchetHeader {
            ratchet_pub: *self.dh_self.public_key().as_bytes(),
            prev_chain_len: self.previous_chain_len,
            msg_num: self.sent,
        };
        let (key, nonce) = message_key(&seed)?;
        let ciphertext = seal(
            &key,
            &nonce,
            &aad(&self.associated_data, &header),
            plaintext,
        )?;
        let bytes = Envelope { header, ciphertext }.encode()?;
        self.send_chain = Some(next_chain);
        self.sent = next_count;
        Ok(bytes)
    }

    /// Decrypt one serialized envelope. The session is unchanged on any error.
    ///
    /// # Errors
    /// A protocol error for a malformed envelope, [`RatchetError::DuplicateMessage`],
    /// [`RatchetError::TooManySkippedKeys`], [`RatchetError::InvalidHeader`],
    /// [`sg_crypto::CryptoError::DecryptionFailed`] (wrapped) for anything that fails
    /// authentication, or [`RatchetError::CounterOverflow`].
    pub fn decrypt(&mut self, envelope: &[u8]) -> Result<Zeroizing<Vec<u8>>, RatchetError> {
        self.decrypt_with_hook(envelope, &NoPqRatchet)
    }

    /// [`Self::decrypt`] with a post-quantum hook.
    ///
    /// # Errors
    /// Same as [`Self::decrypt`].
    pub fn decrypt_with_hook(
        &mut self,
        envelope: &[u8],
        hook: &dyn PqRatchetHook,
    ) -> Result<Zeroizing<Vec<u8>>, RatchetError> {
        let Envelope { header, ciphertext } = Envelope::decode(envelope)?;
        let aad = aad(&self.associated_data, &header);

        // 1. A key cached earlier for an out-of-order message.
        if let Some(position) = self
            .skipped
            .iter()
            .position(|k| k.ratchet_public == header.ratchet_pub && k.index == header.msg_num)
        {
            let entry = self
                .skipped
                .get(position)
                .ok_or(RatchetError::MalformedState)?;
            let (key, nonce) = message_key(&entry.seed)?;
            let plaintext = open(&key, &nonce, &aad, &ciphertext)?;
            self.skipped.remove(position);
            return Ok(plaintext);
        }

        // 2. Everything else is computed on copies and committed only after authentication.
        let remote_public = X25519PublicKey::from_bytes(header.ratchet_pub);
        let mut root_key = copy_secret(&self.root_key);
        let mut new_dh_self: Option<X25519SecretKey> = None;
        let mut send_chain = self.send_chain.as_ref().map(copy_secret);
        let mut recv_chain = self.recv_chain.as_ref().map(copy_secret);
        let (mut sent, mut received, mut previous_chain_len) =
            (self.sent, self.received, self.previous_chain_len);
        let mut new_skipped: Vec<SkippedKey> = Vec::new();

        if self.dh_remote == Some(remote_public) {
            if recv_chain.is_none() {
                return Err(RatchetError::InvalidHeader);
            }
            if header.msg_num < received {
                return Err(RatchetError::DuplicateMessage);
            }
            self.check_skip_budget(u64::from(header.msg_num - received))?;
            skip_to(
                &mut recv_chain,
                &mut received,
                header.msg_num,
                header.ratchet_pub,
                &mut new_skipped,
            )?;
        } else {
            // A new remote ratchet key: first account for the rest of the previous receiving chain.
            let old_remote = self.dh_remote.as_ref();
            let previous_skips = match (&recv_chain, old_remote) {
                (Some(_), Some(_)) => {
                    if header.prev_chain_len < received {
                        return Err(RatchetError::InvalidHeader);
                    }
                    header.prev_chain_len - received
                }
                _ => {
                    if header.prev_chain_len != 0 {
                        return Err(RatchetError::InvalidHeader);
                    }
                    0
                }
            };
            self.check_skip_budget(u64::from(previous_skips) + u64::from(header.msg_num))?;
            if let Some(old) = old_remote {
                skip_to(
                    &mut recv_chain,
                    &mut received,
                    header.prev_chain_len,
                    *old.as_bytes(),
                    &mut new_skipped,
                )?;
            }

            // The Diffie-Hellman ratchet step: derive the receiving chain, then a new sending chain.
            let dh_output = self.dh_self.diffie_hellman(&remote_public)?;
            let local_public = self.dh_self.public_key();
            let extra = hook.extra_root_secret(&RatchetStep {
                direction: RatchetDirection::Receiving,
                local_ratchet_public: local_public.as_bytes(),
                remote_ratchet_public: remote_public.as_bytes(),
            });
            let (next_root, new_recv_chain) = kdf_rk(&root_key, &dh_output, extra.as_ref())?;

            let fresh = X25519SecretKey::generate()?;
            let dh_output = fresh.diffie_hellman(&remote_public)?;
            let fresh_public = fresh.public_key();
            let extra = hook.extra_root_secret(&RatchetStep {
                direction: RatchetDirection::Sending,
                local_ratchet_public: fresh_public.as_bytes(),
                remote_ratchet_public: remote_public.as_bytes(),
            });
            let (next_root, new_send_chain) = kdf_rk(&next_root, &dh_output, extra.as_ref())?;

            root_key = next_root;
            previous_chain_len = sent;
            sent = 0;
            send_chain = Some(new_send_chain);
            recv_chain = Some(new_recv_chain);
            received = 0;
            new_dh_self = Some(fresh);
            skip_to(
                &mut recv_chain,
                &mut received,
                header.msg_num,
                header.ratchet_pub,
                &mut new_skipped,
            )?;
        }

        // The message key for exactly this message.
        let chain = recv_chain.as_ref().ok_or(RatchetError::InvalidHeader)?;
        let (seed, next_chain) = kdf_ck(chain)?;
        received = received
            .checked_add(1)
            .ok_or(RatchetError::CounterOverflow)?;
        let (key, nonce) = message_key(&seed)?;
        let plaintext = open(&key, &nonce, &aad, &ciphertext)?;

        // Authenticated: commit.
        self.root_key = root_key;
        if let Some(fresh) = new_dh_self {
            self.dh_self = fresh;
        }
        self.dh_remote = Some(remote_public);
        self.send_chain = send_chain;
        self.sent = sent;
        self.recv_chain = Some(next_chain);
        self.received = received;
        self.previous_chain_len = previous_chain_len;
        self.skipped.append(&mut new_skipped);
        Ok(plaintext)
    }

    fn check_skip_budget(&self, new_skips: u64) -> Result<(), RatchetError> {
        let per_message = u64::try_from(MAX_SKIPPED_KEYS_PER_SESSION)
            .map_err(|_| RatchetError::CounterOverflow)?;
        let cache =
            u64::try_from(MAX_CACHED_SKIPPED_KEYS).map_err(|_| RatchetError::CounterOverflow)?;
        let cached_now =
            u64::try_from(self.skipped.len()).map_err(|_| RatchetError::CounterOverflow)?;
        if new_skips > per_message || cached_now.saturating_add(new_skips) > cache {
            return Err(RatchetError::TooManySkippedKeys);
        }
        Ok(())
    }

    /// Serialize the whole session. The blob is plaintext secret material; the app must store it
    /// only inside its encrypted database.
    ///
    /// Layout: `version || ad_len(u16) || ad || root_key(32) || dh_self_secret(32) ||
    /// has_remote(u8) [remote_public(32)] || has_send(u8) [send_chain(32)] || sent(u32) ||
    /// has_recv(u8) [recv_chain(32)] || received(u32) || previous_chain_len(u32) ||
    /// n_skipped(u16) || {ratchet_public(32) index(u32) seed(32)}*`.
    ///
    /// # Errors
    /// [`RatchetError::MalformedState`] if a length does not fit its field (cannot happen within
    /// the documented limits).
    pub fn to_bytes(&self) -> Result<Zeroizing<Vec<u8>>, RatchetError> {
        let ad_len =
            u16::try_from(self.associated_data.len()).map_err(|_| RatchetError::MalformedState)?;
        let n_skipped =
            u16::try_from(self.skipped.len()).map_err(|_| RatchetError::MalformedState)?;
        let optional = |present: bool| if present { 33 } else { 1 };
        let size = 1
            + 2
            + self.associated_data.len()
            + 32
            + 32
            + optional(self.dh_remote.is_some())
            + optional(self.send_chain.is_some())
            + 4
            + optional(self.recv_chain.is_some())
            + 4
            + 4
            + 2
            + self.skipped.len() * (32 + 4 + 32);
        let mut w = Writer::with_capacity(size);
        w.u8(PROTOCOL_VERSION);
        w.u16(ad_len);
        w.put(&self.associated_data);
        w.put(self.root_key.expose());
        w.put(self.dh_self.expose_bytes());
        match &self.dh_remote {
            Some(remote) => {
                w.u8(1);
                w.put(remote.as_bytes());
            }
            None => w.u8(0),
        }
        match &self.send_chain {
            Some(chain) => {
                w.u8(1);
                w.put(chain.expose());
            }
            None => w.u8(0),
        }
        w.u32(self.sent);
        match &self.recv_chain {
            Some(chain) => {
                w.u8(1);
                w.put(chain.expose());
            }
            None => w.u8(0),
        }
        w.u32(self.received);
        w.u32(self.previous_chain_len);
        w.u16(n_skipped);
        for k in &self.skipped {
            w.put(&k.ratchet_public);
            w.u32(k.index);
            w.put(k.seed.expose());
        }
        Ok(w.finish())
    }

    /// Restore a session from [`Self::to_bytes`] output, rejecting every structural problem and
    /// every internally inconsistent state.
    ///
    /// # Errors
    /// [`RatchetError::MalformedState`] for truncation, trailing bytes, bad flags, oversized
    /// counts, duplicate skipped keys or an inconsistent combination of fields, and a protocol
    /// error for an unknown version.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, RatchetError> {
        let mut r = Reader::new(bytes);
        check_version(r.u8()?)?;
        let ad_len = usize::from(r.u16()?);
        if ad_len > MAX_ASSOCIATED_DATA_BYTES {
            return Err(RatchetError::MalformedState);
        }
        let associated_data = r.take(ad_len)?.to_vec();
        let root_key = r.secret32()?;
        let dh_self = r.x25519_secret()?;
        let flag = |r: &mut Reader<'_>| -> Result<bool, RatchetError> {
            match r.u8()? {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(RatchetError::MalformedState),
            }
        };
        let dh_remote = if flag(&mut r)? {
            Some(X25519PublicKey::from_bytes(r.array32()?))
        } else {
            None
        };
        let send_chain = if flag(&mut r)? {
            Some(r.secret32()?)
        } else {
            None
        };
        let sent = r.u32()?;
        let recv_chain = if flag(&mut r)? {
            Some(r.secret32()?)
        } else {
            None
        };
        let received = r.u32()?;
        let previous_chain_len = r.u32()?;
        let n_skipped = usize::from(r.u16()?);
        if n_skipped > MAX_CACHED_SKIPPED_KEYS {
            return Err(RatchetError::MalformedState);
        }
        let mut skipped = Vec::with_capacity(n_skipped);
        for _ in 0..n_skipped {
            skipped.push(SkippedKey {
                ratchet_public: r.array32()?,
                index: r.u32()?,
                seed: r.secret32()?,
            });
        }
        r.finish()?;

        // Internal consistency: states the code can never produce are rejected, not repaired.
        let consistent = (send_chain.is_some() || sent == 0)
            && (recv_chain.is_some() || received == 0)
            && (dh_remote.is_some() || recv_chain.is_none())
            && (skipped.is_empty() || dh_remote.is_some());
        let mut pairs: Vec<([u8; 32], u32)> = skipped
            .iter()
            .map(|k| (k.ratchet_public, k.index))
            .collect();
        pairs.sort_unstable();
        let unique = pairs.windows(2).all(|w| w.first() != w.get(1));
        if !consistent || !unique {
            return Err(RatchetError::MalformedState);
        }
        Ok(Self {
            associated_data,
            root_key,
            dh_self,
            dh_remote,
            send_chain,
            sent,
            recv_chain,
            received,
            previous_chain_len,
            skipped,
        })
    }
}

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Session(redacted; sent {}, received {}, {} skipped keys)",
            self.sent,
            self.received,
            self.skipped.len()
        )
    }
}

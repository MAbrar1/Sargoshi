//! The post-quantum hybrid handshake (PQXDH-style, after Signal's published PQXDH
//! specification). Deviations from that specification are listed in docs/architecture/PROTOCOL.md.
//!
//! Initiator (A) with identity `IK_A`, ephemeral `EK_A`, against responder (B) bundle
//! `IK_B, SPK_B, PQSPK_B, [OPK_B]`:
//! ```text
//! DH1 = DH(IK_A, SPK_B)   DH2 = DH(EK_A, IK_B)   DH3 = DH(EK_A, SPK_B)   DH4 = DH(EK_A, OPK_B)
//! (ct, ss) = ML-KEM-768.Encaps(PQSPK_B)
//! SK = HKDF-SHA256(salt = 32 zero bytes, ikm = 0xFF*32 || DH1 || DH2 || DH3 || [DH4] || ss,
//!                  info = "Sargoshi/PQXDH/v1", len = 32)
//! AD = IK_A_dh_pub || IK_B_dh_pub || PQSPK_B_encapsulation_key
//! ```

use sg_common::constants::{
    INFO_PQXDH, MLKEM768_ENCAPSULATION_KEY_LEN, PQXDH_PREFIX_BYTE, PQXDH_PREFIX_LEN,
    PQXDH_SALT_LEN, X25519_KEY_LEN,
};
use sg_crypto::SecretBytes;
use sg_crypto::kdf::hkdf_sha256_secret;
use sg_crypto::kem::{MlKem768Ciphertext, MlKem768PublicKey};
use sg_crypto::x25519::{X25519PublicKey, X25519SecretKey};
use zeroize::Zeroizing;

use crate::error::HandshakeError;
use crate::identity::{IdentityKeyPair, IdentityPublic};
use crate::prekeys::{PreKeyBundle, PrekeySecrets};

/// Everything the responder needs from the initiator, sent ahead of the first ratchet message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialHeader {
    /// The initiator's identity (verified when parsed).
    pub identity: IdentityPublic,
    /// The initiator's ephemeral X25519 public key `EK_A`.
    pub ephemeral: X25519PublicKey,
    /// The ML-KEM-768 ciphertext encapsulated to the responder's post-quantum prekey.
    pub kem_ciphertext: MlKem768Ciphertext,
    /// Identifier of the responder's signed prekey that was used.
    pub signed_prekey_id: u32,
    /// Identifier of the responder's post-quantum prekey that was used.
    pub pq_prekey_id: u32,
    /// Identifier of the one-time prekey that was used, if any.
    pub one_time_prekey_id: Option<u32>,
}

/// The initiator's result: the header to send, and the secrets to start the Double Ratchet.
#[derive(Debug)]
pub struct InitiatorHandshake {
    /// Send this to the responder together with the first ratchet message.
    pub initial_header: InitialHeader,
    /// The shared key `SK`; it becomes the initial root key.
    pub shared_key: SecretBytes<32>,
    /// The associated data `AD` bound into every message of the session.
    pub associated_data: Vec<u8>,
    /// The responder's signed prekey, which acts as its initial ratchet public key.
    pub remote_ratchet_public: X25519PublicKey,
    /// The responder's identity.
    pub remote_identity: IdentityPublic,
}

/// The responder's result.
#[derive(Debug)]
pub struct ResponderHandshake {
    /// The shared key `SK`; it becomes the initial root key.
    pub shared_key: SecretBytes<32>,
    /// The associated data `AD`; identical to the initiator's.
    pub associated_data: Vec<u8>,
    /// The signed prekey secret, which is the responder's initial ratchet key pair.
    pub ratchet_secret: X25519SecretKey,
    /// The initiator's identity.
    pub remote_identity: IdentityPublic,
    /// The one-time prekey that was consumed and deleted, if any.
    pub consumed_one_time_prekey_id: Option<u32>,
}

/// Start a session with the owner of `remote`.
///
/// Verifies every signature in the bundle, generates a fresh ephemeral key and ML-KEM
/// encapsulation from operating system randomness, and derives `SK` and `AD`. The caller must
/// compare `remote.identity` with the identity it already trusts for that contact (or show a
/// safety number on first contact); this function cannot know who the person is.
///
/// # Errors
/// Any signature error from the bundle, [`sg_crypto::CryptoError::WeakSharedSecret`] if the
/// bundle contains a low-order key, or a randomness failure.
pub fn initiate(
    identity: &IdentityKeyPair,
    remote: &PreKeyBundle,
) -> Result<InitiatorHandshake, HandshakeError> {
    let ephemeral = X25519SecretKey::generate()?;
    initiate_inner(identity, remote, &ephemeral, |ek| ek.encapsulate())
}

/// [`initiate`] with a caller-supplied ephemeral key and ML-KEM message, so the written
/// specification can be reproduced byte for byte. Reusing either breaks security.
///
/// # Errors
/// Same as [`initiate`].
#[cfg(feature = "deterministic-kat")]
pub fn initiate_deterministic(
    identity: &IdentityKeyPair,
    remote: &PreKeyBundle,
    ephemeral: &X25519SecretKey,
    kem_message: &[u8; 32],
) -> Result<InitiatorHandshake, HandshakeError> {
    initiate_inner(identity, remote, ephemeral, |ek| {
        ek.encapsulate_deterministic(kem_message)
    })
}

fn initiate_inner(
    identity: &IdentityKeyPair,
    remote: &PreKeyBundle,
    ephemeral: &X25519SecretKey,
    encapsulate: impl FnOnce(
        &MlKem768PublicKey,
    )
        -> Result<(MlKem768Ciphertext, SecretBytes<32>), sg_crypto::CryptoError>,
) -> Result<InitiatorHandshake, HandshakeError> {
    remote.verify()?;
    let spk = &remote.signed_prekey.public;
    let ik_b = remote.identity.dh_key();

    let dh1 = identity.dh().diffie_hellman(spk)?;
    let dh2 = ephemeral.diffie_hellman(ik_b)?;
    let dh3 = ephemeral.diffie_hellman(spk)?;
    let dh4 = remote
        .one_time_prekey
        .as_ref()
        .map(|opk| ephemeral.diffie_hellman(&opk.public))
        .transpose()?;
    let (kem_ciphertext, ss) = encapsulate(&remote.pq_prekey.encapsulation_key)?;

    let shared_key = derive_shared_key(&dh1, &dh2, &dh3, dh4.as_ref(), &ss)?;
    let associated_data = associated_data(
        identity.public().dh_key(),
        ik_b,
        &remote.pq_prekey.encapsulation_key,
    );
    Ok(InitiatorHandshake {
        initial_header: InitialHeader {
            identity: identity.public().clone(),
            ephemeral: ephemeral.public_key(),
            kem_ciphertext,
            signed_prekey_id: remote.signed_prekey.id,
            pq_prekey_id: remote.pq_prekey.id,
            one_time_prekey_id: remote.one_time_prekey.as_ref().map(|opk| opk.id),
        },
        shared_key,
        associated_data,
        remote_ratchet_public: *spk,
        remote_identity: remote.identity.clone(),
    })
}

/// Accept a session started by the owner of `initial.identity`.
///
/// On success the used one-time prekey is deleted from `prekeys` and the initiator's ephemeral
/// key is remembered so the same initial message is rejected if it is presented again. On any
/// error `prekeys` is left unchanged. The caller must persist the updated `prekeys` blob only
/// after the first ratchet message has decrypted successfully, because a corrupted initial
/// header is not detectable until then.
///
/// # Errors
/// [`HandshakeError::ReplayedInitialMessage`], the three `Unknown…PreKey` errors, or a
/// primitive error (for example a low-order ephemeral key).
pub fn accept(
    identity: &IdentityKeyPair,
    prekeys: &mut PrekeySecrets,
    initial: &InitialHeader,
) -> Result<ResponderHandshake, HandshakeError> {
    if prekeys.has_seen(initial.ephemeral.as_bytes()) {
        return Err(HandshakeError::ReplayedInitialMessage);
    }
    let spk = prekeys
        .find_signed(initial.signed_prekey_id)
        .ok_or(HandshakeError::UnknownSignedPreKey)?;
    let pq = prekeys
        .find_pq(initial.pq_prekey_id)
        .ok_or(HandshakeError::UnknownPqPreKey)?;
    let opk = initial
        .one_time_prekey_id
        .map(|id| {
            prekeys
                .find_one_time(id)
                .ok_or(HandshakeError::UnknownOneTimePreKey)
        })
        .transpose()?;

    let dh1 = spk.secret.diffie_hellman(initial.identity.dh_key())?;
    let dh2 = identity.dh().diffie_hellman(&initial.ephemeral)?;
    let dh3 = spk.secret.diffie_hellman(&initial.ephemeral)?;
    let dh4 = opk
        .map(|k| k.secret.diffie_hellman(&initial.ephemeral))
        .transpose()?;
    let ss = pq.secret.decapsulate(&initial.kem_ciphertext)?;

    let shared_key = derive_shared_key(&dh1, &dh2, &dh3, dh4.as_ref(), &ss)?;
    let associated_data = associated_data(
        initial.identity.dh_key(),
        identity.public().dh_key(),
        &pq.secret.public_key()?,
    );
    let ratchet_secret = X25519SecretKey::from_bytes(*spk.secret.expose_bytes());

    prekeys.commit_accept(*initial.ephemeral.as_bytes(), initial.one_time_prekey_id);
    Ok(ResponderHandshake {
        shared_key,
        associated_data,
        ratchet_secret,
        remote_identity: initial.identity.clone(),
        consumed_one_time_prekey_id: initial.one_time_prekey_id,
    })
}

fn derive_shared_key(
    dh1: &SecretBytes<32>,
    dh2: &SecretBytes<32>,
    dh3: &SecretBytes<32>,
    dh4: Option<&SecretBytes<32>>,
    ss: &SecretBytes<32>,
) -> Result<SecretBytes<32>, HandshakeError> {
    let size = PQXDH_PREFIX_LEN + 32 * 3 + dh4.map_or(0, |_| 32) + 32;
    let mut ikm = Zeroizing::new(Vec::with_capacity(size));
    ikm.extend_from_slice(&[PQXDH_PREFIX_BYTE; PQXDH_PREFIX_LEN]);
    ikm.extend_from_slice(dh1.expose());
    ikm.extend_from_slice(dh2.expose());
    ikm.extend_from_slice(dh3.expose());
    if let Some(dh4) = dh4 {
        ikm.extend_from_slice(dh4.expose());
    }
    ikm.extend_from_slice(ss.expose());
    Ok(hkdf_sha256_secret(
        &[0u8; PQXDH_SALT_LEN],
        &ikm,
        INFO_PQXDH,
    )?)
}

fn associated_data(
    ik_a_dh: &X25519PublicKey,
    ik_b_dh: &X25519PublicKey,
    pq_encapsulation_key: &MlKem768PublicKey,
) -> Vec<u8> {
    let mut ad = Vec::with_capacity(2 * X25519_KEY_LEN + MLKEM768_ENCAPSULATION_KEY_LEN);
    ad.extend_from_slice(ik_a_dh.as_bytes());
    ad.extend_from_slice(ik_b_dh.as_bytes());
    ad.extend_from_slice(pq_encapsulation_key.as_bytes());
    ad
}

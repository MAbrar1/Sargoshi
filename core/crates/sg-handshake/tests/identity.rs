//! Identity key tests.
#![allow(
    clippy::expect_used,
    reason = "test helpers: a failed expectation is the intended test failure"
)]

use sg_common::constants::LABEL_IK_DH_SIG;
use sg_crypto::ed25519::SigningKey;
use sg_crypto::x25519::X25519SecretKey;
use sg_handshake::{HandshakeError, IDENTITY_SECRET_BLOB_LEN, IdentityKeyPair, IdentityPublic};

#[test]
fn secret_blob_round_trips_to_the_same_public_identity() {
    let id = IdentityKeyPair::generate().expect("identity");
    let blob = id.to_secret_bytes();
    assert_eq!(blob.len(), IDENTITY_SECRET_BLOB_LEN);
    let restored = IdentityKeyPair::from_secret_bytes(&blob).expect("restore");
    assert_eq!(restored.public(), id.public());
    assert_eq!(restored.to_secret_bytes().as_slice(), blob.as_slice());
}

#[test]
fn secret_blob_parser_is_strict() {
    let blob = IdentityKeyPair::generate()
        .expect("identity")
        .to_secret_bytes();
    for cut in 0..blob.len() {
        assert_eq!(
            IdentityKeyPair::from_secret_bytes(&blob[..cut]).err(),
            Some(HandshakeError::MalformedBlob),
            "cut {cut}"
        );
    }
    let mut longer = blob.to_vec();
    longer.push(0);
    assert_eq!(
        IdentityKeyPair::from_secret_bytes(&longer).err(),
        Some(HandshakeError::MalformedBlob)
    );
    let mut wrong_version = blob.to_vec();
    wrong_version[0] = 9;
    assert!(matches!(
        IdentityKeyPair::from_secret_bytes(&wrong_version),
        Err(HandshakeError::Proto(_))
    ));
}

#[test]
fn identity_never_prints_secrets() {
    let id = IdentityKeyPair::generate().expect("identity");
    assert_eq!(format!("{id:?}"), "IdentityKeyPair(redacted)");
}

#[test]
fn public_identity_requires_a_valid_dh_signature() {
    let id = IdentityKeyPair::generate().expect("identity");
    let public = id.public();
    let signing = public.signing_key().to_bytes();
    let dh = *public.dh_key().as_bytes();
    let sig = *public.dh_signature().as_bytes();
    assert_eq!(
        IdentityPublic::new(&signing, &dh, &sig).as_ref(),
        Ok(public)
    );

    let mut bad_sig = sig;
    bad_sig.iter_mut().take(1).for_each(|b| *b ^= 1);
    assert_eq!(
        IdentityPublic::new(&signing, &dh, &bad_sig),
        Err(HandshakeError::InvalidIdentitySignature)
    );

    // A DH key that the signing key did not certify is refused.
    let other_dh = X25519SecretKey::generate().expect("rng").public_key();
    assert_eq!(
        IdentityPublic::new(&signing, other_dh.as_bytes(), &sig),
        Err(HandshakeError::InvalidIdentitySignature)
    );
    // A different signing key cannot vouch for this DH key.
    let attacker = SigningKey::generate().expect("rng");
    assert_eq!(
        IdentityPublic::new(&attacker.verifying_key().to_bytes(), &dh, &sig),
        Err(HandshakeError::InvalidIdentitySignature)
    );
}

#[test]
fn public_identity_rejects_bad_lengths_and_weak_signing_keys() {
    let id = IdentityKeyPair::generate().expect("identity");
    let p = id.public();
    let (signing, dh, sig) = (
        p.signing_key().to_bytes(),
        *p.dh_key().as_bytes(),
        *p.dh_signature().as_bytes(),
    );
    assert_eq!(
        IdentityPublic::new(&signing[..31], &dh, &sig),
        Err(HandshakeError::InvalidIdentity)
    );
    assert_eq!(
        IdentityPublic::new(&signing, &dh[..31], &sig),
        Err(HandshakeError::InvalidIdentity)
    );
    assert_eq!(
        IdentityPublic::new(&signing, &dh, &sig[..63]),
        Err(HandshakeError::InvalidIdentitySignature)
    );
    // The all-zero encoding is a small-order point and must never be accepted as an identity.
    assert_eq!(
        IdentityPublic::new(&[0u8; 32], &dh, &sig),
        Err(HandshakeError::InvalidIdentity)
    );
}

#[test]
fn certification_uses_the_documented_label() {
    let id = IdentityKeyPair::generate().expect("identity");
    let p = id.public();
    assert!(
        p.signing_key()
            .verify_labeled(LABEL_IK_DH_SIG, p.dh_key().as_bytes(), p.dh_signature())
            .is_ok()
    );
    assert!(
        p.signing_key()
            .verify(p.dh_key().as_bytes(), p.dh_signature())
            .is_err()
    );
}

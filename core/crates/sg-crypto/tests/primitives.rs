//! Behavioural and property tests for the primitive wrappers (known-answer tests live in the
//! sg-testvectors crate).
#![allow(
    clippy::expect_used,
    reason = "test helpers: a failed expectation is the intended test failure"
)]

use proptest::prelude::*;
use sg_common::constants::{AEAD_TAG_LEN, LABEL_IK_DH_SIG};
use sg_crypto::CryptoError;
use sg_crypto::aead::{open, seal};
use sg_crypto::ed25519::{Signature, SigningKey, VerifyingKey};
use sg_crypto::kdf::{hkdf_sha256, hkdf_sha256_secret};
use sg_crypto::kem::MlKem768SecretKey;
use sg_crypto::rng::{fill_random, random_array, random_secret};
use sg_crypto::secret::SecretBytes;
use sg_crypto::x25519::{X25519PublicKey, X25519SecretKey};

#[test]
fn secret_types_never_print_their_contents() {
    let secret = SecretBytes::new([0x41u8; 32]);
    let shown = format!("{secret:?}");
    assert!(shown.contains("redacted"));
    for needle in ["41", "65", "0x41", "A"] {
        assert!(
            !shown.replace("SecretBytes", "").contains(needle),
            "leaked {needle} in {shown}"
        );
    }
    let x = X25519SecretKey::from_bytes([0x42; 32]);
    assert!(!format!("{x:?}").contains("42"));
    let signing = SigningKey::generate().expect("rng");
    assert_eq!(format!("{signing:?}"), "SigningKey(redacted)");
    let (kem, _) = MlKem768SecretKey::generate().expect("rng");
    assert_eq!(format!("{kem:?}"), "MlKem768SecretKey(redacted)");
}

#[test]
fn secret_equality_compares_contents() {
    assert_eq!(SecretBytes::new([1u8; 16]), SecretBytes::new([1u8; 16]));
    assert_ne!(SecretBytes::new([1u8; 16]), SecretBytes::new([2u8; 16]));
    let mut almost = [1u8; 16];
    almost.iter_mut().rev().take(1).for_each(|b| *b = 0);
    assert_ne!(SecretBytes::new([1u8; 16]), SecretBytes::new(almost));
}

#[test]
fn random_values_are_fresh() {
    let a = random_array::<32>().expect("rng");
    let b = random_array::<32>().expect("rng");
    assert_ne!(a, b);
    assert_ne!(
        random_secret::<32>().expect("rng"),
        random_secret::<32>().expect("rng")
    );
    let mut buf = [0u8; 64];
    fill_random(&mut buf).expect("rng");
    assert!(buf.iter().any(|b| *b != 0));
}

proptest! {
    #[test]
    fn x25519_agreement_is_symmetric(a in any::<[u8; 32]>(), b in any::<[u8; 32]>()) {
        let alice = X25519SecretKey::from_bytes(a);
        let bob = X25519SecretKey::from_bytes(b);
        prop_assert_eq!(
            alice.diffie_hellman(&bob.public_key()),
            bob.diffie_hellman(&alice.public_key())
        );
    }

    #[test]
    fn ed25519_sign_and_verify(seed in any::<[u8; 32]>(), msg in proptest::collection::vec(any::<u8>(), 0..512)) {
        let key = SigningKey::from_seed(&SecretBytes::new(seed));
        let public = key.verifying_key();
        // An honestly generated key is always accepted by the strict key parser.
        let parsed = VerifyingKey::from_bytes(&public.to_bytes()).expect("honest key");
        let sig = key.sign(&msg);
        prop_assert_eq!(parsed.verify(&msg, &sig), Ok(()));
        let mut other = msg.clone();
        other.push(0);
        prop_assert_eq!(parsed.verify(&other, &sig), Err(CryptoError::InvalidSignature));
        let mut flipped = *sig.as_bytes();
        flipped.iter_mut().take(1).for_each(|b| *b ^= 1);
        prop_assert_eq!(parsed.verify(&msg, &Signature::from_bytes(flipped)), Err(CryptoError::InvalidSignature));
        let restored = key.seed();
        prop_assert_eq!(restored.expose(), &seed);
    }

    #[test]
    fn aead_round_trip_and_tamper_detection(
        key in any::<[u8; 32]>(), nonce in any::<[u8; 12]>(),
        aad in proptest::collection::vec(any::<u8>(), 0..64),
        msg in proptest::collection::vec(any::<u8>(), 0..300),
    ) {
        let key = SecretBytes::new(key);
        let sealed = seal(&key, &nonce, &aad, &msg).unwrap();
        prop_assert_eq!(sealed.len(), msg.len() + AEAD_TAG_LEN);
        let opened = open(&key, &nonce, &aad, &sealed).unwrap();
        prop_assert_eq!(opened.as_slice(), msg.as_slice());
        for i in 0..sealed.len() {
            let mut bad = sealed.clone();
            bad.iter_mut().skip(i).take(1).for_each(|b| *b ^= 0x80);
            prop_assert_eq!(open(&key, &nonce, &aad, &bad).err(), Some(CryptoError::DecryptionFailed));
        }
        let mut other_aad = aad.clone();
        other_aad.push(1);
        prop_assert!(open(&key, &nonce, &other_aad, &sealed).is_err());
        let mut other_nonce = nonce;
        other_nonce.iter_mut().take(1).for_each(|b| *b ^= 1);
        prop_assert!(open(&key, &other_nonce, &aad, &sealed).is_err());
        let mut wrong_bytes = *key.expose();
        wrong_bytes.iter_mut().take(1).for_each(|b| *b ^= 1);
        let wrong_key = SecretBytes::new(wrong_bytes);
        prop_assert!(open(&wrong_key, &nonce, &aad, &sealed).is_err());
    }

    #[test]
    fn hkdf_empty_salt_equals_zero_salt(ikm in proptest::collection::vec(any::<u8>(), 1..64), info in proptest::collection::vec(any::<u8>(), 0..32)) {
        let mut a = [0u8; 64];
        let mut b = [0u8; 64];
        hkdf_sha256(&[], &ikm, &info, &mut a).unwrap();
        hkdf_sha256(&[0u8; 32], &ikm, &info, &mut b).unwrap();
        prop_assert_eq!(a, b);
    }
}

#[test]
fn x25519_rejects_low_order_peer_keys_and_bad_lengths() {
    let secret = X25519SecretKey::generate().expect("rng");
    let zero = X25519PublicKey::from_bytes([0u8; 32]);
    assert_eq!(
        secret.diffie_hellman(&zero),
        Err(CryptoError::WeakSharedSecret)
    );
    let one = {
        let mut p = [0u8; 32];
        p.iter_mut().take(1).for_each(|b| *b = 1);
        X25519PublicKey::from_bytes(p)
    };
    assert_eq!(
        secret.diffie_hellman(&one),
        Err(CryptoError::WeakSharedSecret)
    );
    assert!(X25519PublicKey::from_slice(&[0u8; 31]).is_err());
    assert!(X25519PublicKey::from_slice(&[0u8; 33]).is_err());
}

#[test]
fn labelled_signatures_are_domain_separated() {
    let key = SigningKey::generate().expect("rng");
    let public = key.verifying_key();
    let data = public.to_bytes();
    let sig = key.sign_labeled(LABEL_IK_DH_SIG, &data);
    assert_eq!(public.verify_labeled(LABEL_IK_DH_SIG, &data, &sig), Ok(()));
    assert!(
        public
            .verify_labeled(b"Sargoshi/SPK/v1", &data, &sig)
            .is_err()
    );
    assert!(
        public.verify(&data, &sig).is_err(),
        "unlabelled message must not verify"
    );
    let mut explicit = LABEL_IK_DH_SIG.to_vec();
    explicit.extend_from_slice(&data);
    assert_eq!(public.verify(&explicit, &sig), Ok(()));
}

#[test]
fn signature_and_key_parsers_check_lengths() {
    assert!(Signature::from_slice(&[0u8; 63]).is_err());
    assert!(Signature::from_slice(&[0u8; 65]).is_err());
    assert!(VerifyingKey::from_slice(&[0u8; 31]).is_err());
}

#[test]
fn hkdf_rejects_over_long_output_and_derives_exact_lengths() {
    let mut too_long = vec![0u8; 8161];
    assert_eq!(
        hkdf_sha256(b"s", b"k", b"i", &mut too_long),
        Err(CryptoError::KdfOutputTooLong)
    );
    let mut longest = vec![0u8; 8160];
    assert_eq!(hkdf_sha256(b"s", b"k", b"i", &mut longest), Ok(()));
    let a: SecretBytes<44> = hkdf_sha256_secret(b"salt", b"ikm", b"info").expect("44 bytes");
    let b: SecretBytes<44> = hkdf_sha256_secret(b"salt", b"ikm", b"info").expect("44 bytes");
    let c: SecretBytes<44> = hkdf_sha256_secret(b"salt", b"ikm", b"info2").expect("44 bytes");
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn aead_rejects_short_input_and_hides_the_reason() {
    let key = SecretBytes::new([9u8; 32]);
    assert_eq!(
        open(&key, &[0u8; 12], b"", &[0u8; AEAD_TAG_LEN - 1]).err(),
        Some(CryptoError::DecryptionFailed)
    );
    assert_eq!(
        CryptoError::DecryptionFailed.to_string(),
        "decryption failed"
    );
}

#[test]
fn every_crypto_error_has_a_distinct_stable_code() {
    use std::collections::HashSet;
    let all = [
        CryptoError::RandomFailure,
        CryptoError::InvalidKey,
        CryptoError::InvalidSignature,
        CryptoError::WeakSharedSecret,
        CryptoError::DecryptionFailed,
        CryptoError::InvalidLength,
        CryptoError::KdfOutputTooLong,
        CryptoError::InvalidCiphertext,
        CryptoError::UnsupportedVersion,
        CryptoError::StreamTruncated,
        CryptoError::StreamChunkOrder,
        CryptoError::StreamBadChunk,
    ];
    let codes: HashSet<u16> = all.iter().map(|e| e.code().as_u16()).collect();
    assert_eq!(codes.len(), all.len());
}

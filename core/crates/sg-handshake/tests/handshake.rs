//! End-to-end handshake behaviour: agreement, replay rejection, state safety and tampering.
#![allow(
    clippy::expect_used,
    reason = "test helpers: a failed expectation is the intended test failure"
)]

mod common;

use common::{bundle, first_one_time, party};
use sg_common::constants::MLKEM768_ENCAPSULATION_KEY_LEN;
use sg_crypto::CryptoError;
use sg_crypto::kem::MlKem768Ciphertext;
use sg_crypto::x25519::X25519PublicKey;
use sg_handshake::{HandshakeError, IdentityKeyPair, InitialHeader, accept, initiate};

#[test]
fn both_sides_agree_with_a_one_time_prekey() {
    let mut bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let opk = first_one_time(&bob);
    let started = initiate(&alice, &bundle(&bob, Some(opk.clone()))).expect("initiate");

    assert_eq!(started.initial_header.one_time_prekey_id, Some(opk.id));
    let accepted =
        accept(&bob.identity, &mut bob.prekeys, &started.initial_header).expect("accept");

    assert_eq!(started.shared_key, accepted.shared_key);
    assert_eq!(started.associated_data, accepted.associated_data);
    assert_eq!(accepted.consumed_one_time_prekey_id, Some(opk.id));
    assert_eq!(accepted.remote_identity, *alice.public());
    assert_eq!(started.remote_identity, *bob.identity.public());
    // The responder's initial ratchet key pair is its signed prekey.
    assert_eq!(
        accepted.ratchet_secret.public_key(),
        started.remote_ratchet_public
    );
    assert_eq!(
        bob.prekeys.one_time_remaining(),
        4,
        "the used one-time prekey is deleted"
    );
}

#[test]
fn both_sides_agree_without_a_one_time_prekey() {
    let mut bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let started = initiate(&alice, &bundle(&bob, None)).expect("initiate");
    assert_eq!(started.initial_header.one_time_prekey_id, None);
    let accepted =
        accept(&bob.identity, &mut bob.prekeys, &started.initial_header).expect("accept");
    assert_eq!(started.shared_key, accepted.shared_key);
    assert_eq!(accepted.consumed_one_time_prekey_id, None);
    assert_eq!(bob.prekeys.one_time_remaining(), 5);
}

#[test]
fn associated_data_has_the_documented_layout() {
    let bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let started = initiate(&alice, &bundle(&bob, None)).expect("initiate");
    let ad = &started.associated_data;
    assert_eq!(ad.len(), 32 + 32 + MLKEM768_ENCAPSULATION_KEY_LEN);
    assert_eq!(&ad[..32], alice.public().dh_key().as_bytes());
    assert_eq!(&ad[32..64], bob.identity.public().dh_key().as_bytes());
    assert_eq!(&ad[64..], bob.upload.pq_prekey.encapsulation_key.as_bytes());
}

#[test]
fn every_session_has_fresh_keys() {
    let bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let b = bundle(&bob, None);
    let first = initiate(&alice, &b).expect("initiate");
    let second = initiate(&alice, &b).expect("initiate");
    assert_ne!(first.shared_key, second.shared_key);
    assert_ne!(
        first.initial_header.ephemeral,
        second.initial_header.ephemeral
    );
    assert_ne!(
        first.initial_header.kem_ciphertext,
        second.initial_header.kem_ciphertext
    );
}

#[test]
fn a_one_time_prekey_cannot_be_used_twice() {
    let mut bob = party();
    let opk = first_one_time(&bob);
    let a1 = IdentityKeyPair::generate().expect("identity");
    let a2 = IdentityKeyPair::generate().expect("identity");
    let h1 = initiate(&a1, &bundle(&bob, Some(opk.clone())))
        .expect("initiate")
        .initial_header;
    let h2 = initiate(&a2, &bundle(&bob, Some(opk)))
        .expect("initiate")
        .initial_header;
    accept(&bob.identity, &mut bob.prekeys, &h1).expect("first accept");
    assert_eq!(
        accept(&bob.identity, &mut bob.prekeys, &h2).err(),
        Some(HandshakeError::UnknownOneTimePreKey)
    );
}

#[test]
fn replaying_an_accepted_initial_message_is_rejected() {
    // With a one-time prekey: caught by the replay memory before the key lookup.
    let mut bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let header = initiate(&alice, &bundle(&bob, Some(first_one_time(&bob))))
        .expect("initiate")
        .initial_header;
    accept(&bob.identity, &mut bob.prekeys, &header).expect("accept");
    assert_eq!(
        accept(&bob.identity, &mut bob.prekeys, &header).err(),
        Some(HandshakeError::ReplayedInitialMessage)
    );

    // Without one: nothing is consumed, so only the replay memory can stop the repeat.
    let header = initiate(&alice, &bundle(&bob, None))
        .expect("initiate")
        .initial_header;
    accept(&bob.identity, &mut bob.prekeys, &header).expect("accept");
    assert_eq!(
        accept(&bob.identity, &mut bob.prekeys, &header).err(),
        Some(HandshakeError::ReplayedInitialMessage)
    );
}

#[test]
fn the_replay_memory_survives_a_serialization_round_trip() {
    let mut bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let header = initiate(&alice, &bundle(&bob, None))
        .expect("initiate")
        .initial_header;
    accept(&bob.identity, &mut bob.prekeys, &header).expect("accept");
    let mut restored =
        sg_handshake::PrekeySecrets::from_bytes(&bob.prekeys.to_bytes().expect("blob"))
            .expect("restore");
    assert_eq!(
        accept(&bob.identity, &mut restored, &header).err(),
        Some(HandshakeError::ReplayedInitialMessage)
    );
}

#[test]
fn a_failed_accept_leaves_the_prekey_state_untouched() {
    let mut bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let before = bob.prekeys.to_bytes().expect("blob");
    let good = initiate(&alice, &bundle(&bob, Some(first_one_time(&bob))))
        .expect("initiate")
        .initial_header;

    let mut cases: Vec<(&str, InitialHeader)> = Vec::new();
    let mut unknown_spk = good.clone();
    unknown_spk.signed_prekey_id = 77;
    cases.push(("unknown signed prekey", unknown_spk));
    let mut unknown_pq = good.clone();
    unknown_pq.pq_prekey_id = 77;
    cases.push(("unknown pq prekey", unknown_pq));
    let mut unknown_opk = good.clone();
    unknown_opk.one_time_prekey_id = Some(77);
    cases.push(("unknown one-time prekey", unknown_opk));
    let mut weak = good;
    weak.ephemeral = X25519PublicKey::from_bytes([0u8; 32]);
    cases.push(("low-order ephemeral key", weak));

    for (name, header) in cases {
        assert!(
            accept(&bob.identity, &mut bob.prekeys, &header).is_err(),
            "{name}"
        );
        assert_eq!(
            bob.prekeys.to_bytes().expect("blob").as_slice(),
            before.as_slice(),
            "{name}"
        );
    }
}

#[test]
fn error_kinds_are_precise() {
    let mut bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let good = initiate(&alice, &bundle(&bob, None))
        .expect("initiate")
        .initial_header;
    let mut h = good.clone();
    h.signed_prekey_id = 9;
    assert_eq!(
        accept(&bob.identity, &mut bob.prekeys, &h).err(),
        Some(HandshakeError::UnknownSignedPreKey)
    );
    let mut h = good.clone();
    h.pq_prekey_id = 9;
    assert_eq!(
        accept(&bob.identity, &mut bob.prekeys, &h).err(),
        Some(HandshakeError::UnknownPqPreKey)
    );
    let mut h = good;
    h.ephemeral = X25519PublicKey::from_bytes([0u8; 32]);
    assert_eq!(
        accept(&bob.identity, &mut bob.prekeys, &h).err(),
        Some(HandshakeError::Crypto(CryptoError::WeakSharedSecret))
    );
}

#[test]
fn a_corrupted_kem_ciphertext_yields_a_different_key_not_a_crash() {
    // ML-KEM uses implicit rejection: the responder derives an unrelated key, so the first
    // authenticated ratchet message fails to decrypt. The handshake itself must not leak a
    // distinguishing error.
    let mut bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let started = initiate(&alice, &bundle(&bob, None)).expect("initiate");
    let mut header = started.initial_header.clone();
    let mut bytes = header.kem_ciphertext.as_bytes().to_vec();
    bytes.iter_mut().take(1).for_each(|b| *b ^= 1);
    header.kem_ciphertext = MlKem768Ciphertext::from_slice(&bytes).expect("length unchanged");
    let accepted = accept(&bob.identity, &mut bob.prekeys, &header).expect("accept");
    assert_ne!(accepted.shared_key, started.shared_key);
}

#[test]
fn a_different_ephemeral_key_changes_the_derived_key() {
    let mut bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let genuine = initiate(&alice, &bundle(&bob, None)).expect("initiate");
    let mut tampered = genuine.initial_header.clone();
    tampered.ephemeral = sg_crypto::x25519::X25519SecretKey::generate()
        .expect("rng")
        .public_key();
    let accepted = accept(&bob.identity, &mut bob.prekeys, &tampered).expect("accept");
    assert_ne!(accepted.shared_key, genuine.shared_key);
}

#[test]
fn dropping_the_one_time_prekey_id_changes_the_derived_key() {
    let mut bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let genuine = initiate(&alice, &bundle(&bob, Some(first_one_time(&bob)))).expect("initiate");
    let mut stripped = genuine.initial_header.clone();
    stripped.one_time_prekey_id = None;
    let accepted = accept(&bob.identity, &mut bob.prekeys, &stripped).expect("accept");
    assert_ne!(accepted.shared_key, genuine.shared_key);
}

#[test]
fn bundle_tampering_is_caught_by_the_initiator() {
    let bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let good = bundle(&bob, Some(first_one_time(&bob)));

    let mut bad_spk_sig = good.clone();
    bad_spk_sig.signed_prekey.public = sg_crypto::x25519::X25519SecretKey::generate()
        .expect("rng")
        .public_key();
    assert_eq!(
        initiate(&alice, &bad_spk_sig).err(),
        Some(HandshakeError::InvalidSignedPreKeySignature)
    );

    let mut bad_pq = good.clone();
    let other = party();
    bad_pq.pq_prekey.encapsulation_key = other.upload.pq_prekey.encapsulation_key.clone();
    assert_eq!(
        initiate(&alice, &bad_pq).err(),
        Some(HandshakeError::InvalidPqPreKeySignature)
    );

    // Someone else's signed prekey under Bob's identity.
    let mut swapped = good.clone();
    swapped.signed_prekey = other.upload.signed_prekey.clone();
    assert_eq!(
        initiate(&alice, &swapped).err(),
        Some(HandshakeError::InvalidSignedPreKeySignature)
    );

    // A server that substitutes its own identity cannot make the old signatures verify.
    let mut impersonated = good;
    impersonated.identity = other.identity.public().clone();
    assert_eq!(
        initiate(&alice, &impersonated).err(),
        Some(HandshakeError::InvalidSignedPreKeySignature)
    );
}

#[test]
fn a_low_order_key_in_a_bundle_is_refused() {
    let bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let mut weak_opk = bundle(&bob, Some(first_one_time(&bob)));
    if let Some(opk) = weak_opk.one_time_prekey.as_mut() {
        opk.public = X25519PublicKey::from_bytes([0u8; 32]);
    }
    assert_eq!(
        initiate(&alice, &weak_opk).err(),
        Some(HandshakeError::Crypto(CryptoError::WeakSharedSecret))
    );
}

#[test]
fn an_initiator_identity_with_a_low_order_dh_key_is_refused() {
    use sg_common::constants::LABEL_IK_DH_SIG;
    use sg_crypto::ed25519::SigningKey;
    use sg_handshake::IdentityPublic;

    // An attacker can validly sign any DH key with their own signing key, including a weak one.
    let mut bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let attacker = SigningKey::generate().expect("rng");
    let weak_dh = [0u8; 32];
    let signature = attacker.sign_labeled(LABEL_IK_DH_SIG, &weak_dh);
    let weak_identity = IdentityPublic::new(
        &attacker.verifying_key().to_bytes(),
        &weak_dh,
        signature.as_bytes(),
    )
    .expect("the certification itself is valid");

    let mut header = initiate(&alice, &bundle(&bob, None))
        .expect("initiate")
        .initial_header;
    header.identity = weak_identity;
    let before = bob.prekeys.to_bytes().expect("blob");
    assert_eq!(
        accept(&bob.identity, &mut bob.prekeys, &header).err(),
        Some(HandshakeError::Crypto(CryptoError::WeakSharedSecret))
    );
    assert_eq!(
        bob.prekeys.to_bytes().expect("blob").as_slice(),
        before.as_slice()
    );
}

#[test]
fn the_initial_header_survives_the_wire_and_still_works() {
    let mut bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let started = initiate(&alice, &bundle(&bob, Some(first_one_time(&bob)))).expect("initiate");
    let bytes = started.initial_header.to_bytes().expect("encode");
    let parsed = InitialHeader::from_bytes(&bytes).expect("decode");
    assert_eq!(parsed, started.initial_header);
    let accepted = accept(&bob.identity, &mut bob.prekeys, &parsed).expect("accept");
    assert_eq!(accepted.shared_key, started.shared_key);
}

#[test]
fn secrets_never_print() {
    let bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let started = initiate(&alice, &bundle(&bob, None)).expect("initiate");
    let shown = format!("{started:?}");
    assert!(shown.contains("redacted"));
    assert!(!shown.contains("shared_key: ["));
}

#[test]
fn every_handshake_error_has_a_stable_distinct_code() {
    use std::collections::HashSet;
    let all = [
        HandshakeError::InvalidIdentity,
        HandshakeError::InvalidIdentitySignature,
        HandshakeError::InvalidSignedPreKeySignature,
        HandshakeError::InvalidPqPreKeySignature,
        HandshakeError::UnknownSignedPreKey,
        HandshakeError::UnknownPqPreKey,
        HandshakeError::UnknownOneTimePreKey,
        HandshakeError::ReplayedInitialMessage,
        HandshakeError::MalformedBlob,
        HandshakeError::TooManyKeys,
        HandshakeError::DuplicateKeyId,
    ];
    let codes: HashSet<u16> = all.iter().map(|e| e.code().as_u16()).collect();
    assert_eq!(codes.len(), all.len());
    assert!(codes.iter().all(|c| (3000..4000).contains(c)));
}

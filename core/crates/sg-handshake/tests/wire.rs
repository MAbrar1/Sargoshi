//! Canonical wire format tests for bundles and the initial header.
#![allow(
    clippy::expect_used,
    reason = "test helpers: a failed expectation is the intended test failure"
)]

mod common;

use common::{bundle, first_one_time, party};
use proptest::prelude::*;
use sg_common::ProtoError;
use sg_handshake::{
    HandshakeError, IdentityKeyPair, InitialHeader, PreKeyBundle, UploadBundle, initiate,
};

#[test]
fn bundle_round_trips_with_and_without_a_one_time_prekey() {
    let bob = party();
    for opk in [None, Some(first_one_time(&bob))] {
        let original = bundle(&bob, opk);
        let bytes = original.to_bytes().expect("encode");
        assert_eq!(PreKeyBundle::from_bytes(&bytes), Ok(original));
    }
}

#[test]
fn bundle_parser_rejects_unknown_fields_versions_and_truncation() {
    let bob = party();
    let bytes = bundle(&bob, Some(first_one_time(&bob)))
        .to_bytes()
        .expect("encode");

    let mut extra = bytes.clone();
    extra.extend_from_slice(&[0x7a, 0x01, 0x00]); // unknown field 15
    assert_eq!(
        PreKeyBundle::from_bytes(&extra),
        Err(HandshakeError::Proto(ProtoError::NonCanonical))
    );

    for v in [0u8, 2, 255] {
        let mut wrong = bytes.clone();
        wrong[0] = v;
        assert_eq!(
            PreKeyBundle::from_bytes(&wrong),
            Err(HandshakeError::Proto(ProtoError::UnsupportedVersion(v)))
        );
    }
    // Protobuf allows exactly one truncation to parse: cutting at the boundary before the
    // trailing optional one-time prekey leaves a valid bundle that simply lacks it. It must
    // never equal the original, and every other cut must be rejected.
    let original = PreKeyBundle::from_bytes(&bytes).expect("decode");
    let accepted: Vec<usize> = (0..bytes.len())
        .filter(|cut| match PreKeyBundle::from_bytes(&bytes[..*cut]) {
            Ok(parsed) => {
                assert_ne!(parsed, original, "cut {cut} equals the original");
                assert_eq!(parsed.one_time_prekey, None, "cut {cut}");
                true
            }
            Err(_) => false,
        })
        .collect();
    assert!(
        accepted.len() <= 1,
        "more than one truncation parsed: {accepted:?}"
    );
    assert!(
        PreKeyBundle::from_bytes(&vec![1u8; sg_common::constants::MAX_BUNDLE_BYTES + 1]).is_err()
    );
}

#[test]
fn no_single_byte_change_yields_the_same_bundle() {
    let bob = party();
    let original = bundle(&bob, Some(first_one_time(&bob)));
    let bytes = original.to_bytes().expect("encode");
    for i in 0..bytes.len() {
        let mut changed = bytes.clone();
        changed[i] ^= 0x01;
        match PreKeyBundle::from_bytes(&changed) {
            Err(_) => {}
            Ok(parsed) => assert_ne!(parsed, original, "byte {i}"),
        }
    }
}

#[test]
fn changes_inside_signed_material_are_always_rejected() {
    // Flipping any bit of a key or signature (as opposed to an unsigned id or timestamp)
    // must fail verification.
    let bob = party();
    let b = bundle(&bob, None);
    let bytes = b.to_bytes().expect("encode");
    let spk = b.signed_prekey.public.as_bytes();
    let at = bytes
        .windows(32)
        .position(|w| w == spk)
        .expect("spk bytes in encoding");
    for i in at..at + 32 {
        let mut changed = bytes.clone();
        changed[i] ^= 0x01;
        assert!(PreKeyBundle::from_bytes(&changed).is_err(), "spk byte {i}");
    }
}

#[test]
fn initial_header_parser_is_strict() {
    let bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let header = initiate(&alice, &bundle(&bob, Some(first_one_time(&bob))))
        .expect("initiate")
        .initial_header;
    let bytes = header.to_bytes().expect("encode");
    assert_eq!(InitialHeader::from_bytes(&bytes), Ok(header));
    let mut extra = bytes.clone();
    extra.extend_from_slice(&[0x7a, 0x01, 0x00]);
    assert_eq!(
        InitialHeader::from_bytes(&extra),
        Err(HandshakeError::Proto(ProtoError::NonCanonical))
    );
    // Same property as for bundles: only the cut before the trailing optional one-time prekey
    // identifier parses, and it yields a different header (the responder then derives a
    // different key and the first ratchet message fails to authenticate).
    let original = InitialHeader::from_bytes(&bytes).expect("decode");
    let accepted: Vec<usize> = (0..bytes.len())
        .filter(|cut| match InitialHeader::from_bytes(&bytes[..*cut]) {
            Ok(parsed) => {
                assert_ne!(parsed, original, "cut {cut} equals the original");
                assert_eq!(parsed.one_time_prekey_id, None, "cut {cut}");
                true
            }
            Err(_) => false,
        })
        .collect();
    assert!(
        accepted.len() <= 1,
        "more than one truncation parsed: {accepted:?}"
    );
}

#[test]
fn a_forged_initiator_identity_in_the_header_is_rejected() {
    let bob = party();
    let alice = IdentityKeyPair::generate().expect("identity");
    let other = IdentityKeyPair::generate().expect("identity");
    let header = initiate(&alice, &bundle(&bob, None))
        .expect("initiate")
        .initial_header;
    let bytes = header.to_bytes().expect("encode");
    // Replace Alice's DH key bytes with another identity's: the signature no longer matches.
    let alice_dh = alice.public().dh_key().as_bytes();
    let at = bytes
        .windows(32)
        .position(|w| w == alice_dh)
        .expect("dh bytes");
    let mut forged = bytes.clone();
    forged[at..at + 32].copy_from_slice(other.public().dh_key().as_bytes());
    assert_eq!(
        InitialHeader::from_bytes(&forged),
        Err(HandshakeError::InvalidIdentitySignature)
    );
}

#[test]
fn upload_bundle_parser_enforces_the_one_time_prekey_limit() {
    let mut p = party();
    let mut upload = p.prekeys.upload_bundle(&p.identity).expect("bundle");
    let extra = p
        .prekeys
        .replenish_one_time_prekeys(1_000, sg_common::constants::MAX_ONE_TIME_PREKEYS_PER_UPLOAD)
        .expect("keys");
    upload.one_time_prekeys.extend(extra);
    let bytes = upload.to_bytes().expect("encode");
    assert_eq!(
        UploadBundle::from_bytes(&bytes, p.identity.public()).err(),
        Some(HandshakeError::TooManyKeys)
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn parsers_never_panic_and_only_accept_canonical_input(junk in proptest::collection::vec(any::<u8>(), 0..400)) {
        if let Ok(b) = PreKeyBundle::from_bytes(&junk) {
            prop_assert_eq!(b.to_bytes().unwrap(), junk.clone());
        }
        if let Ok(h) = InitialHeader::from_bytes(&junk) {
            prop_assert_eq!(h.to_bytes().unwrap(), junk.clone());
        }
        let p = party();
        let _ = UploadBundle::from_bytes(&junk, p.identity.public());
    }
}

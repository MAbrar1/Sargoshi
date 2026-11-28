//! Session state blobs: round trips, strict parsing and mutation fuzzing.
#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "test helpers: a failed expectation or index is the intended test failure"
)]

mod common;

use common::{after_ad, pair};
use proptest::prelude::*;
use sg_common::ProtoError;
use sg_common::constants::MAX_CACHED_SKIPPED_KEYS;
use sg_ratchet::{RatchetError, Session};

fn blob(s: &Session) -> Vec<u8> {
    s.to_bytes().expect("blob").to_vec()
}

/// A mid-conversation Bob with cached skipped keys: the richest state.
fn busy_bob() -> (Session, Session) {
    let (mut alice, mut bob) = pair();
    let m: Vec<Vec<u8>> = (0..4)
        .map(|i| alice.encrypt(format!("m{i}").as_bytes()).expect("encrypt"))
        .collect();
    bob.decrypt(&m[3]).expect("m3 (three keys skipped)");
    (alice, bob)
}

#[test]
fn blobs_round_trip_byte_for_byte_in_every_state() {
    let (mut alice, mut bob) = pair();
    for s in [&alice, &bob] {
        assert_eq!(
            blob(&Session::from_bytes(&blob(s)).expect("restore")),
            blob(s)
        );
    }
    let a0 = alice.encrypt(b"a0").expect("encrypt");
    bob.decrypt(&a0).expect("decrypt");
    let (alice2, bob2) = busy_bob();
    for s in [&alice, &bob, &alice2, &bob2] {
        assert_eq!(
            blob(&Session::from_bytes(&blob(s)).expect("restore")),
            blob(s)
        );
    }
    assert_eq!(bob2.skipped_key_count(), 3);
}

#[test]
fn a_conversation_continues_across_restores_at_every_step() {
    let (mut alice, mut bob) = pair();
    for round in 0..20 {
        let a = alice
            .encrypt(format!("a{round}").as_bytes())
            .expect("encrypt");
        bob = Session::from_bytes(&blob(&bob)).expect("restore bob");
        assert_eq!(
            bob.decrypt(&a).expect("decrypt").as_slice(),
            format!("a{round}").as_bytes()
        );
        let b = bob
            .encrypt(format!("b{round}").as_bytes())
            .expect("encrypt");
        alice = Session::from_bytes(&blob(&alice)).expect("restore alice");
        assert_eq!(
            alice.decrypt(&b).expect("decrypt").as_slice(),
            format!("b{round}").as_bytes()
        );
    }
}

#[test]
fn cached_skipped_keys_survive_a_restore() {
    let (mut alice, mut bob) = pair();
    let m: Vec<Vec<u8>> = (0..4)
        .map(|i| alice.encrypt(format!("m{i}").as_bytes()).expect("encrypt"))
        .collect();
    bob.decrypt(&m[3]).expect("m3");
    let mut restored = Session::from_bytes(&blob(&bob)).expect("restore");
    assert_eq!(restored.skipped_key_count(), 3);
    for i in [1usize, 0, 2] {
        assert_eq!(
            restored.decrypt(&m[i]).expect("cached").as_slice(),
            format!("m{i}").as_bytes()
        );
    }
}

#[test]
fn the_parser_rejects_every_truncation_and_trailing_bytes() {
    let (_, bob) = busy_bob();
    let b = blob(&bob);
    for cut in 0..b.len() {
        assert!(
            Session::from_bytes(&b[..cut]).is_err(),
            "cut {cut} was accepted"
        );
    }
    let mut longer = b.clone();
    longer.push(0);
    assert_eq!(
        Session::from_bytes(&longer).err(),
        Some(RatchetError::MalformedState)
    );
}

#[test]
fn the_parser_rejects_bad_versions_flags_and_sizes() {
    let (alice, bob) = pair();
    let b = blob(&bob);
    let mut wrong_version = b.clone();
    wrong_version[0] = 2;
    assert_eq!(
        Session::from_bytes(&wrong_version).err(),
        Some(RatchetError::Proto(ProtoError::UnsupportedVersion(2)))
    );
    // Optional-field flags must be 0 or 1. For a fresh Bob the first flag follows root and own key.
    let flag_at = after_ad(&b) + 64;
    let mut bad_flag = b.clone();
    bad_flag[flag_at] = 2;
    assert_eq!(
        Session::from_bytes(&bad_flag).err(),
        Some(RatchetError::MalformedState)
    );
    // An oversized associated-data length.
    let mut huge_ad = blob(&alice);
    huge_ad[1] = 0xff;
    huge_ad[2] = 0xff;
    assert_eq!(
        Session::from_bytes(&huge_ad).err(),
        Some(RatchetError::MalformedState)
    );
    // An oversized skipped-key count: the last two bytes of a fresh blob are the count.
    let mut too_many = b.clone();
    let n = too_many.len();
    too_many[n - 2..].copy_from_slice(
        &u16::try_from(MAX_CACHED_SKIPPED_KEYS + 1)
            .expect("fits")
            .to_be_bytes(),
    );
    assert_eq!(
        Session::from_bytes(&too_many).err(),
        Some(RatchetError::MalformedState)
    );
}

#[test]
fn the_parser_rejects_states_the_code_can_never_produce() {
    let (_, bob) = pair(); // fresh responder: no remote key, no chains, all counters zero
    let b = blob(&bob);
    let base = after_ad(&b) + 64; // has_remote flag
    // layout after it: has_remote(1)=0 has_send(1)=0 sent(4) has_recv(1)=0 received(4) pn(4) n_skipped(2)
    let sent_at = base + 2;
    let received_at = base + 2 + 4 + 1;

    let mut sent_without_chain = b.clone();
    sent_without_chain[sent_at + 3] = 1;
    assert_eq!(
        Session::from_bytes(&sent_without_chain).err(),
        Some(RatchetError::MalformedState)
    );

    let mut received_without_chain = b.clone();
    received_without_chain[received_at + 3] = 1;
    assert_eq!(
        Session::from_bytes(&received_without_chain).err(),
        Some(RatchetError::MalformedState)
    );

    // A skipped key but no remote key to belong to.
    let mut orphan = b.clone();
    let n = orphan.len();
    orphan[n - 2..].copy_from_slice(&1u16.to_be_bytes());
    orphan.extend_from_slice(&[7u8; 68]);
    assert_eq!(
        Session::from_bytes(&orphan).err(),
        Some(RatchetError::MalformedState)
    );
}

#[test]
fn the_parser_rejects_duplicate_skipped_keys() {
    let (_, bob) = busy_bob(); // three cached keys, indices 0, 1, 2
    let mut b = blob(&bob);
    let entries_start = b.len() - 3 * 68;
    let first_index_at = entries_start + 32;
    let second_index_at = entries_start + 68 + 32;
    let first: [u8; 4] = b[first_index_at..first_index_at + 4]
        .try_into()
        .expect("4 bytes");
    b[second_index_at..second_index_at + 4].copy_from_slice(&first);
    assert_eq!(
        Session::from_bytes(&b).err(),
        Some(RatchetError::MalformedState)
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn arbitrary_bytes_never_panic_and_only_canonical_blobs_parse(junk in proptest::collection::vec(any::<u8>(), 0..300)) {
        if let Ok(s) = Session::from_bytes(&junk) {
            prop_assert_eq!(s.to_bytes().unwrap().to_vec(), junk);
        }
    }

    #[test]
    fn mutated_valid_blobs_are_rejected_or_round_trip_exactly(
        flips in proptest::collection::vec((any::<prop::sample::Index>(), any::<u8>()), 1..4),
    ) {
        let (_, bob) = busy_bob();
        let mut b = blob(&bob);
        for (at, value) in flips {
            let i = at.index(b.len());
            b[i] ^= value | 1;
        }
        if let Ok(s) = Session::from_bytes(&b) {
            prop_assert_eq!(s.to_bytes().unwrap().to_vec(), b);
        }
    }

    #[test]
    fn decrypt_never_panics_and_never_changes_state_on_junk(
        junk in proptest::collection::vec(any::<u8>(), 0..400),
    ) {
        let (_, mut bob) = busy_bob();
        let before = blob(&bob);
        prop_assert!(bob.decrypt(&junk).is_err());
        prop_assert_eq!(blob(&bob), before);
    }

    #[test]
    fn mutated_genuine_envelopes_are_rejected_without_changing_state(
        flips in proptest::collection::vec((any::<prop::sample::Index>(), 1u8..=255), 1..4),
    ) {
        let (mut alice, mut bob) = pair();
        let _ = alice.encrypt(b"skip me").unwrap();
        let genuine = alice.encrypt(b"genuine").unwrap();
        let mut mutated = genuine.clone();
        for (at, value) in flips {
            let i = at.index(mutated.len());
            mutated[i] ^= value;
        }
        prop_assume!(mutated != genuine);
        let before = blob(&bob);
        prop_assert!(bob.decrypt(&mutated).is_err());
        prop_assert_eq!(blob(&bob), before);
        let opened = bob.decrypt(&genuine).unwrap();
        prop_assert_eq!(opened.as_slice(), b"genuine");
    }
}

//! Behavioural tests of the Double Ratchet.
#![allow(
    clippy::expect_used,
    reason = "test helpers: a failed expectation is the intended test failure"
)]

mod common;

use common::pair;
use sg_common::Envelope;
use sg_common::constants::{MAX_CACHED_SKIPPED_KEYS, MAX_SKIPPED_KEYS_PER_SESSION};
use sg_crypto::CryptoError;
use sg_ratchet::{RatchetError, Session};

fn send(s: &mut Session, text: &str) -> Vec<u8> {
    s.encrypt(text.as_bytes()).expect("encrypt")
}

fn recv(s: &mut Session, envelope: &[u8]) -> String {
    String::from_utf8(s.decrypt(envelope).expect("decrypt").to_vec()).expect("utf8")
}

fn blob(s: &Session) -> Vec<u8> {
    s.to_bytes().expect("blob").to_vec()
}

#[test]
fn a_responder_cannot_send_until_it_has_received() {
    let (mut alice, mut bob) = pair();
    assert!(alice.can_send());
    assert!(!bob.can_send());
    assert_eq!(
        bob.encrypt(b"too early").err(),
        Some(RatchetError::NoSendChain)
    );
    let first = send(&mut alice, "hello");
    assert_eq!(recv(&mut bob, &first), "hello");
    assert!(bob.can_send());
}

#[test]
fn a_long_ping_pong_conversation_works_in_both_directions() {
    let (mut alice, mut bob) = pair();
    for round in 0..60 {
        let a = send(&mut alice, &format!("alice {round}"));
        assert_eq!(recv(&mut bob, &a), format!("alice {round}"));
        let b = send(&mut bob, &format!("bob {round}"));
        assert_eq!(recv(&mut alice, &b), format!("bob {round}"));
    }
    assert_eq!(alice.skipped_key_count() + bob.skipped_key_count(), 0);
}

#[test]
fn bursts_in_one_direction_share_a_chain() {
    let (mut alice, mut bob) = pair();
    let burst: Vec<Vec<u8>> = (0..20)
        .map(|i| send(&mut alice, &format!("m{i}")))
        .collect();
    for (i, m) in burst.iter().enumerate() {
        assert_eq!(recv(&mut bob, m), format!("m{i}"));
    }
    let reply = send(&mut bob, "ok");
    assert_eq!(recv(&mut alice, &reply), "ok");
    let more: Vec<Vec<u8>> = (0..5).map(|i| send(&mut alice, &format!("n{i}"))).collect();
    for (i, m) in more.iter().enumerate() {
        assert_eq!(recv(&mut bob, m), format!("n{i}"));
    }
}

#[test]
fn out_of_order_delivery_within_a_chain() {
    let (mut alice, mut bob) = pair();
    let m: Vec<Vec<u8>> = (0..5).map(|i| send(&mut alice, &format!("m{i}"))).collect();
    assert_eq!(recv(&mut bob, &m[3]), "m3");
    assert_eq!(bob.skipped_key_count(), 3);
    assert_eq!(recv(&mut bob, &m[1]), "m1");
    assert_eq!(bob.skipped_key_count(), 2);
    assert_eq!(recv(&mut bob, &m[4]), "m4");
    assert_eq!(recv(&mut bob, &m[0]), "m0");
    assert_eq!(recv(&mut bob, &m[2]), "m2");
    assert_eq!(bob.skipped_key_count(), 0);
}

#[test]
fn late_messages_from_an_older_chain_still_decrypt_after_the_ratchet_turns() {
    let (mut alice, mut bob) = pair();
    let a: Vec<Vec<u8>> = (0..3).map(|i| send(&mut alice, &format!("a{i}"))).collect();
    assert_eq!(recv(&mut bob, &a[0]), "a0");
    let b0 = send(&mut bob, "b0");
    assert_eq!(recv(&mut alice, &b0), "b0");
    let a3 = send(&mut alice, "a3"); // new chain: its header says the previous chain had 3 messages
    assert_eq!(recv(&mut bob, &a3), "a3");
    assert_eq!(bob.skipped_key_count(), 2, "a1 and a2 are cached");
    assert_eq!(recv(&mut bob, &a[2]), "a2");
    assert_eq!(recv(&mut bob, &a[1]), "a1");
    assert_eq!(bob.skipped_key_count(), 0);
}

#[test]
fn messages_lost_forever_do_not_break_the_session() {
    let (mut alice, mut bob) = pair();
    let _lost1 = send(&mut alice, "lost 1");
    let _lost2 = send(&mut alice, "lost 2");
    let kept = send(&mut alice, "kept");
    assert_eq!(recv(&mut bob, &kept), "kept");
    let reply = send(&mut bob, "reply");
    assert_eq!(recv(&mut alice, &reply), "reply");
    let next = send(&mut alice, "next");
    assert_eq!(recv(&mut bob, &next), "next");
    assert_eq!(bob.skipped_key_count(), 2);
}

#[test]
fn the_per_message_skip_limit_is_exactly_one_thousand() {
    let (mut alice, mut bob) = pair();
    let all: Vec<Vec<u8>> = (0..=MAX_SKIPPED_KEYS_PER_SESSION + 1)
        .map(|_| send(&mut alice, "x"))
        .collect();
    let before = blob(&bob);
    // Message number 1001 would skip 1001 keys: refused, and nothing changes.
    assert_eq!(
        bob.decrypt(&all[MAX_SKIPPED_KEYS_PER_SESSION + 1]).err(),
        Some(RatchetError::TooManySkippedKeys)
    );
    assert_eq!(blob(&bob), before);
    // Message number 1000 skips exactly 1000 keys: accepted.
    assert_eq!(
        bob.decrypt(&all[MAX_SKIPPED_KEYS_PER_SESSION])
            .expect("limit")
            .as_slice(),
        b"x"
    );
    assert_eq!(bob.skipped_key_count(), MAX_SKIPPED_KEYS_PER_SESSION);
}

#[test]
fn the_cache_limit_is_two_thousand_and_eviction_is_explicit() {
    let (mut alice, mut bob) = pair();
    let total = 2 * MAX_SKIPPED_KEYS_PER_SESSION + 4;
    let all: Vec<Vec<u8>> = (0..total).map(|_| send(&mut alice, "x")).collect();
    // First jump: receive #1000 (skips 1000). Second: #2001 (skips 1000). Cache is now full.
    bob.decrypt(&all[MAX_SKIPPED_KEYS_PER_SESSION])
        .expect("first jump");
    bob.decrypt(&all[2 * MAX_SKIPPED_KEYS_PER_SESSION + 1])
        .expect("second jump");
    assert_eq!(bob.skipped_key_count(), MAX_CACHED_SKIPPED_KEYS);
    // One more skip would exceed the cache: refused without changing anything.
    let before = blob(&bob);
    assert_eq!(
        bob.decrypt(&all[2 * MAX_SKIPPED_KEYS_PER_SESSION + 3])
            .err(),
        Some(RatchetError::TooManySkippedKeys)
    );
    assert_eq!(blob(&bob), before);
    // In-order delivery still works with a full cache, and consuming a cached key frees a slot.
    assert!(
        bob.decrypt(&all[2 * MAX_SKIPPED_KEYS_PER_SESSION + 2])
            .is_ok()
    );
    assert!(bob.decrypt(&all[0]).is_ok());
    // Explicit, caller-triggered eviction makes room too.
    assert_eq!(bob.discard_oldest_skipped_keys(10), 10);
    assert_eq!(bob.skipped_key_count(), MAX_CACHED_SKIPPED_KEYS - 11);
    assert_eq!(
        bob.discard_oldest_skipped_keys(usize::MAX),
        MAX_CACHED_SKIPPED_KEYS - 11
    );
}

#[test]
fn a_huge_message_number_is_refused_instantly() {
    let (mut alice, mut bob) = pair();
    let genuine = send(&mut alice, "x");
    let mut parsed = Envelope::decode(&genuine).expect("decode");
    parsed.header.msg_num = u32::MAX;
    let forged = parsed.encode().expect("encode");
    let before = blob(&bob);
    let started = std::time::Instant::now();
    assert_eq!(
        bob.decrypt(&forged).err(),
        Some(RatchetError::TooManySkippedKeys)
    );
    assert!(started.elapsed() < std::time::Duration::from_millis(200));
    assert_eq!(blob(&bob), before);
    assert!(
        bob.decrypt(&genuine).is_ok(),
        "the session is still healthy"
    );
}

#[test]
fn replays_are_rejected_in_every_position_and_change_nothing() {
    let (mut alice, mut bob) = pair();
    let a0 = send(&mut alice, "a0");
    let a1 = send(&mut alice, "a1");
    let a2 = send(&mut alice, "a2");
    bob.decrypt(&a0).expect("a0");
    // Replay of a delivered message in the same chain.
    let state = blob(&bob);
    assert_eq!(bob.decrypt(&a0).err(), Some(RatchetError::DuplicateMessage));
    assert_eq!(blob(&bob), state);
    // Delivery through the skipped-key cache, then replay of it.
    bob.decrypt(&a2).expect("a2 (a1 skipped)");
    bob.decrypt(&a1).expect("a1 from the cache");
    for replay in [&a0, &a1, &a2] {
        let state = blob(&bob);
        assert!(bob.decrypt(replay).is_err());
        assert_eq!(blob(&bob), state);
    }
    // After the ratchet has turned, old messages are still refused.
    let b0 = send(&mut bob, "b0");
    alice.decrypt(&b0).expect("b0");
    let a3 = send(&mut alice, "a3");
    bob.decrypt(&a3).expect("a3");
    for replay in [&a0, &a1, &a2, &a3] {
        let state = blob(&bob);
        assert!(bob.decrypt(replay).is_err(), "replay accepted");
        assert_eq!(blob(&bob), state);
    }
    assert!(alice.decrypt(&b0).is_err());
}

#[test]
fn every_single_byte_change_is_rejected_and_never_damages_the_session() {
    let (mut alice, mut bob) = pair();
    let genuine = send(&mut alice, "authentic message");
    let state = blob(&bob);
    for i in 0..genuine.len() {
        let mut tampered = genuine.clone();
        tampered[i] ^= 0x01;
        assert!(
            bob.decrypt(&tampered).is_err(),
            "byte {i} change was accepted"
        );
        assert_eq!(blob(&bob), state, "byte {i} change altered the session");
    }
    assert_eq!(
        recv(&mut bob, &genuine),
        "authentic message",
        "the genuine message still works"
    );
}

#[test]
fn tampered_header_fields_are_rejected() {
    let (mut alice, mut bob) = pair();
    let _skipped = send(&mut alice, "first");
    let genuine = send(&mut alice, "second");
    let parsed = Envelope::decode(&genuine).expect("decode");
    let state = blob(&bob);
    let mut variants = Vec::new();
    let mut v = parsed.clone();
    v.header.msg_num += 1;
    variants.push(v);
    let mut v = parsed.clone();
    v.header.msg_num = 0;
    variants.push(v);
    let mut v = parsed.clone();
    v.header.prev_chain_len += 1;
    variants.push(v);
    let mut v = parsed.clone();
    v.header.ratchet_pub[0] ^= 1;
    variants.push(v);
    for v in variants {
        assert!(bob.decrypt(&v.encode().expect("encode")).is_err());
        assert_eq!(blob(&bob), state);
    }
}

#[test]
fn a_session_from_another_handshake_cannot_read_the_message() {
    let (mut alice, _) = pair();
    let (_, mut other_bob) = pair();
    let m = send(&mut alice, "secret");
    assert_eq!(
        other_bob.decrypt(&m).err(),
        Some(RatchetError::Crypto(CryptoError::DecryptionFailed))
    );
}

#[test]
fn a_first_message_claiming_a_previous_chain_is_refused() {
    let (mut alice, mut bob) = pair();
    let genuine = send(&mut alice, "x");
    let mut parsed = Envelope::decode(&genuine).expect("decode");
    parsed.header.prev_chain_len = 5;
    let state = blob(&bob);
    assert_eq!(
        bob.decrypt(&parsed.encode().expect("encode")).err(),
        Some(RatchetError::InvalidHeader)
    );
    assert_eq!(blob(&bob), state);
}

#[test]
fn a_message_for_the_initiators_own_starting_key_is_refused() {
    // Before Bob's first reply, Alice's remote key is Bob's signed prekey and she has no receiving
    // chain; a message claiming that key is impossible.
    let (mut alice, mut bob) = pair();
    let first = send(&mut alice, "x");
    bob.decrypt(&first).expect("first");
    let reply = send(&mut bob, "y");
    let mut parsed = Envelope::decode(&reply).expect("decode");
    let alice_state = blob(&alice);
    let signed_prekey_public = {
        // Rebuild what Alice holds as her remote key: it sits right after the AD, root and own key.
        let b = blob(&alice);
        let at = common::after_ad(&b) + 64 + 1;
        let mut key = [0u8; 32];
        key.copy_from_slice(&b[at..at + 32]);
        key
    };
    parsed.header.ratchet_pub = signed_prekey_public;
    assert_eq!(
        alice.decrypt(&parsed.encode().expect("encode")).err(),
        Some(RatchetError::InvalidHeader)
    );
    assert_eq!(blob(&alice), alice_state);
}

#[test]
fn counters_never_wrap() {
    let (alice, _) = pair();
    let mut b = blob(&alice);
    let at = common::after_ad(&b) + 64 + 33 + 33; // `sent` follows root, own key, remote key, send chain
    b[at..at + 4].copy_from_slice(&u32::MAX.to_be_bytes());
    let mut maxed = Session::from_bytes(&b).expect("restore");
    let before = blob(&maxed);
    assert_eq!(
        maxed.encrypt(b"x").err(),
        Some(RatchetError::CounterOverflow)
    );
    assert_eq!(blob(&maxed), before);

    b[at..at + 4].copy_from_slice(&(u32::MAX - 1).to_be_bytes());
    let mut last = Session::from_bytes(&b).expect("restore");
    assert!(last.encrypt(b"last one").is_ok());
    assert_eq!(
        last.encrypt(b"too many").err(),
        Some(RatchetError::CounterOverflow)
    );
}

#[test]
fn an_envelope_larger_than_the_limit_is_refused_without_advancing() {
    let (mut alice, _) = pair();
    let before = blob(&alice);
    let too_big = vec![0u8; sg_common::constants::MAX_ENVELOPE_BYTES];
    assert!(alice.encrypt(&too_big).is_err());
    assert_eq!(blob(&alice), before);
    assert!(alice.encrypt(b"still fine").is_ok());
}

#[test]
fn sessions_never_print_secrets() {
    let (alice, _) = pair();
    let shown = format!("{alice:?}");
    assert!(shown.contains("redacted"));
}

#[test]
fn every_ratchet_error_has_a_stable_distinct_code() {
    use std::collections::HashSet;
    let all = [
        RatchetError::NoSendChain,
        RatchetError::DuplicateMessage,
        RatchetError::TooManySkippedKeys,
        RatchetError::InvalidHeader,
        RatchetError::CounterOverflow,
        RatchetError::MalformedState,
    ];
    let codes: HashSet<u16> = all.iter().map(|e| e.code().as_u16()).collect();
    assert_eq!(codes.len(), all.len());
    assert!(codes.iter().all(|c| (4000..5000).contains(c)));
}

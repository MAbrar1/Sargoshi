//! Shared helpers for the ratchet tests.
#![allow(
    dead_code,
    missing_docs,
    clippy::indexing_slicing,
    clippy::expect_used,
    reason = "test helpers: a failed expectation is the intended test failure"
)]

use sg_common::HEADER_LEN;
use sg_handshake::{IdentityKeyPair, PreKeyBundle, PrekeySecrets, accept, initiate};
use sg_ratchet::{PqRatchetHook, Session};

/// Two sessions that completed a real handshake. Alice is the initiator.
pub fn pair() -> (Session, Session) {
    pair_with(&sg_ratchet::NoPqRatchet, &sg_ratchet::NoPqRatchet)
}

pub fn pair_with(
    alice_hook: &dyn PqRatchetHook,
    bob_hook: &dyn PqRatchetHook,
) -> (Session, Session) {
    let _ = bob_hook;
    let alice_id = IdentityKeyPair::generate().expect("identity");
    let bob_id = IdentityKeyPair::generate().expect("identity");
    let (mut bob_prekeys, upload) =
        PrekeySecrets::generate(&bob_id, 1, 1, 1_700_000_000, 10, 3).expect("prekeys");
    let bundle = PreKeyBundle {
        identity: bob_id.public().clone(),
        signed_prekey: upload.signed_prekey,
        pq_prekey: upload.pq_prekey,
        one_time_prekey: upload.one_time_prekeys.first().cloned(),
    };
    let started = initiate(&alice_id, &bundle).expect("initiate");
    let accepted = accept(&bob_id, &mut bob_prekeys, &started.initial_header).expect("accept");
    let (alice, _) = Session::from_initiator_with_hook(started, alice_hook).expect("alice");
    let bob = Session::from_responder(accepted).expect("bob");
    (alice, bob)
}

/// Offset of the first byte after the associated data in a session blob.
pub fn after_ad(blob: &[u8]) -> usize {
    let len = usize::from(u16::from_be_bytes([blob[1], blob[2]]));
    3 + len
}

pub const ENVELOPE_OVERHEAD: usize = HEADER_LEN;

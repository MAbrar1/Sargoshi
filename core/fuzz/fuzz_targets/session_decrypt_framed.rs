#![no_main]
//! Decrypt on well-framed envelopes with fuzzed header fields and ciphertext. Random bytes rarely
//! pass envelope framing, so this reaches the skip budgets, chain switches and header checks.
use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use sg_common::{Envelope, RatchetHeader};
use sg_handshake::{IdentityKeyPair, PreKeyBundle, PrekeySecrets, accept, initiate};
use sg_ratchet::Session;

/// A responder that has already received one message, so it holds chains and a remote key.
fn session_blob() -> &'static [u8] {
    static BLOB: OnceLock<Vec<u8>> = OnceLock::new();
    BLOB.get_or_init(|| {
        let alice_id = IdentityKeyPair::generate().expect("randomness");
        let bob_id = IdentityKeyPair::generate().expect("randomness");
        let (mut prekeys, upload) = PrekeySecrets::generate(&bob_id, 1, 1, 1, 1, 1).expect("prekeys");
        let bundle = PreKeyBundle {
            identity: bob_id.public().clone(),
            signed_prekey: upload.signed_prekey,
            pq_prekey: upload.pq_prekey,
            one_time_prekey: None,
        };
        let started = initiate(&alice_id, &bundle).expect("initiate");
        let accepted = accept(&bob_id, &mut prekeys, &started.initial_header).expect("accept");
        let (mut alice, _) = Session::from_initiator(started).expect("alice");
        let mut bob = Session::from_responder(accepted).expect("bob");
        bob.decrypt(&alice.encrypt(b"hello").expect("encrypt")).expect("first message");
        bob.to_bytes().expect("blob").to_vec()
    })
}

fuzz_target!(|data: &[u8]| {
    let Some((key, rest)) = data.split_first_chunk::<32>() else { return };
    let Some((prev, rest)) = rest.split_first_chunk::<4>() else { return };
    let Some((number, ciphertext)) = rest.split_first_chunk::<4>() else { return };
    if ciphertext.len() < 16 {
        return;
    }
    let envelope = Envelope {
        header: RatchetHeader {
            ratchet_pub: *key,
            prev_chain_len: u32::from_be_bytes(*prev),
            msg_num: u32::from_be_bytes(*number),
        },
        ciphertext: ciphertext.to_vec(),
    };
    let Ok(bytes) = envelope.encode() else { return };
    let mut session = Session::from_bytes(session_blob()).expect("valid blob");
    let before = session.to_bytes().expect("blob");
    match session.decrypt(&bytes) {
        Ok(_) => panic!("a forged message was accepted"),
        Err(_) => assert_eq!(session.to_bytes().expect("blob").as_slice(), before.as_slice()),
    }
});

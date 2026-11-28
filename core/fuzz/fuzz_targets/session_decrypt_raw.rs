#![no_main]
//! Decrypt on arbitrary bytes: never accepted, and a failure never changes the session.
use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use sg_handshake::{IdentityKeyPair, PreKeyBundle, PrekeySecrets, accept, initiate};
use sg_ratchet::Session;

/// The serialized state of a responder session created by a real handshake.
fn responder_blob() -> &'static [u8] {
    static BLOB: OnceLock<Vec<u8>> = OnceLock::new();
    BLOB.get_or_init(|| {
        let alice = IdentityKeyPair::generate().expect("randomness");
        let bob = IdentityKeyPair::generate().expect("randomness");
        let (mut prekeys, upload) = PrekeySecrets::generate(&bob, 1, 1, 1, 1, 1).expect("prekeys");
        let bundle = PreKeyBundle {
            identity: bob.public().clone(),
            signed_prekey: upload.signed_prekey,
            pq_prekey: upload.pq_prekey,
            one_time_prekey: None,
        };
        let started = initiate(&alice, &bundle).expect("initiate");
        let accepted = accept(&bob, &mut prekeys, &started.initial_header).expect("accept");
        Session::from_responder(accepted).expect("session").to_bytes().expect("blob").to_vec()
    })
}

fuzz_target!(|data: &[u8]| {
    let mut session = Session::from_bytes(responder_blob()).expect("valid blob");
    let before = session.to_bytes().expect("blob");
    match session.decrypt(data) {
        Ok(_) => panic!("a forged message was accepted"),
        Err(_) => assert_eq!(session.to_bytes().expect("blob").as_slice(), before.as_slice()),
    }
});

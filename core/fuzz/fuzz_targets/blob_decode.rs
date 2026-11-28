#![no_main]
//! Secret blob parsers (identity, prekey store, session): only canonical blobs may parse.
use libfuzzer_sys::fuzz_target;
use sg_handshake::{IdentityKeyPair, PrekeySecrets};
use sg_ratchet::Session;

fuzz_target!(|data: &[u8]| {
    if let Ok(identity) = IdentityKeyPair::from_secret_bytes(data) {
        assert_eq!(identity.to_secret_bytes().as_slice(), data);
    }
    if let Ok(prekeys) = PrekeySecrets::from_bytes(data) {
        assert_eq!(prekeys.to_bytes().expect("serializes").as_slice(), data);
    }
    if let Ok(session) = Session::from_bytes(data) {
        assert_eq!(session.to_bytes().expect("serializes").as_slice(), data);
    }
});

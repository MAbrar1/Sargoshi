#![no_main]
//! Bundle and initial header parsers: only canonical, fully verified input may parse.
use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use sg_handshake::{IdentityKeyPair, InitialHeader, PreKeyBundle, UploadBundle};

fn identity() -> &'static IdentityKeyPair {
    static IDENTITY: OnceLock<IdentityKeyPair> = OnceLock::new();
    IDENTITY.get_or_init(|| IdentityKeyPair::generate().expect("operating system randomness"))
}

fuzz_target!(|data: &[u8]| {
    if let Ok(bundle) = PreKeyBundle::from_bytes(data) {
        assert_eq!(bundle.to_bytes().ok().as_deref(), Some(data));
    }
    if let Ok(header) = InitialHeader::from_bytes(data) {
        assert_eq!(header.to_bytes().ok().as_deref(), Some(data));
    }
    if let Ok(upload) = UploadBundle::from_bytes(data, identity().public()) {
        assert_eq!(upload.to_bytes().ok().as_deref(), Some(data));
    }
});

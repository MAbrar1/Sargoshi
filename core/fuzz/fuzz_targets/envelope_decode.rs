#![no_main]
//! Envelope parser: only canonical input may parse, and re-encoding must reproduce it exactly.
use libfuzzer_sys::fuzz_target;
use sg_common::Envelope;

fuzz_target!(|data: &[u8]| {
    if let Ok(envelope) = Envelope::decode(data) {
        assert_eq!(envelope.encode().ok().as_deref(), Some(data), "non-canonical input was accepted");
    }
});

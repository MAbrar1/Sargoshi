//! Property and example tests for the canonical wire formats.
#![allow(
    clippy::expect_used,
    reason = "test helpers: a failed expectation is the intended test failure"
)]

use proptest::prelude::*;
use sg_common::constants::{AEAD_TAG_LEN, MAX_ENVELOPE_BYTES};
use sg_common::{Envelope, HEADER_LEN, ProtoError, RatchetHeader};

fn header(n: u32) -> RatchetHeader {
    RatchetHeader {
        ratchet_pub: [7u8; 32],
        prev_chain_len: 3,
        msg_num: n,
    }
}

fn valid_envelope() -> Vec<u8> {
    Envelope {
        header: header(9),
        ciphertext: vec![0xAB; 48],
    }
    .encode()
    .expect("valid envelope")
}

proptest! {
    #[test]
    fn header_round_trips(key in any::<[u8; 32]>(), prev in any::<u32>(), n in any::<u32>()) {
        let h = RatchetHeader { ratchet_pub: key, prev_chain_len: prev, msg_num: n };
        let bytes = h.to_bytes();
        prop_assert_eq!(bytes.len(), HEADER_LEN);
        prop_assert_eq!(RatchetHeader::from_bytes(&bytes), Ok(h));
    }

    #[test]
    fn header_rejects_every_wrong_length(len in 0usize..200) {
        prop_assume!(len != HEADER_LEN);
        let r = RatchetHeader::from_bytes(&vec![1u8; len]);
        let is_bad_length = matches!(r, Err(ProtoError::BadLength { .. }));
        prop_assert!(is_bad_length);
    }

    #[test]
    fn header_rejects_every_wrong_version(v in any::<u8>()) {
        prop_assume!(v != 1);
        let mut bytes = header(1).to_bytes();
        bytes[0] = v;
        prop_assert_eq!(RatchetHeader::from_bytes(&bytes), Err(ProtoError::UnsupportedVersion(v)));
    }

    #[test]
    fn envelope_round_trips(
        key in any::<[u8; 32]>(), prev in any::<u32>(), n in any::<u32>(),
        ct in proptest::collection::vec(any::<u8>(), AEAD_TAG_LEN..4096),
    ) {
        let e = Envelope {
            header: RatchetHeader { ratchet_pub: key, prev_chain_len: prev, msg_num: n },
            ciphertext: ct,
        };
        let bytes = e.encode().unwrap();
        prop_assert_eq!(Envelope::decode(&bytes), Ok(e));
    }

    #[test]
    fn decode_never_panics_and_only_accepts_what_encode_produces(
        junk in proptest::collection::vec(any::<u8>(), 0..600),
    ) {
        if let Ok(e) = Envelope::decode(&junk) {
            prop_assert_eq!(e.encode().unwrap(), junk);
        }
    }

    #[test]
    fn every_single_byte_flip_in_the_framing_is_rejected_or_changes_the_message(
        idx in 0usize..100, bit in 0u8..8,
    ) {
        let original = valid_envelope();
        let mut mutated = original.clone();
        let i = idx % mutated.len();
        mutated[i] ^= 1 << bit;
        match Envelope::decode(&mutated) {
            Err(_) => {}
            Ok(e) => prop_assert_ne!(e, Envelope::decode(&original).unwrap()),
        }
    }

    #[test]
    fn every_non_one_version_byte_is_rejected(v in any::<u8>()) {
        prop_assume!(v != 1);
        let mut bytes = valid_envelope();
        bytes[0] = v;
        prop_assert_eq!(Envelope::decode(&bytes), Err(ProtoError::UnsupportedVersion(v)));
    }
}

#[test]
fn empty_input_is_truncated() {
    assert_eq!(Envelope::decode(&[]), Err(ProtoError::Truncated));
}

#[test]
fn unknown_field_is_not_canonical() {
    let mut bytes = valid_envelope();
    bytes.extend_from_slice(&[0x1a, 0x01, 0x00]); // field 3, length 1
    assert_eq!(Envelope::decode(&bytes), Err(ProtoError::NonCanonical));
}

#[test]
fn reserved_field_is_rejected_in_version_one() {
    let mut bytes = valid_envelope();
    bytes.extend_from_slice(&[0x7a, 0x01, 0xff]); // field 15, length 1
    assert_eq!(Envelope::decode(&bytes), Err(ProtoError::ReservedFieldSet));
}

#[test]
fn trailing_garbage_is_rejected() {
    let mut bytes = valid_envelope();
    bytes.push(0xff);
    assert!(Envelope::decode(&bytes).is_err());
}

#[test]
fn truncated_envelope_is_rejected_at_every_length() {
    let bytes = valid_envelope();
    for cut in 0..bytes.len() {
        assert!(
            Envelope::decode(&bytes[..cut]).is_err(),
            "cut at {cut} was accepted"
        );
    }
}

#[test]
fn short_ciphertext_is_rejected_on_encode() {
    let e = Envelope {
        header: header(0),
        ciphertext: vec![0; AEAD_TAG_LEN - 1],
    };
    assert!(matches!(
        e.encode(),
        Err(ProtoError::BadLength {
            field: "ciphertext",
            ..
        })
    ));
}

#[test]
fn oversize_envelope_is_rejected_on_encode_and_decode() {
    let e = Envelope {
        header: header(0),
        ciphertext: vec![0; MAX_ENVELOPE_BYTES],
    };
    assert!(matches!(e.encode(), Err(ProtoError::TooLarge { .. })));
    assert!(matches!(
        Envelope::decode(&vec![1u8; MAX_ENVELOPE_BYTES + 1]),
        Err(ProtoError::TooLarge { .. })
    ));
}

#[test]
fn largest_allowed_envelope_round_trips() {
    // Find the largest ciphertext that still fits, proving the limit is reachable and exact.
    let mut lo = AEAD_TAG_LEN;
    let mut hi = MAX_ENVELOPE_BYTES;
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        let e = Envelope {
            header: header(0),
            ciphertext: vec![5; mid],
        };
        if e.encode().is_ok() {
            lo = mid
        } else {
            hi = mid - 1
        }
    }
    let e = Envelope {
        header: header(0),
        ciphertext: vec![5; lo],
    };
    let bytes = e.encode().unwrap();
    assert!(bytes.len() <= MAX_ENVELOPE_BYTES);
    assert_eq!(Envelope::decode(&bytes), Ok(e));
    let too_big = Envelope {
        header: header(0),
        ciphertext: vec![5; lo + 1],
    };
    assert!(too_big.encode().is_err());
}

#[test]
fn header_layout_is_fixed_and_big_endian() {
    let h = RatchetHeader {
        ratchet_pub: [0xAA; 32],
        prev_chain_len: 0x0102_0304,
        msg_num: 0x0A0B_0C0D,
    };
    let b = h.to_bytes();
    assert_eq!(b[0], 1);
    assert_eq!(&b[1..33], &[0xAA; 32]);
    assert_eq!(&b[33..37], &[1, 2, 3, 4]);
    assert_eq!(&b[37..41], &[0x0A, 0x0B, 0x0C, 0x0D]);
}

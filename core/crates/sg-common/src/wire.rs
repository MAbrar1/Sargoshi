//! Canonical wire formats.
//!
//! * [`RatchetHeader`] is a fixed 41-byte layout so that the bytes authenticated as associated
//!   data are canonical by construction: `version(1) || ratchet_pub(32) || prev_len(u32 BE) ||
//!   msg_num(u32 BE)`.
//! * [`Envelope`] is `version(1) || protobuf(body)`. The decoder re-encodes what it parsed and
//!   rejects the input unless the bytes are identical, so unknown fields, reordered fields and
//!   non-minimal encodings are refused (no malleability).
//!
//! Field 15 of the body is reserved for the message-franking commitment and sealed-sender
//! extensions (see docs/architecture/PROTOCOL.md). Version 1 receivers reject it when set.

use prost::Message;

use crate::constants::{AEAD_TAG_LEN, MAX_ENVELOPE_BYTES, X25519_KEY_LEN};
use crate::error::ProtoError;
use crate::version::{PROTOCOL_VERSION, check_version};

/// Length of the serialized [`RatchetHeader`] in bytes.
pub const HEADER_LEN: usize = 1 + X25519_KEY_LEN + 4 + 4;

/// The Double Ratchet message header. Its serialized form is authenticated as associated data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RatchetHeader {
    /// The sender's current ratchet public key (X25519).
    pub ratchet_pub: [u8; X25519_KEY_LEN],
    /// Number of messages in the sender's previous sending chain.
    pub prev_chain_len: u32,
    /// Index of this message in the current sending chain.
    pub msg_num: u32,
}

impl RatchetHeader {
    /// Serialize to the fixed 41-byte canonical form.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_LEN);
        out.push(PROTOCOL_VERSION);
        out.extend_from_slice(&self.ratchet_pub);
        out.extend_from_slice(&self.prev_chain_len.to_be_bytes());
        out.extend_from_slice(&self.msg_num.to_be_bytes());
        out
    }

    /// Parse the fixed 41-byte form.
    ///
    /// # Errors
    /// Fails on a wrong length or an unsupported version.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtoError> {
        if bytes.len() != HEADER_LEN {
            return Err(ProtoError::BadLength {
                field: "header",
                expected: HEADER_LEN,
                actual: bytes.len(),
            });
        }
        let (version, rest) = bytes.split_first().ok_or(ProtoError::Truncated)?;
        check_version(*version)?;
        let (key, rest) = rest
            .split_at_checked(X25519_KEY_LEN)
            .ok_or(ProtoError::Malformed("header key"))?;
        let (prev, num) = rest
            .split_at_checked(4)
            .ok_or(ProtoError::Malformed("header counters"))?;
        let ratchet_pub = <[u8; X25519_KEY_LEN]>::try_from(key)
            .map_err(|_| ProtoError::Malformed("header key"))?;
        let prev_chain_len = u32::from_be_bytes(
            <[u8; 4]>::try_from(prev)
                .map_err(|_| ProtoError::Malformed("previous chain length"))?,
        );
        let msg_num = u32::from_be_bytes(
            <[u8; 4]>::try_from(num).map_err(|_| ProtoError::Malformed("message number"))?,
        );
        Ok(Self {
            ratchet_pub,
            prev_chain_len,
            msg_num,
        })
    }
}

/// Protobuf body of an envelope. Private: callers use [`Envelope`].
#[derive(Clone, PartialEq, Message)]
struct EnvelopeBody {
    #[prost(bytes = "vec", tag = "1")]
    header: Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    ciphertext: Vec<u8>,
    /// Reserved for the franking commitment and sealed-sender extensions; empty in version 1.
    #[prost(bytes = "vec", tag = "15")]
    reserved: Vec<u8>,
}

/// One end-to-end encrypted message as it travels through the gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    /// The authenticated header.
    pub header: RatchetHeader,
    /// AEAD ciphertext including the authentication tag.
    pub ciphertext: Vec<u8>,
}

impl Envelope {
    /// Serialize to `version || protobuf(body)`.
    ///
    /// # Errors
    /// Fails if the ciphertext is shorter than an AEAD tag or the result exceeds
    /// [`MAX_ENVELOPE_BYTES`].
    pub fn encode(&self) -> Result<Vec<u8>, ProtoError> {
        if self.ciphertext.len() < AEAD_TAG_LEN {
            return Err(ProtoError::BadLength {
                field: "ciphertext",
                expected: AEAD_TAG_LEN,
                actual: self.ciphertext.len(),
            });
        }
        let body = EnvelopeBody {
            header: self.header.to_bytes(),
            ciphertext: self.ciphertext.clone(),
            reserved: Vec::new(),
        };
        let mut out = Vec::with_capacity(1 + body.encoded_len());
        out.push(PROTOCOL_VERSION);
        body.encode(&mut out)
            .map_err(|_| ProtoError::Malformed("encode"))?;
        if out.len() > MAX_ENVELOPE_BYTES {
            return Err(ProtoError::TooLarge {
                limit: MAX_ENVELOPE_BYTES,
                actual: out.len(),
            });
        }
        Ok(out)
    }

    /// Parse and strictly validate an envelope.
    ///
    /// # Errors
    /// Fails on empty or oversized input, an unsupported version, malformed or non-canonical
    /// encoding, a set reserved field, a wrong-sized header or a ciphertext shorter than a tag.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtoError> {
        if bytes.is_empty() {
            return Err(ProtoError::Truncated);
        }
        if bytes.len() > MAX_ENVELOPE_BYTES {
            return Err(ProtoError::TooLarge {
                limit: MAX_ENVELOPE_BYTES,
                actual: bytes.len(),
            });
        }
        let (version, rest) = bytes.split_first().ok_or(ProtoError::Truncated)?;
        check_version(*version)?;
        let body =
            EnvelopeBody::decode(rest).map_err(|_| ProtoError::Malformed("envelope body"))?;
        if body.encode_to_vec() != rest {
            return Err(ProtoError::NonCanonical);
        }
        if !body.reserved.is_empty() {
            return Err(ProtoError::ReservedFieldSet);
        }
        let header = RatchetHeader::from_bytes(&body.header)?;
        if body.ciphertext.len() < AEAD_TAG_LEN {
            return Err(ProtoError::BadLength {
                field: "ciphertext",
                expected: AEAD_TAG_LEN,
                actual: body.ciphertext.len(),
            });
        }
        Ok(Self {
            header,
            ciphertext: body.ciphertext,
        })
    }
}

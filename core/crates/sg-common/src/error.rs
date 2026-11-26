//! Error types shared across the core, with stable numeric codes for the FFI boundary.
//!
//! Code ranges (reserved so crates never collide):
//! 1000-1999 protocol and wire format (this crate), 2000-2999 primitives (`sg-crypto`),
//! 3000-3999 identity and handshake (`sg-handshake`), 4000-4999 ratchet (`sg-ratchet`),
//! 5000-5999 groups, 6000-6999 vault and backup, 9000-9999 FFI and internal.

use thiserror::Error;

/// Errors raised while parsing or building wire messages.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProtoError {
    /// The version byte is not the supported protocol version.
    #[error("unsupported protocol version {0}")]
    UnsupportedVersion(u8),
    /// The input is empty.
    #[error("input is empty")]
    Truncated,
    /// The input is larger than the allowed maximum.
    #[error("input of {actual} bytes exceeds the limit of {limit} bytes")]
    TooLarge {
        /// The maximum allowed size in bytes.
        limit: usize,
        /// The size that was presented.
        actual: usize,
    },
    /// The input could not be parsed.
    #[error("malformed message: {0}")]
    Malformed(&'static str),
    /// The input parsed but is not the canonical encoding of what it contains.
    #[error("message is not in canonical form")]
    NonCanonical,
    /// A field has the wrong length.
    #[error("field {field} has length {actual}, expected {expected}")]
    BadLength {
        /// The name of the field.
        field: &'static str,
        /// The required length in bytes.
        expected: usize,
        /// The length that was presented.
        actual: usize,
    },
    /// A field reserved for a later protocol version carries data.
    #[error("a field reserved for a later version is set")]
    ReservedFieldSet,
}

/// Stable numeric error codes that cross the FFI boundary. Values never change once released.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ErrorCode {
    /// See [`ProtoError::UnsupportedVersion`].
    UnsupportedVersion = 1000,
    /// See [`ProtoError::Truncated`].
    Truncated = 1001,
    /// See [`ProtoError::TooLarge`].
    TooLarge = 1002,
    /// See [`ProtoError::Malformed`].
    Malformed = 1003,
    /// See [`ProtoError::NonCanonical`].
    NonCanonical = 1004,
    /// See [`ProtoError::BadLength`].
    BadLength = 1005,
    /// See [`ProtoError::ReservedFieldSet`].
    ReservedFieldSet = 1006,
    /// The operating system random number generator failed.
    RandomFailure = 2000,
    /// A key has the wrong length or is not a valid key.
    InvalidKey = 2001,
    /// A signature did not verify.
    InvalidSignature = 2002,
    /// A key agreement produced a non-contributory (all-zero) shared secret.
    WeakSharedSecret = 2003,
    /// Authenticated decryption failed. No further detail is ever given.
    DecryptionFailed = 2004,
    /// An input has an invalid length.
    InvalidLength = 2005,
    /// More key material was requested than the KDF can produce.
    KdfOutputTooLong = 2006,
    /// A KEM ciphertext is malformed.
    InvalidCiphertext = 2007,
    /// A stream ended without its final chunk.
    StreamTruncated = 2010,
    /// A stream chunk is out of order, repeated, or follows the final chunk.
    StreamChunkOrder = 2011,
    /// A stream chunk has an invalid size.
    StreamBadChunk = 2012,
}

impl ErrorCode {
    /// Every code defined by this crate, for exhaustive tests.
    pub const ALL: [Self; 18] = [
        Self::UnsupportedVersion,
        Self::Truncated,
        Self::TooLarge,
        Self::Malformed,
        Self::NonCanonical,
        Self::BadLength,
        Self::ReservedFieldSet,
        Self::RandomFailure,
        Self::InvalidKey,
        Self::InvalidSignature,
        Self::WeakSharedSecret,
        Self::DecryptionFailed,
        Self::InvalidLength,
        Self::KdfOutputTooLong,
        Self::InvalidCiphertext,
        Self::StreamTruncated,
        Self::StreamChunkOrder,
        Self::StreamBadChunk,
    ];

    /// The numeric value sent across the FFI boundary.
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    /// Parse a numeric code; unknown values yield `None`.
    #[must_use]
    pub const fn from_u16(value: u16) -> Option<Self> {
        match value {
            1000 => Some(Self::UnsupportedVersion),
            1001 => Some(Self::Truncated),
            1002 => Some(Self::TooLarge),
            1003 => Some(Self::Malformed),
            1004 => Some(Self::NonCanonical),
            1005 => Some(Self::BadLength),
            1006 => Some(Self::ReservedFieldSet),
            2000 => Some(Self::RandomFailure),
            2001 => Some(Self::InvalidKey),
            2002 => Some(Self::InvalidSignature),
            2003 => Some(Self::WeakSharedSecret),
            2004 => Some(Self::DecryptionFailed),
            2005 => Some(Self::InvalidLength),
            2006 => Some(Self::KdfOutputTooLong),
            2007 => Some(Self::InvalidCiphertext),
            2010 => Some(Self::StreamTruncated),
            2011 => Some(Self::StreamChunkOrder),
            2012 => Some(Self::StreamBadChunk),
            _ => None,
        }
    }
}

impl ProtoError {
    /// The stable code for this error.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::UnsupportedVersion(_) => ErrorCode::UnsupportedVersion,
            Self::Truncated => ErrorCode::Truncated,
            Self::TooLarge { .. } => ErrorCode::TooLarge,
            Self::Malformed(_) => ErrorCode::Malformed,
            Self::NonCanonical => ErrorCode::NonCanonical,
            Self::BadLength { .. } => ErrorCode::BadLength,
            Self::ReservedFieldSet => ErrorCode::ReservedFieldSet,
        }
    }
}

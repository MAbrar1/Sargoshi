//! Errors of the primitive layer. Messages are deliberately generic: a failed decryption never
//! says which check failed.

use sg_common::ErrorCode;
use thiserror::Error;

/// Errors raised by the primitive wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CryptoError {
    /// The operating system random number generator failed.
    #[error("the system random number generator failed")]
    RandomFailure,
    /// A key has the wrong length or is not a valid key.
    #[error("invalid key")]
    InvalidKey,
    /// A signature did not verify.
    #[error("invalid signature")]
    InvalidSignature,
    /// A key agreement produced an all-zero shared secret.
    #[error("key agreement produced a weak shared secret")]
    WeakSharedSecret,
    /// Authenticated decryption failed.
    #[error("decryption failed")]
    DecryptionFailed,
    /// An input has an invalid length.
    #[error("invalid input length")]
    InvalidLength,
    /// More key material was requested than HKDF can produce.
    #[error("requested key material is too long")]
    KdfOutputTooLong,
    /// A KEM ciphertext is malformed.
    #[error("invalid ciphertext")]
    InvalidCiphertext,
    /// A header carries a protocol version this build does not speak.
    #[error("unsupported protocol version")]
    UnsupportedVersion,
    /// A stream ended without its final chunk.
    #[error("stream is truncated")]
    StreamTruncated,
    /// A stream chunk is out of order, repeated, or follows the final chunk.
    #[error("stream chunk out of order")]
    StreamChunkOrder,
    /// A stream chunk has an invalid size.
    #[error("invalid stream chunk size")]
    StreamBadChunk,
}

impl CryptoError {
    /// The stable code for this error.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::RandomFailure => ErrorCode::RandomFailure,
            Self::InvalidKey => ErrorCode::InvalidKey,
            Self::InvalidSignature => ErrorCode::InvalidSignature,
            Self::WeakSharedSecret => ErrorCode::WeakSharedSecret,
            Self::DecryptionFailed => ErrorCode::DecryptionFailed,
            Self::InvalidLength => ErrorCode::InvalidLength,
            Self::KdfOutputTooLong => ErrorCode::KdfOutputTooLong,
            Self::InvalidCiphertext => ErrorCode::InvalidCiphertext,
            Self::UnsupportedVersion => ErrorCode::UnsupportedVersion,
            Self::StreamTruncated => ErrorCode::StreamTruncated,
            Self::StreamChunkOrder => ErrorCode::StreamChunkOrder,
            Self::StreamBadChunk => ErrorCode::StreamBadChunk,
        }
    }
}

//! Errors of the Double Ratchet. Messages never contain key material, and a failed decryption
//! never says which check failed.

use sg_common::{ErrorCode, ProtoError};
use sg_crypto::CryptoError;
use sg_crypto::codec::CodecError;
use thiserror::Error;

/// Errors raised by session operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RatchetError {
    /// This side has not received a message yet and cannot send.
    #[error("no message has been received yet, so this side cannot send")]
    NoSendChain,
    /// The message was already received.
    #[error("message was already received")]
    DuplicateMessage,
    /// Accepting the message would skip too many message keys.
    #[error("too many skipped messages")]
    TooManySkippedKeys,
    /// The message header is impossible for this session.
    #[error("invalid message header")]
    InvalidHeader,
    /// A message counter would overflow.
    #[error("message counter overflow")]
    CounterOverflow,
    /// A stored session blob is malformed.
    #[error("malformed session state")]
    MalformedState,
    /// A primitive failed, including failed authentication (`DecryptionFailed`).
    #[error(transparent)]
    Crypto(#[from] CryptoError),
    /// The envelope could not be parsed.
    #[error(transparent)]
    Proto(#[from] ProtoError),
}

impl RatchetError {
    /// The stable code for this error.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::NoSendChain => ErrorCode::NoSendChain,
            Self::DuplicateMessage => ErrorCode::DuplicateMessage,
            Self::TooManySkippedKeys => ErrorCode::TooManySkippedKeys,
            Self::InvalidHeader => ErrorCode::InvalidHeader,
            Self::CounterOverflow => ErrorCode::CounterOverflow,
            Self::MalformedState => ErrorCode::MalformedState,
            Self::Crypto(e) => e.code(),
            Self::Proto(e) => e.code(),
        }
    }
}

impl From<CodecError> for RatchetError {
    fn from(_: CodecError) -> Self {
        Self::MalformedState
    }
}

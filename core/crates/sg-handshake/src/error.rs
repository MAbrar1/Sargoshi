//! Errors of the identity, prekey and handshake layer. Messages never contain key material.

use sg_common::{ErrorCode, ProtoError};
use sg_crypto::CryptoError;
use sg_crypto::codec::CodecError;
use thiserror::Error;

/// Errors raised by identity, prekey and handshake operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HandshakeError {
    /// An identity key is malformed or not an acceptable key.
    #[error("invalid identity key")]
    InvalidIdentity,
    /// The signature over the DH identity key does not verify.
    #[error("invalid identity signature")]
    InvalidIdentitySignature,
    /// The signature over a signed prekey does not verify.
    #[error("invalid signed prekey signature")]
    InvalidSignedPreKeySignature,
    /// The signature over a post-quantum prekey does not verify.
    #[error("invalid post-quantum prekey signature")]
    InvalidPqPreKeySignature,
    /// The initial message names a signed prekey this device does not hold.
    #[error("unknown signed prekey")]
    UnknownSignedPreKey,
    /// The initial message names a post-quantum prekey this device does not hold.
    #[error("unknown post-quantum prekey")]
    UnknownPqPreKey,
    /// The initial message names a one-time prekey that is unknown or already used.
    #[error("unknown or already used one-time prekey")]
    UnknownOneTimePreKey,
    /// The initial message has already been accepted.
    #[error("initial message was already accepted")]
    ReplayedInitialMessage,
    /// A stored key blob is malformed.
    #[error("malformed key blob")]
    MalformedBlob,
    /// A key collection is at its size limit.
    #[error("too many keys")]
    TooManyKeys,
    /// A key identifier is already in use.
    #[error("key identifier already in use")]
    DuplicateKeyId,
    /// A primitive failed.
    #[error(transparent)]
    Crypto(#[from] CryptoError),
    /// A wire message could not be parsed.
    #[error(transparent)]
    Proto(#[from] ProtoError),
}

impl HandshakeError {
    /// The stable code for this error.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidIdentity => ErrorCode::InvalidIdentity,
            Self::InvalidIdentitySignature => ErrorCode::InvalidIdentitySignature,
            Self::InvalidSignedPreKeySignature => ErrorCode::InvalidSignedPreKeySignature,
            Self::InvalidPqPreKeySignature => ErrorCode::InvalidPqPreKeySignature,
            Self::UnknownSignedPreKey => ErrorCode::UnknownSignedPreKey,
            Self::UnknownPqPreKey => ErrorCode::UnknownPqPreKey,
            Self::UnknownOneTimePreKey => ErrorCode::UnknownOneTimePreKey,
            Self::ReplayedInitialMessage => ErrorCode::ReplayedInitialMessage,
            Self::MalformedBlob => ErrorCode::MalformedBlob,
            Self::TooManyKeys => ErrorCode::TooManyKeys,
            Self::DuplicateKeyId => ErrorCode::DuplicateKeyId,
            Self::Crypto(e) => e.code(),
            Self::Proto(e) => e.code(),
        }
    }
}

impl From<CodecError> for HandshakeError {
    fn from(_: CodecError) -> Self {
        Self::MalformedBlob
    }
}

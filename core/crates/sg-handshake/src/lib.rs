//! Account identity, prekeys, key bundles and the post-quantum hybrid handshake.
//!
//! Everything here is pure computation on values and byte blobs: this crate never touches the
//! disk, the network or a clock (timestamps are supplied by the caller). The Double Ratchet that
//! consumes the handshake output lives in `sg-ratchet`.
#![forbid(unsafe_code)]

mod codec;
pub mod error;
pub mod handshake;
pub mod identity;
pub mod prekeys;
mod wire;

pub use error::HandshakeError;
#[cfg(feature = "deterministic-kat")]
pub use handshake::initiate_deterministic;
pub use handshake::{InitialHeader, InitiatorHandshake, ResponderHandshake, accept, initiate};
pub use identity::{IDENTITY_SECRET_BLOB_LEN, IdentityKeyPair, IdentityPublic};
pub use prekeys::{
    OneTimePreKeyPublic, PqPreKeyPublic, PreKeyBundle, PrekeySecrets, SignedPreKeyPublic,
    UploadBundle,
};

//! The Double Ratchet: per-message forward secrecy and post-compromise security on top of the
//! handshake in `sg-handshake`. Like the rest of the core it is pure computation: sessions are
//! values that serialize to and from blobs, and nothing touches disk, network or a clock.
#![forbid(unsafe_code)]

pub mod error;
pub mod hook;
mod kdf;
pub mod session;

pub use error::RatchetError;
pub use hook::{NoPqRatchet, PqRatchetHook, RatchetDirection, RatchetStep};
pub use session::{MAX_ASSOCIATED_DATA_BYTES, Session};

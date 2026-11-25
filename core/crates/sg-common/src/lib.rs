//! Shared protocol definitions for Sargoshi: constants, version handling, stable error codes
//! and the canonical wire formats. This crate contains no cryptography; it only defines the
//! values and formats that `sg-crypto`, `sg-handshake`, `sg-ratchet` and the gateway agree on.
#![forbid(unsafe_code)]

pub mod constants;
pub mod error;
pub mod version;
pub mod wire;

pub use error::{ErrorCode, ProtoError};
pub use version::{PROTOCOL_VERSION, check_version};
pub use wire::{Envelope, HEADER_LEN, RatchetHeader};

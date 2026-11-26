//! Wrappers around maintained cryptographic crates. This crate implements no cipher, hash, KEM
//! or constant-time trick of its own (Prompt B hard rule 2). It adds three things: typed keys
//! that zeroize on drop and never print their contents, strict input validation, and the
//! streaming AEAD construction used by the vault and backup engines.
#![forbid(unsafe_code)]

pub mod aead;
pub mod ed25519;
pub mod error;
pub mod kdf;
pub mod kem;
pub mod rng;
pub mod secret;
pub mod stream;
pub mod x25519;

pub use error::CryptoError;
pub use secret::SecretBytes;

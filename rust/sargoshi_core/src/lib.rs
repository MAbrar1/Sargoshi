//! Sargoshi core: recovery file and message padding.
//!
//! This crate composes audited RustCrypto primitives (Argon2id, XChaCha20-Poly1305).
//! It implements no cryptographic primitive itself. The Signal protocol and MLS
//! parts are added in later stages and are not present here.

#![forbid(unsafe_code)]

pub mod padding;
pub mod recovery;

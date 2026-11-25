//! Protocol version handling. Every wire message starts with a version byte and unknown
//! versions are rejected (Prompt B, hard rule 4).

use crate::error::ProtoError;

/// The only protocol version this build speaks.
pub const PROTOCOL_VERSION: u8 = 1;

/// Accept only [`PROTOCOL_VERSION`]. Version 0 and every other value are rejected.
///
/// # Errors
/// Returns [`ProtoError::UnsupportedVersion`] for any other value.
pub fn check_version(version: u8) -> Result<(), ProtoError> {
    if version == PROTOCOL_VERSION {
        Ok(())
    } else {
        Err(ProtoError::UnsupportedVersion(version))
    }
}

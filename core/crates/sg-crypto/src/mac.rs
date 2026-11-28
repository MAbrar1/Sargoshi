//! HMAC-SHA256 (RFC 2104 / RFC 4231), used for the Double Ratchet chain key derivation.

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::error::CryptoError;
use crate::secret::SecretBytes;

/// Compute HMAC-SHA256 of `data` under `key` and return the 32-byte tag as secret material.
///
/// # Errors
/// Returns [`CryptoError::InvalidKey`] only if the MAC implementation rejects the key length
/// (HMAC accepts every length, so this does not happen in practice).
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<SecretBytes<32>, CryptoError> {
    let mut mac =
        <Hmac<Sha256> as KeyInit>::new_from_slice(key).map_err(|_| CryptoError::InvalidKey)?;
    mac.update(data);
    let tag = mac.finalize().into_bytes();
    <[u8; 32]>::try_from(tag.as_slice())
        .map(SecretBytes::new)
        .map_err(|_| CryptoError::InvalidLength)
}

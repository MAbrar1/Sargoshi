//! HKDF-SHA256 (RFC 5869).

use hkdf::Hkdf;
use sg_common::constants::HKDF_SHA256_MAX_OUTPUT;
use sha2::Sha256;
use zeroize::Zeroize;

use crate::error::CryptoError;
use crate::secret::SecretBytes;

/// Derive `out.len()` bytes with HKDF-SHA256. An empty `salt` is equivalent to a salt of 32
/// zero bytes (RFC 5869, section 2.2).
///
/// # Errors
/// Returns [`CryptoError::KdfOutputTooLong`] if `out` is longer than 8160 bytes.
pub fn hkdf_sha256(
    salt: &[u8],
    ikm: &[u8],
    info: &[u8],
    out: &mut [u8],
) -> Result<(), CryptoError> {
    if out.len() > HKDF_SHA256_MAX_OUTPUT {
        return Err(CryptoError::KdfOutputTooLong);
    }
    let salt = if salt.is_empty() { None } else { Some(salt) };
    let hkdf = Hkdf::<Sha256>::new(salt, ikm);
    hkdf.expand(info, out)
        .map_err(|_| CryptoError::KdfOutputTooLong)
}

/// Derive exactly `N` secret bytes with HKDF-SHA256.
///
/// # Errors
/// Returns [`CryptoError::KdfOutputTooLong`] if `N` is larger than 8160.
pub fn hkdf_sha256_secret<const N: usize>(
    salt: &[u8],
    ikm: &[u8],
    info: &[u8],
) -> Result<SecretBytes<N>, CryptoError> {
    let mut tmp = [0u8; N];
    let result = hkdf_sha256(salt, ikm, info, &mut tmp);
    let secret = SecretBytes::new(tmp);
    tmp.zeroize();
    result.map(|()| secret)
}

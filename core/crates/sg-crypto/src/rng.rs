//! Randomness from the operating system only (`getrandom`). There is no seedable generator in
//! this crate; deterministic behaviour exists only behind the `deterministic-kat` feature.

use zeroize::Zeroize;

use crate::error::CryptoError;
use crate::secret::SecretBytes;

/// Fill a buffer with cryptographically secure random bytes.
///
/// # Errors
/// Returns [`CryptoError::RandomFailure`] if the operating system cannot provide randomness.
pub fn fill_random(buf: &mut [u8]) -> Result<(), CryptoError> {
    getrandom::fill(buf).map_err(|_| CryptoError::RandomFailure)
}

/// `N` random bytes as a non-secret array (nonces, identifiers, salts).
///
/// # Errors
/// Returns [`CryptoError::RandomFailure`] on failure.
pub fn random_array<const N: usize>() -> Result<[u8; N], CryptoError> {
    let mut out = [0u8; N];
    fill_random(&mut out)?;
    Ok(out)
}

/// `N` random bytes as secret material.
///
/// # Errors
/// Returns [`CryptoError::RandomFailure`] on failure.
pub fn random_secret<const N: usize>() -> Result<SecretBytes<N>, CryptoError> {
    let mut tmp = [0u8; N];
    let result = fill_random(&mut tmp);
    let secret = SecretBytes::new(tmp);
    tmp.zeroize();
    result.map(|()| secret)
}

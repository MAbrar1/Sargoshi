//! X25519 Diffie-Hellman (RFC 7748) with rejection of all-zero shared secrets.

use core::fmt;

use sg_common::constants::X25519_KEY_LEN;
use subtle::ConstantTimeEq;
use x25519_dalek::{X25519_BASEPOINT_BYTES, x25519};
use zeroize::Zeroize;

use crate::error::CryptoError;
use crate::rng::random_secret;
use crate::secret::SecretBytes;

/// An X25519 public key (32 bytes). Any 32 bytes are a syntactically valid public key; weak
/// points are caught when the shared secret is computed.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct X25519PublicKey([u8; X25519_KEY_LEN]);

impl X25519PublicKey {
    /// Wrap 32 public key bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; X25519_KEY_LEN]) -> Self {
        Self(bytes)
    }

    /// Parse from a slice that must be exactly 32 bytes.
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidKey`] on a wrong length.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, CryptoError> {
        <[u8; X25519_KEY_LEN]>::try_from(bytes)
            .map(Self)
            .map_err(|_| CryptoError::InvalidKey)
    }

    /// The 32 public key bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; X25519_KEY_LEN] {
        &self.0
    }
}

impl fmt::Debug for X25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "X25519PublicKey({} bytes)", X25519_KEY_LEN)
    }
}

/// An X25519 secret key. Zeroized on drop; `Debug` is redacted.
#[derive(Debug, PartialEq, Eq)]
pub struct X25519SecretKey(SecretBytes<X25519_KEY_LEN>);

impl X25519SecretKey {
    /// Generate a fresh secret key from the operating system RNG.
    ///
    /// # Errors
    /// Returns [`CryptoError::RandomFailure`] if randomness is unavailable.
    pub fn generate() -> Result<Self, CryptoError> {
        random_secret().map(Self)
    }

    /// Restore a secret key from its 32 stored bytes (clamping happens during use, per RFC 7748).
    #[must_use]
    pub const fn from_bytes(bytes: [u8; X25519_KEY_LEN]) -> Self {
        Self(SecretBytes::new(bytes))
    }

    /// The 32 secret bytes, for encrypted storage only.
    #[must_use]
    pub const fn expose_bytes(&self) -> &[u8; X25519_KEY_LEN] {
        self.0.expose()
    }

    /// The matching public key.
    #[must_use]
    pub fn public_key(&self) -> X25519PublicKey {
        let mut k = *self.0.expose();
        let public = x25519(k, X25519_BASEPOINT_BYTES);
        k.zeroize();
        X25519PublicKey(public)
    }

    /// Compute the shared secret with a peer.
    ///
    /// # Errors
    /// Returns [`CryptoError::WeakSharedSecret`] when the result is all zero, which happens only
    /// for low-order peer keys (an attack, never an honest peer).
    pub fn diffie_hellman(
        &self,
        peer: &X25519PublicKey,
    ) -> Result<SecretBytes<X25519_KEY_LEN>, CryptoError> {
        let mut k = *self.0.expose();
        let shared = SecretBytes::new(x25519(k, peer.0));
        k.zeroize();
        let all_zero: bool = shared.expose().ct_eq(&[0u8; X25519_KEY_LEN]).into();
        if all_zero {
            Err(CryptoError::WeakSharedSecret)
        } else {
            Ok(shared)
        }
    }
}

//! Ed25519 signatures (RFC 8032) with strict verification, plus the labelled signing used for
//! key certification (`label || data`, Prompt B section 19.2).

use core::fmt;

use curve25519_dalek::edwards::CompressedEdwardsY;
use ed25519_dalek::Signer;
use sg_common::constants::{ED25519_PUBLIC_LEN, ED25519_SIGNATURE_LEN};

use crate::error::CryptoError;
use crate::rng::random_secret;
use crate::secret::SecretBytes;

/// An Ed25519 signature (64 bytes).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signature([u8; ED25519_SIGNATURE_LEN]);

impl Signature {
    /// Wrap 64 signature bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; ED25519_SIGNATURE_LEN]) -> Self {
        Self(bytes)
    }

    /// Parse from a slice that must be exactly 64 bytes.
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidSignature`] on a wrong length.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, CryptoError> {
        <[u8; ED25519_SIGNATURE_LEN]>::try_from(bytes)
            .map(Self)
            .map_err(|_| CryptoError::InvalidSignature)
    }

    /// The 64 signature bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; ED25519_SIGNATURE_LEN] {
        &self.0
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({ED25519_SIGNATURE_LEN} bytes)")
    }
}

/// An Ed25519 public key that has been checked to be a valid curve point encoding.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VerifyingKey(ed25519_dalek::VerifyingKey);

impl VerifyingKey {
    /// Parse a public key. Rejects invalid encodings and every key outside the prime-order
    /// subgroup (small-order and mixed-order keys). Honestly generated keys are always in the
    /// subgroup, so this never rejects an honest peer.
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidKey`] if the bytes are not an acceptable public key.
    pub fn from_bytes(bytes: &[u8; ED25519_PUBLIC_LEN]) -> Result<Self, CryptoError> {
        let key =
            ed25519_dalek::VerifyingKey::from_bytes(bytes).map_err(|_| CryptoError::InvalidKey)?;
        let point = CompressedEdwardsY(*bytes)
            .decompress()
            .ok_or(CryptoError::InvalidKey)?;
        if point.is_small_order() || !point.is_torsion_free() {
            return Err(CryptoError::InvalidKey);
        }
        Ok(Self(key))
    }

    /// Parse from a slice that must be exactly 32 bytes.
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidKey`] on a wrong length or invalid encoding.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, CryptoError> {
        let array =
            <[u8; ED25519_PUBLIC_LEN]>::try_from(bytes).map_err(|_| CryptoError::InvalidKey)?;
        Self::from_bytes(&array)
    }

    /// The 32 public key bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; ED25519_PUBLIC_LEN] {
        self.0.to_bytes()
    }

    /// Verify a signature using strict verification (rejects small-order keys and
    /// non-canonical components).
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidSignature`] if the signature does not verify.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), CryptoError> {
        let sig = ed25519_dalek::Signature::from_bytes(&signature.0);
        self.0
            .verify_strict(message, &sig)
            .map_err(|_| CryptoError::InvalidSignature)
    }

    /// Verify a signature over `label || data`.
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidSignature`] if the signature does not verify.
    pub fn verify_labeled(
        &self,
        label: &[u8],
        data: &[u8],
        signature: &Signature,
    ) -> Result<(), CryptoError> {
        self.verify(&labeled_message(label, data), signature)
    }
}

impl fmt::Debug for VerifyingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VerifyingKey({ED25519_PUBLIC_LEN} bytes)")
    }
}

/// An Ed25519 signing key (32-byte seed). Zeroized on drop; `Debug` is redacted.
pub struct SigningKey(ed25519_dalek::SigningKey);

impl SigningKey {
    /// Generate a fresh signing key from the operating system RNG.
    ///
    /// # Errors
    /// Returns [`CryptoError::RandomFailure`] if randomness is unavailable.
    pub fn generate() -> Result<Self, CryptoError> {
        let seed = random_secret::<ED25519_PUBLIC_LEN>()?;
        Ok(Self::from_seed(&seed))
    }

    /// Restore a signing key from its 32-byte seed.
    #[must_use]
    pub fn from_seed(seed: &SecretBytes<ED25519_PUBLIC_LEN>) -> Self {
        Self(ed25519_dalek::SigningKey::from_bytes(seed.expose()))
    }

    /// The 32-byte seed, for encrypted storage only.
    #[must_use]
    pub fn seed(&self) -> SecretBytes<ED25519_PUBLIC_LEN> {
        SecretBytes::new(self.0.to_bytes())
    }

    /// The matching public key.
    #[must_use]
    pub fn verifying_key(&self) -> VerifyingKey {
        VerifyingKey(self.0.verifying_key())
    }

    /// Sign a message.
    #[must_use]
    pub fn sign(&self, message: &[u8]) -> Signature {
        Signature(self.0.sign(message).to_bytes())
    }

    /// Sign `label || data` (domain-separated certification of keys).
    #[must_use]
    pub fn sign_labeled(&self, label: &[u8], data: &[u8]) -> Signature {
        self.sign(&labeled_message(label, data))
    }
}

impl fmt::Debug for SigningKey {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        _f.write_str("SigningKey(redacted)")
    }
}

fn labeled_message(label: &[u8], data: &[u8]) -> Vec<u8> {
    let mut message = Vec::with_capacity(label.len() + data.len());
    message.extend_from_slice(label);
    message.extend_from_slice(data);
    message
}

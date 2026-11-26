//! ML-KEM-768 (FIPS 203) through the RustCrypto `ml-kem` crate.
//!
//! The secret key is stored as its 64-byte seed (FIPS 203 allows regenerating the full
//! decapsulation key from it), which keeps the stored secret small and canonical.
//! Decapsulation uses the implicit rejection built into ML-KEM: a malformed but correctly
//! sized ciphertext yields a pseudorandom shared secret, not an error.

use core::fmt;

use ml_kem::{Decapsulate, DecapsulationKey768, EncapsulationKey768, KeyExport, Seed};
use sg_common::constants::{
    MLKEM768_CIPHERTEXT_LEN, MLKEM768_ENCAPSULATION_KEY_LEN, MLKEM768_SHARED_SECRET_LEN,
};
use zeroize::Zeroize;

use crate::error::CryptoError;
use crate::rng::{fill_random, random_secret};
use crate::secret::SecretBytes;

/// Length of the secret key seed (`d || z`, FIPS 203).
pub const MLKEM768_SEED_LEN: usize = 64;

/// An ML-KEM-768 encapsulation (public) key, validated on construction.
#[derive(Clone, PartialEq, Eq)]
pub struct MlKem768PublicKey(Vec<u8>);

impl MlKem768PublicKey {
    /// Parse and validate an encapsulation key (length and FIPS 203 modulus check).
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidKey`] if the bytes are not a valid encapsulation key.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != MLKEM768_ENCAPSULATION_KEY_LEN {
            return Err(CryptoError::InvalidKey);
        }
        parse_encapsulation_key(bytes)?;
        Ok(Self(bytes.to_vec()))
    }

    /// The 1,184 encapsulation key bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Encapsulate a fresh shared secret to this key using operating system randomness.
    ///
    /// # Errors
    /// Returns [`CryptoError::RandomFailure`] if randomness is unavailable.
    pub fn encapsulate(&self) -> Result<(MlKem768Ciphertext, SecretBytes<32>), CryptoError> {
        let mut m = [0u8; 32];
        let drawn = fill_random(&mut m);
        let result = drawn.and_then(|()| self.encapsulate_with_message(&m));
        m.zeroize();
        result
    }

    /// Encapsulate with a caller-supplied 32-byte message. Reusing a message breaks security;
    /// this exists only so known-answer tests can reproduce NIST vectors.
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidKey`] if the stored key fails to parse (cannot happen for a
    /// key built with [`Self::from_bytes`]).
    #[cfg(feature = "deterministic-kat")]
    pub fn encapsulate_deterministic(
        &self,
        message: &[u8; 32],
    ) -> Result<(MlKem768Ciphertext, SecretBytes<32>), CryptoError> {
        self.encapsulate_with_message(message)
    }

    fn encapsulate_with_message(
        &self,
        message: &[u8; 32],
    ) -> Result<(MlKem768Ciphertext, SecretBytes<32>), CryptoError> {
        let ek = parse_encapsulation_key(&self.0)?;
        let (ct, ss) = ek.encapsulate_deterministic(&ml_kem::B32::from(*message));
        let ciphertext = MlKem768Ciphertext::from_slice(ct.as_slice())?;
        let secret = secret_from_slice(ss.as_slice())?;
        Ok((ciphertext, secret))
    }
}

impl fmt::Debug for MlKem768PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MlKem768PublicKey({MLKEM768_ENCAPSULATION_KEY_LEN} bytes)"
        )
    }
}

/// An ML-KEM-768 ciphertext (1,088 bytes).
#[derive(Clone, PartialEq, Eq)]
pub struct MlKem768Ciphertext(Vec<u8>);

impl MlKem768Ciphertext {
    /// Wrap ciphertext bytes, checking the length.
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidCiphertext`] on a wrong length.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() == MLKEM768_CIPHERTEXT_LEN {
            Ok(Self(bytes.to_vec()))
        } else {
            Err(CryptoError::InvalidCiphertext)
        }
    }

    /// The 1,088 ciphertext bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for MlKem768Ciphertext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MlKem768Ciphertext({MLKEM768_CIPHERTEXT_LEN} bytes)")
    }
}

/// An ML-KEM-768 secret key held as its seed. Zeroized on drop; `Debug` is redacted.
pub struct MlKem768SecretKey {
    seed: SecretBytes<MLKEM768_SEED_LEN>,
}

impl MlKem768SecretKey {
    /// Generate a fresh key pair from the operating system RNG.
    ///
    /// # Errors
    /// Returns [`CryptoError::RandomFailure`] if randomness is unavailable.
    pub fn generate() -> Result<(Self, MlKem768PublicKey), CryptoError> {
        let secret = Self {
            seed: random_secret()?,
        };
        let public = secret.public_key()?;
        Ok((secret, public))
    }

    /// Restore a key from its 64-byte seed.
    #[must_use]
    pub const fn from_seed(seed: SecretBytes<MLKEM768_SEED_LEN>) -> Self {
        Self { seed }
    }

    /// The 64-byte seed, for encrypted storage only.
    #[must_use]
    pub const fn expose_seed(&self) -> &[u8; MLKEM768_SEED_LEN] {
        self.seed.expose()
    }

    /// The matching encapsulation key.
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidKey`] only if the library produces an invalid key (it never
    /// should; this is checked rather than assumed).
    pub fn public_key(&self) -> Result<MlKem768PublicKey, CryptoError> {
        let dk = self.decapsulation_key();
        let bytes = dk.encapsulation_key().to_bytes();
        MlKem768PublicKey::from_bytes(bytes.as_slice())
    }

    /// Recover the shared secret from a ciphertext.
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidKey`] if the internal representation is invalid (cannot
    /// happen for keys built by this crate).
    pub fn decapsulate(
        &self,
        ciphertext: &MlKem768Ciphertext,
    ) -> Result<SecretBytes<MLKEM768_SHARED_SECRET_LEN>, CryptoError> {
        let dk = self.decapsulation_key();
        let ss = dk
            .decapsulate_slice(ciphertext.as_bytes())
            .map_err(|_| CryptoError::InvalidCiphertext)?;
        secret_from_slice(ss.as_slice())
    }

    fn decapsulation_key(&self) -> DecapsulationKey768 {
        DecapsulationKey768::from_seed(Seed::from(*self.seed.expose()))
    }
}

impl fmt::Debug for MlKem768SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MlKem768SecretKey(redacted)")
    }
}

fn parse_encapsulation_key(bytes: &[u8]) -> Result<EncapsulationKey768, CryptoError> {
    let key =
        ml_kem::Key::<EncapsulationKey768>::try_from(bytes).map_err(|_| CryptoError::InvalidKey)?;
    EncapsulationKey768::new(&key).map_err(|_| CryptoError::InvalidKey)
}

fn secret_from_slice(bytes: &[u8]) -> Result<SecretBytes<32>, CryptoError> {
    <[u8; 32]>::try_from(bytes)
        .map(SecretBytes::new)
        .map_err(|_| CryptoError::InvalidLength)
}

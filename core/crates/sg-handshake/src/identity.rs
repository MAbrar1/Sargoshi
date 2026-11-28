//! The long-term account identity: an Ed25519 signing key and an X25519 DH key, where the signing
//! key certifies the DH key (`"Sargoshi/IK-DH/v1" || dh_public`). The safety number is computed
//! from this identity, so it does not change when a device is added.

use core::fmt;

use sg_common::constants::{ED25519_PUBLIC_LEN, LABEL_IK_DH_SIG, X25519_KEY_LEN};
use sg_common::{PROTOCOL_VERSION, check_version};
use sg_crypto::SecretBytes;
use sg_crypto::ed25519::{Signature, SigningKey, VerifyingKey};
use sg_crypto::x25519::{X25519PublicKey, X25519SecretKey};
use zeroize::Zeroizing;

use crate::error::HandshakeError;
use sg_crypto::codec::{Reader, Writer};

/// Length of the serialized identity secret blob: `version || signing_seed || dh_secret`.
pub const IDENTITY_SECRET_BLOB_LEN: usize = 1 + ED25519_PUBLIC_LEN + X25519_KEY_LEN;

/// The public half of an identity. Every value of this type has been verified: the DH key's
/// signature checked out under the signing key, and both keys are acceptable encodings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityPublic {
    signing: VerifyingKey,
    dh: X25519PublicKey,
    dh_signature: Signature,
}

impl IdentityPublic {
    /// Parse and verify an identity from raw bytes.
    ///
    /// # Errors
    /// [`HandshakeError::InvalidIdentity`] for an unacceptable key and
    /// [`HandshakeError::InvalidIdentitySignature`] if the DH key's signature does not verify.
    pub fn new(signing: &[u8], dh: &[u8], dh_signature: &[u8]) -> Result<Self, HandshakeError> {
        let signing =
            VerifyingKey::from_slice(signing).map_err(|_| HandshakeError::InvalidIdentity)?;
        let dh = X25519PublicKey::from_slice(dh).map_err(|_| HandshakeError::InvalidIdentity)?;
        let dh_signature = Signature::from_slice(dh_signature)
            .map_err(|_| HandshakeError::InvalidIdentitySignature)?;
        signing
            .verify_labeled(LABEL_IK_DH_SIG, dh.as_bytes(), &dh_signature)
            .map_err(|_| HandshakeError::InvalidIdentitySignature)?;
        Ok(Self {
            signing,
            dh,
            dh_signature,
        })
    }

    /// The Ed25519 verifying key.
    #[must_use]
    pub const fn signing_key(&self) -> &VerifyingKey {
        &self.signing
    }

    /// The X25519 identity key used in the handshake.
    #[must_use]
    pub const fn dh_key(&self) -> &X25519PublicKey {
        &self.dh
    }

    /// The signature that certifies the DH key.
    #[must_use]
    pub const fn dh_signature(&self) -> &Signature {
        &self.dh_signature
    }
}

/// An identity with its secret keys. Zeroized on drop; `Debug` is redacted.
pub struct IdentityKeyPair {
    signing: SigningKey,
    dh: X25519SecretKey,
    public: IdentityPublic,
}

impl IdentityKeyPair {
    /// Create a fresh identity from the operating system RNG.
    ///
    /// # Errors
    /// Fails only if randomness is unavailable.
    pub fn generate() -> Result<Self, HandshakeError> {
        let signing = SigningKey::generate()?;
        let dh = X25519SecretKey::generate()?;
        Ok(Self::assemble(signing, dh))
    }

    fn assemble(signing: SigningKey, dh: X25519SecretKey) -> Self {
        let dh_public = dh.public_key();
        let dh_signature = signing.sign_labeled(LABEL_IK_DH_SIG, dh_public.as_bytes());
        let public = IdentityPublic {
            signing: signing.verifying_key(),
            dh: dh_public,
            dh_signature,
        };
        Self {
            signing,
            dh,
            public,
        }
    }

    /// The public identity.
    #[must_use]
    pub const fn public(&self) -> &IdentityPublic {
        &self.public
    }

    pub(crate) const fn signing(&self) -> &SigningKey {
        &self.signing
    }

    pub(crate) const fn dh(&self) -> &X25519SecretKey {
        &self.dh
    }

    /// Serialize the secret keys: `version || signing_seed(32) || dh_secret(32)`. The blob is
    /// plaintext secret material; the app must store it only inside its encrypted database.
    #[must_use]
    pub fn to_secret_bytes(&self) -> Zeroizing<Vec<u8>> {
        let mut w = Writer::with_capacity(IDENTITY_SECRET_BLOB_LEN);
        w.u8(PROTOCOL_VERSION);
        w.put(self.signing.seed().expose());
        w.put(self.dh.expose_bytes());
        w.finish()
    }

    /// Restore an identity from [`Self::to_secret_bytes`] output.
    ///
    /// # Errors
    /// [`HandshakeError::MalformedBlob`] for a wrong length and a protocol error for an unknown
    /// version.
    pub fn from_secret_bytes(bytes: &[u8]) -> Result<Self, HandshakeError> {
        if bytes.len() != IDENTITY_SECRET_BLOB_LEN {
            return Err(HandshakeError::MalformedBlob);
        }
        let mut r = Reader::new(bytes);
        check_version(r.u8()?)?;
        let seed: SecretBytes<ED25519_PUBLIC_LEN> = r.secret32()?;
        let dh = r.x25519_secret()?;
        r.finish()?;
        Ok(Self::assemble(SigningKey::from_seed(&seed), dh))
    }
}

impl fmt::Debug for IdentityKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("IdentityKeyPair(redacted)")
    }
}

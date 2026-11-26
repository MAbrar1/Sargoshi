//! ChaCha20-Poly1305 (RFC 8439). The caller supplies the nonce: in Sargoshi every key is used
//! exactly once or has a strictly increasing counter nonce (see `stream`).

use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};
use sg_common::constants::{AEAD_KEY_LEN, AEAD_NONCE_LEN, AEAD_TAG_LEN};
use zeroize::Zeroizing;

use crate::error::CryptoError;
use crate::secret::SecretBytes;

/// Encrypt and authenticate. The output is `ciphertext || 16-byte tag`.
///
/// # Errors
/// Returns [`CryptoError::InvalidLength`] if the plaintext is too long for the cipher.
pub fn seal(
    key: &SecretBytes<AEAD_KEY_LEN>,
    nonce: &[u8; AEAD_NONCE_LEN],
    aad: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = ChaCha20Poly1305::new(&Key::from(*key.expose()));
    cipher
        .encrypt(
            &Nonce::from(*nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| CryptoError::InvalidLength)
}

/// Verify and decrypt `ciphertext || tag`. The plaintext is zeroized when dropped.
///
/// # Errors
/// Returns [`CryptoError::DecryptionFailed`] for any failure, without saying which check failed.
pub fn open(
    key: &SecretBytes<AEAD_KEY_LEN>,
    nonce: &[u8; AEAD_NONCE_LEN],
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    if ciphertext.len() < AEAD_TAG_LEN {
        return Err(CryptoError::DecryptionFailed);
    }
    let cipher = ChaCha20Poly1305::new(&Key::from(*key.expose()));
    cipher
        .decrypt(
            &Nonce::from(*nonce),
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map(Zeroizing::new)
        .map_err(|_| CryptoError::DecryptionFailed)
}

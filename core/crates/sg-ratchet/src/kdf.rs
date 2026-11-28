//! The three Double Ratchet key derivations, exactly as in Prompt B section 19.2.

use sg_common::constants::{INFO_DR_MESSAGE, INFO_DR_ROOT};
use sg_crypto::SecretBytes;
use sg_crypto::kdf::hkdf_sha256_secret;
use sg_crypto::mac::hmac_sha256;
use zeroize::Zeroizing;

use crate::error::RatchetError;
use sg_crypto::CryptoError;

/// A fresh copy of a 32-byte secret (needed to work on a trial copy of the state).
pub(crate) fn copy_secret(secret: &SecretBytes<32>) -> SecretBytes<32> {
    SecretBytes::new(*secret.expose())
}

fn split64(bytes: &SecretBytes<64>) -> Result<(SecretBytes<32>, SecretBytes<32>), RatchetError> {
    let (first, rest) = bytes
        .expose()
        .split_first_chunk::<32>()
        .ok_or(RatchetError::Crypto(CryptoError::InvalidLength))?;
    let second = <[u8; 32]>::try_from(rest).map_err(|_| CryptoError::InvalidLength)?;
    Ok((SecretBytes::new(*first), SecretBytes::new(second)))
}

/// `KDF_RK`: `HKDF-SHA256(salt = root_key, ikm = dh_output [|| extra], info =
/// "Sargoshi/DR/v1/root")` expanded to 64 bytes, split into the new root key and a chain key.
/// `extra` is `None` unless a post-quantum hook supplies a secret.
pub(crate) fn kdf_rk(
    root_key: &SecretBytes<32>,
    dh_output: &SecretBytes<32>,
    extra: Option<&SecretBytes<32>>,
) -> Result<(SecretBytes<32>, SecretBytes<32>), RatchetError> {
    let mut ikm = Zeroizing::new(Vec::with_capacity(32 + extra.map_or(0, |_| 32)));
    ikm.extend_from_slice(dh_output.expose());
    if let Some(extra) = extra {
        ikm.extend_from_slice(extra.expose());
    }
    let out: SecretBytes<64> = hkdf_sha256_secret(root_key.expose(), &ikm, INFO_DR_ROOT)?;
    split64(&out)
}

/// `KDF_CK`: the message key seed is `HMAC-SHA256(chain_key, 0x01)` and the next chain key is
/// `HMAC-SHA256(chain_key, 0x02)`.
pub(crate) fn kdf_ck(
    chain_key: &SecretBytes<32>,
) -> Result<(SecretBytes<32>, SecretBytes<32>), RatchetError> {
    let seed = hmac_sha256(chain_key.expose(), &[0x01])?;
    let next = hmac_sha256(chain_key.expose(), &[0x02])?;
    Ok((seed, next))
}

/// Message key expansion: `HKDF-SHA256(seed, info = "Sargoshi/DR/v1/message")` to 44 bytes, a
/// 32-byte ChaCha20-Poly1305 key followed by a 12-byte nonce. Every key is used exactly once.
pub(crate) fn message_key(
    seed: &SecretBytes<32>,
) -> Result<(SecretBytes<32>, [u8; 12]), RatchetError> {
    let expanded: SecretBytes<44> = hkdf_sha256_secret(&[], seed.expose(), INFO_DR_MESSAGE)?;
    let (key, nonce) = expanded
        .expose()
        .split_first_chunk::<32>()
        .ok_or(RatchetError::Crypto(CryptoError::InvalidLength))?;
    let nonce = <[u8; 12]>::try_from(nonce).map_err(|_| CryptoError::InvalidLength)?;
    Ok((SecretBytes::new(*key), nonce))
}

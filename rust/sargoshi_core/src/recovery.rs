//! Recovery file: identity keys only, never message history.
//!
//! Format (all integers big-endian):
//!   magic "SGRF" | version u8 | m_kib u32 | t u32 | p u32 | salt[16] | nonce[24] | ciphertext+tag
//! The whole header is bound as AEAD associated data, so tampering with the
//! Argon2 parameters or salt makes decryption fail.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305,
    aead::{Aead, Payload},
};
use zeroize::{Zeroize, Zeroizing};

const MAGIC: &[u8; 4] = b"SGRF";
const VERSION: u8 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
const HEADER_LEN: usize = 4 + 1 + 4 + 4 + 4 + SALT_LEN + NONCE_LEN;

/// Spec floor: never below 19 MiB of Argon2 memory.
pub const MIN_M_KIB: u32 = 19 * 1024;
/// Upper bounds stop a hostile file from forcing huge work on restore.
pub const MAX_M_KIB: u32 = 1024 * 1024;
pub const MAX_T: u32 = 10;
pub const MAX_P: u32 = 8;
pub const MIN_PASSPHRASE_CHARS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
}

impl KdfParams {
    /// Conservative default. The app calibrates per device and stores the result.
    pub const DEFAULT: KdfParams = KdfParams { m_kib: MIN_M_KIB, t: 3, p: 1 };

    fn valid(&self) -> bool {
        (MIN_M_KIB..=MAX_M_KIB).contains(&self.m_kib)
            && (1..=MAX_T).contains(&self.t)
            && (1..=MAX_P).contains(&self.p)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecoveryError {
    WeakPassphrase,
    BadParams,
    BadFormat,
    /// Wrong passphrase or corrupted or tampered file. Deliberately not distinguished.
    Decrypt,
    Random,
    Kdf,
}

fn derive_key(pass: &str, salt: &[u8], p: KdfParams) -> Result<Zeroizing<[u8; 32]>, RecoveryError> {
    let params = Params::new(p.m_kib, p.t, p.p, Some(32)).map_err(|_| RecoveryError::BadParams)?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; 32]);
    argon
        .hash_password_into(pass.as_bytes(), salt, key.as_mut())
        .map_err(|_| RecoveryError::Kdf)?;
    Ok(key)
}

pub fn create(identity_keys: &[u8], passphrase: &str, params: KdfParams) -> Result<Vec<u8>, RecoveryError> {
    if passphrase.chars().count() < MIN_PASSPHRASE_CHARS {
        return Err(RecoveryError::WeakPassphrase);
    }
    if !params.valid() {
        return Err(RecoveryError::BadParams);
    }
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    getrandom::fill(&mut salt).map_err(|_| RecoveryError::Random)?;
    getrandom::fill(&mut nonce).map_err(|_| RecoveryError::Random)?;

    let mut header = Vec::with_capacity(HEADER_LEN);
    header.extend_from_slice(MAGIC);
    header.push(VERSION);
    header.extend_from_slice(&params.m_kib.to_be_bytes());
    header.extend_from_slice(&params.t.to_be_bytes());
    header.extend_from_slice(&params.p.to_be_bytes());
    header.extend_from_slice(&salt);
    header.extend_from_slice(&nonce);

    let key = derive_key(passphrase, &salt, params)?;
    let cipher = XChaCha20Poly1305::new((&*key).into());
    let ct = cipher
        .encrypt(&nonce.into(), Payload { msg: identity_keys, aad: &header })
        .map_err(|_| RecoveryError::Decrypt)?;
    salt.zeroize();

    header.extend_from_slice(&ct);
    Ok(header)
}

pub fn restore(file: &[u8], passphrase: &str) -> Result<Zeroizing<Vec<u8>>, RecoveryError> {
    if file.len() < HEADER_LEN + 16 || &file[..4] != MAGIC || file[4] != VERSION {
        return Err(RecoveryError::BadFormat);
    }
    let u32_at = |i: usize| u32::from_be_bytes([file[i], file[i + 1], file[i + 2], file[i + 3]]);
    let params = KdfParams { m_kib: u32_at(5), t: u32_at(9), p: u32_at(13) };
    if !params.valid() {
        return Err(RecoveryError::BadParams);
    }
    let salt = &file[17..17 + SALT_LEN];
    let nonce: [u8; NONCE_LEN] = file[17 + SALT_LEN..HEADER_LEN]
        .try_into()
        .map_err(|_| RecoveryError::BadFormat)?;
    let (header, ct) = file.split_at(HEADER_LEN);

    let key = derive_key(passphrase, salt, params)?;
    let cipher = XChaCha20Poly1305::new((&*key).into());
    let plain = cipher
        .decrypt(&nonce.into(), Payload { msg: ct, aad: header })
        .map_err(|_| RecoveryError::Decrypt)?;
    Ok(Zeroizing::new(plain))
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEYS: &[u8] = b"identity-key-bytes-for-test-only";

    #[test]
    fn roundtrip() {
        let f = create(KEYS, "correct horse battery", KdfParams::DEFAULT).unwrap();
        assert_eq!(&restore(&f, "correct horse battery").unwrap()[..], KEYS);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let f = create(KEYS, "correct horse battery", KdfParams::DEFAULT).unwrap();
        assert_eq!(restore(&f, "wrong horse battery").unwrap_err(), RecoveryError::Decrypt);
    }

    #[test]
    fn tampering_any_byte_fails() {
        let f = create(KEYS, "correct horse battery", KdfParams::DEFAULT).unwrap();
        // Skip the params bytes: changing them is caught earlier as BadParams or Decrypt.
        for i in [0usize, 4, 17, 30, 45, f.len() - 1] {
            let mut g = f.clone();
            g[i] ^= 1;
            assert!(restore(&g, "correct horse battery").is_err(), "byte {i}");
        }
    }

    #[test]
    fn rejects_weak_inputs() {
        assert_eq!(create(KEYS, "short", KdfParams::DEFAULT).unwrap_err(), RecoveryError::WeakPassphrase);
        let weak = KdfParams { m_kib: 1024, ..KdfParams::DEFAULT };
        assert_eq!(create(KEYS, "correct horse battery", weak).unwrap_err(), RecoveryError::BadParams);
    }

    #[test]
    fn hostile_params_rejected_on_restore() {
        let mut f = create(KEYS, "correct horse battery", KdfParams::DEFAULT).unwrap();
        f[5..9].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(restore(&f, "correct horse battery").unwrap_err(), RecoveryError::BadParams);
    }

    #[test]
    fn two_files_differ() {
        let a = create(KEYS, "correct horse battery", KdfParams::DEFAULT).unwrap();
        let b = create(KEYS, "correct horse battery", KdfParams::DEFAULT).unwrap();
        assert_ne!(a, b);
    }
}

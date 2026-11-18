//! Wraps the vault master key under a key derived from the PIN and the
//! platform pepper (Argon2id). Changing the PIN re-wraps the same master key,
//! so sealed items do not need re-encrypting.
//!
//! Format (big-endian): magic "SGKW" | version u8 | m_kib u32 | t u32 | salt[16] | nonce[24] | ciphertext+tag
//! The header is bound as associated data.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305,
    aead::{Aead, Payload},
};
use zeroize::Zeroizing;

const MAGIC: &[u8; 4] = b"SGKW";
const VERSION: u8 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
const HEADER_LEN: usize = 4 + 1 + 4 + 4 + SALT_LEN + NONCE_LEN;
pub const MASTER_LEN: usize = 32;
pub const PEPPER_LEN: usize = 32;
pub const MIN_PIN_CHARS: usize = 4;
pub const MIN_M_KIB: u32 = 19 * 1024;
pub const MAX_M_KIB: u32 = 256 * 1024;
pub const MAX_T: u32 = 10;

#[derive(Debug, PartialEq, Eq)]
pub enum WrapError {
    PinTooShort,
    BadPepper,
    BadMaster,
    BadParams,
    BadFormat,
    /// Wrong PIN, wrong pepper or tampered blob. Deliberately not distinguished.
    Unwrap,
    Random,
    Kdf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrapParams {
    pub m_kib: u32,
    pub t: u32,
}

impl WrapParams {
    pub const DEFAULT: WrapParams = WrapParams {
        m_kib: MIN_M_KIB,
        t: 3,
    };

    fn valid(&self) -> bool {
        (MIN_M_KIB..=MAX_M_KIB).contains(&self.m_kib) && (1..=MAX_T).contains(&self.t)
    }
}

fn kek(
    pin: &str,
    pepper: &[u8],
    salt: &[u8],
    p: WrapParams,
) -> Result<Zeroizing<[u8; 32]>, WrapError> {
    if pepper.len() != PEPPER_LEN {
        return Err(WrapError::BadPepper);
    }
    let params = Params::new(p.m_kib, p.t, 1, Some(32)).map_err(|_| WrapError::BadParams)?;
    let argon = Argon2::new_with_secret(pepper, Algorithm::Argon2id, Version::V0x13, params)
        .map_err(|_| WrapError::BadPepper)?;
    let mut out = Zeroizing::new([0u8; 32]);
    argon
        .hash_password_into(pin.as_bytes(), salt, out.as_mut())
        .map_err(|_| WrapError::Kdf)?;
    Ok(out)
}

pub fn wrap(
    master: &[u8],
    pin: &str,
    pepper: &[u8],
    params: WrapParams,
) -> Result<Vec<u8>, WrapError> {
    if master.len() != MASTER_LEN {
        return Err(WrapError::BadMaster);
    }
    if pin.chars().count() < MIN_PIN_CHARS {
        return Err(WrapError::PinTooShort);
    }
    if !params.valid() {
        return Err(WrapError::BadParams);
    }
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    getrandom::fill(&mut salt).map_err(|_| WrapError::Random)?;
    getrandom::fill(&mut nonce).map_err(|_| WrapError::Random)?;
    let mut header = Vec::with_capacity(HEADER_LEN);
    header.extend_from_slice(MAGIC);
    header.push(VERSION);
    header.extend_from_slice(&params.m_kib.to_be_bytes());
    header.extend_from_slice(&params.t.to_be_bytes());
    header.extend_from_slice(&salt);
    header.extend_from_slice(&nonce);
    let key = kek(pin, pepper, &salt, params)?;
    let ct = XChaCha20Poly1305::new((&*key).into())
        .encrypt(
            &nonce.into(),
            Payload {
                msg: master,
                aad: &header,
            },
        )
        .map_err(|_| WrapError::Unwrap)?;
    header.extend_from_slice(&ct);
    Ok(header)
}

pub fn unwrap(blob: &[u8], pin: &str, pepper: &[u8]) -> Result<Zeroizing<Vec<u8>>, WrapError> {
    if blob.len() != HEADER_LEN + MASTER_LEN + 16 || &blob[..4] != MAGIC || blob[4] != VERSION {
        return Err(WrapError::BadFormat);
    }
    let u32_at = |i: usize| u32::from_be_bytes([blob[i], blob[i + 1], blob[i + 2], blob[i + 3]]);
    let params = WrapParams {
        m_kib: u32_at(5),
        t: u32_at(9),
    };
    if !params.valid() {
        return Err(WrapError::BadParams);
    }
    let salt = &blob[13..13 + SALT_LEN];
    let nonce: [u8; NONCE_LEN] = blob[13 + SALT_LEN..HEADER_LEN]
        .try_into()
        .map_err(|_| WrapError::BadFormat)?;
    let (header, ct) = blob.split_at(HEADER_LEN);
    let key = kek(pin, pepper, salt, params)?;
    let plain = XChaCha20Poly1305::new((&*key).into())
        .decrypt(
            &nonce.into(),
            Payload {
                msg: ct,
                aad: header,
            },
        )
        .map_err(|_| WrapError::Unwrap)?;
    Ok(Zeroizing::new(plain))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PEPPER: [u8; 32] = [9u8; 32];
    const MASTER: [u8; 32] = [3u8; 32];

    #[test]
    fn roundtrip() {
        let w = wrap(&MASTER, "2468", &PEPPER, WrapParams::DEFAULT).unwrap();
        assert_eq!(&unwrap(&w, "2468", &PEPPER).unwrap()[..], &MASTER);
    }

    #[test]
    fn wrong_pin_or_pepper_fails() {
        let w = wrap(&MASTER, "2468", &PEPPER, WrapParams::DEFAULT).unwrap();
        assert_eq!(unwrap(&w, "2469", &PEPPER).unwrap_err(), WrapError::Unwrap);
        assert_eq!(
            unwrap(&w, "2468", &[8u8; 32]).unwrap_err(),
            WrapError::Unwrap
        );
        assert_eq!(
            unwrap(&w, "2468", &[9u8; 5]).unwrap_err(),
            WrapError::BadPepper
        );
    }

    #[test]
    fn rejects_bad_inputs() {
        assert_eq!(
            wrap(&MASTER, "12", &PEPPER, WrapParams::DEFAULT).unwrap_err(),
            WrapError::PinTooShort
        );
        assert_eq!(
            wrap(&[1u8; 5], "2468", &PEPPER, WrapParams::DEFAULT).unwrap_err(),
            WrapError::BadMaster
        );
        let weak = WrapParams { m_kib: 1024, t: 3 };
        assert_eq!(
            wrap(&MASTER, "2468", &PEPPER, weak).unwrap_err(),
            WrapError::BadParams
        );
    }

    #[test]
    fn tamper_and_hostile_params_fail() {
        let w = wrap(&MASTER, "2468", &PEPPER, WrapParams::DEFAULT).unwrap();
        for i in [0usize, 4, 20, 40, w.len() - 1] {
            let mut t = w.clone();
            t[i] ^= 1;
            assert!(unwrap(&t, "2468", &PEPPER).is_err(), "byte {i}");
        }
        let mut h = w.clone();
        h[5..9].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(
            unwrap(&h, "2468", &PEPPER).unwrap_err(),
            WrapError::BadParams
        );
        assert_eq!(
            unwrap(&w[..20], "2468", &PEPPER).unwrap_err(),
            WrapError::BadFormat
        );
    }

    #[test]
    fn rewrapping_under_a_new_pin_keeps_the_master_key() {
        let w1 = wrap(&MASTER, "2468", &PEPPER, WrapParams::DEFAULT).unwrap();
        let m = unwrap(&w1, "2468", &PEPPER).unwrap();
        let w2 = wrap(&m, "8642", &PEPPER, WrapParams::DEFAULT).unwrap();
        assert_ne!(w1, w2);
        assert_eq!(&unwrap(&w2, "8642", &PEPPER).unwrap()[..], &MASTER);
        assert!(unwrap(&w2, "2468", &PEPPER).is_err());
    }
}

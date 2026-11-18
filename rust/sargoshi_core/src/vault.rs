//! Vault PIN verifier record.
//!
//! A PIN has little entropy, so the record is keyed with a 32-byte pepper that
//! the platform keeps in hardware-backed storage (Android Keystore). A copy of
//! the record alone cannot be brute-forced offline without the pepper.
//!
//! Format (big-endian): magic "SGPV" | version u8 | m_kib u32 | t u32 | p u32 | salt[16] | tag[32]

use argon2::{Algorithm, Argon2, Params, Version};
use zeroize::Zeroizing;

const MAGIC: &[u8; 4] = b"SGPV";
const VERSION: u8 = 1;
const SALT_LEN: usize = 16;
const TAG_LEN: usize = 32;
const RECORD_LEN: usize = 4 + 1 + 12 + SALT_LEN + TAG_LEN;

pub const MIN_PIN_CHARS: usize = 4;
pub const PEPPER_LEN: usize = 32;
pub const MIN_M_KIB: u32 = 19 * 1024;
pub const MAX_M_KIB: u32 = 256 * 1024;
pub const MAX_T: u32 = 10;

#[derive(Debug, PartialEq, Eq)]
pub enum VaultError {
    PinTooShort,
    BadPepper,
    BadParams,
    BadFormat,
    Random,
    Kdf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultParams {
    pub m_kib: u32,
    pub t: u32,
}

impl VaultParams {
    pub const DEFAULT: VaultParams = VaultParams {
        m_kib: MIN_M_KIB,
        t: 3,
    };

    fn valid(&self) -> bool {
        (MIN_M_KIB..=MAX_M_KIB).contains(&self.m_kib) && (1..=MAX_T).contains(&self.t)
    }
}

fn tag(
    pin: &str,
    pepper: &[u8],
    salt: &[u8],
    p: VaultParams,
) -> Result<Zeroizing<[u8; TAG_LEN]>, VaultError> {
    if pepper.len() != PEPPER_LEN {
        return Err(VaultError::BadPepper);
    }
    let params = Params::new(p.m_kib, p.t, 1, Some(TAG_LEN)).map_err(|_| VaultError::BadParams)?;
    let argon = Argon2::new_with_secret(pepper, Algorithm::Argon2id, Version::V0x13, params)
        .map_err(|_| VaultError::BadPepper)?;
    let mut out = Zeroizing::new([0u8; TAG_LEN]);
    argon
        .hash_password_into(pin.as_bytes(), salt, out.as_mut())
        .map_err(|_| VaultError::Kdf)?;
    Ok(out)
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

pub fn enroll(pin: &str, pepper: &[u8], params: VaultParams) -> Result<Vec<u8>, VaultError> {
    if pin.chars().count() < MIN_PIN_CHARS {
        return Err(VaultError::PinTooShort);
    }
    if !params.valid() {
        return Err(VaultError::BadParams);
    }
    let mut salt = [0u8; SALT_LEN];
    getrandom::fill(&mut salt).map_err(|_| VaultError::Random)?;
    let t = tag(pin, pepper, &salt, params)?;
    let mut rec = Vec::with_capacity(RECORD_LEN);
    rec.extend_from_slice(MAGIC);
    rec.push(VERSION);
    rec.extend_from_slice(&params.m_kib.to_be_bytes());
    rec.extend_from_slice(&params.t.to_be_bytes());
    rec.extend_from_slice(&1u32.to_be_bytes());
    rec.extend_from_slice(&salt);
    rec.extend_from_slice(&*t);
    Ok(rec)
}

/// `Ok(true)` on a match, `Ok(false)` on a wrong PIN. Errors mean the record
/// or pepper is unusable, which the caller must not treat as "wrong PIN".
pub fn verify(record: &[u8], pin: &str, pepper: &[u8]) -> Result<bool, VaultError> {
    if record.len() != RECORD_LEN || &record[..4] != MAGIC || record[4] != VERSION {
        return Err(VaultError::BadFormat);
    }
    let u32_at =
        |i: usize| u32::from_be_bytes([record[i], record[i + 1], record[i + 2], record[i + 3]]);
    let params = VaultParams {
        m_kib: u32_at(5),
        t: u32_at(9),
    };
    if !params.valid() || u32_at(13) != 1 {
        return Err(VaultError::BadParams);
    }
    let salt = &record[17..17 + SALT_LEN];
    let stored = &record[17 + SALT_LEN..];
    // Always run the KDF, even for a too-short PIN, so timing does not leak length rules.
    let t = tag(pin, pepper, salt, params)?;
    Ok(ct_eq(&*t, stored) && pin.chars().count() >= MIN_PIN_CHARS)
}

#[cfg(test)]
mod tests {
    /// RFC 9106 section 5.3: Argon2id v1.3 with a secret value (the mechanism
    /// the pepper uses), 32 KiB, 3 passes, 4 lanes, 32-byte tag.
    #[test]
    fn matches_rfc9106_argon2id_vector() {
        use argon2::{AssociatedData, ParamsBuilder};
        let ad = AssociatedData::new(&[4u8; 12]).unwrap();
        let params = ParamsBuilder::new()
            .m_cost(32)
            .t_cost(3)
            .p_cost(4)
            .output_len(32)
            .data(ad)
            .build()
            .unwrap();
        let argon = Argon2::new_with_secret(&[3u8; 8], Algorithm::Argon2id, Version::V0x13, params)
            .unwrap();
        let mut out = [0u8; 32];
        argon
            .hash_password_into(&[1u8; 32], &[2u8; 16], &mut out)
            .unwrap();
        let expected = "0d640df58d78766c08c037a34a8b53c9d01ef0452d75b65eb52520e96b01e659";
        let got: String = out.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(got, expected);
    }

    use super::*;

    const PEPPER: [u8; 32] = [7u8; 32];

    #[test]
    fn right_pin_matches_wrong_pin_does_not() {
        let r = enroll("2468", &PEPPER, VaultParams::DEFAULT).unwrap();
        assert_eq!(verify(&r, "2468", &PEPPER), Ok(true));
        assert_eq!(verify(&r, "2469", &PEPPER), Ok(false));
    }

    #[test]
    fn wrong_pepper_does_not_match() {
        let r = enroll("2468", &PEPPER, VaultParams::DEFAULT).unwrap();
        assert_eq!(verify(&r, "2468", &[8u8; 32]), Ok(false));
        assert_eq!(verify(&r, "2468", &[7u8; 31]), Err(VaultError::BadPepper));
    }

    #[test]
    fn rejects_short_pin_and_weak_params() {
        assert_eq!(
            enroll("123", &PEPPER, VaultParams::DEFAULT),
            Err(VaultError::PinTooShort)
        );
        let weak = VaultParams { m_kib: 1024, t: 3 };
        assert_eq!(enroll("1234", &PEPPER, weak), Err(VaultError::BadParams));
    }

    #[test]
    fn same_pin_gives_different_records() {
        let a = enroll("2468", &PEPPER, VaultParams::DEFAULT).unwrap();
        let b = enroll("2468", &PEPPER, VaultParams::DEFAULT).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn malformed_or_hostile_records_are_errors() {
        let mut r = enroll("2468", &PEPPER, VaultParams::DEFAULT).unwrap();
        assert_eq!(
            verify(&r[..10], "2468", &PEPPER),
            Err(VaultError::BadFormat)
        );
        let mut hostile = r.clone();
        hostile[5..9].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(
            verify(&hostile, "2468", &PEPPER),
            Err(VaultError::BadParams)
        );
        r[40] ^= 1;
        assert_eq!(verify(&r, "2468", &PEPPER), Ok(false));
    }

    #[test]
    fn tampered_salt_does_not_match() {
        let mut r = enroll("2468", &PEPPER, VaultParams::DEFAULT).unwrap();
        r[20] ^= 1;
        assert_eq!(verify(&r, "2468", &PEPPER), Ok(false));
    }
}

//! Authenticated encryption for vault items: XChaCha20-Poly1305 with a random
//! 24-byte nonce per item. Output is `nonce || ciphertext+tag`. The caller
//! binds context (item id, kind) through `aad`, so a sealed item cannot be
//! moved to another slot unnoticed.

use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305,
    aead::{Aead, Payload},
};
use zeroize::Zeroizing;

pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;

#[derive(Debug, PartialEq, Eq)]
pub enum AeadError {
    BadKey,
    Random,
    /// Wrong key, wrong context, or tampered data. Deliberately not distinguished.
    Open,
}

pub fn generate_key() -> Result<Zeroizing<[u8; KEY_LEN]>, AeadError> {
    let mut k = Zeroizing::new([0u8; KEY_LEN]);
    getrandom::fill(k.as_mut()).map_err(|_| AeadError::Random)?;
    Ok(k)
}

pub fn seal(key: &[u8], aad: &[u8], plain: &[u8]) -> Result<Vec<u8>, AeadError> {
    let mut nonce = [0u8; NONCE_LEN];
    getrandom::fill(&mut nonce).map_err(|_| AeadError::Random)?;
    seal_with_nonce(key, &nonce, aad, plain)
}

/// Encryption with a caller-chosen nonce. Private on purpose: a repeated
/// nonce under one key breaks the cipher, so only `seal` (random nonce) and
/// known-answer tests may call it.
fn seal_with_nonce(
    key: &[u8],
    nonce: &[u8; NONCE_LEN],
    aad: &[u8],
    plain: &[u8],
) -> Result<Vec<u8>, AeadError> {
    let key: &[u8; KEY_LEN] = key.try_into().map_err(|_| AeadError::BadKey)?;
    let cipher = XChaCha20Poly1305::new(key.into());
    let ct = cipher
        .encrypt(&(*nonce).into(), Payload { msg: plain, aad })
        .map_err(|_| AeadError::Open)?;
    let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
    out.extend_from_slice(nonce);
    out.extend_from_slice(&ct);
    Ok(out)
}

pub fn open(key: &[u8], aad: &[u8], sealed: &[u8]) -> Result<Zeroizing<Vec<u8>>, AeadError> {
    let key: &[u8; KEY_LEN] = key.try_into().map_err(|_| AeadError::BadKey)?;
    if sealed.len() < NONCE_LEN + TAG_LEN {
        return Err(AeadError::Open);
    }
    let (nonce, ct) = sealed.split_at(NONCE_LEN);
    let nonce: [u8; NONCE_LEN] = nonce.try_into().map_err(|_| AeadError::Open)?;
    let cipher = XChaCha20Poly1305::new(key.into());
    let plain = cipher
        .decrypt(&nonce.into(), Payload { msg: ct, aad })
        .map_err(|_| AeadError::Open)?;
    Ok(Zeroizing::new(plain))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_empty() {
        let k = generate_key().unwrap();
        for msg in [&b""[..], b"hello", &[7u8; 5000]] {
            let s = seal(&*k, b"item:1", msg).unwrap();
            assert_eq!(&open(&*k, b"item:1", &s).unwrap()[..], msg);
        }
    }

    #[test]
    fn wrong_key_context_or_tamper_fails() {
        let k = generate_key().unwrap();
        let other = generate_key().unwrap();
        let s = seal(&*k, b"item:1", b"secret").unwrap();
        assert_eq!(open(&*other, b"item:1", &s).unwrap_err(), AeadError::Open);
        assert_eq!(open(&*k, b"item:2", &s).unwrap_err(), AeadError::Open);
        for i in [0, NONCE_LEN, s.len() - 1] {
            let mut t = s.clone();
            t[i] ^= 1;
            assert_eq!(
                open(&*k, b"item:1", &t).unwrap_err(),
                AeadError::Open,
                "byte {i}"
            );
        }
        assert_eq!(open(&*k, b"item:1", &s[..10]).unwrap_err(), AeadError::Open);
    }

    #[test]
    fn nonces_are_fresh_and_keys_validated() {
        let k = generate_key().unwrap();
        assert_ne!(
            seal(&*k, b"a", b"x").unwrap(),
            seal(&*k, b"a", b"x").unwrap()
        );
        assert_eq!(seal(&[1u8; 5], b"", b"x").unwrap_err(), AeadError::BadKey);
        assert_eq!(
            open(&[1u8; 33], b"", &[0u8; 60]).unwrap_err(),
            AeadError::BadKey
        );
    }

    fn unhex(h: &str) -> Vec<u8> {
        let h: String = h.split_whitespace().collect();
        (0..h.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
            .collect()
    }

    /// draft-irtf-cfrg-xchacha, appendix A.3.1 (published AEAD_XChaCha20_Poly1305 vector).
    #[test]
    fn matches_published_xchacha20_poly1305_vector() {
        let plain = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";
        let aad = unhex("50515253c0c1c2c3c4c5c6c7");
        let key = unhex("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f");
        let nonce: [u8; NONCE_LEN] = unhex("404142434445464748494a4b4c4d4e4f5051525354555657")
            .try_into()
            .unwrap();
        let expected_ct = unhex(
            "bd6d179d3e83d43b9576579493c0e939572a1700252bfaccbed2902c21396cbb
             731c7f1b0b4aa6440bf3a82f4eda7e39ae64c6708c54c216cb96b72e1213b452
             2f8c9ba40db5d945b11b69b982c1bb9e3f3fac2bc369488f76b2383565d3fff9
             21f9664c97637da9768812f615c68b13b52e",
        );
        let expected_tag = unhex("c0875924c1c7987947deafd8780acf49");

        let sealed = seal_with_nonce(&key, &nonce, &aad, plain).unwrap();
        assert_eq!(&sealed[..NONCE_LEN], &nonce[..]);
        assert_eq!(
            &sealed[NONCE_LEN..NONCE_LEN + plain.len()],
            &expected_ct[..]
        );
        assert_eq!(&sealed[NONCE_LEN + plain.len()..], &expected_tag[..]);
        assert_eq!(&open(&key, &aad, &sealed).unwrap()[..], &plain[..]);
    }
}

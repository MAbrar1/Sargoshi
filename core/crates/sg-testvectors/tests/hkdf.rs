//! HKDF-SHA256 against Wycheproof (which includes the RFC 5869 cases).
use sg_crypto::CryptoError;
use sg_crypto::kdf::hkdf_sha256;
use sg_testvectors::{field, hex, load};

type TestResult = Result<(), String>;

#[test]
fn wycheproof_hkdf_sha256() -> TestResult {
    let records = load("hkdf_sha256.txt")?;
    let (mut valid, mut invalid) = (0, 0);
    for r in &records {
        let id = field(r, 0)?;
        let (ikm, salt, info) = (hex(field(r, 2)?)?, hex(field(r, 3)?)?, hex(field(r, 4)?)?);
        let size: usize = field(r, 5)?.parse().map_err(|e| format!("size: {e}"))?;
        let expected = hex(field(r, 6)?)?;
        let mut out = vec![0u8; size];
        match field(r, 1)? {
            "valid" => {
                hkdf_sha256(&salt, &ikm, &info, &mut out).map_err(|e| format!("tc {id}: {e}"))?;
                assert_eq!(out, expected, "tc {id}");
                valid += 1;
            }
            _ => {
                assert_eq!(
                    hkdf_sha256(&salt, &ikm, &info, &mut out),
                    Err(CryptoError::KdfOutputTooLong),
                    "tc {id}"
                );
                invalid += 1;
            }
        }
    }
    assert!(valid >= 80, "only {valid} valid vectors ran");
    assert!(invalid >= 1, "no over-long output case ran");
    Ok(())
}

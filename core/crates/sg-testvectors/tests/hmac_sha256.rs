//! HMAC-SHA256 against Wycheproof (which includes the RFC 4231 cases and forged tags).
use sg_crypto::mac::hmac_sha256;
use sg_testvectors::{field, hex, load};

type TestResult = Result<(), String>;

#[test]
fn wycheproof_hmac_sha256() -> TestResult {
    let records = load("hmac_sha256.txt")?;
    let (mut valid, mut invalid) = (0, 0);
    for r in &records {
        let id = field(r, 0)?;
        let (key, msg, tag) = (hex(field(r, 2)?)?, hex(field(r, 3)?)?, hex(field(r, 4)?)?);
        let computed = hmac_sha256(&key, &msg).map_err(|e| format!("tc {id}: {e}"))?;
        match field(r, 1)? {
            "valid" => {
                assert_eq!(computed.expose().as_slice(), tag.as_slice(), "tc {id}");
                valid += 1;
            }
            _ => {
                assert_ne!(
                    computed.expose().as_slice(),
                    tag.as_slice(),
                    "tc {id}: forged tag matches"
                );
                invalid += 1;
            }
        }
    }
    // The vector file is pinned by upstream commit, so exact counts also detect a shrinking set.
    assert_eq!((valid, invalid), (33, 54));
    Ok(())
}

#[test]
fn rfc4231_test_case_1() -> TestResult {
    // RFC 4231 section 4.2: key = 0x0b * 20, data = "Hi There".
    let tag = hmac_sha256(&[0x0b; 20], b"Hi There").map_err(|e| e.to_string())?;
    let expected = hex("b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7")?;
    assert_eq!(tag.expose().as_slice(), expected.as_slice());
    Ok(())
}

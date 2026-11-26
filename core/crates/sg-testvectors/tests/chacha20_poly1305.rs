//! ChaCha20-Poly1305 against Wycheproof (which includes the RFC 8439 cases and tag forgeries).
use sg_crypto::SecretBytes;
use sg_crypto::aead::{open, seal};
use sg_testvectors::{field, hex, load};

type TestResult = Result<(), String>;

#[test]
fn wycheproof_chacha20_poly1305() -> TestResult {
    let records = load("chacha20_poly1305.txt")?;
    let (mut valid, mut invalid) = (0, 0);
    for r in &records {
        let id = field(r, 0)?;
        let key: [u8; 32] = hex(field(r, 2)?)?
            .as_slice()
            .try_into()
            .map_err(|_| format!("tc {id}: key"))?;
        let nonce: [u8; 12] = hex(field(r, 3)?)?
            .as_slice()
            .try_into()
            .map_err(|_| format!("tc {id}: iv"))?;
        let (aad, msg) = (hex(field(r, 4)?)?, hex(field(r, 5)?)?);
        let mut sealed = hex(field(r, 6)?)?;
        sealed.extend_from_slice(&hex(field(r, 7)?)?);
        let key = SecretBytes::new(key);
        match field(r, 1)? {
            "valid" => {
                assert_eq!(
                    seal(&key, &nonce, &aad, &msg),
                    Ok(sealed.clone()),
                    "tc {id}"
                );
                let opened =
                    open(&key, &nonce, &aad, &sealed).map_err(|e| format!("tc {id}: {e}"))?;
                assert_eq!(opened.as_slice(), msg.as_slice(), "tc {id}");
                valid += 1;
            }
            _ => {
                assert!(
                    open(&key, &nonce, &aad, &sealed).is_err(),
                    "tc {id}: forgery accepted"
                );
                invalid += 1;
            }
        }
    }
    assert!(valid >= 50, "only {valid} valid vectors ran");
    assert!(invalid >= 50, "only {invalid} forgery cases ran");
    Ok(())
}

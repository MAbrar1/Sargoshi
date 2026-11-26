//! X25519 against Wycheproof (which includes the RFC 7748 cases, twists and low-order points).
use sg_crypto::CryptoError;
use sg_crypto::x25519::{X25519PublicKey, X25519SecretKey};
use sg_testvectors::{field, hex, load};

type TestResult = Result<(), String>;

#[test]
fn wycheproof_x25519() -> TestResult {
    let records = load("x25519.txt")?;
    let (mut agreed, mut rejected) = (0, 0);
    for r in &records {
        let id = field(r, 0)?;
        let result = field(r, 1)?;
        let (public, private, shared) =
            (hex(field(r, 2)?)?, hex(field(r, 3)?)?, hex(field(r, 4)?)?);
        let Ok(public) = X25519PublicKey::from_slice(&public) else {
            assert_ne!(
                result, "valid",
                "tc {id}: a valid vector has a malformed public key"
            );
            rejected += 1;
            continue;
        };
        let private: [u8; 32] = private
            .as_slice()
            .try_into()
            .map_err(|_| format!("tc {id}: private key length"))?;
        let outcome = X25519SecretKey::from_bytes(private).diffie_hellman(&public);
        match (result, outcome) {
            ("valid", Ok(secret)) => {
                assert_eq!(secret.expose().as_slice(), shared.as_slice(), "tc {id}");
                agreed += 1;
            }
            ("acceptable", Ok(secret)) => {
                assert_eq!(secret.expose().as_slice(), shared.as_slice(), "tc {id}");
                agreed += 1;
            }
            ("acceptable", Err(e)) => {
                assert_eq!(e, CryptoError::WeakSharedSecret, "tc {id}");
                assert!(
                    shared.iter().all(|b| *b == 0),
                    "tc {id}: rejected a non-zero secret"
                );
                rejected += 1;
            }
            ("invalid", Err(_)) => rejected += 1,
            (kind, other) => return Err(format!("tc {id}: result {kind} but got {other:?}")),
        }
    }
    assert!(agreed >= 400, "only {agreed} agreements ran");
    assert!(rejected >= 1, "no low-order or malformed case ran");
    Ok(())
}

#[test]
fn public_key_derivation_matches_the_clamped_basepoint_multiplication() -> TestResult {
    // RFC 7748 section 6.1 Alice: the published private and public keys.
    let private = hex("77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a")?;
    let public = hex("8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a")?;
    let private: [u8; 32] = private
        .as_slice()
        .try_into()
        .map_err(|_| "length".to_owned())?;
    let derived = X25519SecretKey::from_bytes(private).public_key();
    assert_eq!(derived.as_bytes().as_slice(), public.as_slice());
    Ok(())
}

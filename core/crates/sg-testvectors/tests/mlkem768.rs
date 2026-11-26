//! ML-KEM-768 against NIST ACVP vectors (FIPS 203): key generation, encapsulation,
//! decapsulation including implicit rejection, and key validity checks.
#![allow(
    deprecated,
    reason = "NIST ACVP publishes ML-KEM keys in the expanded format that the library deprecated for applications; it is used here only to check the library against NIST"
)]

use ml_kem::{Decapsulate, DecapsulationKey768, ExpandedKeyEncoding, KeyExport};
use sg_crypto::SecretBytes;
use sg_crypto::kem::{MlKem768Ciphertext, MlKem768PublicKey, MlKem768SecretKey};
use sg_testvectors::{field, hex, load};

type TestResult = Result<(), String>;

fn expanded(dk: &[u8]) -> Result<DecapsulationKey768, String> {
    let array = ml_kem::ExpandedDecapsulationKey::<ml_kem::MlKem768>::try_from(dk)
        .map_err(|_| "expanded key length".to_owned())?;
    DecapsulationKey768::from_expanded(&array).map_err(|_| "invalid expanded key".to_owned())
}

#[test]
fn acvp_key_generation_from_seed() -> TestResult {
    let records = load("mlkem768_keygen.txt")?;
    assert_eq!(records.len(), 25);
    for r in &records {
        let id = field(r, 0)?;
        let mut seed = hex(field(r, 1)?)?; // d
        seed.extend_from_slice(&hex(field(r, 2)?)?); // z
        let (ek, dk) = (hex(field(r, 3)?)?, hex(field(r, 4)?)?);
        let seed: [u8; 64] = seed
            .as_slice()
            .try_into()
            .map_err(|_| format!("tc {id}: seed"))?;
        let secret = MlKem768SecretKey::from_seed(SecretBytes::new(seed));
        let public = secret.public_key().map_err(|e| format!("tc {id}: {e}"))?;
        assert_eq!(
            public.as_bytes(),
            ek.as_slice(),
            "tc {id}: encapsulation key"
        );
        // The expanded decapsulation key derived by the library from the same seed must match
        // the NIST value as well.
        let library_dk = DecapsulationKey768::from_seed(ml_kem::Seed::from(seed));
        assert_eq!(
            library_dk.to_expanded_bytes().as_slice(),
            dk.as_slice(),
            "tc {id}: dk"
        );
    }
    Ok(())
}

#[test]
fn acvp_encapsulation_with_fixed_message() -> TestResult {
    let records = load("mlkem768_encaps.txt")?;
    assert_eq!(records.len(), 25);
    for r in &records {
        let id = field(r, 0)?;
        let ek = MlKem768PublicKey::from_bytes(&hex(field(r, 1)?)?)
            .map_err(|e| format!("tc {id}: {e}"))?;
        let m: [u8; 32] = hex(field(r, 2)?)?
            .as_slice()
            .try_into()
            .map_err(|_| format!("tc {id}: m"))?;
        let (c, k) = (hex(field(r, 3)?)?, hex(field(r, 4)?)?);
        let (ciphertext, shared) = ek
            .encapsulate_deterministic(&m)
            .map_err(|e| format!("tc {id}: {e}"))?;
        assert_eq!(ciphertext.as_bytes(), c.as_slice(), "tc {id}: ciphertext");
        assert_eq!(
            shared.expose().as_slice(),
            k.as_slice(),
            "tc {id}: shared secret"
        );
    }
    Ok(())
}

#[test]
fn acvp_decapsulation_including_implicit_rejection() -> TestResult {
    let records = load("mlkem768_decaps.txt")?;
    assert_eq!(records.len(), 10);
    for r in &records {
        let id = field(r, 0)?;
        let dk = expanded(&hex(field(r, 1)?)?).map_err(|e| format!("tc {id}: {e}"))?;
        let (c, k) = (hex(field(r, 2)?)?, hex(field(r, 3)?)?);
        let shared = dk
            .decapsulate_slice(&c)
            .map_err(|_| format!("tc {id}: ciphertext length"))?;
        assert_eq!(shared.as_slice(), k.as_slice(), "tc {id}");
    }
    Ok(())
}

#[test]
fn acvp_encapsulation_key_check_matches_wrapper_validation() -> TestResult {
    let records = load("mlkem768_ek_check.txt")?;
    assert_eq!(records.len(), 10);
    let mut rejected = 0;
    for r in &records {
        let id = field(r, 0)?;
        let passed = field(r, 2)? == "1";
        let result = MlKem768PublicKey::from_bytes(&hex(field(r, 1)?)?);
        assert_eq!(result.is_ok(), passed, "tc {id}");
        rejected += usize::from(!passed);
    }
    assert!(rejected >= 1, "no invalid key case ran");
    Ok(())
}

#[test]
fn acvp_decapsulation_key_check_matches_library_validation() -> TestResult {
    let records = load("mlkem768_dk_check.txt")?;
    assert_eq!(records.len(), 10);
    for r in &records {
        let id = field(r, 0)?;
        let passed = field(r, 2)? == "1";
        let result = hex(field(r, 1)?).and_then(|dk| expanded(&dk));
        assert_eq!(result.is_ok(), passed, "tc {id}");
    }
    Ok(())
}

#[test]
fn wrapper_round_trip_and_implicit_rejection() -> TestResult {
    let (secret, public) = MlKem768SecretKey::generate().map_err(|e| e.to_string())?;
    let (ciphertext, sender_secret) = public.encapsulate().map_err(|e| e.to_string())?;
    let receiver_secret = secret.decapsulate(&ciphertext).map_err(|e| e.to_string())?;
    assert_eq!(sender_secret, receiver_secret);

    // A corrupted but correctly sized ciphertext decapsulates to a different, unrelated secret.
    let mut bytes = ciphertext.as_bytes().to_vec();
    bytes.iter_mut().take(1).for_each(|b| *b ^= 1);
    let corrupted = MlKem768Ciphertext::from_slice(&bytes).map_err(|e| e.to_string())?;
    let rejected = secret.decapsulate(&corrupted).map_err(|e| e.to_string())?;
    assert_ne!(sender_secret, rejected);

    // A wrong-sized ciphertext is refused outright.
    assert!(MlKem768Ciphertext::from_slice(&bytes[1..]).is_err());
    assert!(MlKem768PublicKey::from_bytes(&public.as_bytes()[1..]).is_err());
    Ok(())
}

#[test]
fn encapsulation_key_bytes_round_trip() -> TestResult {
    let (secret, public) = MlKem768SecretKey::generate().map_err(|e| e.to_string())?;
    assert_eq!(
        MlKem768PublicKey::from_bytes(public.as_bytes()),
        Ok(public.clone())
    );
    let restored = MlKem768SecretKey::from_seed(SecretBytes::new(*secret.expose_seed()));
    assert_eq!(restored.public_key(), Ok(public));
    let library = DecapsulationKey768::from_seed(ml_kem::Seed::from(*secret.expose_seed()));
    assert_eq!(library.encapsulation_key().to_bytes().len(), 1184);
    Ok(())
}

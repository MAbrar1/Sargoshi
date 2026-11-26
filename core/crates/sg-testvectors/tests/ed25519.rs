//! Ed25519 against Wycheproof, verified with the strict verifier Sargoshi uses.
use sg_crypto::SecretBytes;
use sg_crypto::ed25519::{Signature, SigningKey, VerifyingKey};
use sg_testvectors::{field, hex, load};

type TestResult = Result<(), String>;

#[test]
fn wycheproof_ed25519() -> TestResult {
    let records = load("ed25519.txt")?;
    let (mut accepted, mut rejected, mut refused_weak_keys) = (0, 0, 0);
    for r in &records {
        let id = field(r, 0)?;
        let result = field(r, 1)?;
        let (pk, msg, sig) = (hex(field(r, 2)?)?, hex(field(r, 3)?)?, hex(field(r, 4)?)?);
        let verdict = VerifyingKey::from_slice(&pk).and_then(|key| {
            let signature = Signature::from_slice(&sig)?;
            key.verify(&msg, &signature)
        });
        match (result, verdict.is_ok()) {
            ("valid", true) => accepted += 1,
            ("invalid", false) => rejected += 1,
            // Wycheproof lets verifiers choose for these. Sargoshi policy is to refuse every
            // small-order or mixed-order public key; any other category needs a human decision.
            ("acceptable", false) => {
                let flags = field(r, 5)?;
                assert!(
                    flags == "SmallOrderPublicKey" || flags == "MixedOrderPublicKey",
                    "tc {id}: new acceptable category {flags} needs review"
                );
                refused_weak_keys += 1;
            }
            (kind, ok) => return Err(format!("tc {id}: vector is {kind} but accepted={ok}")),
        }
    }
    assert!(accepted >= 50, "only {accepted} valid vectors ran");
    assert!(rejected >= 50, "only {rejected} invalid vectors ran");
    assert!(
        refused_weak_keys >= 20,
        "only {refused_weak_keys} weak-key vectors ran"
    );
    Ok(())
}

#[test]
fn rfc8032_test_vector_1_signs_and_verifies() -> TestResult {
    // RFC 8032 section 7.1, TEST 1 (empty message).
    let seed = hex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")?;
    let public = hex("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")?;
    let signature = hex(
        "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b",
    )?;
    let seed: [u8; 32] = seed
        .as_slice()
        .try_into()
        .map_err(|_| "seed length".to_owned())?;
    let key = SigningKey::from_seed(&SecretBytes::new(seed));
    assert_eq!(key.verifying_key().to_bytes().as_slice(), public.as_slice());
    assert_eq!(key.sign(b"").as_bytes().as_slice(), signature.as_slice());
    Ok(())
}

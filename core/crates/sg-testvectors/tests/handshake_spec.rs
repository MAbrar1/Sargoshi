//! Conformance of the handshake to the written specification (Prompt B, section 19.2). The
//! expected shared key and associated data are rebuilt here with raw primitives and the literal
//! label strings from the specification, then compared with the real implementation. Both sides
//! must also agree with each other.
use sg_crypto::SecretBytes;
use sg_crypto::kdf::hkdf_sha256_secret;
use sg_crypto::x25519::X25519SecretKey;
use sg_handshake::{
    IdentityKeyPair, OneTimePreKeyPublic, PreKeyBundle, PrekeySecrets, accept,
    initiate_deterministic,
};

type TestResult = Result<(), String>;

struct Fixture {
    alice: IdentityKeyPair,
    bob: IdentityKeyPair,
    bob_prekeys: PrekeySecrets,
    bundle: PreKeyBundle,
    opk: OneTimePreKeyPublic,
}

fn fixture() -> Result<Fixture, String> {
    let alice = IdentityKeyPair::generate().map_err(|e| e.to_string())?;
    let bob = IdentityKeyPair::generate().map_err(|e| e.to_string())?;
    let (bob_prekeys, upload) =
        PrekeySecrets::generate(&bob, 7, 8, 1_700_000_000, 50, 3).map_err(|e| e.to_string())?;
    let opk = upload
        .one_time_prekeys
        .first()
        .cloned()
        .ok_or("no one-time prekey")?;
    let bundle = PreKeyBundle {
        identity: bob.public().clone(),
        signed_prekey: upload.signed_prekey,
        pq_prekey: upload.pq_prekey,
        one_time_prekey: None,
    };
    Ok(Fixture {
        alice,
        bob,
        bob_prekeys,
        bundle,
        opk,
    })
}

fn alice_dh_secret(alice: &IdentityKeyPair) -> X25519SecretKey {
    // The identity secret blob is `version || signing_seed(32) || dh_secret(32)`.
    let blob = alice.to_secret_bytes();
    let mut dh = [0u8; 32];
    dh.iter_mut()
        .zip(blob.iter().skip(33))
        .for_each(|(d, s)| *d = *s);
    X25519SecretKey::from_bytes(dh)
}

fn check(with_one_time_prekey: bool) -> TestResult {
    let mut f = fixture()?;
    if with_one_time_prekey {
        f.bundle.one_time_prekey = Some(f.opk.clone());
    }
    let ephemeral = X25519SecretKey::from_bytes([0x11; 32]);
    let kem_message = [0x22u8; 32];
    let started = initiate_deterministic(&f.alice, &f.bundle, &ephemeral, &kem_message)
        .map_err(|e| e.to_string())?;

    // --- rebuild the specification by hand
    let ik_a = alice_dh_secret(&f.alice);
    let ik_b_pub = f.bundle.identity.dh_key();
    let spk_b = &f.bundle.signed_prekey.public;
    let dh1 = ik_a.diffie_hellman(spk_b).map_err(|e| e.to_string())?;
    let dh2 = ephemeral
        .diffie_hellman(ik_b_pub)
        .map_err(|e| e.to_string())?;
    let dh3 = ephemeral.diffie_hellman(spk_b).map_err(|e| e.to_string())?;
    let (ct, ss) = f
        .bundle
        .pq_prekey
        .encapsulation_key
        .encapsulate_deterministic(&kem_message)
        .map_err(|e| e.to_string())?;

    let mut ikm = vec![0xFFu8; 32];
    ikm.extend_from_slice(dh1.expose());
    ikm.extend_from_slice(dh2.expose());
    ikm.extend_from_slice(dh3.expose());
    if with_one_time_prekey {
        let dh4 = ephemeral
            .diffie_hellman(&f.opk.public)
            .map_err(|e| e.to_string())?;
        ikm.extend_from_slice(dh4.expose());
    }
    ikm.extend_from_slice(ss.expose());
    assert_eq!(
        ikm.len(),
        32 + 32 * if with_one_time_prekey { 5 } else { 4 }
    );
    let expected: SecretBytes<32> =
        hkdf_sha256_secret(&[0u8; 32], &ikm, b"Sargoshi/PQXDH/v1").map_err(|e| e.to_string())?;

    let mut expected_ad = Vec::new();
    expected_ad.extend_from_slice(f.alice.public().dh_key().as_bytes());
    expected_ad.extend_from_slice(ik_b_pub.as_bytes());
    expected_ad.extend_from_slice(f.bundle.pq_prekey.encapsulation_key.as_bytes());

    assert_eq!(
        started.shared_key, expected,
        "shared key differs from the specification"
    );
    assert_eq!(
        started.associated_data, expected_ad,
        "associated data differs from the specification"
    );
    assert_eq!(started.initial_header.kem_ciphertext, ct);
    assert_eq!(started.initial_header.ephemeral, ephemeral.public_key());
    assert_eq!(started.initial_header.signed_prekey_id, 7);
    assert_eq!(started.initial_header.pq_prekey_id, 8);
    assert_eq!(
        started.initial_header.one_time_prekey_id.is_some(),
        with_one_time_prekey
    );

    // --- the responder must reach the same key and data
    let accepted =
        accept(&f.bob, &mut f.bob_prekeys, &started.initial_header).map_err(|e| e.to_string())?;
    assert_eq!(accepted.shared_key, expected);
    assert_eq!(accepted.associated_data, expected_ad);
    Ok(())
}

#[test]
fn handshake_with_a_one_time_prekey_matches_the_specification() -> TestResult {
    check(true)
}

#[test]
fn handshake_without_a_one_time_prekey_matches_the_specification() -> TestResult {
    check(false)
}

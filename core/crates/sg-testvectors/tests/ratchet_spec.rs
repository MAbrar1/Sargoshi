//! Conformance of the Double Ratchet to its written specification (Prompt B, section 19.2).
//! A real conversation is run through the public API, and every key, nonce, header and
//! ciphertext is rebuilt here with raw primitives and the literal label strings, then compared
//! with the session state (read through the documented blob layout) and the real envelopes.
use sg_common::Envelope;
use sg_crypto::SecretBytes;
use sg_crypto::aead::seal;
use sg_crypto::kdf::hkdf_sha256_secret;
use sg_crypto::mac::hmac_sha256;
use sg_crypto::x25519::{X25519PublicKey, X25519SecretKey};
use sg_handshake::{IdentityKeyPair, PreKeyBundle, PrekeySecrets, accept, initiate};
use sg_ratchet::Session;

type TestResult = Result<(), String>;

/// The documented session blob, parsed by hand: `version || ad_len(u16) || ad || root(32) ||
/// dh_self(32) || has_remote [remote(32)] || has_send [send(32)] || sent(u32) || has_recv
/// [recv(32)] || received(u32) || previous_chain_len(u32) || n_skipped(u16) || ...`.
struct Parsed {
    ad: Vec<u8>,
    root: [u8; 32],
    dh_self: [u8; 32],
    dh_remote: Option<[u8; 32]>,
    send: Option<[u8; 32]>,
    sent: u32,
    recv: Option<[u8; 32]>,
    received: u32,
    previous: u32,
    skipped: u16,
}

fn parse(blob: &[u8]) -> Result<Parsed, String> {
    let mut at = 0usize;
    let mut take = |n: usize| -> Result<Vec<u8>, String> {
        let end = at.checked_add(n).ok_or("overflow")?;
        let slice = blob.get(at..end).ok_or("blob too short")?.to_vec();
        at = end;
        Ok(slice)
    };
    let arr32 =
        |v: Vec<u8>| -> Result<[u8; 32], String> { v.try_into().map_err(|_| "length".to_owned()) };
    let u32_of = |v: Vec<u8>| -> Result<u32, String> {
        Ok(u32::from_be_bytes(
            v.try_into().map_err(|_| "length".to_owned())?,
        ))
    };
    assert_eq!(take(1)?, vec![1], "version");
    let ad_len = usize::from(u16::from_be_bytes(
        take(2)?.try_into().map_err(|_| "ad len")?,
    ));
    let ad = take(ad_len)?;
    let root = arr32(take(32)?)?;
    let dh_self = arr32(take(32)?)?;
    let optional = |take: &mut dyn FnMut(usize) -> Result<Vec<u8>, String>| -> Result<Option<[u8; 32]>, String> {
        match take(1)?.first().copied() {
            Some(0) => Ok(None),
            Some(1) => Ok(Some(arr32(take(32)?)?)),
            _ => Err("bad flag".to_owned()),
        }
    };
    let dh_remote = optional(&mut take)?;
    let send = optional(&mut take)?;
    let sent = u32_of(take(4)?)?;
    let recv = optional(&mut take)?;
    let received = u32_of(take(4)?)?;
    let previous = u32_of(take(4)?)?;
    let skipped = u16::from_be_bytes(take(2)?.try_into().map_err(|_| "skipped")?);
    Ok(Parsed {
        ad,
        root,
        dh_self,
        dh_remote,
        send,
        sent,
        recv,
        received,
        previous,
        skipped,
    })
}

fn secret(bytes: [u8; 32]) -> SecretBytes<32> {
    SecretBytes::new(bytes)
}

fn public_of(secret_bytes: [u8; 32]) -> [u8; 32] {
    *X25519SecretKey::from_bytes(secret_bytes)
        .public_key()
        .as_bytes()
}

fn dh(secret_bytes: [u8; 32], public: [u8; 32]) -> Result<SecretBytes<32>, String> {
    X25519SecretKey::from_bytes(secret_bytes)
        .diffie_hellman(&X25519PublicKey::from_bytes(public))
        .map_err(|e| e.to_string())
}

/// `KDF_RK`, written out from the specification text.
fn kdf_rk(root: &[u8; 32], dh_out: &SecretBytes<32>) -> Result<([u8; 32], [u8; 32]), String> {
    let out: SecretBytes<64> = hkdf_sha256_secret(root, dh_out.expose(), b"Sargoshi/DR/v1/root")
        .map_err(|e| e.to_string())?;
    let (a, b) = out.expose().split_at(32);
    Ok((
        a.try_into().map_err(|_| "split")?,
        b.try_into().map_err(|_| "split")?,
    ))
}

/// One message from a chain: returns the ciphertext and the next chain key.
fn chain_message(
    chain: &[u8; 32],
    ad: &[u8],
    dh_public: [u8; 32],
    previous: u32,
    n: u32,
    plaintext: &[u8],
) -> Result<(Vec<u8>, [u8; 32]), String> {
    let seed = hmac_sha256(chain, &[0x01]).map_err(|e| e.to_string())?;
    let next = hmac_sha256(chain, &[0x02]).map_err(|e| e.to_string())?;
    let expanded: SecretBytes<44> =
        hkdf_sha256_secret(&[], seed.expose(), b"Sargoshi/DR/v1/message")
            .map_err(|e| e.to_string())?;
    let (key, nonce) = expanded.expose().split_at(32);
    let key = secret(key.try_into().map_err(|_| "key")?);
    let nonce: [u8; 12] = nonce.try_into().map_err(|_| "nonce")?;
    let mut header = vec![1u8];
    header.extend_from_slice(&dh_public);
    header.extend_from_slice(&previous.to_be_bytes());
    header.extend_from_slice(&n.to_be_bytes());
    let mut aad = ad.to_vec();
    aad.extend_from_slice(&header);
    let ciphertext = seal(&key, &nonce, &aad, plaintext).map_err(|e| e.to_string())?;
    Ok((ciphertext, *next.expose()))
}

fn check_envelope(
    bytes: &[u8],
    expected_ciphertext: &[u8],
    dh_public: [u8; 32],
    previous: u32,
    n: u32,
) -> TestResult {
    let envelope = Envelope::decode(bytes).map_err(|e| e.to_string())?;
    assert_eq!(envelope.header.ratchet_pub, dh_public, "header ratchet key");
    assert_eq!(
        envelope.header.prev_chain_len, previous,
        "header previous chain length"
    );
    assert_eq!(envelope.header.msg_num, n, "header message number");
    assert_eq!(
        envelope.ciphertext, expected_ciphertext,
        "ciphertext differs from the specification"
    );
    Ok(())
}

#[test]
fn a_conversation_matches_the_specification_step_by_step() -> TestResult {
    // --- a real handshake, keeping copies of what the specification needs
    let alice_id = IdentityKeyPair::generate().map_err(|e| e.to_string())?;
    let bob_id = IdentityKeyPair::generate().map_err(|e| e.to_string())?;
    let (mut bob_prekeys, upload) =
        PrekeySecrets::generate(&bob_id, 1, 1, 1_700_000_000, 10, 1).map_err(|e| e.to_string())?;
    let bundle = PreKeyBundle {
        identity: bob_id.public().clone(),
        signed_prekey: upload.signed_prekey,
        pq_prekey: upload.pq_prekey,
        one_time_prekey: upload.one_time_prekeys.first().cloned(),
    };
    let started = initiate(&alice_id, &bundle).map_err(|e| e.to_string())?;
    let sk = *started.shared_key.expose();
    let spk_public = *started.remote_ratchet_public.as_bytes();
    let accepted =
        accept(&bob_id, &mut bob_prekeys, &started.initial_header).map_err(|e| e.to_string())?;
    let bob_spk_secret = *accepted.ratchet_secret.expose_bytes();
    let (mut alice, _) = Session::from_initiator(started).map_err(|e| e.to_string())?;
    let mut bob = Session::from_responder(accepted).map_err(|e| e.to_string())?;

    // --- initiator start: (root, sending chain) = KDF_RK(SK, DH(DHs, SPK_B))
    let a0 = parse(&alice.to_bytes().map_err(|e| e.to_string())?)?;
    let (root0, chain0) = kdf_rk(&sk, &dh(a0.dh_self, spk_public)?)?;
    assert_eq!(a0.root, root0, "initial root key");
    assert_eq!(a0.send, Some(chain0), "initial sending chain");
    assert_eq!(a0.dh_remote, Some(spk_public));
    assert_eq!(
        (a0.recv, a0.sent, a0.received, a0.previous),
        (None, 0, 0, 0)
    );
    let alice_pub = public_of(a0.dh_self);

    // --- responder start: root key is SK, own ratchet key is the signed prekey, nothing else
    let b0 = parse(&bob.to_bytes().map_err(|e| e.to_string())?)?;
    assert_eq!((b0.root, b0.dh_self), (sk, bob_spk_secret));
    assert_eq!((b0.dh_remote, b0.send, b0.recv), (None, None, None));
    assert_eq!(a0.ad, b0.ad, "both sides bind the same associated data");

    // --- Alice's first two messages, from the first sending chain
    let m0 = alice.encrypt(b"first").map_err(|e| e.to_string())?;
    let m1 = alice.encrypt(b"second").map_err(|e| e.to_string())?;
    let (ct0, chain1) = chain_message(&chain0, &a0.ad, alice_pub, 0, 0, b"first")?;
    let (ct1, chain2) = chain_message(&chain1, &a0.ad, alice_pub, 0, 1, b"second")?;
    check_envelope(&m0, &ct0, alice_pub, 0, 0)?;
    check_envelope(&m1, &ct1, alice_pub, 0, 1)?;
    assert_eq!(
        parse(&alice.to_bytes().map_err(|e| e.to_string())?)?.send,
        Some(chain2)
    );

    // --- Bob receives both: the DH ratchet step on his side
    assert_eq!(
        bob.decrypt(&m0).map_err(|e| e.to_string())?.as_slice(),
        b"first"
    );
    assert_eq!(
        bob.decrypt(&m1).map_err(|e| e.to_string())?.as_slice(),
        b"second"
    );
    let b1 = parse(&bob.to_bytes().map_err(|e| e.to_string())?)?;
    let (root1, recv_chain) = kdf_rk(&sk, &dh(bob_spk_secret, alice_pub)?)?;
    let (root2, bob_send_chain) = kdf_rk(&root1, &dh(b1.dh_self, alice_pub)?)?;
    let recv_after_two = hmac_sha256(
        &hmac_sha256(&recv_chain, &[2])
            .map_err(|e| e.to_string())?
            .expose()[..],
        &[2],
    )
    .map_err(|e| e.to_string())?;
    assert_eq!(b1.root, root2, "responder root key after the ratchet step");
    assert_eq!(b1.send, Some(bob_send_chain), "responder sending chain");
    assert_eq!(
        b1.recv,
        Some(*recv_after_two.expose()),
        "responder receiving chain advanced twice"
    );
    assert_eq!(
        (b1.dh_remote, b1.sent, b1.received, b1.previous, b1.skipped),
        (Some(alice_pub), 0, 2, 0, 0)
    );
    assert_ne!(
        b1.dh_self, bob_spk_secret,
        "the signed prekey is replaced by a fresh ratchet key"
    );
    let bob_pub = public_of(b1.dh_self);

    // --- Bob's reply starts his chain at message 0 with previous length 0
    let reply = bob.encrypt(b"reply").map_err(|e| e.to_string())?;
    let (ct_reply, _) = chain_message(&bob_send_chain, &b1.ad, bob_pub, 0, 0, b"reply")?;
    check_envelope(&reply, &ct_reply, bob_pub, 0, 0)?;

    // --- Alice receives it: her DH ratchet step, and PN is the length of her first chain (2)
    assert_eq!(
        alice.decrypt(&reply).map_err(|e| e.to_string())?.as_slice(),
        b"reply"
    );
    let a1 = parse(&alice.to_bytes().map_err(|e| e.to_string())?)?;
    let (a_root1, a_recv_chain) = kdf_rk(&root0, &dh(a0.dh_self, bob_pub)?)?;
    let (a_root2, a_send_chain) = kdf_rk(&a_root1, &dh(a1.dh_self, bob_pub)?)?;
    assert_eq!(
        a1.root, a_root2,
        "initiator root key after the ratchet step"
    );
    assert_eq!(a1.send, Some(a_send_chain), "initiator new sending chain");
    assert_eq!(
        a1.recv,
        Some(
            *hmac_sha256(&a_recv_chain, &[2])
                .map_err(|e| e.to_string())?
                .expose()
        )
    );
    assert_eq!(
        (a1.sent, a1.received, a1.previous),
        (0, 1, 2),
        "PN equals the old chain length"
    );
    let alice_new_pub = public_of(a1.dh_self);
    assert_ne!(alice_new_pub, alice_pub);

    // --- Alice's next message announces the previous chain length 2
    let again = alice.encrypt(b"again").map_err(|e| e.to_string())?;
    let (ct_again, _) = chain_message(&a_send_chain, &a1.ad, alice_new_pub, 2, 0, b"again")?;
    check_envelope(&again, &ct_again, alice_new_pub, 2, 0)?;
    assert_eq!(
        bob.decrypt(&again).map_err(|e| e.to_string())?.as_slice(),
        b"again"
    );
    Ok(())
}

//! The post-quantum hook: a pure extension point that mixes an optional extra secret into the
//! root key derivation. Sargoshi ships only the no-op default; these tests prove the seam works.
#![allow(
    clippy::expect_used,
    reason = "test helpers: a failed expectation is the intended test failure"
)]

mod common;

use std::cell::RefCell;

use common::pair_with;
use sg_crypto::SecretBytes;
use sg_ratchet::{NoPqRatchet, PqRatchetHook, RatchetDirection, RatchetStep};

/// A hook that supplies the same fixed secret for every step.
struct Fixed([u8; 32]);

impl PqRatchetHook for Fixed {
    fn extra_root_secret(&self, _step: &RatchetStep<'_>) -> Option<SecretBytes<32>> {
        Some(SecretBytes::new(self.0))
    }
}

/// A hook whose secret is the XOR of the two ratchet public keys of the step. Both sides of a
/// step see the same pair with the roles swapped, so they agree only if the context is oriented
/// correctly.
struct FromPublicKeys;

impl PqRatchetHook for FromPublicKeys {
    fn extra_root_secret(&self, step: &RatchetStep<'_>) -> Option<SecretBytes<32>> {
        let mut out = [0u8; 32];
        for (o, (a, b)) in out.iter_mut().zip(
            step.local_ratchet_public
                .iter()
                .zip(step.remote_ratchet_public),
        ) {
            *o = a ^ b;
        }
        Some(SecretBytes::new(out))
    }
}

struct Recorder(RefCell<Vec<RatchetDirection>>);

impl PqRatchetHook for Recorder {
    fn extra_root_secret(&self, step: &RatchetStep<'_>) -> Option<SecretBytes<32>> {
        self.0.borrow_mut().push(step.direction);
        None
    }
}

fn converse(
    alice: &mut sg_ratchet::Session,
    bob: &mut sg_ratchet::Session,
    alice_hook: &dyn PqRatchetHook,
    bob_hook: &dyn PqRatchetHook,
    rounds: usize,
) {
    for i in 0..rounds {
        let a = alice.encrypt(format!("a{i}").as_bytes()).expect("encrypt");
        assert_eq!(
            bob.decrypt_with_hook(&a, bob_hook)
                .expect("bob decrypts")
                .as_slice(),
            format!("a{i}").as_bytes()
        );
        let b = bob.encrypt(format!("b{i}").as_bytes()).expect("encrypt");
        assert_eq!(
            alice
                .decrypt_with_hook(&b, alice_hook)
                .expect("alice decrypts")
                .as_slice(),
            format!("b{i}").as_bytes()
        );
    }
}

#[test]
fn sessions_that_share_a_hook_secret_converse_normally() {
    let hook = Fixed([0x42; 32]);
    let (mut alice, mut bob) = pair_with(&hook, &hook);
    converse(&mut alice, &mut bob, &hook, &hook, 10);
}

#[test]
fn the_context_is_oriented_consistently_on_both_sides() {
    let hook = FromPublicKeys;
    let (mut alice, mut bob) = pair_with(&hook, &hook);
    converse(&mut alice, &mut bob, &hook, &hook, 10);
}

#[test]
fn a_hook_used_by_only_one_side_breaks_authentication_without_damage() {
    let hook = Fixed([0x42; 32]);
    let (mut alice, mut bob) = pair_with(&hook, &NoPqRatchet);
    let first = alice.encrypt(b"x").expect("encrypt");
    let before = bob.to_bytes().expect("blob").to_vec();
    assert!(bob.decrypt(&first).is_err());
    assert_eq!(bob.to_bytes().expect("blob").to_vec(), before);
}

#[test]
fn different_hook_secrets_do_not_interoperate() {
    let (mut alice, mut bob) = pair_with(&Fixed([1; 32]), &Fixed([1; 32]));
    let first = alice.encrypt(b"x").expect("encrypt");
    assert!(bob.decrypt_with_hook(&first, &Fixed([2; 32])).is_err());
    assert!(bob.decrypt_with_hook(&first, &Fixed([1; 32])).is_ok());
}

#[test]
fn the_hook_is_consulted_once_per_dh_step_and_never_inside_a_chain() {
    let recorder = Recorder(RefCell::new(Vec::new()));
    let (mut alice, mut bob) = pair_with(&recorder, &NoPqRatchet);
    assert_eq!(
        *recorder.0.borrow(),
        vec![RatchetDirection::Sending],
        "initiator start"
    );
    recorder.0.borrow_mut().clear();

    let a0 = alice.encrypt(b"0").expect("encrypt");
    let a1 = alice.encrypt(b"1").expect("encrypt");
    // Bob's first message from a new remote key is a ratchet step: receive then send.
    bob.decrypt_with_hook(&a0, &recorder).expect("a0");
    assert_eq!(
        *recorder.0.borrow(),
        vec![RatchetDirection::Receiving, RatchetDirection::Sending]
    );
    recorder.0.borrow_mut().clear();
    // Further messages in the same chain never consult it.
    bob.decrypt_with_hook(&a1, &recorder).expect("a1");
    assert!(recorder.0.borrow().is_empty());
}

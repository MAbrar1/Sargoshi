//! Prekey generation, rotation, pruning, upload bundles and the secret blob.
#![allow(
    clippy::expect_used,
    reason = "test helpers: a failed expectation is the intended test failure"
)]

mod common;

use common::party;
use sg_common::constants::{
    MAX_ONE_TIME_PREKEYS_PER_UPLOAD, MAX_STORED_ONE_TIME_PREKEYS, MAX_STORED_SIGNED_PREKEYS,
    ONE_TIME_PREKEY_REPLENISH_BELOW,
};
use sg_handshake::{HandshakeError, IdentityKeyPair, PrekeySecrets, UploadBundle};

#[test]
fn generated_prekeys_are_signed_by_the_identity() {
    let p = party();
    let signing = p.identity.public().signing_key();
    assert_eq!(p.upload.signed_prekey.verify(signing), Ok(()));
    assert_eq!(p.upload.pq_prekey.verify(signing), Ok(()));
    assert_eq!(p.upload.one_time_prekeys.len(), 5);
    let ids: Vec<u32> = p.upload.one_time_prekeys.iter().map(|k| k.id).collect();
    assert_eq!(ids, vec![100, 101, 102, 103, 104]);
    assert_eq!(p.upload.verify(p.identity.public()), Ok(()));
}

#[test]
fn another_identity_cannot_vouch_for_the_prekeys() {
    let p = party();
    let stranger = IdentityKeyPair::generate().expect("identity");
    assert_eq!(
        p.upload.verify(stranger.public()),
        Err(HandshakeError::InvalidSignedPreKeySignature)
    );
}

#[test]
fn upload_bundle_wire_round_trip_verifies_signatures() {
    let p = party();
    let bytes = p.upload.to_bytes().expect("encode");
    assert_eq!(
        UploadBundle::from_bytes(&bytes, p.identity.public()),
        Ok(p.upload.clone())
    );
    let stranger = IdentityKeyPair::generate().expect("identity");
    assert!(UploadBundle::from_bytes(&bytes, stranger.public()).is_err());
}

#[test]
fn one_time_prekey_replenishment_follows_the_thresholds() {
    let mut p = party();
    assert_eq!(p.prekeys.one_time_remaining(), 5);
    assert!(p.prekeys.needs_replenishment());
    let added = p
        .prekeys
        .replenish_one_time_prekeys(1_000, 100)
        .expect("replenish");
    assert_eq!(added.len(), 100);
    assert_eq!(p.prekeys.one_time_remaining(), 105);
    assert!(!p.prekeys.needs_replenishment());
    assert_eq!(ONE_TIME_PREKEY_REPLENISH_BELOW, 25);
}

#[test]
fn identifier_and_size_limits_are_enforced() {
    let mut p = party();
    assert_eq!(
        p.prekeys.replenish_one_time_prekeys(100, 1).err(),
        Some(HandshakeError::DuplicateKeyId)
    );
    assert_eq!(
        p.prekeys.replenish_one_time_prekeys(u32::MAX - 1, 5).err(),
        Some(HandshakeError::DuplicateKeyId)
    );
    assert_eq!(
        p.prekeys
            .replenish_one_time_prekeys(5_000, MAX_ONE_TIME_PREKEYS_PER_UPLOAD + 1)
            .err(),
        Some(HandshakeError::TooManyKeys)
    );
    let mut next = 10_000;
    while p.prekeys.one_time_remaining() + MAX_ONE_TIME_PREKEYS_PER_UPLOAD
        <= MAX_STORED_ONE_TIME_PREKEYS
    {
        p.prekeys
            .replenish_one_time_prekeys(next, MAX_ONE_TIME_PREKEYS_PER_UPLOAD)
            .expect("fill");
        next += 1_000;
    }
    assert_eq!(
        p.prekeys
            .replenish_one_time_prekeys(next, MAX_ONE_TIME_PREKEYS_PER_UPLOAD)
            .err(),
        Some(HandshakeError::TooManyKeys)
    );
    // Signed prekey identifiers must be unique too.
    assert_eq!(
        p.prekeys.rotate_signed_prekeys(&p.identity, 1, 2, 1).err(),
        Some(HandshakeError::DuplicateKeyId)
    );
    assert_eq!(
        p.prekeys.rotate_signed_prekeys(&p.identity, 2, 1, 1).err(),
        Some(HandshakeError::DuplicateKeyId)
    );
}

#[test]
fn rotation_keeps_old_keys_until_pruned_and_never_removes_the_newest() {
    let mut p = party();
    let (spk2, pq2) = p
        .prekeys
        .rotate_signed_prekeys(&p.identity, 2, 2, 1_700_100_000)
        .expect("rotate");
    assert_eq!(spk2.verify(p.identity.public().signing_key()), Ok(()));
    assert_eq!(pq2.verify(p.identity.public().signing_key()), Ok(()));
    // The upload bundle now advertises the newest pair.
    let upload = p.prekeys.upload_bundle(&p.identity).expect("bundle");
    assert_eq!((upload.signed_prekey.id, upload.pq_prekey.id), (2, 2));

    assert_eq!(
        p.prekeys.prune_before(1_700_050_000),
        2,
        "old signed and pq keys are removed"
    );
    assert_eq!(
        p.prekeys.prune_before(u64::MAX),
        0,
        "the newest keys are always kept"
    );
    let upload = p.prekeys.upload_bundle(&p.identity).expect("bundle");
    assert_eq!((upload.signed_prekey.id, upload.pq_prekey.id), (2, 2));
}

#[test]
fn signed_prekey_collection_has_a_limit() {
    let mut p = party();
    for i in 2..=u32::try_from(MAX_STORED_SIGNED_PREKEYS).expect("fits") {
        p.prekeys
            .rotate_signed_prekeys(&p.identity, i, i, u64::from(i))
            .expect("rotate");
    }
    assert_eq!(
        p.prekeys
            .rotate_signed_prekeys(&p.identity, 99, 99, 99)
            .err(),
        Some(HandshakeError::TooManyKeys)
    );
}

#[test]
fn secret_blob_round_trips_exactly() {
    let mut p = party();
    p.prekeys
        .rotate_signed_prekeys(&p.identity, 2, 2, 1_700_100_000)
        .expect("rotate");
    let blob = p.prekeys.to_bytes().expect("blob");
    let restored = PrekeySecrets::from_bytes(&blob).expect("restore");
    assert_eq!(
        restored.to_bytes().expect("blob").as_slice(),
        blob.as_slice()
    );
    assert_eq!(
        restored.one_time_remaining(),
        p.prekeys.one_time_remaining()
    );
    // The restored store signs identical bundles (Ed25519 is deterministic).
    assert_eq!(
        restored.upload_bundle(&p.identity).expect("bundle"),
        p.prekeys.upload_bundle(&p.identity).expect("bundle")
    );
}

#[test]
fn secret_blob_parser_rejects_every_truncation_and_trailing_data() {
    let p = party();
    let blob = p.prekeys.to_bytes().expect("blob");
    for cut in 0..blob.len() {
        assert!(
            PrekeySecrets::from_bytes(&blob[..cut]).is_err(),
            "cut {cut} was accepted"
        );
    }
    let mut longer = blob.to_vec();
    longer.push(0);
    assert_eq!(
        PrekeySecrets::from_bytes(&longer).err(),
        Some(HandshakeError::MalformedBlob)
    );
    let mut wrong_version = blob.to_vec();
    wrong_version[0] = 3;
    assert!(matches!(
        PrekeySecrets::from_bytes(&wrong_version),
        Err(HandshakeError::Proto(_))
    ));
}

#[test]
fn secret_blob_parser_rejects_oversized_counts_and_duplicate_ids() {
    let p = party();
    let blob = p.prekeys.to_bytes().expect("blob");
    let mut huge_signed = blob.to_vec();
    huge_signed[1] = 0xff;
    huge_signed[2] = 0xff;
    assert_eq!(
        PrekeySecrets::from_bytes(&huge_signed).err(),
        Some(HandshakeError::TooManyKeys)
    );

    // Two signed prekeys with the same identifier: take a two-key blob and copy the first id.
    let mut q = party();
    q.prekeys
        .rotate_signed_prekeys(&q.identity, 2, 2, 5)
        .expect("rotate");
    let mut dup = q.prekeys.to_bytes().expect("blob").to_vec();
    let second_id_at = 1 + 2 + (4 + 8 + 32);
    dup[second_id_at..second_id_at + 4].copy_from_slice(&1u32.to_be_bytes());
    assert_eq!(
        PrekeySecrets::from_bytes(&dup).err(),
        Some(HandshakeError::DuplicateKeyId)
    );
}

#[test]
fn prekey_secrets_never_print_key_material() {
    let p = party();
    let shown = format!("{:?}", p.prekeys);
    assert!(shown.contains("redacted"));
    assert!(shown.contains("1 signed"));
}

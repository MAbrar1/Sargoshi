//! Shared helpers for the handshake tests.
#![allow(
    dead_code,
    missing_docs,
    clippy::expect_used,
    reason = "test helpers: a failed expectation is the intended test failure"
)]

use sg_handshake::{
    IdentityKeyPair, OneTimePreKeyPublic, PreKeyBundle, PrekeySecrets, UploadBundle,
};

pub struct Party {
    pub identity: IdentityKeyPair,
    pub prekeys: PrekeySecrets,
    pub upload: UploadBundle,
}

pub fn party() -> Party {
    let identity = IdentityKeyPair::generate().expect("identity");
    let (prekeys, upload) =
        PrekeySecrets::generate(&identity, 1, 1, 1_700_000_000, 100, 5).expect("prekeys");
    Party {
        identity,
        prekeys,
        upload,
    }
}

/// The bundle a server would hand out for this party, optionally with one of its one-time keys.
pub fn bundle(party: &Party, one_time: Option<OneTimePreKeyPublic>) -> PreKeyBundle {
    PreKeyBundle {
        identity: party.identity.public().clone(),
        signed_prekey: party.upload.signed_prekey.clone(),
        pq_prekey: party.upload.pq_prekey.clone(),
        one_time_prekey: one_time,
    }
}

pub fn first_one_time(party: &Party) -> OneTimePreKeyPublic {
    party
        .upload
        .one_time_prekeys
        .first()
        .cloned()
        .expect("one-time prekey")
}

//! Checks the constants against the authoritative specification text and against each other.

use sg_common::constants::*;
use sg_common::{ErrorCode, PROTOCOL_VERSION, check_version};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn blueprint() -> Result<String, std::io::Error> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../docs/blueprint/Sargoshi-Final-Blueprint-v5.md"
    );
    std::fs::read_to_string(path)
}

#[test]
fn every_label_appears_verbatim_in_the_blueprint() -> TestResult {
    let text = blueprint()?;
    for label in [
        LABEL_IK_DH_SIG,
        LABEL_SPK_SIG,
        LABEL_PQSPK_SIG,
        INFO_PQXDH,
        INFO_DR_ROOT,
        INFO_DR_MESSAGE,
        INFO_BACKUP,
    ] {
        let s = std::str::from_utf8(label)?;
        assert!(text.contains(s), "label {s} is not in the blueprint");
    }
    Ok(())
}

#[test]
fn numeric_limits_match_the_blueprint_wording() -> TestResult {
    let text = blueprint()?;
    for needle in [
        "100 per device, replenished when fewer than 25 remain",
        "Maximum skipped message keys 1,000 per session and 2,000 cached",
        "next multiple of 160 bytes",
        "public key 1,184 bytes, ciphertext 1,088 bytes",
        "SHA-512 iterated 5,200 times",
        "24-word recovery phrase",
        "1 MiB chunks",
        "random 16-byte salt",
    ] {
        assert!(text.contains(needle), "spec wording not found: {needle}");
    }
    assert_eq!(ONE_TIME_PREKEYS_PER_DEVICE, 100);
    assert_eq!(ONE_TIME_PREKEY_REPLENISH_BELOW, 25);
    assert_eq!(MAX_SKIPPED_KEYS_PER_SESSION, 1_000);
    assert_eq!(MAX_CACHED_SKIPPED_KEYS, 2_000);
    assert_eq!(PADDING_BLOCK_BYTES, 160);
    assert_eq!(MLKEM768_ENCAPSULATION_KEY_LEN, 1184);
    assert_eq!(MLKEM768_CIPHERTEXT_LEN, 1088);
    assert_eq!(SAFETY_NUMBER_ITERATIONS, 5_200);
    assert_eq!(RECOVERY_PHRASE_WORDS, 24);
    assert_eq!(BACKUP_CHUNK_BYTES, 1_048_576);
    assert_eq!(ARGON2_SALT_LEN, 16);
    Ok(())
}

#[test]
fn safety_number_shape_is_twelve_groups_of_five() {
    assert_eq!(SAFETY_NUMBER_DIGITS, 12 * SAFETY_NUMBER_GROUP_DIGITS);
}

#[test]
fn labels_are_distinct_and_namespaced() {
    let all = [
        LABEL_IK_DH_SIG,
        LABEL_SPK_SIG,
        LABEL_PQSPK_SIG,
        INFO_PQXDH,
        INFO_DR_ROOT,
        INFO_DR_MESSAGE,
        INFO_BACKUP,
    ];
    for (i, a) in all.iter().enumerate() {
        assert!(a.starts_with(b"Sargoshi/"));
        for b in &all[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

#[test]
fn argon2_floor_is_not_below_the_documented_minimum() {
    const {
        assert!(ARGON2_MIN_MEMORY_KIB >= 19 * 1024);
        assert!(ARGON2_MIN_ITERATIONS >= 2);
        assert!(ARGON2_MIN_PARALLELISM >= 1);
    }
}

#[test]
fn stream_label_is_distinct_from_every_blueprint_label() {
    let blueprint_labels = [
        LABEL_IK_DH_SIG,
        LABEL_SPK_SIG,
        LABEL_PQSPK_SIG,
        INFO_PQXDH,
        INFO_DR_ROOT,
        INFO_DR_MESSAGE,
        INFO_BACKUP,
    ];
    assert!(INFO_STREAM.starts_with(b"Sargoshi/"));
    assert!(!blueprint_labels.contains(&INFO_STREAM));
    assert_eq!(STREAM_HEADER_LEN, 17);
    assert_eq!(HKDF_SHA256_MAX_OUTPUT, 8160);
}

#[test]
fn only_version_one_is_accepted() {
    assert_eq!(PROTOCOL_VERSION, 1);
    for v in 0..=u8::MAX {
        assert_eq!(check_version(v).is_ok(), v == 1, "version {v}");
    }
}

#[test]
fn error_codes_are_unique_stable_and_round_trip() {
    let mut seen = std::collections::HashSet::new();
    for code in ErrorCode::ALL {
        assert!(seen.insert(code.as_u16()), "duplicate code");
        assert_eq!(ErrorCode::from_u16(code.as_u16()), Some(code));
        assert!((1000..4000).contains(&code.as_u16()));
    }
    assert_eq!(ErrorCode::from_u16(0), None);
    assert_eq!(ErrorCode::from_u16(999), None);
    assert_eq!(ErrorCode::from_u16(2008), None);
    assert_eq!(ErrorCode::from_u16(3011), None);
    assert_eq!(ErrorCode::from_u16(4000), None);
    // Released codes never change: this pins the wire values.
    assert_eq!(ErrorCode::UnsupportedVersion.as_u16(), 1000);
    assert_eq!(ErrorCode::ReservedFieldSet.as_u16(), 1006);
    assert_eq!(ErrorCode::RandomFailure.as_u16(), 2000);
    assert_eq!(ErrorCode::StreamBadChunk.as_u16(), 2012);
    assert_eq!(ErrorCode::InvalidIdentity.as_u16(), 3000);
    assert_eq!(ErrorCode::DuplicateKeyId.as_u16(), 3010);
}

//! Adversarial tests for the streaming AEAD: round trips at every boundary, and every way a
//! hostile storage layer could truncate, reorder, duplicate, splice or modify a stream.
#![allow(
    clippy::expect_used,
    reason = "test helpers: a failed expectation is the intended test failure"
)]

use proptest::prelude::*;
use sg_common::constants::{AEAD_TAG_LEN, MAX_STREAM_CHUNK_BYTES, STREAM_HEADER_LEN};
use sg_crypto::CryptoError;
use sg_crypto::secret::SecretBytes;
use sg_crypto::stream::{StreamDecryptor, StreamEncryptor};

const CHUNK: usize = 64;

fn key() -> SecretBytes<32> {
    SecretBytes::new([7u8; 32])
}

/// Encrypt `data` as a stream. When `empty_final` is true and the data is an exact multiple of
/// the chunk size, the final chunk is empty; otherwise the last piece carries the final flag.
fn encrypt(data: &[u8], aad: &[u8], empty_final: bool) -> (Vec<u8>, Vec<Vec<u8>>) {
    let mut enc = StreamEncryptor::new(&key(), CHUNK, aad).expect("encryptor");
    let header = enc.header().to_vec();
    let mut pieces: Vec<&[u8]> = data.chunks(CHUNK).collect();
    if pieces.is_empty() || (empty_final && data.len().is_multiple_of(CHUNK)) {
        pieces.push(&[]);
    }
    let last = pieces.len() - 1;
    let chunks = pieces
        .iter()
        .enumerate()
        .map(|(i, piece)| enc.encrypt_chunk(piece, i == last).expect("chunk"))
        .collect();
    (header, chunks)
}

fn decrypt_all(header: &[u8], chunks: &[Vec<u8>], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let mut dec = StreamDecryptor::new(&key(), header, CHUNK, aad)?;
    let mut out = Vec::new();
    for chunk in chunks {
        let (plain, _) = dec.decrypt_chunk(chunk)?;
        out.extend_from_slice(&plain);
    }
    dec.finish()?;
    Ok(out)
}

proptest! {
    #[test]
    fn round_trips_at_every_length(data in proptest::collection::vec(any::<u8>(), 0..400), empty_final in any::<bool>()) {
        let (header, chunks) = encrypt(&data, b"aad", empty_final);
        prop_assert_eq!(header.len(), STREAM_HEADER_LEN);
        prop_assert_eq!(decrypt_all(&header, &chunks, b"aad").unwrap(), data);
    }
}

#[test]
fn boundary_lengths_round_trip_with_both_final_styles() {
    for len in [0, 1, CHUNK - 1, CHUNK, CHUNK + 1, 2 * CHUNK, 3 * CHUNK + 5] {
        let data: Vec<u8> = (0..len)
            .map(|i| u8::try_from(i % 251).expect("fits"))
            .collect();
        for empty_final in [false, true] {
            let (header, chunks) = encrypt(&data, b"", empty_final);
            assert_eq!(
                decrypt_all(&header, &chunks, b""),
                Ok(data.clone()),
                "len {len}"
            );
        }
    }
}

#[test]
fn a_full_size_final_chunk_is_reported_as_final() {
    let data = vec![1u8; CHUNK];
    let (header, chunks) = encrypt(&data, b"", false);
    assert_eq!(chunks.len(), 1);
    let mut dec = StreamDecryptor::new(&key(), &header, CHUNK, b"").expect("decryptor");
    let (plain, is_final) = dec.decrypt_chunk(&chunks[0]).expect("chunk");
    assert!(is_final);
    assert_eq!(plain.as_slice(), data.as_slice());
    assert!(dec.is_finished());
}

#[test]
fn dropping_the_final_chunk_is_detected_as_truncation() {
    let data = vec![3u8; CHUNK * 3 + 10];
    let (header, mut chunks) = encrypt(&data, b"", false);
    chunks.pop();
    assert_eq!(
        decrypt_all(&header, &chunks, b""),
        Err(CryptoError::StreamTruncated)
    );
}

#[test]
fn dropping_a_middle_chunk_fails_authentication() {
    let (header, mut chunks) = encrypt(&[3u8; CHUNK * 4], b"", true);
    chunks.remove(1);
    assert_eq!(
        decrypt_all(&header, &chunks, b""),
        Err(CryptoError::DecryptionFailed)
    );
}

#[test]
fn reordering_chunks_fails_authentication() {
    let (header, mut chunks) = encrypt(&[3u8; CHUNK * 4], b"", true);
    chunks.swap(0, 1);
    assert_eq!(
        decrypt_all(&header, &chunks, b""),
        Err(CryptoError::DecryptionFailed)
    );
}

#[test]
fn duplicating_a_chunk_fails_authentication() {
    let (header, mut chunks) = encrypt(&[3u8; CHUNK * 4], b"", true);
    let copy = chunks[1].clone();
    chunks.insert(2, copy);
    assert_eq!(
        decrypt_all(&header, &chunks, b""),
        Err(CryptoError::DecryptionFailed)
    );
}

#[test]
fn chunks_after_the_final_chunk_are_refused() {
    let (header, chunks) = encrypt(&[3u8; CHUNK + 3], b"", false);
    let mut dec = StreamDecryptor::new(&key(), &header, CHUNK, b"").expect("decryptor");
    for chunk in &chunks {
        dec.decrypt_chunk(chunk).expect("chunk");
    }
    assert_eq!(
        dec.decrypt_chunk(&chunks[0]).err(),
        Some(CryptoError::StreamChunkOrder)
    );
}

#[test]
fn every_single_byte_change_anywhere_is_detected() {
    let data = vec![9u8; CHUNK * 2 + 20];
    let (header, chunks) = encrypt(&data, b"ctx", false);
    for (ci, chunk) in chunks.iter().enumerate() {
        for bi in 0..chunk.len() {
            let mut bad = chunks.clone();
            bad[ci][bi] ^= 0x01;
            assert!(
                decrypt_all(&header, &bad, b"ctx").is_err(),
                "chunk {ci} byte {bi}"
            );
        }
    }
    for bi in 0..header.len() {
        let mut bad = header.clone();
        bad[bi] ^= 0x01;
        assert!(
            decrypt_all(&bad, &chunks, b"ctx").is_err(),
            "header byte {bi}"
        );
    }
}

#[test]
fn wrong_key_wrong_aad_and_wrong_chunk_size_are_refused() {
    let (header, chunks) = encrypt(&[5u8; CHUNK * 2 + 1], b"right", false);
    assert!(decrypt_all(&header, &chunks, b"wrong").is_err());
    assert!(decrypt_all(&header, &chunks, b"").is_err());
    let mut other_key =
        StreamDecryptor::new(&SecretBytes::new([8u8; 32]), &header, CHUNK, b"right")
            .expect("decryptor");
    assert!(other_key.decrypt_chunk(&chunks[0]).is_err());
    let mut other_size =
        StreamDecryptor::new(&key(), &header, CHUNK * 2, b"right").expect("decryptor");
    assert!(other_size.decrypt_chunk(&chunks[0]).is_err());
}

#[test]
fn a_failed_chunk_does_not_advance_the_decryptor() {
    let (header, chunks) = encrypt(&[5u8; CHUNK * 2], b"", true);
    let mut dec = StreamDecryptor::new(&key(), &header, CHUNK, b"").expect("decryptor");
    assert!(dec.decrypt_chunk(&chunks[1]).is_err());
    assert!(dec.decrypt_chunk(&chunks[0]).is_ok());
    assert!(dec.decrypt_chunk(&chunks[1]).is_ok());
}

#[test]
fn chunks_of_different_streams_cannot_be_spliced() {
    let data = vec![6u8; CHUNK * 3];
    let (header_a, chunks_a) = encrypt(&data, b"", true);
    let (_, chunks_b) = encrypt(&data, b"", true);
    let mut mixed = chunks_a.clone();
    mixed[1] = chunks_b[1].clone();
    assert!(decrypt_all(&header_a, &mixed, b"").is_err());
}

#[test]
fn two_streams_under_one_key_never_share_ciphertext() {
    let data = vec![1u8; CHUNK];
    let (header_a, chunks_a) = encrypt(&data, b"", false);
    let (header_b, chunks_b) = encrypt(&data, b"", false);
    assert_ne!(header_a, header_b);
    assert_ne!(chunks_a[0], chunks_b[0]);
}

#[test]
fn header_problems_are_reported_precisely() {
    let (header, chunks) = encrypt(b"hello", b"", false);
    assert!(matches!(
        StreamDecryptor::new(&key(), &header[..header.len() - 1], CHUNK, b""),
        Err(CryptoError::InvalidLength)
    ));
    let mut wrong_version = header.clone();
    wrong_version[0] = 2;
    assert!(matches!(
        StreamDecryptor::new(&key(), &wrong_version, CHUNK, b""),
        Err(CryptoError::UnsupportedVersion)
    ));
    assert_eq!(decrypt_all(&header, &chunks, b""), Ok(b"hello".to_vec()));
}

#[test]
fn encryptor_enforces_chunk_rules() {
    assert!(matches!(
        StreamEncryptor::new(&key(), 0, b""),
        Err(CryptoError::StreamBadChunk)
    ));
    assert!(matches!(
        StreamEncryptor::new(&key(), MAX_STREAM_CHUNK_BYTES + 1, b""),
        Err(CryptoError::StreamBadChunk)
    ));
    let mut enc = StreamEncryptor::new(&key(), CHUNK, b"").expect("encryptor");
    assert_eq!(
        enc.encrypt_chunk(&[0u8; CHUNK - 1], false),
        Err(CryptoError::StreamBadChunk)
    );
    assert_eq!(
        enc.encrypt_chunk(&[0u8; CHUNK + 1], true),
        Err(CryptoError::StreamBadChunk)
    );
    assert!(enc.encrypt_chunk(&[0u8; CHUNK], false).is_ok());
    assert!(enc.encrypt_chunk(&[], true).is_ok());
    assert_eq!(
        enc.encrypt_chunk(&[], true),
        Err(CryptoError::StreamChunkOrder)
    );
}

#[test]
fn decryptor_refuses_impossible_chunk_lengths() {
    let (header, _) = encrypt(b"x", b"", false);
    let mut dec = StreamDecryptor::new(&key(), &header, CHUNK, b"").expect("decryptor");
    assert_eq!(
        dec.decrypt_chunk(&[0u8; CHUNK + AEAD_TAG_LEN + 1]).err(),
        Some(CryptoError::StreamBadChunk)
    );
    assert_eq!(
        dec.decrypt_chunk(&[0u8; AEAD_TAG_LEN - 1]).err(),
        Some(CryptoError::StreamBadChunk)
    );
}

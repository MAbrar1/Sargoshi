//! Conformance of the streaming AEAD to its written specification. The expected ciphertext is
//! rebuilt here from the formulas in `sg-crypto/src/stream.rs` and docs/architecture/PROTOCOL.md
//! using only HKDF and ChaCha20-Poly1305, then compared with the real encryptor's output.
use sg_common::constants::INFO_STREAM;
use sg_crypto::SecretBytes;
use sg_crypto::aead::seal;
use sg_crypto::kdf::hkdf_sha256_secret;
use sg_crypto::stream::{StreamDecryptor, StreamEncryptor};

type TestResult = Result<(), String>;

fn expected_chunk(
    master: &SecretBytes<32>,
    stream_id: &[u8; 16],
    chunk_size: u32,
    caller_aad: &[u8],
    index: u64,
    is_final: bool,
    plaintext: &[u8],
) -> Result<Vec<u8>, String> {
    let mut header = vec![1u8];
    header.extend_from_slice(stream_id);
    let key: SecretBytes<32> =
        hkdf_sha256_secret(stream_id, master.expose(), INFO_STREAM).map_err(|e| e.to_string())?;
    let mut nonce = [0u8; 12];
    nonce
        .iter_mut()
        .skip(4)
        .zip(index.to_be_bytes())
        .for_each(|(n, b)| *n = b);
    let mut aad = header;
    aad.extend_from_slice(&chunk_size.to_be_bytes());
    aad.extend_from_slice(&index.to_be_bytes());
    aad.push(u8::from(is_final));
    aad.extend_from_slice(
        &u32::try_from(caller_aad.len())
            .map_err(|e| e.to_string())?
            .to_be_bytes(),
    );
    aad.extend_from_slice(caller_aad);
    seal(&key, &nonce, &aad, plaintext).map_err(|e| e.to_string())
}

#[test]
fn encryptor_output_matches_the_written_construction() -> TestResult {
    let master = SecretBytes::new([0x5au8; 32]);
    let stream_id = [0xc3u8; 16];
    let aad = b"sargoshi-backup-test";
    let chunks: [(&[u8], bool); 3] = [(&[1u8; 32], false), (&[2u8; 32], false), (&[3u8; 5], true)];

    let mut enc = StreamEncryptor::new_with_stream_id(&master, stream_id, 32, aad)
        .map_err(|e| e.to_string())?;
    let mut expected_header = vec![1u8];
    expected_header.extend_from_slice(&stream_id);
    assert_eq!(enc.header().as_slice(), expected_header.as_slice());

    let mut sealed = Vec::new();
    for (i, (plaintext, is_final)) in chunks.iter().enumerate() {
        let index = u64::try_from(i).map_err(|e| e.to_string())?;
        let actual = enc
            .encrypt_chunk(plaintext, *is_final)
            .map_err(|e| e.to_string())?;
        let expected = expected_chunk(&master, &stream_id, 32, aad, index, *is_final, plaintext)?;
        assert_eq!(actual, expected, "chunk {i}");
        sealed.push(actual);
    }

    let mut dec =
        StreamDecryptor::new(&master, &expected_header, 32, aad).map_err(|e| e.to_string())?;
    for (i, ((plaintext, is_final), chunk)) in chunks.iter().zip(&sealed).enumerate() {
        let (opened, final_flag) = dec
            .decrypt_chunk(chunk)
            .map_err(|e| format!("chunk {i}: {e}"))?;
        assert_eq!(opened.as_slice(), *plaintext);
        assert_eq!(final_flag, *is_final);
    }
    dec.finish().map_err(|e| e.to_string())
}

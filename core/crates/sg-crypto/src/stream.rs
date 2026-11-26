//! Streaming AEAD for large data (vault files and backups, Prompt C).
//!
//! Construction (a Sargoshi design built only from HKDF-SHA256 and ChaCha20-Poly1305; it is
//! listed for cryptographer review in docs/architecture/AUDIT-PREP.md):
//!
//! * A stream starts with a 17-byte header: `version || stream_id(16 random bytes)`.
//! * The per-stream key is `HKDF-SHA256(salt = stream_id, ikm = master_key, info =
//!   "Sargoshi/Stream/v1")`, so the same master key can protect many streams without ever
//!   repeating a (key, nonce) pair.
//! * Chunk `i` is sealed with nonce `0x00000000 || i (u64 big-endian)` and associated data
//!   `header || chunk_size(u32) || i(u64) || final_flag(1) || len(caller_aad)(u32) || caller_aad`.
//! * Every chunk except the last carries exactly `chunk_size` bytes; the last carries at most
//!   `chunk_size` bytes (possibly zero) and has the final flag set.
//!
//! Reordering, dropping, duplicating or truncating chunks, changing the chunk size and
//! mixing chunks of different streams all fail authentication; a stream that ends without its
//! final chunk is reported as truncated by [`StreamDecryptor::finish`].

use sg_common::constants::{
    AEAD_KEY_LEN, AEAD_NONCE_LEN, AEAD_TAG_LEN, INFO_STREAM, MAX_STREAM_CHUNK_BYTES,
    MAX_STREAM_CHUNKS, STREAM_HEADER_LEN, STREAM_ID_LEN,
};
use sg_common::{PROTOCOL_VERSION, check_version};
use zeroize::Zeroizing;

use crate::aead::{open, seal};
use crate::error::CryptoError;
use crate::kdf::hkdf_sha256_secret;
use crate::rng::random_array;
use crate::secret::SecretBytes;

/// Serialized stream header.
pub type StreamHeader = [u8; STREAM_HEADER_LEN];

fn validate_chunk_size(chunk_size: usize) -> Result<u32, CryptoError> {
    if chunk_size == 0 || chunk_size > MAX_STREAM_CHUNK_BYTES {
        return Err(CryptoError::StreamBadChunk);
    }
    u32::try_from(chunk_size).map_err(|_| CryptoError::StreamBadChunk)
}

fn derive_stream_key(
    master: &SecretBytes<AEAD_KEY_LEN>,
    header: &StreamHeader,
) -> Result<SecretBytes<AEAD_KEY_LEN>, CryptoError> {
    let (_, stream_id) = header.split_first().ok_or(CryptoError::InvalidLength)?;
    hkdf_sha256_secret(stream_id, master.expose(), INFO_STREAM)
}

fn nonce_for(index: u64) -> [u8; AEAD_NONCE_LEN] {
    let [b0, b1, b2, b3, b4, b5, b6, b7] = index.to_be_bytes();
    [0, 0, 0, 0, b0, b1, b2, b3, b4, b5, b6, b7]
}

fn chunk_aad(
    header: &StreamHeader,
    chunk_size: u32,
    index: u64,
    is_final: bool,
    caller_aad: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let aad_len = u32::try_from(caller_aad.len()).map_err(|_| CryptoError::InvalidLength)?;
    let mut aad = Vec::with_capacity(STREAM_HEADER_LEN + 4 + 8 + 1 + 4 + caller_aad.len());
    aad.extend_from_slice(header);
    aad.extend_from_slice(&chunk_size.to_be_bytes());
    aad.extend_from_slice(&index.to_be_bytes());
    aad.push(u8::from(is_final));
    aad.extend_from_slice(&aad_len.to_be_bytes());
    aad.extend_from_slice(caller_aad);
    Ok(aad)
}

/// Encrypts a stream chunk by chunk.
#[derive(Debug)]
pub struct StreamEncryptor {
    key: SecretBytes<AEAD_KEY_LEN>,
    header: StreamHeader,
    chunk_size: usize,
    chunk_size_u32: u32,
    caller_aad: Vec<u8>,
    next_index: u64,
    finished: bool,
}

impl StreamEncryptor {
    /// Start a new stream under `master_key`.
    ///
    /// # Errors
    /// Returns [`CryptoError::StreamBadChunk`] for a chunk size of zero or above 16 MiB, and
    /// [`CryptoError::RandomFailure`] if randomness is unavailable.
    pub fn new(
        master_key: &SecretBytes<AEAD_KEY_LEN>,
        chunk_size: usize,
        caller_aad: &[u8],
    ) -> Result<Self, CryptoError> {
        let stream_id = random_array::<STREAM_ID_LEN>()?;
        Self::with_stream_id(master_key, stream_id, chunk_size, caller_aad)
    }

    /// Start a stream with a caller-chosen stream identifier. Reusing an identifier under the
    /// same master key breaks security; this exists only so conformance tests can reproduce the
    /// documented construction byte for byte.
    ///
    /// # Errors
    /// Same as [`Self::new`], without the randomness failure.
    #[cfg(feature = "deterministic-kat")]
    pub fn new_with_stream_id(
        master_key: &SecretBytes<AEAD_KEY_LEN>,
        stream_id: [u8; STREAM_ID_LEN],
        chunk_size: usize,
        caller_aad: &[u8],
    ) -> Result<Self, CryptoError> {
        Self::with_stream_id(master_key, stream_id, chunk_size, caller_aad)
    }

    fn with_stream_id(
        master_key: &SecretBytes<AEAD_KEY_LEN>,
        stream_id: [u8; STREAM_ID_LEN],
        chunk_size: usize,
        caller_aad: &[u8],
    ) -> Result<Self, CryptoError> {
        let chunk_size_u32 = validate_chunk_size(chunk_size)?;
        u32::try_from(caller_aad.len()).map_err(|_| CryptoError::InvalidLength)?;
        let mut bytes = Vec::with_capacity(STREAM_HEADER_LEN);
        bytes.push(PROTOCOL_VERSION);
        bytes.extend_from_slice(&stream_id);
        let header =
            StreamHeader::try_from(bytes.as_slice()).map_err(|_| CryptoError::InvalidLength)?;
        let key = derive_stream_key(master_key, &header)?;
        Ok(Self {
            key,
            header,
            chunk_size,
            chunk_size_u32,
            caller_aad: caller_aad.to_vec(),
            next_index: 0,
            finished: false,
        })
    }

    /// The header that must be stored or sent ahead of the first chunk.
    #[must_use]
    pub const fn header(&self) -> StreamHeader {
        self.header
    }

    /// Seal the next chunk. Every chunk except the final one must be exactly `chunk_size` bytes;
    /// the final chunk may be shorter or empty.
    ///
    /// # Errors
    /// Returns [`CryptoError::StreamChunkOrder`] after the final chunk and
    /// [`CryptoError::StreamBadChunk`] for a wrongly sized chunk or too many chunks.
    pub fn encrypt_chunk(
        &mut self,
        plaintext: &[u8],
        is_final: bool,
    ) -> Result<Vec<u8>, CryptoError> {
        if self.finished {
            return Err(CryptoError::StreamChunkOrder);
        }
        if self.next_index >= MAX_STREAM_CHUNKS {
            return Err(CryptoError::StreamBadChunk);
        }
        let size_ok = if is_final {
            plaintext.len() <= self.chunk_size
        } else {
            plaintext.len() == self.chunk_size
        };
        if !size_ok {
            return Err(CryptoError::StreamBadChunk);
        }
        let aad = chunk_aad(
            &self.header,
            self.chunk_size_u32,
            self.next_index,
            is_final,
            &self.caller_aad,
        )?;
        let sealed = seal(&self.key, &nonce_for(self.next_index), &aad, plaintext)?;
        self.next_index += 1;
        self.finished = is_final;
        Ok(sealed)
    }
}

/// Decrypts a stream chunk by chunk, enforcing order and the final chunk.
#[derive(Debug)]
pub struct StreamDecryptor {
    key: SecretBytes<AEAD_KEY_LEN>,
    header: StreamHeader,
    chunk_size: usize,
    chunk_size_u32: u32,
    caller_aad: Vec<u8>,
    next_index: u64,
    finished: bool,
}

impl StreamDecryptor {
    /// Open a stream from its header.
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidLength`] for a header of the wrong length,
    /// [`CryptoError::UnsupportedVersion`] for an unknown version and
    /// [`CryptoError::StreamBadChunk`] for an invalid chunk size.
    pub fn new(
        master_key: &SecretBytes<AEAD_KEY_LEN>,
        header: &[u8],
        chunk_size: usize,
        caller_aad: &[u8],
    ) -> Result<Self, CryptoError> {
        let chunk_size_u32 = validate_chunk_size(chunk_size)?;
        u32::try_from(caller_aad.len()).map_err(|_| CryptoError::InvalidLength)?;
        let header = StreamHeader::try_from(header).map_err(|_| CryptoError::InvalidLength)?;
        let (version, _) = header.split_first().ok_or(CryptoError::InvalidLength)?;
        check_version(*version).map_err(|_| CryptoError::UnsupportedVersion)?;
        let key = derive_stream_key(master_key, &header)?;
        Ok(Self {
            key,
            header,
            chunk_size,
            chunk_size_u32,
            caller_aad: caller_aad.to_vec(),
            next_index: 0,
            finished: false,
        })
    }

    /// Verify and decrypt the next chunk. Returns the plaintext and whether this was the final
    /// chunk. A failed chunk leaves the decryptor unchanged.
    ///
    /// # Errors
    /// Returns [`CryptoError::StreamChunkOrder`] after the final chunk,
    /// [`CryptoError::StreamBadChunk`] for an impossible chunk length and
    /// [`CryptoError::DecryptionFailed`] if authentication fails.
    pub fn decrypt_chunk(
        &mut self,
        ciphertext: &[u8],
    ) -> Result<(Zeroizing<Vec<u8>>, bool), CryptoError> {
        if self.finished {
            return Err(CryptoError::StreamChunkOrder);
        }
        let full = self.chunk_size + AEAD_TAG_LEN;
        if ciphertext.len() < AEAD_TAG_LEN || ciphertext.len() > full {
            return Err(CryptoError::StreamBadChunk);
        }
        let nonce = nonce_for(self.next_index);
        let try_open = |is_final: bool| -> Result<Zeroizing<Vec<u8>>, CryptoError> {
            let aad = chunk_aad(
                &self.header,
                self.chunk_size_u32,
                self.next_index,
                is_final,
                &self.caller_aad,
            )?;
            open(&self.key, &nonce, &aad, ciphertext)
        };
        let (plaintext, is_final) = if ciphertext.len() < full {
            (try_open(true)?, true)
        } else {
            match try_open(false) {
                Ok(plaintext) => (plaintext, false),
                Err(_) => (try_open(true)?, true),
            }
        };
        self.next_index += 1;
        self.finished = is_final;
        Ok((plaintext, is_final))
    }

    /// Whether the final chunk has been seen.
    #[must_use]
    pub const fn is_finished(&self) -> bool {
        self.finished
    }

    /// Call at the end of the input: succeeds only if the final chunk was seen.
    ///
    /// # Errors
    /// Returns [`CryptoError::StreamTruncated`] if the stream ended early.
    pub fn finish(self) -> Result<(), CryptoError> {
        if self.finished {
            Ok(())
        } else {
            Err(CryptoError::StreamTruncated)
        }
    }
}

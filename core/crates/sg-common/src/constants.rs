//! Protocol constants. Every value here is copied from docs/blueprint (Prompt B, section 19.2)
//! and is tested against the literal text of that specification in `tests/constants.rs`.

/// Domain separation label for the signature over the DH identity key.
pub const LABEL_IK_DH_SIG: &[u8] = b"Sargoshi/IK-DH/v1";
/// Domain separation label for the signature over a signed prekey.
pub const LABEL_SPK_SIG: &[u8] = b"Sargoshi/SPK/v1";
/// Domain separation label for the signature over a post-quantum signed prekey.
pub const LABEL_PQSPK_SIG: &[u8] = b"Sargoshi/PQSPK/v1";

/// HKDF info string of the PQXDH-style handshake.
pub const INFO_PQXDH: &[u8] = b"Sargoshi/PQXDH/v1";
/// HKDF info string of the Double Ratchet root key derivation.
pub const INFO_DR_ROOT: &[u8] = b"Sargoshi/DR/v1/root";
/// HKDF info string of the Double Ratchet message key expansion.
pub const INFO_DR_MESSAGE: &[u8] = b"Sargoshi/DR/v1/message";
/// HKDF info string of the backup key derived from the recovery phrase.
pub const INFO_BACKUP: &[u8] = b"Sargoshi/Backup/v1";

/// HKDF info string used to derive a per-stream key in the streaming AEAD wrapper.
/// This label is a Sargoshi addition and is not in Prompt B (DECISIONS D-20).
pub const INFO_STREAM: &[u8] = b"Sargoshi/Stream/v1";

/// Length of the HKDF salt used by the handshake (32 zero bytes).
pub const PQXDH_SALT_LEN: usize = 32;
/// Length of the 0xFF prefix of the handshake input key material.
pub const PQXDH_PREFIX_LEN: usize = 32;
/// Value of every byte of the handshake input key material prefix.
pub const PQXDH_PREFIX_BYTE: u8 = 0xFF;

/// Length of an X25519 public or secret key.
pub const X25519_KEY_LEN: usize = 32;
/// Length of an Ed25519 public key.
pub const ED25519_PUBLIC_LEN: usize = 32;
/// Length of an Ed25519 signature.
pub const ED25519_SIGNATURE_LEN: usize = 64;
/// Length of a ChaCha20-Poly1305 key.
pub const AEAD_KEY_LEN: usize = 32;
/// Length of a ChaCha20-Poly1305 nonce.
pub const AEAD_NONCE_LEN: usize = 12;
/// Length of a ChaCha20-Poly1305 authentication tag.
pub const AEAD_TAG_LEN: usize = 16;
/// Length of an ML-KEM-768 encapsulation (public) key in bytes (FIPS 203).
pub const MLKEM768_ENCAPSULATION_KEY_LEN: usize = 1184;
/// Length of an ML-KEM-768 ciphertext in bytes (FIPS 203).
pub const MLKEM768_CIPHERTEXT_LEN: usize = 1088;
/// Length of an ML-KEM-768 shared secret in bytes (FIPS 203).
pub const MLKEM768_SHARED_SECRET_LEN: usize = 32;

/// One-time prekeys generated per device.
pub const ONE_TIME_PREKEYS_PER_DEVICE: usize = 100;
/// A device replenishes its one-time prekeys when fewer than this many remain.
pub const ONE_TIME_PREKEY_REPLENISH_BELOW: usize = 25;

/// Maximum skipped message keys accepted from a single session.
pub const MAX_SKIPPED_KEYS_PER_SESSION: usize = 1_000;
/// Maximum cached skipped message keys across a session.
pub const MAX_CACHED_SKIPPED_KEYS: usize = 2_000;

/// Plaintext is padded to the next multiple of this many bytes before encryption.
pub const PADDING_BLOCK_BYTES: usize = 160;

/// Number of iterations of SHA-512 in the safety number derivation.
pub const SAFETY_NUMBER_ITERATIONS: u32 = 5_200;
/// Total digits of a safety number (12 groups of 5).
pub const SAFETY_NUMBER_DIGITS: usize = 60;
/// Digits per group of a safety number.
pub const SAFETY_NUMBER_GROUP_DIGITS: usize = 5;

/// Number of words of the recovery phrase (256 bits of entropy).
pub const RECOVERY_PHRASE_WORDS: usize = 24;
/// Size of one backup chunk (1 MiB).
pub const BACKUP_CHUNK_BYTES: usize = 1024 * 1024;

/// Length of the random salt used with Argon2id for the Vault key.
pub const ARGON2_SALT_LEN: usize = 16;
/// Lowest Argon2id memory cost (KiB) Sargoshi will ever accept: 19 MiB.
/// VERIFY: confirm against the current OWASP Password Storage Cheat Sheet before B5 ships.
pub const ARGON2_MIN_MEMORY_KIB: u32 = 19 * 1024;
/// Lowest Argon2id iteration count Sargoshi will ever accept.
/// VERIFY: confirm against the current OWASP Password Storage Cheat Sheet before B5 ships.
pub const ARGON2_MIN_ITERATIONS: u32 = 2;
/// Lowest Argon2id parallelism Sargoshi will ever accept.
pub const ARGON2_MIN_PARALLELISM: u32 = 1;

/// Upper bound on the size of one serialized envelope. This limit is a Sargoshi decision (it is
/// not in Prompt B) so that a hostile peer cannot make a receiver allocate without bound.
pub const MAX_ENVELOPE_BYTES: usize = 128 * 1024;

/// Length of the random stream identifier in a streaming AEAD header.
pub const STREAM_ID_LEN: usize = 16;
/// Length of a streaming AEAD header: version byte plus stream identifier.
pub const STREAM_HEADER_LEN: usize = 1 + STREAM_ID_LEN;
/// Largest chunk size the streaming AEAD wrapper accepts (16 MiB).
pub const MAX_STREAM_CHUNK_BYTES: usize = 16 * 1024 * 1024;
/// Largest number of chunks in one stream; keeps the 96-bit nonce counter far from wrapping.
pub const MAX_STREAM_CHUNKS: u64 = 1 << 32;
/// Largest output HKDF-SHA256 can produce: 255 blocks of 32 bytes (RFC 5869).
pub const HKDF_SHA256_MAX_OUTPUT: usize = 255 * 32;

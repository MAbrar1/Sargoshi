# PROTOCOL

Status: milestones B1, B2 and B3 of Prompt B (blueprint 19.2). This document describes only what is implemented and tested. Sections for later milestones are listed in section 7 and are added when their code exists. Every deviation from Prompt B is in section 6. This protocol has not been reviewed by a cryptographer (risk R-01) and must not be described as audited.

## 1. Conventions
- Integers on the wire are big-endian. Keys are X25519 and Ed25519 unless stated.
- All constants live in `core/crates/sg-common/src/constants.rs` and are tested against the blueprint text (`tests/constants.rs` fails if a label or number in code no longer appears in the specification).
- The core is stateless and never touches disk or network (Prompt B hard rule 5).

## 2. Versioning
The protocol version is 1. Every wire message begins with a version byte. A receiver accepts exactly version 1; version 0 and every other value are rejected with `UnsupportedVersion` (tested for all 256 byte values).

## 3. Domain-separation labels
| Use | Label |
|---|---|
| Signature over the DH identity key | `Sargoshi/IK-DH/v1` |
| Signature over a signed prekey | `Sargoshi/SPK/v1` |
| Signature over a post-quantum signed prekey | `Sargoshi/PQSPK/v1` |
| Handshake HKDF info | `Sargoshi/PQXDH/v1` |
| Double Ratchet root KDF info | `Sargoshi/DR/v1/root` |
| Double Ratchet message key info | `Sargoshi/DR/v1/message` |
| Backup key HKDF info | `Sargoshi/Backup/v1` |

## 4. Wire formats
### 4.1 Ratchet header (41 bytes, fixed)
`version(1) || ratchet_pub(32) || previous_chain_length(u32) || message_number(u32)`.
The serialized header is authenticated as associated data together with the session AD (defined in the handshake section). A fixed layout is used so the authenticated bytes are canonical by construction.

### 4.2 Envelope
`version(1) || protobuf(body)` with body fields: 1 `header` (bytes, exactly the 41-byte header), 2 `ciphertext` (bytes, at least 16 bytes: the AEAD tag), 15 `reserved` (bytes, must be empty in version 1).
Canonical rule: the decoder re-encodes the parsed body and requires the bytes to be identical to the input. Unknown fields, reordered fields, non-minimal varints and trailing bytes are therefore rejected (`NonCanonical` or `Malformed`). A non-empty field 15 is rejected with `ReservedFieldSet`. The total size may not exceed 128 KiB (`TooLarge`).
Field 15 is reserved for the message-franking commitment and for sealed-sender extensions. When sealed sender ships, the sender identity moves inside the encrypted payload and delivery uses an access key derived from the recipient's profile key; that change will use a new protocol version, not a silent extension of version 1.

## 5. Error codes
Stable numeric codes (never renumbered): 1000 UnsupportedVersion, 1001 Truncated, 1002 TooLarge, 1003 Malformed, 1004 NonCanonical, 1005 BadLength, 1006 ReservedFieldSet. Ranges: 2000s primitives, 3000s identity and handshake, 4000s ratchet, 5000s groups, 6000s vault and backup, 9000s FFI and internal.

## 6. Primitives and policies (B2)
All primitives come from maintained crates; no cipher, hash, KEM or constant-time trick is written by this project. Exact versions, licences and audit status are in docs/ledger/DEPENDENCIES.md.

| Primitive | Standard | Wrapper in `sg-crypto` | Policy |
|---|---|---|---|
| X25519 | RFC 7748 | `x25519` | An all-zero shared secret (low-order peer key) is rejected with `WeakSharedSecret`. |
| Ed25519 | RFC 8032 | `ed25519` | Strict verification. Public keys must be in the prime-order subgroup (small-order and mixed-order keys are rejected). Key certification signs `label || data`. |
| HKDF-SHA256 | RFC 5869 | `kdf` | At most 8160 output bytes. An empty salt is the same as 32 zero bytes. |
| ChaCha20-Poly1305 | RFC 8439 | `aead` | The caller supplies the 96-bit nonce; every key is used once or with a strictly increasing counter. A failed decryption never says why. |
| ML-KEM-768 | FIPS 203 | `kem` | The secret key is stored as its 64-byte seed. Encapsulation keys are length- and modulus-checked. Decapsulation uses ML-KEM implicit rejection (a corrupted ciphertext yields an unrelated secret, not an error). |
| Randomness | operating system | `rng` | `getrandom` only. No seedable generator exists outside the `deterministic-kat` test feature. |
| Secrets | | `secret` | `SecretBytes<N>` zeroizes on drop, compares in constant time, has no `Clone`, and its `Debug` output is redacted. |

### 6.1 Streaming AEAD (for vault files and backups)
Header: `version(1) || stream_id(16, random)`. Per-stream key: `HKDF-SHA256(salt = stream_id, ikm = master_key, info = "Sargoshi/Stream/v1")`. Chunk `i` is sealed with nonce `0x00000000 || i(u64 BE)` and associated data `header || chunk_size(u32 BE) || i(u64 BE) || final_flag(1) || len(caller_aad)(u32 BE) || caller_aad`. Every chunk except the last holds exactly `chunk_size` bytes; the last holds at most `chunk_size` bytes (possibly none) and carries the final flag. The default chunk size for backups is 1 MiB; the maximum is 16 MiB and at most 2^32 chunks.
Detected: truncation (missing final chunk), dropped, duplicated, reordered or spliced chunks, any single-byte change in a header or chunk, a different chunk size, key or caller data. A failed chunk leaves the decryptor unchanged. This construction is a Sargoshi design (DECISIONS D-20) and is on the cryptographer's review list.

### 6.2 Test evidence
Known-answer vectors, generated by `tools/gen-vectors.mjs` from pinned upstream commits (hashes in `core/crates/sg-testvectors/vectors/SOURCES.md`): HKDF-SHA256 86 (Wycheproof), X25519 518 (Wycheproof, includes low-order and twist cases), ChaCha20-Poly1305 316 (Wycheproof, includes forgeries), Ed25519 210 (Wycheproof, includes malformed and weak-key cases), ML-KEM-768 80 (NIST ACVP: key generation 25, encapsulation 25, decapsulation 10, encapsulation-key check 10, decapsulation-key check 10), plus RFC 7748 section 6.1 and RFC 8032 test 1. A conformance test rebuilds the streaming construction from the formulas above and compares it with the real encryptor. Property tests cover symmetry, tamper detection and round trips.

## 7. Identity, prekeys and handshake (B3)
Crate `sg-handshake`. Everything is pure computation on values and byte blobs; timestamps are supplied by the caller.

### 7.1 Identity
An account identity is an Ed25519 signing key plus an X25519 DH key. The signing key certifies the DH key: `signature = Ed25519(label || dh_public)` with label `Sargoshi/IK-DH/v1`. An `IdentityPublic` value can only be built through a constructor that verifies this certification and refuses invalid, small-order and mixed-order keys, so every value of the type is verified. Secret blob (65 bytes): `version || signing_seed(32) || dh_secret(32)`, plaintext secret material that the app stores only inside its encrypted database.

### 7.2 Prekeys
- Signed prekey (X25519): signature over `Sargoshi/SPK/v1 || public`.
- Post-quantum signed prekey (ML-KEM-768 encapsulation key): signature over `Sargoshi/PQSPK/v1 || encapsulation_key`. It is a single reusable key rotated on a schedule, not a one-time key.
- One-time prekeys (X25519), unsigned: 100 per device, replenished when fewer than 25 remain.
Signatures are never stored; Ed25519 is deterministic, so they are recomputed from the identity whenever a bundle is built. Limits: at most 16 signed and 16 post-quantum prekeys kept for acceptance, at most 1,024 one-time prekeys stored, at most 256 per upload. Rotation adds a new pair and keeps the old ones until `prune_before(cutoff)`, which always keeps the newest of each kind. Identifiers must be unique.
Secret store blob: `version || n_signed(u16) || {id u32, created_at u64, secret 32}* || n_pq(u16) || {id u32, created_at u64, seed 64}* || n_one_time(u16) || {id u32, secret 32}* || n_seen(u32) || {ephemeral 32}*`. Parsing is bounds-checked, rejects truncation, trailing bytes, oversized counts and duplicate identifiers. The writer is created with its exact final size so a secret is never copied by a reallocation.

### 7.3 Bundles and the initial header
Wire format `version || protobuf(body)` with the canonical re-encoding rule of section 4.2 (maximum 64 KiB). Parsing a `PreKeyBundle` verifies the identity certification and both prekey signatures; parsing an `UploadBundle` verifies them under the identity the server has on record; parsing an `InitialHeader` verifies the initiator's identity. The initial header carries: the initiator's identity (signing key, DH key, certification), the ephemeral key `EK_A`, the ML-KEM ciphertext, and the identifiers of the signed prekey, post-quantum prekey and (optional) one-time prekey used. The two required identifiers have explicit presence on the wire, so a message that lacks them is rejected rather than read as identifier 0. As with any protobuf message, cutting a bundle or header at the boundary before its trailing optional one-time prekey field yields another valid message without that key; the result is never equal to the original and at most one such cut parses (tested).

### 7.4 Handshake
```
DH1 = DH(IK_A, SPK_B)   DH2 = DH(EK_A, IK_B)   DH3 = DH(EK_A, SPK_B)   DH4 = DH(EK_A, OPK_B)   (DH4 only with a one-time prekey)
(ct, ss) = ML-KEM-768.Encaps(PQSPK_B)
SK = HKDF-SHA256(salt = 32 zero bytes, ikm = 0xFF*32 || DH1 || DH2 || DH3 || [DH4] || ss, info = "Sargoshi/PQXDH/v1", L = 32)
AD = IK_A_dh_public || IK_B_dh_public || PQSPK_B_encapsulation_key        (32 + 32 + 1184 bytes)
```
`IK` are the X25519 identity keys. The initiator verifies every signature in the bundle before any secret operation. `SK` becomes the Double Ratchet root key and `AD` is bound into every message (section 4.1). The responder's signed prekey doubles as its initial ratchet key pair: the initiator receives its public half, the responder keeps the secret half.
Responder (`accept`): rejects a repeated ephemeral key (`ReplayedInitialMessage`), looks up the three prekeys by identifier, computes the same values, and only on success deletes the used one-time prekey and remembers the ephemeral key (the 4,096 most recent are kept; older replays are stopped by prekey rotation). On any error the store is left unchanged (tested byte for byte). A corrupted ML-KEM ciphertext is not an error at this stage: ML-KEM's implicit rejection gives an unrelated key, so the first ratchet message fails to authenticate. For this reason the caller persists the updated store only after the first ratchet message has decrypted.
The caller, not this crate, must compare the identity in a bundle or header with the identity it trusts for that contact, or show a safety number on first contact.

### 7.5 Test evidence
43 behavioural tests in `sg-handshake` plus two conformance tests in `sg-testvectors` that rebuild the shared key, associated data and ciphertext from the formulas above using raw primitives and the literal label strings, with and without a one-time prekey, and check that both sides agree. Covered attack and failure cases: replay with and without a one-time prekey (also across a serialization round trip), one-time prekey reuse, unknown identifiers, low-order ephemeral keys, low-order keys in bundles, a low-order DH key inside a validly signed initiator identity, forged and substituted identities and prekeys, corrupted ciphertexts, dropped header fields, every truncation of every blob and message, single-byte changes of a bundle, signed-material changes, oversized counts, duplicate identifiers, and arbitrary junk input (never panics, only canonical input is accepted).

## 8. Deviations from Prompt B
1. Crate names follow the repository layout (DECISIONS D-17): `sg-common` replaces `sg-proto`, `sg-testvectors` replaces `sg-testvec`, `sg-group` replaces `sg-mls`; handshake and ratchet are separate crates `sg-handshake` and `sg-ratchet`.
2. The ratchet header is a fixed binary layout rather than a protobuf message (still contains exactly the fields Prompt B lists).
3. Maximum envelope size 128 KiB is a Sargoshi decision (not in Prompt B).
4. `cargo audit` is replaced by `cargo deny check advisories` (DECISIONS D-19).
5. The streaming AEAD derives a per-stream key and uses a new label `Sargoshi/Stream/v1` (DECISIONS D-20).
6. Ed25519 public keys are additionally required to be in the prime-order subgroup (DECISIONS D-22).
7. ML-KEM-768 secret keys are stored as seeds (DECISIONS D-21).
8. The responder keeps a bounded memory of accepted ephemeral keys to reject replays, and the core's `accept` returns the updated prekey store, so the FFI `session_accept` returns the updated prekey blob in addition to the session blob and consumed one-time prekey identifiers (DECISIONS D-25).
9. Prekey signatures are recomputed instead of stored; the post-quantum prekey is a single rotated key (DECISIONS D-26).

## 9. Sections added by later milestones
B4 Double Ratchet, skipped-key limits, replay rejection, state serialization; B5 safety number, padding, recovery phrase, backup and vault key derivation; B6 FFI error mapping. Open items marked `VERIFY:` in the code are listed in AUDIT-PREP.md.

# AUDIT PREPARATION

Status: milestones B1, B2 and B3. For the independent cryptographer review required before public launch (risk R-01, EXTERNAL.md).

## Scope today
`core/crates/sg-common` (constants, version check, error codes, canonical header and envelope), `core/crates/sg-crypto` (wrappers for X25519, Ed25519, HKDF-SHA256, ChaCha20-Poly1305, ML-KEM-768 and the streaming AEAD) `core/crates/sg-handshake` (identity, prekeys, bundles, initial header, hybrid handshake) and `core/crates/sg-testvectors` (known-answer and specification conformance tests). No primitive is written by the project; the original constructions are the streaming AEAD (PROTOCOL.md 6.1) and the composition of primitives in the handshake (PROTOCOL.md 7.4).

## How to reproduce the evidence
`node tools/gen-vectors.mjs --check --online` (re-downloads upstream vectors and compares byte for byte), `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, `node tools/check-dep-age.mjs`, `node tools/gate.mjs --full`.

## Dependencies in scope
See docs/ledger/DEPENDENCIES.md (versions, licences, audit status, 97-crate lockfile record). Independent audit status of the crypto crates is unknown for the pinned versions.

## Open items marked VERIFY
- Argon2id minimum parameters (19 MiB, 2 iterations, 1 lane) must be confirmed against the current OWASP guidance before B5 ships (`constants.rs`).
- Independent audit status of every crypto crate version pinned (DEPENDENCIES.md); the chacha20poly1305 audit coverage for 0.11.
- Licence terms of the NIST ACVP-Server repository (vectors/SOURCES.md).
- Constant-time behaviour of the pinned crates on the target Android CPUs (not measured).

## Questions for the reviewer (growing list)
1. Is a 128 KiB envelope limit appropriate, and should field 15 stay rejected-when-set in version 1?
2. Is the fixed 41-byte header preferable to a length-prefixed encoding for associated data?
3. Which rekey cadence in PQ-RATCHET.md do you recommend?
4. Streaming AEAD (PROTOCOL.md 6.1): is deriving a per-stream key from a random stream id plus a counter nonce sound, and is the associated-data layout complete?
5. Is the Ed25519 policy (strict verification plus a prime-order-subgroup requirement on public keys) right for identity and prekey signatures?
6. RustCrypto `ml-kem` 0.3.4 versus `libcrux-ml-kem` for ML-KEM-768; is seed-only storage of the secret key acceptable?
7. Is the `deterministic-kat` feature isolation (off in app builds) sufficient?
8. Handshake (PROTOCOL.md 7.4): the associated data contains the DH identity keys and the post-quantum prekey but not the Ed25519 signing keys; is that sufficient, and is the key schedule input order sound?
9. Is a single reusable post-quantum prekey with scheduled rotation acceptable, and what rotation interval do you recommend?
10. Is the replay design (bounded memory of ephemeral keys plus prekey rotation) adequate?
11. VERIFY: compare this handshake with Signal's published PQXDH specification line by line and list every difference (the author recalls identity-key, KEM-parameter and info-string differences but has not rechecked the current text).

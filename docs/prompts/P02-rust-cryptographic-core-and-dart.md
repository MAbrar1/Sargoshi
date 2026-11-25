# P02 — Rust cryptographic core and Dart bindings

Depends on: P00. Ledger rows owned by this phase: 0 (infrastructure phase; its proof is the gate and the tests below).

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Build the vetted-primitive crypto core exactly as specified by Prompt B (blueprint lines 2705-2858): identity, prekeys, hybrid key agreement, Double Ratchet, safety numbers, padding, backup and vault key derivation helpers, recovery phrase, streaming AEAD, FFI. No invented primitives, ever.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §14 (lines 2151-2239) Security and cryptography (14.1 to 14.3)
- §19.2 (lines 2705-2859) Prompt B: ROLE, HARD RULES, WORKSPACE, ACCOUNT IDENTITY AND DEVICE LINKING, CRYPTO SPECIFICATION, FFI, TESTS (this text is the authoritative crypto spec)
- §12.4 (lines 1783-1802) key hierarchy rows
- §8.3 (lines 1359-1379) and 8.4 for the streaming AEAD requirements used later by the Vault

## Build (do every item, in order, as real working code)

1. WORKSPACE. Root Cargo.toml workspace with members core/crates/* and server/crates/*; rust-toolchain.toml; deny.toml; rustfmt.toml. Crates now: sg-proto, sg-crypto, sg-testvec, sg-ffi. (sg-mls, sg-vault, sg-backup come in their phases.) `#![forbid(unsafe_code)]` in every crate except sg-ffi.
2. DEPENDENCIES. Before pinning any crate run `cargo info <crate>` (or read crates.io) and record version, licence, release date (must be at least 14 days old), and audit status ("unknown" if you do not know) in docs/ledger/DEPENDENCIES.md. Candidates are listed in Prompt B HARD RULES 1. For ML-KEM-768 pick one maintained implementation and record why. Check the licence of every crate; stop on anything copyleft.
3. IMPLEMENT milestones B1 to B6 of Prompt B in order: proto and constants; primitive wrappers including the streaming AEAD wrapper (ChaCha20-Poly1305 STREAM) used later by the Vault; identity and prekeys; PQXDH-style handshake; Double Ratchet; safety number, padding, backup key derivation (24-word phrase, HKDF), vault key derivation (Argon2id with parameters stored in the blob); FFI facade through flutter_rust_bridge with typed errors and no panics across the boundary (catch_unwind).
4. The recovery key is an ACCOUNT-level secret: expose `recovery_phrase_generate()` and `recovery_key_derive(phrase, passphrase?)` now because ACC-13 (P06) and BKP-04 (P12) both use it.
5. BUILD FOR ANDROID. Cross-compile sg-ffi for aarch64, armv7 and x86_64 Android targets and load it from Flutter (flutter_rust_bridge with its recommended build integration; verify the current docs). Record the chosen method in DECISIONS.md.
6. DOCS. Write docs/architecture/PROTOCOL.md and THREAT_MODEL.md as you go so a reviewer can audit from the docs alone, plus PQ-RATCHET.md and AUDIT-PREP.md as design notes (the sparse ML-KEM ratchet is NOT implemented; a no-op PqRatchetHook trait exists). Every deviation from Prompt B is written down. The UI label is "Post-quantum protected (beta)" and nothing may say audited, unhackable or military-grade.

## Tests (required)

- Known-answer tests: HKDF RFC 5869, X25519 RFC 7748, ChaCha20-Poly1305 RFC 8439, ML-KEM-768 vectors from the crate or NIST. Property tests (proptest): out-of-order delivery, skipped keys up to the limit, replay rejection, state round trip, tampered header and ciphertext rejected. Fuzz targets for envelope and bundle parsing (cargo-fuzz needs nightly; if unavailable record it in VERIFY.md and keep the targets compiling).
- A Dart test that loads the real library, completes a handshake and exchanges 100 messages each way.

## Done only when ALL are true

- Every ledger row of P02 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P02` shows no open rows.
- `node tools/gate.mjs --phase P02 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- `cargo test --workspace`, clippy with -D warnings, `cargo deny check` pass.
- You wrote no cryptographic primitive. Say so explicitly in PROGRESS.md and list every crate used.
- PROGRESS.md updated and `CURRENT_PHASE: P03` set; all work committed.

## Stop conditions specific to this phase

- Stop and report if a requirement cannot be met with a maintained crate. The owner must get a cryptographer to review PROTOCOL.md before any public launch (R-01); say this plainly in PROGRESS.md.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

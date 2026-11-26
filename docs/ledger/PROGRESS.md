# PROGRESS

CURRENT_PHASE: P00

Rules: one line `CURRENT_PHASE: Pxx` above (read by the tools). Update this file before every stop. Newest session first.

## Resume instructions
1. Read AGENTS.md. 2. `node tools/ledger.mjs status --phase <CURRENT_PHASE>`. 3. Open docs/prompts/<CURRENT_PHASE>-*.md. 4. `node tools/ledger.mjs next`.

## Session log
| Date | Phase | Done (ids) | Open / blocked | Next exact step |
|---|---|---|---|---|
| 25 Nov 2026 | P02 (Rust track, D-15) B2 | B2 done: sg-crypto (X25519, Ed25519, HKDF-SHA256, ChaCha20-Poly1305, ML-KEM-768, OS RNG, zeroizing secret types, streaming AEAD with per-stream key), sg-testvectors with 1,200 known-answer records (Wycheproof + NIST ACVP at pinned commits, re-downloaded and compared byte for byte), streaming conformance test, 67 tests total. Findings fixed: mixed-order Ed25519 key accepted by strict verifier (D-22); gate hook for generators was not really checking (gen-vectors --check now real). `node tools/gate.mjs --full` PASS; check-dep-age OK (97 crates). | VERIFY items in AUDIT-PREP.md; crate audit status unknown; no Android cross-compile yet (needs owner machine) | Milestone B3: identity keys, prekeys, bundle, PQXDH-style handshake in sg-handshake (read Prompt B lines about key types and handshake; verify every API from source; keep deterministic tests behind the same feature) |
| 25 Nov 2026 | P02 (Rust track, D-15) B1 | B1 done: workspace (Cargo.toml, rust-toolchain.toml, rustfmt, clippy, deny), crate sg-common (constants verified against blueprint text, versioning, error codes, canonical header+envelope), 23 tests incl. proptest, docs PROTOCOL/THREAT_MODEL/PQ-RATCHET/AUDIT-PREP, tools/check-dep-age.mjs (found and fixed 9 transitive crates younger than 14 days), CI rust.yml (not run: no GitHub here). `node tools/gate.mjs --full` PASS. No cryptographic primitive written. | B2..B14 of Prompt B; P00 toolchain item and all Flutter phases need the owner's machine | Resume: export PATH=$HOME/mamba/envs/sg/bin:$PATH (see LOCAL-DEV), then milestone B2: primitive wrappers + known-answer tests (verify crate versions with `cargo info` and crates.io API, 14-day rule, then `node tools/check-dep-age.mjs`) |
| 25 Nov 2026 | P00 | skeleton dirs (apps, core, server, infra) with READMEs; git init; ledger init (883 rows); gate PASS; deliberate TODO break test: gate FAILED on reality, PASSED after removal; Rust 1.100 + cargo-deny + Postgres 19 + Redis 8 installed in sandbox and cargo verified against crates.io | P00 toolchain check on owner machine (Flutter, Android SDK/NDK, Docker, emulator) - see D-15 and EXTERNAL.md | Rust track: read docs/prompts/P02-*.md and blueprint lines 2705-2858, start sg crates |
| (pack created) | P00 | none | everything | Run the P00 prompt |

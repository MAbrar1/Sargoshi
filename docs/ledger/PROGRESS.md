# PROGRESS

CURRENT_PHASE: P00

Rules: one line `CURRENT_PHASE: Pxx` above (read by the tools). Update this file before every stop. Newest session first.

## Resume instructions
1. Read AGENTS.md. 2. `node tools/ledger.mjs status --phase <CURRENT_PHASE>`. 3. Open docs/prompts/<CURRENT_PHASE>-*.md. 4. `node tools/ledger.mjs next`.

## Session log
| Date | Phase | Done (ids) | Open / blocked | Next exact step |
|---|---|---|---|---|
| 25 Nov 2026 | P02 (Rust track, D-15) B1 | B1 done: workspace (Cargo.toml, rust-toolchain.toml, rustfmt, clippy, deny), crate sg-common (constants verified against blueprint text, versioning, error codes, canonical header+envelope), 23 tests incl. proptest, docs PROTOCOL/THREAT_MODEL/PQ-RATCHET/AUDIT-PREP, tools/check-dep-age.mjs (found and fixed 9 transitive crates younger than 14 days), CI rust.yml (not run: no GitHub here). `node tools/gate.mjs --full` PASS. No cryptographic primitive written. | B2..B14 of Prompt B; P00 toolchain item and all Flutter phases need the owner's machine | Resume: export PATH=$HOME/mamba/envs/sg/bin:$PATH (see LOCAL-DEV), then milestone B2: primitive wrappers + known-answer tests (verify crate versions with `cargo info` and crates.io API, 14-day rule, then `node tools/check-dep-age.mjs`) |
| 25 Nov 2026 | P00 | skeleton dirs (apps, core, server, infra) with READMEs; git init; ledger init (883 rows); gate PASS; deliberate TODO break test: gate FAILED on reality, PASSED after removal; Rust 1.100 + cargo-deny + Postgres 19 + Redis 8 installed in sandbox and cargo verified against crates.io | P00 toolchain check on owner machine (Flutter, Android SDK/NDK, Docker, emulator) - see D-15 and EXTERNAL.md | Rust track: read docs/prompts/P02-*.md and blueprint lines 2705-2858, start sg crates |
| (pack created) | P00 | none | everything | Run the P00 prompt |

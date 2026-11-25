# AUDIT PREPARATION

Status: milestone B1. For the independent cryptographer review required before public launch (risk R-01, EXTERNAL.md).

## Scope today
`core/crates/sg-common`: constants, version check, error codes, canonical header and envelope. No cryptographic code exists yet; no primitive has been written by the project and none will be.

## How to reproduce the evidence
`cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, `node tools/check-dep-age.mjs`, `node tools/gate.mjs --full`.

## Dependencies in scope
prost 0.14.5, thiserror 2.0.21, proptest 1.12.0 (tests). Independent audit status of each: unknown. Full lockfile record in docs/ledger/DEPENDENCIES.md.

## Open items marked VERIFY
- Argon2id minimum parameters (19 MiB, 2 iterations, 1 lane) must be confirmed against the current OWASP guidance before B5 ships (`constants.rs`).
- Choice of the ML-KEM-768 implementation is made and justified in B2.

## Questions for the reviewer (growing list)
1. Is a 128 KiB envelope limit appropriate, and should field 15 stay rejected-when-set in version 1?
2. Is the fixed 41-byte header preferable to a length-prefixed encoding for associated data?
3. Which rekey cadence in PQ-RATCHET.md do you recommend?

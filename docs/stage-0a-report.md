# Stage 0A report: threat model and architecture decisions

Status: PLANNING ARTIFACTS ADDED; Stage 0A is not marked complete. No security implementation or audit is claimed.

## Added

- `docs/threat-model.md`: assets, adversaries, trust boundaries, intended controls, explicit limits, and verification items.
- `docs/adr/ADR-002-network-layer.md`: proposed Kotlin/OkHttp boundary; production endpoint and pins remain blockers.
- `docs/adr/ADR-003-database.md`: proposed Drift/SQLCipher design; package and isolate wiring remain unverified.
- `docs/adr/ADR-004-rust-binding.md`: proposes an FFI spike with flutter_rust_bridge, subject to compatibility verification; no cryptographic library is claimed selected.
- `docs/adr/ADR-005-tech-risks.md`: active technical risk register.

## Not built

No application code, network client, database, FFI binding, crypto protocol, onboarding, or feature UI was implemented in this stage. Existing provisional ADR-006 choices have not been treated as legal approval or a production configuration.

## Verification

No tests were run because this change contains documentation only. After checkout, run:

```sh
git diff --check
```

## TODO(verify)

- Confirm protocol/library compatibility and license obligations with legal/security review.
- Obtain the real App Link/API domain and primary/backup SPKI pins, expiry, and rotation plan.
- Verify Drift/SQLCipher package, FTS5/WAL behavior, 16 KB support, and isolate key handoff.
- Verify Flutter/Rust binding generator compatibility and all native ABI/page-size outputs.
- Confirm group MLS implementation, AI model policy, and 32-bit ABI decision in a finalized owner ADR.
- Review Android version/OEM-specific requirements against supported devices.

## Owner decisions / blockers

The owner authorized choosing best options, but no production hostname/pin values can be responsibly invented. The backend owner must supply those. Legal review is still required for crypto/dependency licensing. Stage 0A remains open until these decisions and verification items are reviewed.

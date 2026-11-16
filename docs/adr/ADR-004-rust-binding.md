# ADR-004: Rust core and Flutter FFI boundary

Status: PROPOSED for Stage 0A; no production cryptography is selected or claimed by this ADR.

## Decision

Keep cryptographic operations in the Rust core and expose an asynchronous Dart interface through generated FFI bindings. Evaluate `flutter_rust_bridge` as the preferred binding generator, subject to a build/maintenance/license/ABI spike before adoption. Keep a narrow, versioned interface for `createSession`, `encrypt`, `decrypt`, `deriveCallKey`, `padMessage`, `createRecoveryFile`, and `restoreRecoveryFile`. All operations must be asynchronous to callers and must not block Flutter's UI isolate.

Use audited protocol libraries only; do not implement primitives or protocol state machines from scratch. Use `zeroize` and constant-time operations where supported by the selected libraries, and test against published vectors. The fake implementation may be used only in debug/test builds, with a persistent demo warning; release CI must reject its inclusion.

## Rationale

A generated binding reduces handwritten ABI and serialization glue, while Rust keeps crypto out of Dart presentation logic. The generator is a tooling choice, not a cryptographic guarantee. If the proposed generator cannot meet target ABI, 16 KB, async, or release-verification requirements, stop and compare alternatives before committing to it.

## Consequences

- Stage 2 or the crypto integration stage must add a binding spike and release artifact inspection before real key material is introduced.
- FFI types must avoid unnecessary secret copies and must not expose raw private-key bytes to UI-facing APIs.
- All native artifacts must be built and checked for arm64-v8a and armeabi-v7a, plus debug-only x86_64 if retained.
- Release CI must fail on fake-core symbols/flags and include Rust tests, lint, dependency audit, and eventual fuzzing.

## TODO(verify)

- Verify flutter_rust_bridge current maintenance, license, generated-code workflow, Flutter 3.50 compatibility, and Android ABI support.
- Verify selected protocol libraries, their audit history, licenses, and support for required hybrid key agreement and MLS.
- Verify Rust zeroization/constant-time guarantees at the FFI and compiler boundary; do not claim perfect erasure.
- Verify 16 KB alignment for Rust and all transitive native artifacts on the actual release build.
- Confirm release fake-core exclusion can be enforced robustly in CI.

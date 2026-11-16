# ADR-003: Encrypted local database

Status: PROPOSED for Stage 0A; package wiring remains unverified.

## Decision

Use Drift over SQLCipher for application persistence. Run database work off the UI isolate; use WAL mode, `secure_delete`, and FTS5 where supported. Keep Vault data in a separately keyed store. The SQLCipher key is unwrapped in native Kotlin, supplied once to the database opener, and temporary buffers are cleared promptly. Do not persist database keys in Dart preferences or ordinary files.

Do not select a concrete Flutter SQLCipher package/version until its maintenance status, native ABI coverage, FTS5/WAL behavior, 16 KB page compatibility, and Drift/background-isolate integration have been tested. Database open must fail closed rather than silently fall back to plaintext.

## Rationale

Drift supplies typed queries/migrations and reactive access. SQLCipher provides encrypted database pages, while native Keystore ownership limits key handling in Dart. A separate Vault key/store reduces accidental cross-feature access; it does not make a compromised process safe.

## Consequences

- Stage 2 must implement and test the native key handoff, isolate lifecycle, migration, recovery, and failure behavior before storing real user data.
- FTS5 must index only eligible content; Vault, Whisper, expired/disappearing content, and AI inputs must be excluded per product rules.
- WAL and deletion semantics require device-level verification; secure deletion is not a guarantee of flash overwrite.
- The key briefly existing in process memory remains a documented residual risk.

## TODO(verify)

- Verify the correct maintained 2026 Drift/SQLCipher package and licensing.
- Verify FTS5, WAL, `secure_delete`, migration, and background-isolate support in the selected build.
- Verify 16 KB page compatibility for every bundled native library and ABI.
- Prototype the least-exposure Kotlin-to-Drift-isolate key handoff and review memory zeroization limits.
- Define the separate Vault database/key lifecycle and cryptographic erase behavior.

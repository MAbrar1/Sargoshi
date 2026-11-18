# Stage 0B to 3 report (not release-ready; Stage 1 to 3 are demo scaffolds)

Status on `main` (CI: checks, flutter, rust, android all green; 80 Flutter tests pass; analyzer clean).

## Real and verified in CI
- Flutter 3.50.0 project skeleton, Android build (R8, 16 KB ELF alignment check, merged-manifest allow-list, backup exclusions, TLS-only config).
- `Measured<T>`, Moonlit Ember tokens, bundled subset fonts (193,008 B) and a custom 17-glyph icon font, reproducible from `tool/`.
- Performance tier classifier (thresholds provisional, ADR-007) and the Pigeon channel to Kotlin.
- Rust core: Argon2id + XChaCha20-Poly1305 recovery file and message padding (9 tests, clippy clean).
- CI guards: banned words, contrast, directional layout, ARB, hardcoded strings, manifest, ELF, icons.
- `sargoshi_ffi` C ABI crate (pad/unpad, recovery create/restore, zeroing buffers, ABI version check) and `dart:ffi` bindings. Dart tests run against the host-built library in CI (88 tests total). Packaged into each per-ABI APK via cargo-ndk with 16 KB alignment (CI-checked). Measured release APKs: arm64 16.62 MiB (budget 45), armeabi-v7a 14.28, x86_64 17.91. Not yet loaded at runtime on a device or emulator.
- Vault PIN verifier in the Rust core: Argon2id (min 19 MiB) keyed with a 32-byte platform pepper, constant-time compare, hostile parameters rejected (6 Rust tests). FFI ABI v2. Dart `NativePinVerifier` evaluates real and decoy records every time (5 tests against the real library). `IsolatePinVerifier` runs the derivation on a background isolate. The vault controller and screen are now asynchronous (busy state, overlapping attempts ignored, results discarded if the vault is locked mid-check, a failed check shows "could not run" and is not counted as a wrong PIN). The app still ships the demo verifier: `VaultPepperStore.kt` (random 32-byte secret sealed by a non-exportable Keystore AES-GCM key, StrongBox when available, stored in the no-backup directory; `read` never creates, so a lost secret cannot become "wrong PIN") and the Dart `KeystorePepperSource` (method channel `app.sargoshi/vault_pepper`, mocked-channel tests) are built and compile in CI but have NEVER run on a device or emulator. Not done: stored PIN records, enrollment flow, and switching the app from the demo verifier.
- Layout tests at 320x568 and text x1.0/1.3/2.0 for chat list, thread and vault.

## Demo only (no real security)
- Vault screen: demo PIN verifier (PIN 1234, decoy 0000). No Keystore, no SQLCipher, no key derivation. Entry is a hidden long-press on the wordmark. A visible banner says this is a demo.
- Trust and key-change review: in-memory state machine. While a key change is under review, sent drafts are queued and released on accept.
- No CryptoCore or FakeCryptoCore, no Signal or MLS, no networking, no storage.

## Not built yet
Kotlin lint, Phosphor icon package, tier calibration on devices and diagnostics row, "Always light" persistence, CryptoCore with a "not in release" CI check, real Drift/SQLCipher storage, Runtime load test of the native library on an emulator or device, go_router wiring, explicit dependency versions (several use `any`, pinned only by `pubspec.lock`), Riverpod 3 migration.

## TODO(verify)
1. Tier thresholds on real devices.
2. Argon2 calibration on target phones; audit status of the new RustCrypto majors.
3. AGPL-3.0 compatibility of all dependencies; legal review.
4. All Stage 0A items (SQLCipher wiring, key handoff, Android 14+ behaviours, PiP, OEM intents, Phosphor licence).

## Decisions needed from you
1. Confirm or reverse the choices in ADR-006.
2. Provide the real domain for App Links and the pinned API host.

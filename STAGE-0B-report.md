# Stage 0B report: project skeleton (NOT DONE, partially verified)

Per spec section 14, a stage is not done while required checks are unrun. The Dart SDK could not be downloaded in this environment (storage.googleapis.com returned 403 through the proxy), so no Dart or Flutter code has been compiled, analyzed or tested.

## Built
- ADR-006 recording owner decisions made on your behalf (AGPL-3.0 with libsignal, OpenMLS capped at 64, no bundled AI model, keep armeabi-v7a behind a size gate, placeholder domain).
- pubspec.yaml, analysis_options.yaml (avoid_print is an error), l10n.yaml, English ARB.
- `Measured<T>`: private constructor, factories require a non-empty source, `unknown` for unavailable values.
- Colour tokens (dark and light), radii and type scale from the spec; dark and light ThemeData.
- Minimal app entry using localizations, plus a Measured unit test.
- Four CI scripts and a GitHub Actions workflow.

## Verified (actually run)
| Check | Result |
|---|---|
| Banned words over lib, test, android, docs, tool, store | Pass, 14 files |
| Contrast, 29 text/background pairs in both themes | 29/29 pass (lowest: Coral Deep on Parchment 4.93:1) |
| Left/right layout ban | Pass |
| ARB validity | Pass |

## Not verified
- `flutter pub get`, `flutter gen-l10n`, `flutter analyze`, `flutter test` (including measured_test.dart). The Dart code is unreviewed by a compiler and may have errors.
- The `Measured` "compile error without a source" goal: the private constructor and required named `source` enforce it at compile time, but an empty string is only caught by an assert in debug builds.

## Not built yet (Stage 0B items still open)
- Bundled fonts (Inter, Instrument Serif, JetBrains Mono): need a networked environment to download, subset and licence-check. Total payload must be under 250 KB.
- Icon font (Phosphor package plus custom glyph font).
- Kotlin performance tier detector and Pigeon channel.
- Android project (android/ directory, manifest, Gradle, backup rules). It does not exist yet, so the project cannot build an APK.
- Dart lints for hardcoded strings (script not written), Kotlin lint config, banned-words test as a Dart test.
- Most CI items listed at the bottom of ci.yml.

## TODO(verify)
1. ADR-001: confirm Flutter 3.47.1 / Dart 3.13.1 with `flutter --version`; check `sdk: ^3.12.0` in pubspec.
2. Package majors for flutter_riverpod, go_router, flutter_lints.
3. All Stage 0A items (SQLCipher wiring, key handoff, Rust binding, LiveKit encryption options, Android 14+ behaviours, PiP, OEM intents, Phosphor licence, RSuperellipse, font versions).
4. AGPL-3.0 compatibility of every dependency; legal review.

## Commands to run on a machine with Flutter
```
flutter pub get
flutter gen-l10n
dart format --output=none --set-exit-if-changed .
flutter analyze
flutter test
python3 tool/ci/banned_words.py
python3 tool/ci/check_contrast.py
python3 tool/ci/check_directional.py
python3 tool/ci/check_arb.py
```

## Decisions needed from you
1. Confirm or reverse the five choices in ADR-006.
2. Provide the real domain for App Links and the pinned API host.
3. Where should the build and tests run? Without Flutter, Android SDK and a device matrix, I cannot verify Stages 1 onward. Options: you run the commands above and paste results; or provide an environment where the SDKs can be downloaded.

## Addendum: Rust core slice (built and verified here)
`rust/sargoshi_core` compiles and passes 9 unit tests and `clippy -D warnings`:
- Recovery file: Argon2id (min 19 MiB, hostile parameters rejected on restore) plus XChaCha20-Poly1305, header bound as associated data, passphrase minimum 8 characters. Identity keys only.
- Message padding: length-prefixed, power-of-two buckets to 16 KiB then 16 KiB multiples; strict unpad.
- Uses audited RustCrypto crates only; `#![forbid(unsafe_code)]`; Cargo.lock committed.
Not done: published test vectors, fuzzing, zeroize memory checks, cargo-deny/audit, the FFI bridge to Dart, and all Signal/MLS code. The crates are new major versions: TODO(verify) their audit status and the Argon2 calibration on target phones.

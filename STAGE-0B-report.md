# Stage 0B report: project skeleton

Status: PARTIALLY COMPLETE. The skeleton builds and passes CI. Four spec items for 0B are still open (below), so the stage is not marked done.

## Verified by CI on GitHub (all four jobs green)
| Job | What it proves |
|---|---|
| checks | Banned words, contrast (29/29 pairs, both themes), no left/right layout APIs, ARB valid |
| flutter | Flutter 3.50.0 (Dart 3.14.0): pub get, gen-l10n, format, analyze (no issues), test |
| rust | sargoshi_core: fmt, clippy -D warnings, 9 tests (recovery file, padding) |
| android | Release APKs build per ABI; merged manifest passes the allow-list; native libraries pass the 16 KB alignment check |

Measured release APK sizes (skeleton only, not a budget result): arm64-v8a 14.9 MB, armeabi-v7a 12.4 MB.
The merged manifest contains only INTERNET plus the app-internal receiver permission, allowBackup=false, cleartext off, and MainActivity as the only exported component (the exported ProfileInstallReceiver was removed with tools:node="remove").
ELF check: 6 native libraries across 3 ABIs, all 16 KB aligned. This covers only the Flutter engine and app library. SQLCipher, the Rust core, WebRTC and plugins are not in the app yet.

## Built
ADR-001 (measured) and ADR-006; Flutter app skeleton with Measured<T>, tokens, dark and light themes, English ARB; Android project (minSdk 26, target and compile 36, R8 and resource shrinking on, Log stripping, backup exclusion rules, TLS-only config, predictive back flag); Rust core slice; CI.

## Not built (still open in Stage 0B)
1. Bundled fonts (Inter, Instrument Serif, JetBrains Mono): need download, subsetting and licence checks; the ci sandbox here cannot fetch them.
2. Icon font (Phosphor package plus custom glyph font).
3. Kotlin performance-tier detector and its Pigeon channel.
4. Dart lint for hardcoded strings; detekt/ktlint; the banned-words check as a Dart test.

## Known deviations and risks
- material_ui and cupertino_ui are pinned as `any` and pubspec.lock is not committed yet. TODO(verify): commit pubspec.lock so versions are fixed.
- Release builds are signed with the debug key for now. Real signing (Play App Signing) is a Stage 10 item.
- The launcher icon is a placeholder circle.
- Gradle wrapper jar was recovered from Flutter's template and its checksum matched the CI-generated file.
- ADR-006 decisions (AGPL-3.0, OpenMLS, no bundled model, keep armeabi-v7a, placeholder domain) are still awaiting your confirmation.

## TODO(verify)
Everything listed in Stage 0A section 4, plus: Dart 3.14.0 vs pubspec `sdk: ^3.12.0`; AGPL compatibility of dependencies; the DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION entry (added by AndroidX, app-internal).

## Run locally
flutter pub get && flutter gen-l10n && flutter analyze && flutter test
python3 tool/ci/check_manifest.py <merged manifest>; python3 tool/ci/check_elf_alignment.py <apk>
cd rust/sargoshi_core && cargo test --release

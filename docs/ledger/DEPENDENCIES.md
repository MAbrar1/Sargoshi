# DEPENDENCIES

Toolchain versions are recorded by P00 from the real machine. Every library is added here when first used: name, exact version, licence, source (pub.dev / crates.io / Maven), why, and the date you verified it exists and is maintained.

## Toolchain (filled by P00)
| Tool | Version |
|---|---|
| rustc / cargo / clippy / rustfmt | 1.100.0 / 1.100.0 / 0.1.100 / 1.11.0 (conda-forge, build sandbox) |
| cargo-deny | 0.22.1 (conda-forge) |
| PostgreSQL | 19.0 (conda-forge, native, used instead of Docker in the sandbox) |
| Redis | 8.10.3 (conda-forge, native, used instead of Docker in the sandbox) |
| node | v22.23.5 |
| git | 2.43.0 |
| Flutter / Dart / Android SDK / NDK / adb / Docker / Gradle | NOT AVAILABLE in the build sandbox (verified 25 Nov 2026); required on the owner's machine, see EXTERNAL.md |
| Rust Android targets (aarch64-linux-android, armv7-linux-androideabi, x86_64-linux-android) | NOT INSTALLED in the build sandbox |

## Libraries
| Ecosystem | Name | Version | Licence | Why | Verified |
|---|---|---|---|---|---|

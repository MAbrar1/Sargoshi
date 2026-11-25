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
| crates.io | prost | 0.14.5 | Apache-2.0 | Protobuf encoding of the envelope body in sg-common; decoder re-encodes to enforce canonical form | 25 Nov 2026 (published 10 Nov 2026). Audit status: unknown |
| crates.io | thiserror | 2.0.21 | MIT OR Apache-2.0 | Error type derive in sg-common | 25 Nov 2026 (published 23 Sep 2026). Audit status: unknown |
| crates.io | proptest | 1.12.0 | MIT OR Apache-2.0 | Property tests (dev-dependency only) | 25 Nov 2026 (published 10 Oct 2026). Audit status: unknown |

### Lockfile supply-chain record (B1, 25 Nov 2026)
- All 58 crates.io packages in Cargo.lock were checked with `node tools/check-dep-age.mjs`: every one is at least 14 days old (Prompt B hard rule 7).
- Cargo had resolved nine transitive crates that were too new; they were downgraded with `cargo update --precise`: anyhow 1.0.105 -> 1.0.104, bytes 1.12.2 -> 1.12.1, errno 0.3.15 -> 0.3.14, hashbrown 0.17.2 -> 0.17.1, prost-derive 0.14.6 -> 0.14.5, rand 0.10.4 -> 0.10.3, regex-syntax 0.8.12 -> 0.8.11, toml_edit 0.25.21 -> 0.25.19, toml_parser 1.1.7 -> 1.1.5.
- `cargo deny check` passes (advisories, bans, licences, sources). One locked crate (r-efi, Windows/UEFI only) is licensed `MIT OR Apache-2.0 OR LGPL-2.1-or-later`; Sargoshi uses it under MIT/Apache, no copyleft applies.
- Re-run `node tools/check-dep-age.mjs` after every `cargo update` or new dependency.


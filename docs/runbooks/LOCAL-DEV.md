# LOCAL-DEV

## The gate
- `node tools/gate.mjs` project checks (tree, reality, registry, generators)
- `node tools/gate.mjs --full` plus flutter analyze/test and cargo fmt/clippy/test/deny when those projects exist
- `node tools/gate.mjs --phase P07B` plus: every row up to that phase must be terminal

## Ledger
- `node tools/ledger.mjs status` / `next` / `start` / `claim` / `waive`

## Rust toolchain on the build sandbox (verified commands, 25 Nov 2026)
```
curl -O https://conda.anaconda.org/conda-forge/linux-64/micromamba-2.10.0-0.tar.bz2   # then extract bin/micromamba
export MAMBA_ROOT_PREFIX=$HOME/mamba
micromamba create -y -n sg -c conda-forge rust cargo-deny postgresql redis-server
export PATH=$MAMBA_ROOT_PREFIX/envs/sg/bin:$PATH
rustc --version && cargo --version && cargo deny --version
```

## Owner machine (required before P01; the P00 toolchain check)
Run and paste the output: `flutter --version`, `flutter doctor -v`, `dart --version`, `rustc --version`, `cargo --version`,
`rustup target list --installed`, `java -version`, `adb version`, `docker --version`, `docker compose version`, `node --version`, `git --version`.
Needed: Android SDK platform 36 + build-tools + NDK; Rust targets aarch64-linux-android, armv7-linux-androideabi, x86_64-linux-android;
an emulator (`emulator -list-avds`) or a phone (`adb devices`).

## Known-answer vectors
- `node tools/gen-vectors.mjs` regenerates `core/crates/sg-testvectors/vectors/` from pinned upstream commits (needs network)
- `node tools/gen-vectors.mjs --check` offline integrity check (the gate runs this); add `--online` to re-download upstream and compare
- `node tools/check-dep-age.mjs` after every `cargo update` or new dependency

## Fuzzing (`core/fuzz`)
Five libFuzzer targets: `envelope_decode`, `handshake_decode`, `blob_decode`, `session_decrypt_raw`, `session_decrypt_framed`.

With cargo-fuzz and a nightly toolchain (recommended for a long campaign; the `--fuzz-dir` flag has not been checked against the installed cargo-fuzz version here, VERIFY): `cargo install cargo-fuzz --locked`, then from the repository root `cargo +nightly fuzz run --fuzz-dir core/fuzz envelope_decode`.

Without cargo-fuzz, on a stable toolchain (verified on the build sandbox; coverage-guided but without AddressSanitizer). The explicit `--target` keeps the instrumentation flags away from build scripts:
```
export CARGO_TARGET_DIR=$HOME/fuzz-target
export RUSTFLAGS="--cfg fuzzing -Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-level=4 -Cllvm-args=-sanitizer-coverage-inline-8bit-counters -Cllvm-args=-sanitizer-coverage-pc-table -Cllvm-args=-sanitizer-coverage-trace-compares -Cdebug-assertions -Coverflow-checks"
cargo build --release --target x86_64-unknown-linux-gnu --manifest-path core/fuzz/Cargo.toml --bins
$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/envelope_decode -max_total_time=60 -max_len=8192 <corpus-dir> [seed-dir]
```
Valid seed inputs matter: canonical parsers reject random bytes at the first signature check, so generate seeds from a real handshake and conversation. A crash is written as `crash-<hash>` in the working directory.
Supply-chain checks for the fuzz crate: `node tools/check-dep-age.mjs` (covers both lockfiles) and `cargo deny --manifest-path core/fuzz/Cargo.toml check`.

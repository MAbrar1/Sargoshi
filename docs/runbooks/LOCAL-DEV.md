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

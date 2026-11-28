# core/fuzz
libFuzzer targets (cargo-fuzz) for every parser and for the ratchet's decrypt path. This crate has its own workspace and needs a nightly Rust toolchain plus `cargo install cargo-fuzz --locked`.
Run one target: `cargo +nightly fuzz run --fuzz-dir core/fuzz envelope_decode` (from the repository root).
Each target asserts the same two properties: parsers accept only canonical input (re-encoding reproduces it exactly), and a failed decrypt never changes a session.
Corpora, artifacts and coverage are ignored by git.

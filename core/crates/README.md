# core/crates
Rust crates of the cryptographic and protocol core. All are named `sg-*` and live in the root workspace.
Only `sg-ffi` may contain `unsafe`; every other crate root has `#![forbid(unsafe_code)]`.
Crate names follow docs/ledger/PROJECT-TREE.md (see DECISIONS D-17 for the mapping from Prompt B names).

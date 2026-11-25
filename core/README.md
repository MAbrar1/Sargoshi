# core
The shared Rust workspace for cryptography and protocol logic.
Allowed here: `crates/` holding `sg-*` crates (created in P02): identity, prekeys, ratchet, vault keys, FFI.
Only `sg-ffi` may contain `unsafe`; no custom cryptographic primitives, only maintained audited crates.
Not allowed: UI code, server-only code, secrets or logs of key material.

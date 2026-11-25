# server
The Rust gateway: accounts, key bundles, message relay, media, push, device linking, abuse controls.
Allowed here: `crates/` (gateway crates, from P03) and `migrations/` (forward-only SQL, from P03).
The server only ever sees ciphertext and routing metadata; it must never hold message plaintext or user keys.
Not allowed: client code, configuration with real credentials.

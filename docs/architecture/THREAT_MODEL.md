# THREAT MODEL

Status: milestone B1. Covers what exists today and states what is out of scope. Updated by every later milestone. Not reviewed by a cryptographer.

## 1. Assets
Message plaintext; long-term identity keys; per-device prekeys and ratchet state; recovery phrase and backup key; Vault key; the social graph and timing of conversations.

## 2. Adversaries considered
- A network attacker who can read, drop, replay, reorder or modify traffic.
- A malicious or compromised gateway operator (stores only ciphertext by design; sees routing metadata).
- A malicious peer sending hostile input (oversized, malformed or crafted messages).
- A person with brief physical access to an unlocked or locked phone (handled by later Vault and app-lock phases).

## 3. What B1 provides
- Strict, canonical parsing of the outer envelope and header: no unknown fields, no alternative encodings, a hard size limit, rejection of every unsupported version. Verified by property tests including decoding of arbitrary bytes without panic.
- Stable error codes so failures never carry secrets (the types in this crate hold no key material).

## 4. Known limits (stated now so nobody assumes otherwise)
- The gateway sees who talks to whom, when, and how big envelopes are. Sealed sender and message franking are not implemented; field 15 is reserved for them.
- Ongoing post-quantum rekeying is not implemented (see PQ-RATCHET.md). The only acceptable label is "Post-quantum protected (beta)". Nothing may say audited, unhackable or military-grade.
- A compromised endpoint (malware, rooted device with the app unlocked) defeats any end-to-end protocol.
- No traffic-analysis resistance beyond padding to 160-byte blocks (padding is added in B5).

## 4a. Open questions for the cryptographer
Collected in AUDIT-PREP.md.

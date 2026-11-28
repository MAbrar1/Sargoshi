# THREAT MODEL

Status: milestones B1, B2, B3 and B4. Covers what exists today and states what is out of scope. Updated by every later milestone. Not reviewed by a cryptographer.

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

## 3a. What B2 adds
- Every primitive is checked against independent vectors (Wycheproof, NIST ACVP, RFCs), including attack cases: low-order X25519 points, forged AEAD tags, malformed and weak-key Ed25519 signatures, invalid ML-KEM keys.
- Secret keys zeroize on drop, never print, and compare in constant time.
- Large data is protected by a streaming AEAD that detects truncation, reordering, duplication and splicing.

## 3b. What B3 adds
- A hybrid handshake: the initial session key depends on X25519 and on ML-KEM-768, so recorded traffic stays protected unless both are broken.
- Mutual authentication of the identity DH keys through DH1 and DH2, and ephemeral and one-time keys for forward secrecy of the initial key.
- Every signature in a bundle is verified before any secret operation; a parsed bundle, upload or header is always a verified value.
- Replays of an accepted initial message are rejected, a used one-time prekey is deleted, and a failed acceptance never changes stored state.

## 3c. What B4 adds
- Forward secrecy per message: a message key is derived once, used once and deleted; chain keys only move forward.
- Post-compromise security: after a key compromise, the session heals once both sides have exchanged a message in each direction (a new Diffie-Hellman ratchet round), as long as the attacker does not keep stealing keys.
- Forged, corrupted, replayed or reordered messages cannot corrupt a session or advance its state, and a hostile header costs a bounded amount of work (limits are checked before any key is derived).
- Session state blobs are parsed strictly; the parsers and the decrypt path were fuzzed (about 25 million executions, no crash).

## 4. Known limits (stated now so nobody assumes otherwise)
- Message keys that were skipped (out-of-order or lost messages) stay cached, up to 2,000, until the late message arrives or the app discards them; for those messages forward secrecy lasts only until then. The app should discard old skipped keys by age.
- Ratchet headers are not encrypted: the gateway sees each message's ratchet public key and counters, and so the rhythm of a conversation.
- Post-compromise security is classical only: without the post-quantum ratchet (see PQ-RATCHET.md), a future quantum attacker who recorded traffic and later obtains one chain key can follow the conversation until the next Diffie-Hellman step that it cannot compute.
- The responder cannot send before receiving, so a first message from the responder's side must wait.
- Message length is visible unless the caller pads (padding to 160-byte blocks arrives in B5).
- The post-quantum prekey is reusable and rotated on a schedule; until it is rotated and pruned, an attacker who later steals its secret and has recorded the initial messages can recompute their post-quantum contribution.
- The replay memory holds the 4,096 most recent ephemeral keys; replays older than that are stopped only by prekey rotation.
- Trust in an identity is decided by the app (first contact and the safety number, milestone B5). A server that substitutes a complete bundle at first contact cannot be detected without that comparison.
- The prekey, ratchet and one-time identifiers in the initial header are not authenticated by the handshake; tampering with them only causes the session to fail.
- A server can withhold one-time prekeys; the handshake then runs without DH4 (still secure, with weaker replay properties, which the replay memory covers).
- Zeroization is best effort: copies of a secret can remain in registers, stack frames or allocator memory, and the core does not lock memory pages. Callers that use `expose` accessors can copy secrets.
- Constant-time behaviour is the dependencies' claim; it has not been independently measured here.
- The gateway sees who talks to whom, when, and how big envelopes are. Sealed sender and message franking are not implemented; field 15 is reserved for them.
- Ongoing post-quantum rekeying is not implemented (see PQ-RATCHET.md). The only acceptable label is "Post-quantum protected (beta)". Nothing may say audited, unhackable or military-grade.
- A compromised endpoint (malware, rooted device with the app unlocked) defeats any end-to-end protocol.
- No traffic-analysis resistance beyond padding to 160-byte blocks (padding is added in B5).

## 4a. Open questions for the cryptographer
Collected in AUDIT-PREP.md.

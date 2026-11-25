# PQ-RATCHET (design note, not implemented)

Prompt B forbids implementing ongoing post-quantum rekeying (the sparse ML-KEM ratchet). The initial handshake is post-quantum hybrid (X25519 plus ML-KEM-768) in milestone B3. After the handshake, the Double Ratchet is classical only, so an attacker who records traffic today and later breaks X25519 could read messages encrypted after the handshake's protection no longer matters for the chain keys. A `PqRatchetHook` trait with a no-op default will exist in `sg-ratchet` (B4) so a real implementation can be added without changing the session format.

## Sizes (ML-KEM-768, FIPS 203)
Encapsulation key 1,184 bytes; ciphertext 1,088 bytes; shared secret 32 bytes. One completed rekey exchange costs 1,184 + 1,088 = 2,272 bytes of extra traffic.

## Message flow (sketch for review)
1. Side A attaches a fresh ML-KEM encapsulation key to an outgoing message.
2. Side B encapsulates to it and attaches the ciphertext to its next message.
3. Both sides mix the shared secret into the root key through the existing root KDF, so a later break of X25519 alone no longer exposes the chains.

## Cadence options and cost
| Cadence | Extra bytes per message on average |
|---|---|
| every 25 messages | about 91 |
| every 50 messages | about 45 |
| every 100 messages | about 23 |
| on every ratchet turn | 2,272 per turn (heavy for chatty conversations) |
The choice trades protection window against bandwidth and battery. It needs the cryptographer's input.

## Honest status
Not implemented. The app label is "Post-quantum protected (beta)".

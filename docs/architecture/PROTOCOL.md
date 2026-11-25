# PROTOCOL

Status: milestone B1 of Prompt B (blueprint 19.2). This document describes only what is implemented and tested. Sections for later milestones are listed in section 7 and are added when their code exists. Every deviation from Prompt B is in section 6. This protocol has not been reviewed by a cryptographer (risk R-01) and must not be described as audited.

## 1. Conventions
- Integers on the wire are big-endian. Keys are X25519 and Ed25519 unless stated.
- All constants live in `core/crates/sg-common/src/constants.rs` and are tested against the blueprint text (`tests/constants.rs` fails if a label or number in code no longer appears in the specification).
- The core is stateless and never touches disk or network (Prompt B hard rule 5).

## 2. Versioning
The protocol version is 1. Every wire message begins with a version byte. A receiver accepts exactly version 1; version 0 and every other value are rejected with `UnsupportedVersion` (tested for all 256 byte values).

## 3. Domain-separation labels
| Use | Label |
|---|---|
| Signature over the DH identity key | `Sargoshi/IK-DH/v1` |
| Signature over a signed prekey | `Sargoshi/SPK/v1` |
| Signature over a post-quantum signed prekey | `Sargoshi/PQSPK/v1` |
| Handshake HKDF info | `Sargoshi/PQXDH/v1` |
| Double Ratchet root KDF info | `Sargoshi/DR/v1/root` |
| Double Ratchet message key info | `Sargoshi/DR/v1/message` |
| Backup key HKDF info | `Sargoshi/Backup/v1` |

## 4. Wire formats
### 4.1 Ratchet header (41 bytes, fixed)
`version(1) || ratchet_pub(32) || previous_chain_length(u32) || message_number(u32)`.
The serialized header is authenticated as associated data together with the session AD (defined in the handshake section). A fixed layout is used so the authenticated bytes are canonical by construction.

### 4.2 Envelope
`version(1) || protobuf(body)` with body fields: 1 `header` (bytes, exactly the 41-byte header), 2 `ciphertext` (bytes, at least 16 bytes: the AEAD tag), 15 `reserved` (bytes, must be empty in version 1).
Canonical rule: the decoder re-encodes the parsed body and requires the bytes to be identical to the input. Unknown fields, reordered fields, non-minimal varints and trailing bytes are therefore rejected (`NonCanonical` or `Malformed`). A non-empty field 15 is rejected with `ReservedFieldSet`. The total size may not exceed 128 KiB (`TooLarge`).
Field 15 is reserved for the message-franking commitment and for sealed-sender extensions. When sealed sender ships, the sender identity moves inside the encrypted payload and delivery uses an access key derived from the recipient's profile key; that change will use a new protocol version, not a silent extension of version 1.

## 5. Error codes
Stable numeric codes (never renumbered): 1000 UnsupportedVersion, 1001 Truncated, 1002 TooLarge, 1003 Malformed, 1004 NonCanonical, 1005 BadLength, 1006 ReservedFieldSet. Ranges: 2000s primitives, 3000s identity and handshake, 4000s ratchet, 5000s groups, 6000s vault and backup, 9000s FFI and internal.

## 6. Deviations from Prompt B
1. Crate names follow the repository layout (DECISIONS D-17): `sg-common` replaces `sg-proto`, `sg-testvectors` replaces `sg-testvec`, `sg-group` replaces `sg-mls`; handshake and ratchet are separate crates `sg-handshake` and `sg-ratchet`.
2. The ratchet header is a fixed binary layout rather than a protobuf message (still contains exactly the fields Prompt B lists).
3. Maximum envelope size 128 KiB is a Sargoshi decision (not in Prompt B).
4. `cargo audit` is replaced by `cargo deny check advisories` (DECISIONS D-19).

## 7. Sections added by later milestones
B2 primitives and streaming AEAD; B3 identity, prekeys, bundle format and handshake; B4 Double Ratchet, skipped-key limits, replay rejection, state serialization; B5 safety number, padding, recovery phrase, backup and vault key derivation; B6 FFI error mapping. Open items marked `VERIFY:` in the code are listed in AUDIT-PREP.md.

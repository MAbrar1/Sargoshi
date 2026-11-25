# P10 — Vault core: format, keys, lock model, viewers, leak harness

Depends on: P09 P02. Ledger rows owned by this phase: 59 (Registry features 14, Settings 8, Vault leak vectors 25, Screens 12)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Make the two Vault promises true and testable: nothing leaves the Vault to the gallery, other apps, the system or any backup. Implement exactly Prompt C (blueprint lines 2860-2950) for the Vault half.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §8 (lines 1337-1485) The Private Vault (all of 8.1 to 8.11)
- §19.3 (lines 2860-2951) Prompt C: ROLE, DELIVERABLES 1,3,4,5, HARD RULES, SPECIFICATION VAULT FORMAT AND KEYS, FLOWS, TESTS (Vault parts), V5 addendum
- §7.9 (lines 1043-1078) VLT (P10 rows)
- §13.3 (lines 1863-2150) group D
- spec-data: leak-matrix.csv

## Build (do every item, in order, as real working code)

1. sg-vault crate: SGO1 object format (header, 64 KiB chunks, STREAM construction), key hierarchy (VMK, Argon2id PIN key, Keystore HMAC entanglement, HKDF unlock key, biometric copy, recovery wrap, derived keys), seeking, tamper and truncation detection, secure delete. It reuses the P02 streaming AEAD. Expose to Kotlin through the sg-ffi facade (UniFFI or JNI; decide, record in DECISIONS.md; unsafe only in sg-ffi).
2. Kotlin `:engine` Gradle module (apps/mobile/android/engine): Keystore entanglement, BiometricPrompt, lock model with the exact lockout delays and the signed no-backup counter, secure-window plumbing, import with Photo Picker and the per-API remove-original rules (Android 11+ `MediaStore.createDeleteRequest`, Android 10 recoverable-security flow, Android 9 legacy), export with metadata stripping, trash with 30-day purge, Vault SQLCipher database, Media3 custom decrypting data source (no temp files, no local server, no listening sockets), in-memory image and PDF viewing, camera and recorder writing straight to the encryptor (the scratch-file muxer exception only as Prompt C describes), Vault inbox for messages that arrive while locked, move chat in and out.
3. Every Vault Activity or route: FLAG_SECURE, recents screenshot disabled, excluded from recents, overlay hiding and obscured-touch filtering, autofill off, suggestions off, copy disabled in viewers, zero-size image cache for Vault images. The Vault has its own navigator that is cleared on lock; Vault routes are never deep-linkable (6.12). PIN pad from P01.
4. UI: Vault lock, setup (PIN of 6 or more digits or passphrase of 8 or more with strength meter and the plain-words limits text), home, gallery with privacy blur grid, viewer, files, audio, chats, trash, import and export flows, Vault camera. Add the Vault card to Secure setup.
5. Leak harness (8.10) as instrumented tests plus a host-side plaintext scanner: implement the 9 steps for the vectors owned by this phase (LKV rows with build_phase P10). Run them on an emulator or device and append each real result to docs/quality/DEVICE-RUNS.md (`LKV-01 | pixel-emulator | android-14 | <command> | PASS`).

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P10 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (14)

| ID | What | Notes |
|---|---|---|
| VLT-01 | Vault space with its own lock: PIN (6 or more digits) or passphrase (8 or more characters), optional biometrics | bp line 1047 |
| VLT-02 | Vault Gallery: photos, videos and GIFs with albums, favourites, slideshow and encrypted thumbnails | bp line 1048 |
| VLT-03 | Vault Files: documents, PDFs, archives and any file type, with an in-app viewer | bp line 1049 |
| VLT-04 | Vault Audio: audio files and voice notes with player, speed control and local transcript | bp line 1050 |
| VLT-06 | Vault Chats: move any chat in, including history and media. Messages that arrive while locked wait sealed | bp line 1052 |
| VLT-09 | Vault camera and recorder: photos, video and voice go straight into the Vault and are never written to the gallery | bp line 1055 |
| VLT-10 | Import from gallery or files with "Remove original" (system consent) and a leftover-trace report | bp line 1056 |
| VLT-11 | Export with unlock and a warning, to a place you choose, with location and camera data removed | bp line 1057 |
| VLT-12 | Viewers that decrypt on the fly: no temporary plaintext file, no local web server. Screenshots, recording and casting blocked. Hidden from recent apps | bp line 1058 |
| VLT-13 | Zero system footprint: nothing in MediaStore, shared storage, system thumbnails, search, widgets, shortcuts, notifications, autofill, clipboard, Android backup or the ... | bp line 1059 |
| VLT-14 | Lock model: auto-lock choices, lock on screen off, lock on app switch, biometric re-authentication on each open, escalating lockout | bp line 1060 |
| VLT-19 | Vault trash with 30-day auto-purge, inside the Vault | bp line 1065 |
| VLT-20 | Secure delete: destroy the item key, plus best-effort overwrite (honest note for flash storage) | bp line 1066 |
| VLT-23 | Privacy blur grid: Vault thumbnails stay blurred until tapped | bp line 1069 |

### Settings (8) — ids: STD-01, STD-02, STD-03, STD-04, STD-05, STD-06, STD-07, STD-13

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Vault leak vectors (25)

| ID | What | Notes |
|---|---|---|
| LKV-01 | Phone gallery and MediaStore |  |
| LKV-02 | System thumbnail caches |  |
| LKV-03 | Google Photos and other cloud backup of originals |  |
| LKV-04 | Android Auto Backup and device-to-device transfer (both on by default) |  |
| LKV-05 | File managers and other apps |  |
| LKV-06 | Temporary plaintext files and caches |  |
| LKV-07 | Local web server for video |  |
| LKV-08 | Share sheet and "Open with" |  |
| LKV-09 | Screenshots, screen recording, casting, recent apps |  |
| LKV-10 | Notifications |  |
| LKV-11 | Widgets, shortcuts, Direct Share, assistants |  |
| LKV-12 | System and app search |  |
| LKV-13 | Clipboard |  |
| LKV-14 | Keyboard learning and suggestions |  |
| LKV-15 | Autofill |  |
| LKV-16 | Accessibility services reading the screen |  |
| LKV-17 | Overlays and tapjacking |  |
| LKV-18 | Logs and crash reports |  |
| LKV-20 | Memory |  |
| LKV-21 | Before first unlock after reboot |  |
| LKV-22 | ADB backup and debugging |  |
| LKV-23 | Deleted originals on flash storage |  |
| LKV-24 | Photos of the screen by a camera |  |
| LKV-25 | Contacts and calendar providers |  |
| LKV-26 | Links opened from notes |  |

### Screens (12)

| ID | What | Notes |
|---|---|---|
| SCR-081 | Vault lock |  |
| SCR-082 | Vault setup |  |
| SCR-083 | Vault home |  |
| SCR-084 | Vault gallery and albums |  |
| SCR-085 | Vault viewer |  |
| SCR-086 | Vault files and viewer |  |
| SCR-087 | Vault audio |  |
| SCR-088 | Vault camera and recorder |  |
| SCR-089 | Vault import and remove originals |  |
| SCR-090 | Vault export |  |
| SCR-091 | Vault chats |  |
| SCR-092 | Vault trash |  |


## Tests (required)

- Property tests: any byte flip, truncation or reorder of an object fails; random seek equals sequential read; wrap/unwrap round trips; lockout timing table. Known-answer tests reused from P02. Unit tests for every flow. Device runs for the harness.

## Done only when ALL are true

- Every ledger row of P10 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P10` shows no open rows.
- `node tools/gate.mjs --phase P10 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P10 VLT row claimed and every LKV row of this phase has a PASS line in DEVICE-RUNS.md. If no emulator or device is available STOP and ask the owner; do not claim.
- PROGRESS.md updated and `CURRENT_PHASE: P11` set; all work committed.

## Stop conditions specific to this phase

- If a Vault security requirement cannot be met on a given API level, say so and document it in docs/architecture/THREAT-MODEL-VAULT.md instead of weakening the promise.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

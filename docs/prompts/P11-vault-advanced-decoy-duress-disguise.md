# P11 — Vault advanced: decoy, duress, disguise, notes, IDs, checkup, footprint

Depends on: P10. Ledger rows owned by this phase: 29 (Registry features 12, Alerts 1, Settings 6, Journeys 1, Screens 9)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Complete the Vault: notes, private contacts, ID wallet, decoy and duress, disguise, intruder log, send-from-Vault, storage manager, checkup, footprint scan, and the bulk move-to-Vault action.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §8.3 (lines 1359-1379) to 8.8 (decoy, duress, disguise, intruder log, screens)
- §19.3 (lines 2860-2951) Prompt C FLOWS (footprint scan, checkup) and TESTS
- §7.9 (lines 1043-1078) VLT (P11 rows) and MSG-31
- §17 (lines 2445-2477) rows on disguised icons, broad media permission, intruder photo

## Build (do every item, in order, as real working code)

1. Notes (rich text, checklists, attachments, no sync), private contacts that never enter the phone Contacts, ID wallet with expiry chips and generic reminder text ("Document reminder"), decoy vault (same-size wrapped keys, random bytes when disabled, constant-time unlock across paths) and duress PIN (opens decoy, hides the real entry until the real PIN is entered twice, optional quiet alert to one trusted contact, never wipes by default), disguised icon through activity aliases chosen by the person (Calculator, Notes, Clock) with the hidden entry gesture, intruder log with consent screen (photo off by default), send from Vault with no cache copy and Message Rules applied, storage manager (compress, recompress), Vault checkup (every object authenticates, stray temp files, orphans, risky settings: non-system accessibility services, overlay permission, screen recorders, USB debugging, root), footprint scan with on-device perceptual hashes (Photo Picker or chosen-folder fallback when broad media access is refused) and system delete requests, MSG-31 bulk action to move messages to the Vault.
2. Record the Play-policy risk of VLT-17 and the broad media permission in docs/compliance/PLAY-CHECKLIST.md.

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P11 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (12)

| ID | What | Notes |
|---|---|---|
| MSG-31 | Bulk actions: select many messages to forward, star, delete, export or move to the Vault | bp line 935 |
| VLT-05 | Vault Notes: rich text, checklists and attachments | bp line 1051 |
| VLT-07 | Vault Contacts: private contact cards that never enter the phone Contacts | bp line 1053 |
| VLT-08 | ID wallet: scans of CNIC, passport, licence and cards, with expiry reminders | bp line 1054 |
| VLT-15 | Decoy vault: a second PIN opens a believable vault with content you choose | bp line 1061 |
| VLT-16 | Duress PIN: opens the decoy, hides the real Vault entry, can quietly alert a trusted contact | bp line 1062 |
| VLT-17 | Disguised app icon and name (Calculator, Notes, Clock) chosen by you, with the Vault entry hidden behind a gesture | bp line 1063 |
| VLT-18 | Intruder log: time and count of wrong attempts, with an optional front-camera photo kept inside the Vault (off by default) | bp line 1064 |
| VLT-24 | Send from the Vault straight into a chat with no cache or gallery copy. The attachment follows Message Rules | bp line 1070 |
| VLT-25 | Vault storage manager: size per album, compress photos, recompress video | bp line 1071 |
| VLT-26 | Vault checkup: verifies every item decrypts, finds stray temporary files, warns about risky settings (accessibility services, overlay permission, screen recorders, USB... | bp line 1072 |
| VLT-27 | Footprint scan: finds leftover copies of Vault items in the gallery, Downloads and chat media folders using on-device image hashes, and offers to remove them (needs me... | bp line 1073 |

### Alerts (1)

| ID | What | Notes |
|---|---|---|
| ALR-15 | Vault checkup found a risk | bp line 1737 |

### Settings (6) — ids: STD-08, STD-09, STD-10, STD-11, STD-14, STD-15

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Journeys (1)

| ID | What | Notes |
|---|---|---|
| JNY-06 | Vault import |  |

### Screens (9)

| ID | What | Notes |
|---|---|---|
| SCR-093 | Vault notes and editor |  |
| SCR-094 | Vault contacts |  |
| SCR-095 | ID wallet |  |
| SCR-096 | Decoy and duress setup |  |
| SCR-097 | Disguise and hidden entry |  |
| SCR-098 | Intruder log |  |
| SCR-099 | Vault storage manager |  |
| SCR-100 | Vault checkup |  |
| SCR-101 | Footprint scan |  |


## Tests (required)

- Tests: decoy indistinguishability (same sizes, same timing within a tolerance), duress behaviour, checkup finds an injected stray file and a corrupted object, footprint scan finds a copy in a test folder, intruder log. Add the leak vectors owned by this phase to the harness and run on device.

## Done only when ALL are true

- Every ledger row of P11 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P11` shows no open rows.
- `node tools/gate.mjs --phase P11 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P11 row claimed and DEVICE-RUNS.md updated.
- PROGRESS.md updated and `CURRENT_PHASE: P12` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

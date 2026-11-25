# P20 — Trust and safety extensions

Depends on: P19. Ledger rows owned by this phase: 17 (Registry features 9, Alerts 1, Settings 1, Journeys 1, Screens 5)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Sealed sender, message franking, look-alike warnings, link warnings, check-in timer, SOS, help directory, restrict, QR safety scanner, Pakistan scam intelligence in the chat flow.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §7.8 (lines 1018-1042) PRV-04, §7.10 (lines 1079-1100) SAF (P20 rows), §14.2 (lines 2157-2176) rows Sealed sender and Message franking, §14.4 (lines 2181-2194)
- §3 (lines 253-295) Journey J15
- §16.3 (lines 2378-2434) copy (Check-in, SOS, Restrict, Scam pack, OTP guard)
- §11.7 (lines 1712-1748) ALR-16

## Build (do every item, in order, as real working code)

1. Sealed sender (sender identity inside the encrypted envelope, delivery access key derived from the recipient profile key; update PROTOCOL.md and keep the cryptographer note), message franking (commitment, franking key, server tag, reporter reveals the key; reports become verified), look-alike username warning and Mint verified badge, link safety warnings, safety check-in timer (server-held scheduled envelope with heartbeat and honest limits in copy), SOS (hold button, last location, optional 10 second audio clip), local help directory (numbers verified before each release; record the verification date and source in the file, never invent numbers: if you cannot verify a number, leave it out and list it in EXTERNAL.md), restrict with keyword filters, QR scanner with a safety check before opening links or payment codes.

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P20 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (9)

| ID | What | Notes |
|---|---|---|
| PRV-04 | Sealed sender: the server cannot see who sent a message. Envelope designed in MVP, shipped in v1 | bp line 1025 |
| SAF-08 | Link safety warnings | bp line 1090 |
| SAF-09 | Look-alike warning when a new contact's username is close to one you already have; Mint verified badge after you compare safety numbers | bp line 1091 |
| SAF-10 | Message franking so moderators can verify that a reported message was really sent | bp line 1092 |
| SAF-11 | Safety check-in timer: if you do not confirm, trusted contacts get your last shared location and a message | bp line 1093 |
| SAF-14 | SOS: hold a button to send an alert, the last location and an optional 10-second audio clip to trusted contacts. Different from the check-in timer | bp line 1096 |
| SAF-15 | Local help directory: offline list of reporting and helpline contacts for cybercrime, harassment and child safety. Numbers verified before each release | bp line 1097 |
| SAF-16 | Restrict: a person’s messages go quietly to Requests and they are not told. Plus keyword filters | bp line 1098 |
| SAF-17 | QR scanner with a safety check before a link or payment code opens | bp line 1099 |

### Alerts (1)

| ID | What | Notes |
|---|---|---|
| ALR-16 | Check-in timer about to expire | bp line 1738 |

### Settings (1) — ids: STC-09

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Journeys (1)

| ID | What | Notes |
|---|---|---|
| JNY-15 | Scam attempt |  |

### Screens (5)

| ID | What | Notes |
|---|---|---|
| SCR-177 | SOS |  |
| SCR-178 | Check-in timer |  |
| SCR-179 | Local help directory |  |
| SCR-180 | QR scanner with safety check |  |
| SCR-181 | Restrict and keyword filters |  |


## Tests (required)

- Crypto property tests for sealed sender and franking; check-in fault tests (missed heartbeat releases, heartbeat cancels); JNY-15 end to end (OTP guard, scam pack, restrict).

## Done only when ALL are true

- Every ledger row of P20 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P20` shows no open rows.
- `node tools/gate.mjs --phase P20 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P20 row claimed.
- PROGRESS.md updated and `CURRENT_PHASE: P21` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

# P13 — Backup, restore, export and import UI; migrations

Depends on: P12. Ledger rows owned by this phase: 51 (Registry features 20, Alerts 2, Settings 13, Journeys 2, Vault leak vectors 2, Screens 12)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

All backup rows with their real UI, the restore wizard, exports, WhatsApp and Telegram import, phone-to-phone transfer, smart storage, forecast.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §12 (lines 1749-1837) (all), §7.14 (lines 1165-1188) BKP and GRW-01, GRW-02, §13.3 (lines 1863-2150) group J
- §16.3 (lines 2378-2434) copy rows (Backup, Backup health, Restore, Drill failed)
- §3 (lines 253-295) Journeys J4 and J8
- §11.7 (lines 1712-1748) ALR-05, ALR-17

## Build (do every item, in order, as real working code)

1. Backup dashboard with the readiness score and exact copy, destinations, versions, scope, schedule and conditions, recovery key manager (view, test, change, revoke, delete cloud backups), key quiz and reminders feeding the score, restore wizard (new phone, same phone, one chat, chosen version) with chats-first progress and resume, drill results (ALR-17), smart storage offload with tap-to-restore, storage forecast, export chats as HTML, PDF, JSON or text with media and optional encryption, import from WhatsApp export (text with media, locale date formats, senders mapped, messages marked Imported) and Telegram JSON export, phone-to-phone transfer over local Wi-Fi with a QR handshake and encrypted stream, Vault backup and Vault restore on their own screens, encrypted history sync to a newly linked device (BKP-17), progress notifications through ProgressReporter with generic text.

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P13 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (20)

| ID | What | Notes |
|---|---|---|
| BKP-01 | Local encrypted backup to a folder you choose, including SD card or USB-OTG storage | bp line 1169 |
| BKP-02 | Sargoshi Secure Backup in the cloud, zero-knowledge | bp line 1170 |
| BKP-03 | Your own cloud: Google Drive app data, S3-compatible storage or WebDAV | bp line 1171 |
| BKP-04 | 24-word recovery key with optional passphrase, and a printable Recovery Kit with QR | bp line 1172 |
| BKP-05 | Incremental, deduplicated backups with versions (7 daily, 4 weekly, 6 monthly) | bp line 1173 |
| BKP-06 | Schedules and conditions: daily, weekly or manual; Wi-Fi only; charging only; battery above 30% | bp line 1174 |
| BKP-07 | Scope control: chats, media types, size caps, last N days of media, text only | bp line 1175 |
| BKP-08 | Vault backup with its own key, off by default | bp line 1176 |
| BKP-09 | Verification: integrity checks and monthly restore drills on a random sample | bp line 1177 |
| BKP-10 | Restore wizard: new phone, same phone, one chat or a chosen version. Chats first, media in the background, resumes after interruption | bp line 1178 |
| BKP-11 | Phone-to-phone transfer over local Wi-Fi, encrypted, no internet | bp line 1179 |
| BKP-13 | Export a chat or all chats as HTML, PDF, JSON or text with media, optionally encrypted | bp line 1181 |
| BKP-14 | Backup health dashboard: last success, size, next run, restore-readiness score | bp line 1182 |
| BKP-15 | Smart storage: offload old media to your backup and keep thumbnails, restore on tap | bp line 1183 |
| BKP-16 | Change or revoke the recovery key, delete cloud backups | bp line 1184 |
| BKP-17 | Encrypted history sync to a newly linked device | bp line 1185 |
| BKP-18 | Recovery key quiz with reminders at 7, 30 and 90 days, feeding the readiness score | bp line 1186 |
| BKP-19 | Storage forecast: how big the backup will be in 3 and 12 months at the current rate, and what each destination will cost to hold | bp line 1187 |
| GRW-01 | Import WhatsApp chat exports (text with media) on the phone | bp line 1290 |
| GRW-02 | Import Telegram exports (JSON) | bp line 1291 |

### Alerts (2)

| ID | What | Notes |
|---|---|---|
| ALR-05 | No backup, or backup older than 7 days with failures, or older than 30 days | bp line 1727 |
| ALR-17 | Restore drill failed | bp line 1739 |

### Settings (13) — ids: STD-17, STG-10, STJ-01, STJ-02, STJ-03, STJ-04, STJ-05, STJ-06, STJ-07, STJ-08, STJ-10, STJ-11, STJ-12

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Journeys (2)

| ID | What | Notes |
|---|---|---|
| JNY-04 | New phone |  |
| JNY-08 | Backup and restore |  |

### Vault leak vectors (2)

| ID | What | Notes |
|---|---|---|
| LKV-19 | Sargoshi's own backups |  |
| LKV-28 | Live progress and ongoing notifications (NTF-16) |  |

### Screens (12)

| ID | What | Notes |
|---|---|---|
| SCR-102 | Backup dashboard |  |
| SCR-103 | Destinations setup |  |
| SCR-104 | Versions list |  |
| SCR-105 | Scope and schedule |  |
| SCR-106 | Recovery key manager |  |
| SCR-107 | Restore wizard and progress |  |
| SCR-108 | Restore drill result |  |
| SCR-109 | Export chats |  |
| SCR-110 | Import from WhatsApp and Telegram |  |
| SCR-111 | Phone-to-phone transfer |  |
| SCR-112 | Storage forecast and smart storage |  |
| SCR-113 | Vault backup and restore |  |


## Tests (required)

- JNY-04 and JNY-08 end to end; import tests with real sample export files you write; transfer test between two app stacks over loopback; ALR-05 and ALR-17 triggers; goldens.

## Done only when ALL are true

- Every ledger row of P13 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P13` shows no open rows.
- `node tools/gate.mjs --phase P13 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P13 row claimed. LKV rows of this phase PASS in DEVICE-RUNS.md.
- PROGRESS.md updated and `CURRENT_PHASE: P14A` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

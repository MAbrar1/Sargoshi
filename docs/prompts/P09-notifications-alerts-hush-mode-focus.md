# P09 — Notifications, alerts, Hush mode, focus

Depends on: P08. Ledger rows owned by this phase: 42 (Registry features 18, Alerts 1, Settings 15, Screens 8)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

The full notification pipeline and the alert system: content-free push, local notification building, the 8-rule engine in the exact order, channels, Hush mode, health check, alert cards and sheets, focus profiles, prayer times, driving mode, live progress.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §11 (lines 1642-1748) Notifications and alerts (all), spec-data: alerts.csv alert-levels.csv notification-channels.csv notification-rules-order.csv
- §7.15 (lines 1189-1212) NTF rows
- §13.3 (lines 1863-2150) group F
- §8.6 (lines 1408-1441) leak vectors 10 and 28 (notification text)
- §16.3 (lines 2378-2434) copy rows (Hush, Notification health, New device)

## Build (do every item, in order, as real working code)

1. Create the 12 channels (importance, sound as in notification-channels.csv). `NotificationRulesEngine` is a pure Dart class that applies the 8 rules in exactly the order of notification-rules-order.csv; one unit test per rule and per interaction.
2. FCM background handler (`@pragma("vm:entry-point")` top-level function): fetch pending, decrypt, run the engine, post; if anything fails within 10 seconds post the generic "You have new messages". Vault chat messages never produce text (the Vault arrives in P10 and extends the handler).
3. Hush mode with schedule and exceptions, priority contacts, repeat-caller breakthrough, per-chat sound/vibration/mute, privacy levels, grouping and rate limits, badge rules, notification actions (Reply, Mark as read, Mute 1 hour) that work from the shade, notification centre with alert history, health check that detects battery restrictions and permission/channel blocks and opens the exact settings page per brand (Xiaomi, Oppo, Vivo, Samsung, Infinix/Tecno, Pixel).
4. Alert framework: SgAlertCard, banner and sheet by severity; Critical needs a choice and cannot be swiped away; Hush never holds Critical. Implement every ALR row whose build_phase is P09 now and give the other ALR rows their reusable hooks.
5. Focus profiles, prayer-time engine (calculation on the phone from a chosen city and method, no location permission; choose a maintained calculation package or implement the published methods and test against published tables), driving mode (car Bluetooth trigger, text-to-speech, voice replies through the platform recogniser), highlights mode per group (the group UI lands in P14A; build the setting plumbing and test with a synthetic chat), Android 16 progress notifications through `ProgressReporter` with a standard ongoing-notification fallback (generic text only).

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P09 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (18)

| ID | What | Notes |
|---|---|---|
| NTF-01 | Content-free push payloads | bp line 1193 |
| NTF-02 | Notification channels by category | bp line 1194 |
| NTF-03 | Per-chat sound, vibration and mute | bp line 1195 |
| NTF-04 | Hush mode: scheduled quiet hours with exceptions | bp line 1196 |
| NTF-05 | Notification privacy levels | bp line 1197 |
| NTF-06 | Alert system with four severity levels (ALR-01 to ALR-23) | bp line 1198 |
| NTF-07 | Notification health check in Settings: shows whether battery rules are blocking pushes | bp line 1199 |
| NTF-08 | In-app notification centre with alert history | bp line 1200 |
| NTF-09 | Reply, Mark as read and Mute for 1 hour from the notification | bp line 1201 |
| NTF-10 | Smart grouping and rate limits | bp line 1202 |
| NTF-11 | Priority contacts that can break through Hush mode | bp line 1203 |
| NTF-12 | Repeat-caller breakthrough: a second call within 3 minutes rings through | bp line 1204 |
| NTF-13 | Badge count rules (count chats, not messages; muted chats excluded) | bp line 1205 |
| NTF-14 | Prayer-time engine: times calculated on the phone from a city and method you choose. No location permission needed | bp line 1206 |
| NTF-16 | Live progress notifications: Android 16 progress style for backup, restore, large uploads, presenting and check-in timers. Older Android uses a standard ongoing notifi... | bp line 1208 |
| NTF-17 | Driving mode: starts on a car Bluetooth connection or by hand. Reads messages aloud, takes voice replies, rings only priority contacts | bp line 1209 |
| NTF-18 | Highlights mode per group: notify only for mentions, replies and saved contacts | bp line 1210 |
| NTF-19 | Focus profiles: Work, Study, Sleep, Prayer, Driving and custom. Each bundles a Hush schedule, allowed contacts and which chats show | bp line 1211 |

### Alerts (1)

| ID | What | Notes |
|---|---|---|
| ALR-06 | Push health check failed | bp line 1728 |

### Settings (15) — ids: STF-01, STF-02, STF-04, STF-05, STF-06, STF-08, STF-09, STF-11, STF-12, STF-13, STF-14, STF-15, STF-16, STF-17, STF-18

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Screens (8)

| ID | What | Notes |
|---|---|---|
| SCR-073 | Notification centre and alert history |  |
| SCR-074 | Alert sheet (Critical) |  |
| SCR-075 | Hush mode |  |
| SCR-076 | Priority contacts |  |
| SCR-077 | Notification health check |  |
| SCR-078 | Focus profiles |  |
| SCR-079 | Prayer times settings |  |
| SCR-080 | Driving mode |  |


## Tests (required)

- Rules engine tests (all 8 rules and their order), notification text tests (no text for Vault, generic fallback path), instrumented notification-action tests, prayer times against published tables for at least 3 cities and 2 methods.

## Done only when ALL are true

- Every ledger row of P09 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P09` shows no open rows.
- `node tools/gate.mjs --phase P09 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P09 row claimed. ALR rows of this phase claimed.
- PROGRESS.md updated and `CURRENT_PHASE: P10` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

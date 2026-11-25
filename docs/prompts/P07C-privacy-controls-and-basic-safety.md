# P07C — Privacy controls and basic safety

Depends on: P07B. Ledger rows owned by this phase: 44 (Registry features 18, Alerts 1, Settings 15, Journeys 1, Screens 9)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Privacy and safety basics: disappearing timers, visibility controls, screenshot blocking, clipboard guard, privacy blur, requests inbox, report, block, appeals, child-safety page, Trust screen and safety numbers.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §7.8 (lines 1018-1042) PRV (P07C rows), §7.10 (lines 1079-1100) SAF (P07C rows)
- §14.2 (lines 2157-2176) Layers (safety number, verification), §14.4 (lines 2181-2194) Abuse controls
- §16.3 (lines 2378-2434) copy rows (Requests, Safety number, Key change, Report)
- §11.7 (lines 1712-1748) ALR-02
- §3 (lines 253-295) Journey J9

## Build (do every item, in order, as real working code)

1. Disappearing timers per chat plus default (the timer really deletes), hide last seen and online, who can add or call you, screenshot blocking through FLAG_SECURE on chosen scopes (Android), strip tracking parameters, hide app preview in recents, incognito keyboard flag, hide profile photo from non-contacts, per-chat receipt and typing overrides, clipboard guard (sensitive flag, 30 second auto-clear), privacy blur in the chat list, capture notice where Android reports it.
2. Requests inbox (no media or links until accepted), report with franking left for P20 (reports are labelled unverified), block, added-by-stranger preview comes with groups in P14A, spam filter rules plus the server limits, child-safety reporting and the public CSAE standards page content (docs/compliance/CSAE-STANDARDS.md and an in-app page), appeals with the human-review reason shown.
3. Trust screen: safety number (the P02 function), QR compare, key history, verified badge. ALR-02 banner is a chat banner that cannot be dismissed without Verify or Continue anyway.

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P07C --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (18)

| ID | What | Notes |
|---|---|---|
| PRV-01 | Disappearing timers per chat, plus a default timer for new chats (a single message can override it with RSL-01) | bp line 1022 |
| PRV-02 | Hide last seen and online status | bp line 1023 |
| PRV-03 | Control who can add you and who can call you | bp line 1024 |
| PRV-05 | Android: block screenshots. iOS: alert on screenshot or screen recording (iOS cannot block them) | bp line 1026 |
| PRV-06 | Strip tracking parameters from links | bp line 1027 |
| PRV-07 | Hide app preview in the app switcher | bp line 1028 |
| PRV-08 | Incognito keyboard flag in text fields | bp line 1029 |
| PRV-09 | Hide profile photo from people who are not contacts | bp line 1030 |
| PRV-11 | Per-chat overrides for read receipts and typing indicator | bp line 1032 |
| PRV-15 | Clipboard guard: sensitive copy flag, auto-clear after 30 seconds, no copying from the Vault or Whisper messages | bp line 1036 |
| PRV-16 | Privacy blur in the chat list: hide previews and photos until tapped | bp line 1037 |
| PRV-20 | Capture notice: where Android reports a screenshot or recording, show a notice in chats with Whisper or Confidential rules. Does not prevent capture | bp line 1041 |
| SAF-01 | Report and block, from chats, groups, channels and profiles | bp line 1083 |
| SAF-02 | A report sends the selected messages from the reporter's phone, nothing else | bp line 1084 |
| SAF-03 | Requests inbox: messages from unknown senders wait for approval, with no media or links until you accept | bp line 1085 |
| SAF-04 | Spam filter: on-device rules plus server rate limits | bp line 1086 |
| SAF-05 | Child-safety reporting, public CSAE standards page, named safety contact | bp line 1087 |
| SAF-06 | Appeals reviewed by a human, with a clear reason for every ban | bp line 1088 |

### Alerts (1)

| ID | What | Notes |
|---|---|---|
| ALR-02 | A contact's safety number changed | bp line 1724 |

### Settings (15) — ids: STB-01, STB-02, STB-03, STB-04, STB-05, STB-06, STB-07, STB-09, STB-12, STB-13, STB-14, STB-15, STB-16, STB-21, STE-01

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Journeys (1)

| ID | What | Notes |
|---|---|---|
| JNY-09 | Report and appeal |  |

### Screens (9)

| ID | What | Notes |
|---|---|---|
| SCR-053 | Trust screen (safety number, QR) |  |
| SCR-054 | Key-change banner and sheet |  |
| SCR-055 | Requests inbox |  |
| SCR-056 | Disappearing timer sheet |  |
| SCR-057 | Privacy settings pages |  |
| SCR-058 | Report flow |  |
| SCR-059 | Block list |  |
| SCR-060 | Appeal flow |  |
| SCR-061 | Child safety standards page |  |


## Tests (required)

- Tests: timer deletion, visibility rules enforced at the server and client, FLAG_SECURE present (instrumented), clipboard auto-clear, request gating. JNY-09 end to end. Goldens for Trust screen, Requests, banners.

## Done only when ALL are true

- Every ledger row of P07C is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P07C` shows no open rows.
- `node tools/gate.mjs --phase P07C --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P07C row claimed. ALR-02 claimed.
- PROGRESS.md updated and `CURRENT_PHASE: P08` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

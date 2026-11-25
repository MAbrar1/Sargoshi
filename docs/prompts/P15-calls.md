# P15 — Calls

Depends on: P14B. Ledger rows owned by this phase: 41 (Registry features 23, Settings 10, Screens 8)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Real end-to-end encrypted voice and video calls (1-on-1 and group up to 8), native call integration, handover, call links, lobby, hold and waiting, missed-call messages, scheduling, screening, effects, captions hook.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §9.1 (lines 1488-1500) Call design, §7.11 (lines 1101-1130) CAL (P15 rows) and PRV-12
- §19.4 (lines 2952-3009) Prompt D (all) - the authoritative engine spec
- §14.2 (lines 2157-2176) Calls row, §15.3 (lines 2278-2303) call budgets, §11.7 (lines 1712-1748) ALR rows for calls
- §6.17 (lines 827-862) Figure 8 text (call with presenting)

## Build (do every item, in order, as real working code)

1. Decide the media stack and record it: WebRTC with a self-hosted LiveKit SFU (add livekit and TURN to infra/compose, TURN over TCP 443 fallback) and the Flutter client. REQUIRED: frame-level end-to-end encryption keyed from the chat session through a `CallKeyProvider`. If the chosen client library cannot do end-to-end frame encryption with externally supplied keys, STOP and tell the owner (Prompt D hard rule 1); do not ship unencrypted-through-SFU calls.
2. Android: self-managed Telecom through the Jetpack core-telecom library (verify), high-priority incoming call notification, Bluetooth and hearing-aid routing, proximity sensor, network handover Wi-Fi to mobile, low-data mode, noise suppression, picture-in-picture, call waiting and hold, missed-call quick actions and missed-call messages, call links with lobby, scheduled calls with calendar entry and reminder, caller screening, 4-emoji verification, reactions, raise hand, in-call chat, quality report (statistics only), video effects behind a capability check (T2 and up), always-relay option PRV-12.
3. Call state machine documented with a diagram in docs/architecture/CALLS.md.

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P15 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (23)

| ID | What | Notes |
|---|---|---|
| PRV-12 | "Always relay calls": all call traffic goes through our relay, so the other person never sees your IP address | bp line 1033 |
| CAL-01 | 1-on-1 voice call, end-to-end encrypted | bp line 1105 |
| CAL-02 | 1-on-1 video call | bp line 1106 |
| CAL-03 | Native call screen (Telecom on Android, CallKit on iOS) | bp line 1107 |
| CAL-04 | Call history | bp line 1108 |
| CAL-05 | Low-data call mode | bp line 1109 |
| CAL-06 | Missed-call quick actions: Message, Call back, Remind me | bp line 1110 |
| CAL-07 | Group calls up to 8 people | bp line 1111 |
| CAL-08 | Call waiting and hold | bp line 1112 |
| CAL-09 | Call links | bp line 1113 |
| CAL-10 | Noise suppression and adaptive bitrate | bp line 1114 |
| CAL-11 | Raise hand and call reactions | bp line 1115 |
| CAL-12 | In-call chat | bp line 1116 |
| CAL-13 | Picture-in-picture during video calls | bp line 1117 |
| CAL-14 | Call verification with a 4-emoji code | bp line 1118 |
| CAL-15 | Lobby for call links: the host admits people | bp line 1119 |
| CAL-16 | Network handover without dropping the call (Wi-Fi to mobile data) | bp line 1120 |
| CAL-17 | Missed-call message: when a call is not answered, the caller can leave a voice or video message | bp line 1121 |
| CAL-18 | Scheduled calls with a calendar entry, reminder and join link | bp line 1122 |
| CAL-20 | Video effects: background blur and replacement, low-light boost, framing (T2 and above) | bp line 1124 |
| CAL-23 | Call quality report: after a poor call, one tap sends network statistics only | bp line 1127 |
| CAL-24 | Caller screening: calls from unknown people ring silently with a prompt and appear in Requests | bp line 1128 |
| CAL-25 | Audio routing: earpiece, speaker, Bluetooth and hearing-aid choices with a remembered preference and proximity sensor | bp line 1129 |

### Settings (10) — ids: STB-11, STH-01, STH-02, STH-03, STH-11, STH-12, STH-13, STH-14, STH-15, STH-16

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Screens (8)

| ID | What | Notes |
|---|---|---|
| SCR-131 | Call history |  |
| SCR-132 | Call screen |  |
| SCR-133 | Incoming call (Telecom) |  |
| SCR-134 | Join by link and lobby |  |
| SCR-135 | Scheduled calls |  |
| SCR-136 | Missed-call message |  |
| SCR-137 | Audio routing sheet |  |
| SCR-138 | In-call chat and reactions |  |


## Tests (required)

- Instrumented tests per Android version you can run (record which in DEVICE-RUNS.md; do not claim versions you did not run), network handover test, a test proving the relay only sees ciphertext frames, notification-text test, battery run notes against 15.3 on whatever device exists (label results as measurements).

## Done only when ALL are true

- Every ledger row of P15 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P15` shows no open rows.
- `node tools/gate.mjs --phase P15 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P15 row claimed; CAL and PRV-12 rows have PASS lines in DEVICE-RUNS.md.
- PROGRESS.md updated and `CURRENT_PHASE: P16` set; all work committed.

## Stop conditions specific to this phase

- Stop at the E2EE decision point if the library cannot do it.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

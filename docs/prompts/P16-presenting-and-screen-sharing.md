# P16 — Presenting and screen sharing

Depends on: P15. Ledger rows owned by this phase: 29 (Registry features 15, Alerts 1, Settings 7, Journeys 1, Screens 5)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Source-aware presenting: entire screen, one app, files and photos, camera, whiteboard, audio only, with presenter privacy, quality modes, audience controls, recording to the Vault.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §9.2 (lines 1501-1513) to 9.7 (version matrix, source sheet, safety, audience, builder notes)
- §7.12 (lines 1131-1152) SHR (P16 rows)
- §19.4 (lines 2952-3009) Prompt D hard rules 2 to 6
- §11.7 (lines 1712-1748) ALR-14

## Build (do every item, in order, as real working code)

1. MediaProjection only (no accessibility service, no overlay tricks): new consent per share, foreground service of the media-projection type, correct callbacks, single-app sharing where the system offers it, honest reasons for unavailable sources (9.3), secure windows stay black, notification text hidden while sharing, Do Not Disturb prompt, pause/resume/stop from the bar, status chip, optional floating control (permission), quality modes with frame-rate and resolution caps, switch source mid-call (new consent), audience controls and follow presenter, viewer laser pointer (no remote control), whiteboard with a conflict-free data type over a data channel (up to 8 people, named cursors, export to the Vault), freeze frame, presenter notes and timer, recording only by the host with an indicator for everyone and saved straight to the Vault, live progress notification.

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P16 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (15)

| ID | What | Notes |
|---|---|---|
| SHR-01 | Share the entire screen | bp line 1135 |
| SHR-02 | Share one app through the system picker (Android 14 and up where the phone offers it) | bp line 1136 |
| SHR-03 | Present from inside Sargoshi: photo, video, PDF, slides, document or note. Works on every supported Android version | bp line 1137 |
| SHR-04 | Share the camera as a source, with flip, torch and zoom | bp line 1138 |
| SHR-05 | Share device audio alone or with the screen | bp line 1139 |
| SHR-06 | Shared whiteboard: pen, shapes, text, laser pointer, per-person cursors, export to the Vault | bp line 1140 |
| SHR-07 | Presenter privacy shield: hides Sargoshi notification text, prompts for Do Not Disturb, shows black for secure windows | bp line 1141 |
| SHR-08 | Pause, resume and stop from a floating control (optional permission) or the status chip | bp line 1142 |
| SHR-09 | Quality modes: Text and slides, Smooth motion, Auto, with frame rate and resolution caps | bp line 1143 |
| SHR-10 | Switch source in the middle of a call (asks the system for new consent) | bp line 1144 |
| SHR-11 | Audience controls: who can view, who can present, viewer zoom and pan, follow presenter | bp line 1145 |
| SHR-12 | Record the session with a recording indicator for everyone, saved into the Vault | bp line 1146 |
| SHR-13 | Viewer pointer: viewers draw a temporary laser pointer on the presented screen. No remote control, by design | bp line 1147 |
| SHR-15 | Presenter notes and a private timer for slides and files | bp line 1149 |
| SHR-16 | Freeze frame: pause a video or screen and annotate the still | bp line 1150 |

### Alerts (1)

| ID | What | Notes |
|---|---|---|
| ALR-14 | Screen share or call in progress | bp line 1736 |

### Settings (7) — ids: STH-04, STH-05, STH-06, STH-07, STH-08, STH-09, STH-17

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Journeys (1)

| ID | What | Notes |
|---|---|---|
| JNY-07 | Presenting |  |

### Screens (5)

| ID | What | Notes |
|---|---|---|
| SCR-139 | Present sheet |  |
| SCR-140 | Presenter bar and floating control |  |
| SCR-141 | Whiteboard |  |
| SCR-142 | Presenter notes and timer |  |
| SCR-143 | Audience controls |  |


## Tests (required)

- Per-Android-version source tests for the versions you can run, black-frame test for secure windows, recording lands only in the Vault (leak check), whiteboard CRDT property tests, notification-text test. JNY-07.

## Done only when ALL are true

- Every ledger row of P16 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P16` shows no open rows.
- `node tools/gate.mjs --phase P16 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P16 row claimed with PASS lines in DEVICE-RUNS.md.
- PROGRESS.md updated and `CURRENT_PHASE: P17` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

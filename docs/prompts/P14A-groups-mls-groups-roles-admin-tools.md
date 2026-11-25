# P14A — Groups: MLS groups, roles, admin tools

Depends on: P13. Ledger rows owned by this phase: 27 (Registry features 16, Settings 2, Journeys 1, Screens 8)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Real group chat with OpenMLS: groups up to 100 (then 1,000), roles, topics, links, approvals, slow mode, announcement mode, join questions, admin log, events, polls, history for new members, mentions, member labels, ban list and timed mute.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §7.7 (lines 992-1017) GRP (P14A rows), MSG-01, MSG-16, SAF-07
- §14.2 (lines 2157-2176) Groups row, §15.4 (lines 2304-2309) Scale and cost notes
- §19.2 (lines 2705-2859) Prompt B milestone B11 (sg-mls and delivery-service endpoints)
- §3 (lines 253-295) Journey J3

## Build (do every item, in order, as real working code)

1. sg-mls crate wrapping OpenMLS on a standard classical suite (record OpenMLS version, licence and audit status in DEPENDENCIES.md); server delivery-service endpoints (key packages, commit and welcome fan-out) in sg-gateway; labels say groups are classical encryption until an audited PQ suite exists.
2. Group UI for every P14A row; admin actions are MLS-authenticated, the server never learns roles or content. Added-by-a-non-contact preview with "Leave and report". Test 1,000-member groups before promising them (a synthetic load test that reports commit size and time).

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P14A --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (16)

| ID | What | Notes |
|---|---|---|
| MSG-01 | 1-on-1 and group chat | bp line 905 |
| MSG-16 | Polls, with optional anonymous voting | bp line 920 |
| GRP-01 | Groups up to 100 with owner, admin and member roles | bp line 996 |
| GRP-02 | Groups up to 1,000 | bp line 997 |
| GRP-03 | Topics (threads) inside large groups | bp line 998 |
| GRP-04 | Join links with approval and a member limit | bp line 999 |
| GRP-06 | Slow mode | bp line 1001 |
| GRP-07 | Announcement-only mode | bp line 1002 |
| GRP-08 | Join questions that admins review | bp line 1003 |
| GRP-09 | Admin log: who changed what | bp line 1004 |
| GRP-10 | Events with RSVP (yes, maybe, no), plus one, end time, pinning and reminders | bp line 1005 |
| GRP-12 | Group history for new members: admins choose 0, 25 or 100 recent messages, sent encrypted | bp line 1007 |
| GRP-13 | Mention everyone, with an admin switch | bp line 1008 |
| GRP-14 | Member labels: self-chosen or admin-set tags shown next to names (separate from the admin role) | bp line 1009 |
| GRP-19 | Ban list and timed mute for members, with an expiry | bp line 1014 |
| SAF-07 | Added to a group by a non-contact: show a preview and "Leave and report" in one tap | bp line 1089 |

### Settings (2) — ids: STE-05, STE-27

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Journeys (1)

| ID | What | Notes |
|---|---|---|
| JNY-03 | Group |  |

### Screens (8)

| ID | What | Notes |
|---|---|---|
| SCR-114 | Create group |  |
| SCR-115 | Group info, members, roles |  |
| SCR-116 | Join links and approvals |  |
| SCR-117 | Admin log |  |
| SCR-118 | Events and RSVP |  |
| SCR-119 | Polls |  |
| SCR-120 | Slow mode, announcement-only, mentions |  |
| SCR-121 | Ban list and timed mute |  |


## Tests (required)

- MLS property tests (add, remove, update, concurrent commits), two- and three-device integration tests, JNY-03, goldens.

## Done only when ALL are true

- Every ledger row of P14A is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P14A` shows no open rows.
- `node tools/gate.mjs --phase P14A --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P14A row claimed.
- PROGRESS.md updated and `CURRENT_PHASE: P14B` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

# P14B — Communities, channels, group admin automation, Moments

Depends on: P14A. Ledger rows owned by this phase: 26 (Registry features 14, Settings 2, Journeys 1, Screens 9)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Communities, broadcast channels, captcha and welcome, auto-moderation on admin devices (with honest labels), ownership succession, templates, and Moments.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §7.7 (lines 992-1017) GRP (P14B rows), PER-03, §7.13 (lines 1153-1164) MOM
- §14.9 (lines 2225-2239) row on GRP-17, §3 (lines 253-295) Journey J14
- §6.17 (lines 827-862) Figure 12 text (Communities)

## Build (do every item, in order, as real working code)

1. Communities (directory object on the server holding only an encrypted directory and invite tokens; membership and messages stay in MLS groups), announcement channel, shared rules, one invite link with approval, broadcast channels up to 1,000 encrypted members, join as a persona, captcha and welcome message, auto-moderation rules that run only on online admin devices (the settings screen says so), ownership succession after 7 days, group templates (Class, Family, Team, Neighbourhood), Moments (24-hour, audience control, encrypted, replies go to the chat, view list, close friends, mute, question and poll stickers, highlights).

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P14B --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (14)

| ID | What | Notes |
|---|---|---|
| PER-03 | Join public channels as a persona | bp line 899 |
| GRP-05 | Broadcast channels | bp line 1000 |
| GRP-15 | Communities: an umbrella with an announcement channel, linked groups, shared rules and one invite link | bp line 1010 |
| GRP-16 | Join captcha and a welcome message for new members | bp line 1011 |
| GRP-17 | Auto-moderation rules (keywords, links from new members, rate limits) that run on online admin devices, with honest labels about that limit | bp line 1012 |
| GRP-18 | Ownership succession: if the owner leaves or is inactive, ownership moves to an admin after 7 days | bp line 1013 |
| GRP-21 | Group templates: Class, Family, Team and Neighbourhood, each a preset of roles, rules and notification defaults | bp line 1016 |
| MOM-01 | 24-hour moments with audience control, encrypted | bp line 1157 |
| MOM-02 | Replies go to the chat | bp line 1158 |
| MOM-03 | Optional view list | bp line 1159 |
| MOM-04 | Close friends list | bp line 1160 |
| MOM-05 | Mute someone's moments | bp line 1161 |
| MOM-06 | Question and poll stickers in moments | bp line 1162 |
| MOM-07 | Highlights: keep chosen moments on the profile with audience control | bp line 1163 |

### Settings (2) — ids: STE-28, STE-29

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Journeys (1)

| ID | What | Notes |
|---|---|---|
| JNY-14 | Community admin |  |

### Screens (9)

| ID | What | Notes |
|---|---|---|
| SCR-122 | Community |  |
| SCR-123 | Broadcast channel |  |
| SCR-124 | Captcha and welcome |  |
| SCR-125 | Auto-moderation rules |  |
| SCR-126 | Ownership succession |  |
| SCR-127 | Group templates |  |
| SCR-128 | Moments feed |  |
| SCR-129 | Moment viewer and creator |  |
| SCR-130 | Close friends and highlights |  |


## Tests (required)

- JNY-14 end to end (community with four groups, captcha, auto-moderation with an offline admin, succession, ban list); Moments expiry test; goldens.

## Done only when ALL are true

- Every ledger row of P14B is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P14B` shows no open rows.
- `node tools/gate.mjs --phase P14B --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P14B row claimed.
- PROGRESS.md updated and `CURRENT_PHASE: P15` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

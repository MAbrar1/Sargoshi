# P07B — Message rules, advanced messaging and organisation

Depends on: P07A. Ledger rows owned by this phase: 43 (Registry features 18, Settings 12, Screens 13)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Message Rules with honest labels, Whisper Send, and the advanced messaging and organisation rows.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §7.4 (lines 937-945) RSL, the P07B rows of §7.3 (lines 901-936) and §7.6 (lines 975-991)
- §16.3 (lines 2378-2434) Rules sheet row and Delete dialog
- §6.11 (lines 607-682) chat anatomy parts 7 and 8 (whisper bubble, rule badge)
- §6.16 (lines 816-826) motion (Whisper Send spring)

## Build (do every item, in order, as real working code)

1. RSL-01 to RSL-04: per-message rules (allow forward, allow copy, hide from search and AI, expire after 30 seconds to 4 weeks, delete after read, require unlock to open), chat rules and defaults for new chats, rule badges visible to both sides, the "What this does and does not stop" sheet, templates Whisper, Confidential, Ephemeral, Custom. Rules travel INSIDE the encrypted payload and are enforced by the receiving client (forward blocked, copy blocked, excluded from search index and AI index, timers actually delete the stored message and its media and the search index entry). The copy must never claim what a phone cannot enforce.
2. Whisper Send (hold send and slide up, haptics, spring animation, faded smaller bubble that vanishes after reading) for text, voice and photos.
3. Scheduled send and time capsules (server release from P04), repeating scheduled messages, advanced search with filters and saved searches and jump to date, keep-this-message consent, edit history, threads, saved collections, text stickers, custom reactions, link to message (`?msg=` deep link, members only), chat lock (light), smart folders by rules, mentions and replies filter, needs-reply markers with reminders.

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P07B --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (18)

| ID | What | Notes |
|---|---|---|
| MSG-17 | Whisper Send (signature): hold send and slide up. Applies the Whisper rule set (RSL-04) with a small, faded bubble that vanishes after reading. Works for text, voice a... | bp line 921 |
| MSG-22 | Advanced search: filters for person, chat, date range, type, has link and has file; jump to a date; saved searches | bp line 926 |
| MSG-23 | Keep this message: in a disappearing chat, ask to keep a message and the sender approves. The kept copy is marked | bp line 927 |
| MSG-24 | Edit history: optional per chat, shows earlier versions of an edited message to everyone in the chat | bp line 928 |
| MSG-25 | Threads: reply in a side thread in any chat or group without flooding the timeline | bp line 929 |
| MSG-26 | Saved collections: put starred messages and saved items into named collections | bp line 930 |
| MSG-27 | Repeating scheduled messages: daily, weekly or monthly, with an end date | bp line 931 |
| MSG-28 | Text stickers: turn a short phrase into a styled sticker | bp line 932 |
| MSG-29 | Custom reactions: any emoji or sticker as a reaction, with favourites | bp line 933 |
| MSG-30 | Link to message: a private link that opens that message for members of the chat only | bp line 934 |
| RSL-01 | Message rules per message: allow forward, allow copy, hide from search and AI, expire after (30 seconds to 4 weeks), delete after read, require unlock to open | bp line 941 |
| RSL-02 | Chat rules for everything in a chat: block export, block auto-download, block AI processing, block forwarding. Default rules for new chats | bp line 942 |
| RSL-03 | Rule badges visible to both sides, and a "What this does and does not stop" sheet that is honest about screenshots and cameras | bp line 943 |
| RSL-04 | Rule templates: Whisper, Confidential, Ephemeral, Custom (view-once media is MED-04) | bp line 944 |
| ORG-09 | Chat lock (light): hide a chat behind app unlock and biometrics without moving it into the Vault. The Vault stays the strong option | bp line 987 |
| ORG-10 | Smart folders that fill by rules: unread, mentions, groups, people, labels | bp line 988 |
| ORG-11 | Mentions and replies filter in the chat list | bp line 989 |
| ORG-12 | Needs-reply and follow-up markers with reminders | bp line 990 |

### Settings (12) — ids: STE-02, STE-03, STE-16, STE-17, STE-18, STE-19, STE-20, STE-21, STE-22, STE-23, STE-24, STE-25

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Screens (13)

| ID | What | Notes |
|---|---|---|
| SCR-040 | Message rules sheet |  |
| SCR-041 | What this does and does not stop |  |
| SCR-042 | Whisper Send gesture |  |
| SCR-043 | Scheduled and repeating messages |  |
| SCR-044 | Thread view |  |
| SCR-045 | Saved collections |  |
| SCR-046 | Advanced search filters |  |
| SCR-047 | Edit history sheet |  |
| SCR-048 | Keep this message consent |  |
| SCR-049 | Bulk select bar |  |
| SCR-050 | Smart folders editor |  |
| SCR-051 | Needs-reply markers |  |
| SCR-052 | Text sticker maker and custom reactions |  |


## Tests (required)

- Tests that prove enforcement: a message with forward disabled cannot be forwarded through any UI path; an expired message is gone from the database, the FTS index and the media folder; hide-from-search messages never appear in search results. Two-device tests for rules arriving and being honoured.
- Goldens for rules sheet, badges and whisper bubbles (LTR, RTL).

## Done only when ALL are true

- Every ledger row of P07B is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P07B` shows no open rows.
- `node tools/gate.mjs --phase P07B --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P07B row claimed.
- PROGRESS.md updated and `CURRENT_PHASE: P07C` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

# P07A — Chat core: 1-on-1 messaging, personas, organisation basics

Depends on: P06. Ledger rows owned by this phase: 57 (Registry features 29, Settings 11, Journeys 1, Screens 16)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

The real chat: encrypted 1-on-1 messages end to end, replies, reactions, mentions, edit, delete, undo send, drafts, receipts, typing, link previews, offline queue view, search, folders, pins, mute, archive, Saved Messages.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §7.2 (lines 893-900) PER, §7.3 (lines 901-936) MSG (rows listed below), §7.6 (lines 975-991) ORG
- §6.11 (lines 607-682) chat anatomy and screen blueprints (Chat list, Chat), Figure 4 and 6 text
- §6.7 (lines 539-550) UX patterns, §6.16 (lines 816-826) motion catalogue
- §15.5 (lines 2310-2317) Ordering and idempotency
- §16.3 (lines 2378-2434) copy rows (Chat list empty, Message states, Undo, Message menu, Delete dialog)

## Build (do every item, in order, as real working code)

1. Chat list (72 dp rows, filter chips, Requests row only after P07C, FAB, skeleton loading), chat screen with the 11 anatomy parts, composer that grows to 6 lines, message bubble variants, date chips, status ticks (Sending, Sent, Delivered, Read, "Not sent. Tap to try again."), optimistic UI, Undo snackbar for 5 seconds.
2. Every message action in the long-press menu works for real: Reply, React, Copy, Forward, Edit, Star, Pin, Remind me (a real local scheduled notification), Delete for everyone and for me. Edits and deletes travel as messages and apply on arrival (15.5). Swipe right to reply, double-tap to react; the remapping settings arrive in P18 but gestures read their action from SettingsResolver now.
3. Drafts per chat (persisted), delivered/read states and typing indicator each optional through settings, link previews generated on the sender phone with tracking parameters stripped (PRV-06 is in P07C), full-text search on the phone with FTS (verify FTS5 in the SQLCipher build you chose), waiting-to-send queue screen (NET-06), pin/mute/archive/folders/starred/Saved Messages/unread filter/mark unread/multiple pinned messages, personas and burner usernames using the P04 endpoints.
4. Message info, quote part of a message, chat wallpapers and per-chat accent, contact card sharing, rich text (bold, italic, monospace, strikethrough, quote, spoiler), silent send (the notification is suppressed at the receiver by a flag inside the encrypted payload), and the Hush-independent notification channel hooks come in P09.

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P07A --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (29)

| ID | What | Notes |
|---|---|---|
| PER-01 | Personas: a different name, photo and bio per chat or group | bp line 897 |
| PER-02 | Burner usernames: temporary usernames with an end date, revocable any time | bp line 898 |
| MSG-02 | Replies, reactions, mentions | bp line 906 |
| MSG-03 | Edit and delete for everyone | bp line 907 |
| MSG-04 | Forward messages (limits are set by Message Rules, RSL-01 and RSL-02) | bp line 908 |
| MSG-05 | Drafts saved per chat | bp line 909 |
| MSG-06 | Delivered and read states, each optional | bp line 910 |
| MSG-07 | Typing indicator, optional | bp line 911 |
| MSG-08 | Link previews generated on the sender's phone | bp line 912 |
| MSG-09 | Offline queue with automatic retry | bp line 913 |
| MSG-10 | Full-text search on the phone | bp line 914 |
| MSG-11 | Scheduled send and time capsules (deliver now, opens on the date you choose) | bp line 915 |
| MSG-12 | Remind me about this message | bp line 916 |
| MSG-13 | Message info: who got and read it (groups up to 50) | bp line 917 |
| MSG-14 | Quote part of a message | bp line 918 |
| MSG-15 | Chat wallpapers and per-chat accent colour | bp line 919 |
| MSG-18 | Undo send within 5 seconds | bp line 922 |
| MSG-19 | Share a contact card | bp line 923 |
| MSG-20 | Rich text: bold, italic, monospace, strikethrough, quote and spoiler | bp line 924 |
| MSG-21 | Silent send (no notification for the receiver) | bp line 925 |
| ORG-01 | Pin, mute, archive | bp line 979 |
| ORG-02 | Chat folders | bp line 980 |
| ORG-03 | Starred messages | bp line 981 |
| ORG-04 | Saved Messages (note to self) | bp line 982 |
| ORG-05 | Unread filter and swipe actions | bp line 983 |
| ORG-06 | Several pinned messages per chat | bp line 984 |
| ORG-07 | Mark chat as unread | bp line 985 |
| ORG-08 | Colour labels for chats | bp line 986 |
| NET-06 | Waiting-to-send queue view ("3 messages waiting") | bp line 1261 |

### Settings (11) — ids: STA-05, STA-06, STB-08, STE-04, STE-06, STE-12, STE-13, STE-14, STE-15, STF-03, STG-07

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Journeys (1)

| ID | What | Notes |
|---|---|---|
| JNY-02 | Send and receive |  |

### Screens (16)

| ID | What | Notes |
|---|---|---|
| SCR-024 | Chat list |  |
| SCR-025 | Chat |  |
| SCR-026 | Message long-press menu |  |
| SCR-027 | Reaction picker |  |
| SCR-028 | Forward picker |  |
| SCR-029 | Search in chat |  |
| SCR-030 | Global search |  |
| SCR-031 | Chat info |  |
| SCR-032 | Chat media gallery |  |
| SCR-033 | Message info |  |
| SCR-034 | Waiting-to-send queue |  |
| SCR-035 | Folders manager |  |
| SCR-036 | Archived chats |  |
| SCR-037 | Starred messages |  |
| SCR-038 | Saved Messages |  |
| SCR-039 | Personas manager |  |


## Tests (required)

- Widget and integration tests per row. Two-device integration tests for send, edit, delete, reaction, offline queue then reconnect, duplicate and out-of-order delivery. JNY-02 end to end.
- Golden tests: Chat list and Chat at 360, 412, 600, 840 and 1280 dp, LTR and RTL, text scale 2.0 (tag SG:SCR-0xx).

## Done only when ALL are true

- Every ledger row of P07A is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P07A` shows no open rows.
- `node tools/gate.mjs --phase P07A --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P07A row claimed. Screenshots exported to docs/quality/screens/P07A/.
- PROGRESS.md updated and `CURRENT_PHASE: P07B` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

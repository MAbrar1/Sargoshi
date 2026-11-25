# P21 — Version 2 rows: build what is self-contained, document the rest honestly

Depends on: P20. Ledger rows owned by this phase: 38 (Registry features 38)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Handle the 38 rows the blueprint marks v2. Build the ones that need no outside partner, licence or second client, and give the others a design note, an interface and an honest ledger status.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §7 (lines 863-1336) rows with phase v2 (registry.csv build_phase P21, v2_mode column)
- §18.2 (lines 2484-2492) Scope tiers
- §19.4 (lines 2952-3009) and 19.3 statements about v2 design notes only (CAL-19 CAL-21 CAL-22 SHR-17 VLT-30 VLT-31)

## Build (do every item, in order, as real working code)

1. Rules by v2_mode: TRY_BUILD rows are built for real, in this order of value: ACC-19, GRP-11, GRP-20, PRV-19, MED-24, MED-21, NTF-15, ADM-09, BOT-01, BOT-02, BOT-03, MED-14, BIZ-05, BIZ-04, BIZ-01, BIZ-02, BIZ-03. DESIGN_ONLY rows get docs/architecture/V2-<ID>.md (goal, threat model, interface sketch, what is needed, why not now) and `ledger waive --status DESIGN_ONLY`. EXTERNAL_BLOCKED rows (LOC-05, LOC-08 need native translators; PAY-01 needs a licensed partner and regulatory approval) get the infrastructure that can be real (for example extra locale scaffolding) and an EXTERNAL.md entry. DEFERRED rows (BKP-12, DSK-01, DSK-02, DSK-04 need other clients) get a note explaining the dependency.
2. You may not waive any MVP or v1 row here. The ledger tool refuses it.

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P21 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (38)

| ID | What | Notes |
|---|---|---|
| ACC-19 | Legacy contact: a trusted person can ask to close the account or post a memorial notice. Never gives access to chats | bp line 889; v2 mode TRY_BUILD |
| MED-14 | Nearby share with no internet, over local Wi-Fi | bp line 963; v2 mode TRY_BUILD |
| MED-21 | Content credentials: show provenance (C2PA) on photos that carry it. Never claims to detect fakes | bp line 970; v2 mode TRY_BUILD |
| MED-24 | Document preview in chat for PDF, Word, Excel and PowerPoint, rendered on the phone | bp line 973; v2 mode TRY_BUILD |
| GRP-11 | Shared checklists | bp line 1006; v2 mode TRY_BUILD |
| GRP-20 | Group notes: one pinned, editable note | bp line 1015; v2 mode TRY_BUILD |
| PRV-19 | Private link viewer: opens links in an isolated in-app tab with no stored cookies or history and trackers removed | bp line 1040; v2 mode TRY_BUILD |
| VLT-28 | Optional encrypted Vault sync to a linked device | bp line 1074; v2 mode DESIGN_ONLY |
| VLT-30 | Vault secrets: logins, recovery codes, Wi-Fi keys and card notes with no autofill and no clipboard by default | bp line 1076; v2 mode DESIGN_ONLY |
| VLT-31 | Secure one-time share link: a view-only copy for one recipient that expires. Leaves the Vault by design and says so | bp line 1077; v2 mode DESIGN_ONLY |
| SAF-12 | Family accounts: parent-managed accounts for pre-teens. Visible to the child, no message reading, no hidden mode. The parent controls who can contact the child and whi... | bp line 1094; v2 mode DESIGN_ONLY |
| CAL-19 | Huddles: a drop-in audio room inside a group | bp line 1123; v2 mode DESIGN_ONLY |
| CAL-21 | Call handoff between linked devices | bp line 1125; v2 mode DESIGN_ONLY |
| CAL-22 | Classroom and webinar mode: up to 100 viewers, speakers on stage, raise-hand queue, view-only audience. The SFU sees participants, not content | bp line 1126; v2 mode DESIGN_ONLY |
| SHR-14 | Two presenters at the same time | bp line 1148; v2 mode DESIGN_ONLY |
| SHR-17 | Scenes: save and switch layouts such as slides with camera | bp line 1151; v2 mode DESIGN_ONLY |
| BKP-12 | Cross-platform restore (Android, iOS, desktop) | bp line 1180; v2 mode DEFERRED |
| NTF-15 | Push fallback for phones without Google services: UnifiedPush or a battery-guarded persistent connection | bp line 1207; v2 mode TRY_BUILD |
| LOC-05 | Interface in Punjabi, Pashto and Sindhi | bp line 1247; v2 mode EXTERNAL_BLOCKED |
| LOC-08 | Interface in Saraiki and Balochi, with community translation review | bp line 1250; v2 mode EXTERNAL_BLOCKED |
| NET-08 | Nearby messaging: short text and voice notes over Bluetooth and Wi-Fi Direct when the network is down, relayed through nearby Sargoshi phones, end to end encrypted. No... | bp line 1263; v2 mode DESIGN_ONLY |
| ADM-09 | Reproducible builds and a public guide to verify a build | bp line 1282; v2 mode TRY_BUILD |
| GRW-06 | Find friends privately: contact matching that never uploads your address book | bp line 1295; v2 mode DESIGN_ONLY |
| DSK-01 | Windows desktop app as a linked device | bp line 1303; v2 mode DEFERRED |
| DSK-02 | Web app as a linked device | bp line 1304; v2 mode DEFERRED |
| DSK-04 | Present a desktop window or monitor from the desktop app | bp line 1306; v2 mode DEFERRED |
| DSK-05 | Wear OS companion: notification actions, quick replies and the Hush toggle. No Vault | bp line 1307; v2 mode DESIGN_ONLY |
| DSK-06 | Android Auto: read aloud and voice reply, tied to Driving mode. No Vault | bp line 1308; v2 mode DESIGN_ONLY |
| BOT-01 | Bot API and webhooks | bp line 1315; v2 mode TRY_BUILD |
| BOT-02 | Mini-app container for polls, tools and services | bp line 1316; v2 mode TRY_BUILD |
| BOT-03 | Bot permission labels: every bot declares which messages it can read | bp line 1317; v2 mode TRY_BUILD |
| PAY-01 | Wallet and transfers through licensed local partners. Needs regulatory approval first | bp line 1324; v2 mode EXTERNAL_BLOCKED |
| PAY-02 | Payment request cards with deep links to local payment apps. Sargoshi holds no money and takes no custody. Needs a legal and technical review | bp line 1325; v2 mode DESIGN_ONLY |
| BIZ-01 | Verified business profile and catalogue | bp line 1331; v2 mode TRY_BUILD |
| BIZ-02 | Quick replies | bp line 1332; v2 mode TRY_BUILD |
| BIZ-03 | Business hours and away message | bp line 1333; v2 mode TRY_BUILD |
| BIZ-04 | API for shops and schools | bp line 1334; v2 mode TRY_BUILD |
| BIZ-05 | Verified official accounts through domain proof | bp line 1335; v2 mode TRY_BUILD |


## Tests (required)

- Tests for every TRY_BUILD row you complete. A summary table in docs/ledger/REALITY.md of every v2 row, its final status and why.

## Done only when ALL are true

- Every ledger row of P21 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P21` shows no open rows.
- `node tools/gate.mjs --phase P21 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- All P21 rows are terminal with honest statuses.
- PROGRESS.md updated and `CURRENT_PHASE: P22` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

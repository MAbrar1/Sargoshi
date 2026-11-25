# P17 — On-device AI layer

Depends on: P08 P11. Ledger rows owned by this phase: 69 (Registry features 7, AI features 36, Alerts 1, Settings 19, Screens 6)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Build the on-device AI layer and all 36 AI features, evaluation harness first, with honest capability gating and per-space indexes.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §10 (lines 1569-1641) AI system (all) and spec-data/ai-features.csv
- §19.5 (lines 3010-3064) Prompt E (all) - authoritative
- §13.3 (lines 1863-2150) group I, §7 (lines 863-1336) rows LOC-06, MED-13, MED-23, VLT-21, VLT-22, VLT-29, SAF-13
- §17 (lines 2445-2477) (model licences)

## Build (do every item, in order, as real working code)

1. Evaluation harness FIRST (Urdu and Roman Urdu test sets you write and label as self-written; never invent benchmark numbers; report only what you measure). Capability detector T1/T2/T3, ModelManager (download with SHA-256 verify, pause, resume, delete, storage view, Wi-Fi only default), every model manifest with name, version, licence, size, languages, hash; ship only permissive licences you have confirmed.
2. Order from Prompt E: AI-29 redact assist and AI-35 OTP guard (rules), AI-31 scam packs (signed rule files; initial Pakistan pack written by you and marked draft, native review listed in EXTERNAL.md), AI-10 Roman Urdu converter (300+ self-written phrases), AI-03 smart replies, AI-04 catch me up, AI-01 translate and language detection with pack download, AI-06 transcription, AI-34 voice-note translation, per-space search index (normal and Vault separate; the Vault index encrypted and deleted on lock timeout), AI-33 image descriptions, then the rest by tier. AI-30 and AI-32 are v2: build if self-contained, otherwise design note.
3. A privacy test proves no network call happens from any AI feature without an explicit Cloud tap (AI-22). Suggestions never auto-send. Unsupported features hide themselves; Settings > AI explains why.
4. Dependent rows: LOC-06 Urdu typing aids, MED-13 sticker maker, MED-23 automatic face blur, VLT-21 Vault search, VLT-22 Vault AI, VLT-29 smart ID capture, SAF-13 scam library.

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P17 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (7)

| ID | What | Notes |
|---|---|---|
| MED-13 | On-device sticker maker | bp line 962 |
| MED-23 | Automatic face blur: a per-chat option that blurs every detected face before a photo or video is sent | bp line 972 |
| VLT-21 | Vault search across names, notes and text found in images, indexed only inside the Vault | bp line 1067 |
| VLT-22 | Vault AI on the phone: text-in-image search, auto-albums (documents, receipts, screenshots), duplicate finder | bp line 1068 |
| VLT-29 | Smart ID capture: detect the document type and expiry date with on-device text recognition (Latin digits first) and set the reminder | bp line 1075 |
| SAF-13 | Pakistan scam library: signed, versioned rule packs for wallet-agent fraud, prize and aid scams, fake officers, job offers and OTP requests | bp line 1095 |
| LOC-06 | Urdu typing aids: phonetic Urdu layout in the composer, Roman Urdu conversion and word suggestions from an on-device lexicon | bp line 1248 |

### AI features (36)

| ID | What | Notes |
|---|---|---|
| AI-01 | Translate message | bp line 1605 |
| AI-02 | Auto-translate chat | bp line 1606 |
| AI-03 | Smart replies | bp line 1607 |
| AI-04 | Catch me up | bp line 1608 |
| AI-05 | Voice note summary | bp line 1609 |
| AI-06 | Voice note transcript | bp line 1610 |
| AI-07 | Tone rewrite | bp line 1611 |
| AI-08 | Fix spelling and grammar | bp line 1612 |
| AI-09 | Urdu voice typing | bp line 1613 |
| AI-10 | Roman Urdu to Urdu script | bp line 1614 |
| AI-11 | Scam and phishing warning | bp line 1615 |
| AI-12 | Search by meaning | bp line 1616 |
| AI-13 | Smart actions | bp line 1617 |
| AI-14 | Copy text from image | bp line 1618 |
| AI-15 | Image descriptions | bp line 1619 |
| AI-16 | Sensitive image blur | bp line 1620 |
| AI-17 | Live captions in calls | bp line 1621 |
| AI-18 | Call notes | bp line 1622 |
| AI-19 | Urgent message detection | bp line 1623 |
| AI-20 | Digest of muted chats | bp line 1624 |
| AI-21 | Announcement helper | bp line 1625 |
| AI-22 | Cloud assistant | bp line 1626 |
| AI-23 | Auto-reply when busy | bp line 1627 |
| AI-24 | Model manager | bp line 1628 |
| AI-25 | Ask this chat | bp line 1629 |
| AI-26 | Word lookup | bp line 1630 |
| AI-27 | Document summary | bp line 1631 |
| AI-28 | Smart privacy suggestions | bp line 1632 |
| AI-29 | Redact assist | bp line 1633 |
| AI-30 | Live call translation | bp line 1634; v2 mode TRY_BUILD |
| AI-31 | Pakistan scam packs | bp line 1635 |
| AI-32 | Voice commands | bp line 1636; v2 mode TRY_BUILD |
| AI-33 | Add a description | bp line 1637 |
| AI-34 | Voice-note translation | bp line 1638 |
| AI-35 | OTP and secret guard | bp line 1639 |
| AI-36 | Calm send | bp line 1640 |

### Alerts (1)

| ID | What | Notes |
|---|---|---|
| ALR-08 | Scam or phishing risk (AI-11) | bp line 1730 |

### Settings (19) — ids: STD-16, STE-07, STE-26, STF-07, STH-10, STI-01, STI-02, STI-03, STI-04, STI-05, STI-06, STI-07, STI-08, STI-09, STI-10, STI-11, STL-04, STL-06, STM-07

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Screens (6)

| ID | What | Notes |
|---|---|---|
| SCR-144 | AI sheet |  |
| SCR-145 | AI and models |  |
| SCR-146 | Catch me up |  |
| SCR-147 | Translate and rewrite sheets |  |
| SCR-148 | Scam and OTP guard sheets |  |
| SCR-149 | Redact assist |  |


## Tests (required)

- Harness reports with real measurements in docs/quality/AI-EVAL.md. Unit tests per feature. Privacy/no-network test. Performance on the lowest tier device available (record).

## Done only when ALL are true

- Every ledger row of P17 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P17` shows no open rows.
- `node tools/gate.mjs --phase P17 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P17 row and AI row claimed or honestly marked (AI-30, AI-32 only).
- PROGRESS.md updated and `CURRENT_PHASE: P18` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

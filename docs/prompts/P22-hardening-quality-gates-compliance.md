# P22 — Hardening, quality gates, compliance

Depends on: P21. Ledger rows owned by this phase: 7 (Registry features 3, Alerts 1, Vault leak vectors 1, Screens 2)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Prove it: risk register verification, performance and battery budgets, accessibility audit, goldens at five widths, adaptive pass for tablets and desktops, supply-chain checks, docs for auditors, store compliance checklists.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §4 (lines 296-344) Risk register (spec-data/risks.csv), §15.3 (lines 2278-2303) budgets, §17 (lines 2445-2477) Store, legal and Pakistan compliance
- §20 (lines 3065-3206) Test plan, launch checklist and UX research plan (spec-data: test-matrix.csv beta-checklist.csv ux-studies.csv design-gates.csv)
- §18.6 (lines 2527-2539) Design gates, §6.8 (lines 551-554) Accessibility targets
- §14.6 (lines 2204-2207) Incident response

## Build (do every item, in order, as real working code)

1. Walk risks.csv: for every R-xx write in docs/quality/RISK-VERIFICATION.md the concrete test, file or document that detects or mitigates it, or mark it EXTERNAL with why.
2. Performance and battery: measure the budgets in 15.3 on the devices you have, record numbers with the device and method in docs/quality/PERFORMANCE.md (a budget is a target; never write a measurement you did not take). Install size under 45 MB per ABI split.
3. Accessibility audit of every screen: contrast, 48 dp targets, 200% text, labels, focus order, reduce motion (LOC-02, DSK-03, DSK-07 rows); TalkBack walkthrough notes; goldens for every main screen at 360, 412, 600, 840 and 1280 dp in LTR and RTL; tablet and desktop three-pane behaviour, keyboard shortcuts (Ctrl K, Ctrl N, Esc), drag and drop, hover and right-click menus.
4. Supply chain: `cargo deny`, `cargo audit`, SBOM generation (CycloneDX) committed under docs/compliance, pinned versions at least 14 days old, dependency licence report, R8/ProGuard configuration, release signing configuration that reads secrets from the environment (never commit a keystore). ALR-09 (integrity or root warning; never blocks).
5. Compliance documents (drafts for counsel, labelled as such): Play checklist (target API, Data Safety answers, foreground-service declaration, child-safety self-certification), privacy policy template, terms template, CSAE page, vulnerability disclosure (`security.txt`), transparency policy, KEY-CEREMONY.md for the offline update-signing key (the owner holds it), INCIDENT-RESPONSE.md.
6. LKV-29 companion check (Vault text never reaches Wear or car surfaces; the surfaces are DESIGN_ONLY so test the notification text that would be bridged).
7. Design gates G1 to G6 and the UX studies need real people: list them as EXTERNAL_BLOCKED in EXTERNAL.md with the exact protocol from ux-studies.csv. Do not invent study results.

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P22 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (3)

| ID | What | Notes |
|---|---|---|
| LOC-02 | Text scales to 200%; TalkBack and VoiceOver labels; WCAG AA contrast; no meaning by colour alone | bp line 1244 |
| DSK-03 | Tablet and foldable layouts | bp line 1305 |
| DSK-07 | Keyboard, mouse and Chromebook support: hover, right-click menus, shortcuts | bp line 1309 |

### Alerts (1)

| ID | What | Notes |
|---|---|---|
| ALR-09 | Integrity check failed, or rooted or jailbroken | bp line 1731 |

### Vault leak vectors (1)

| ID | What | Notes |
|---|---|---|
| LKV-29 | Wear OS and Android Auto surfaces (DSK-05, DSK-06) |  |

### Screens (2)

| ID | What | Notes |
|---|---|---|
| SCR-182 | Tablet and desktop three panes |  |
| SCR-183 | Right-to-left variants of all screens |  |


## Tests (required)

- The whole test suite on the full matrix you can run; list what you could not run.

## Done only when ALL are true

- Every ledger row of P22 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P22` shows no open rows.
- `node tools/gate.mjs --phase P22 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- `node tools/gate.mjs --phase P22` passes.
- PROGRESS.md updated and `CURRENT_PHASE: P23` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

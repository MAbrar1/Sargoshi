# P23 — Release candidate and handover

Depends on: P22. Ledger rows owned by this phase: 0 (infrastructure phase; its proof is the gate and the tests below).

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Final audit and honest handover. Nothing new is built; everything claimed is re-verified.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §20.2 (lines 3085-3105) Closed-beta checklist (spec-data/beta-checklist.csv), §18.3 (lines 2493-2503) Milestones
- docs/ledger/REALITY.md and EXTERNAL.md

## Build (do every item, in order, as real working code)

1. Run `node tools/gate.mjs --full` and `node tools/check-registry.mjs --final`. Fix every finding.
2. For each row of beta-checklist.csv write the evidence (command output summary, file, device run) in docs/quality/BETA-CHECKLIST-EVIDENCE.md or mark it EXTERNAL.
3. Build the release: `flutter build apk --release --split-per-abi` and an app bundle; record sizes. Write README.md (what Sargoshi is, how to run, how to test, where everything is) and docs/runbooks/RELEASE.md.
4. Consolidate EXTERNAL.md into a single checklist for the owner: credentials to create (Firebase, SMS provider, Google OAuth, domain, signing keys), people to hire or ask (cryptographer, native Urdu reviewer, counsel, child-safety adviser), devices to test on, studies to run, store accounts.
5. REALITY.md: a final truthful table of every ledger status count and every non-DONE row with its reason. State plainly that the cryptography is unaudited, groups are classical, and which features depend on external services.

## Tests (required)

- Everything already tested; this phase only re-runs and records.

## Done only when ALL are true

- Every ledger row of P23 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P23` shows no open rows.
- `node tools/gate.mjs --phase P23 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- `node tools/gate.mjs --full` passes and `node tools/check-registry.mjs --final` passes.
- PROGRESS.md updated; all work committed; final report written.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

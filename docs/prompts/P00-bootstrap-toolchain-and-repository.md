# P00 — Bootstrap, toolchain and repository

Depends on: none. Ledger rows owned by this phase: 0 (infrastructure phase; its proof is the gate and the tests below).

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Prove the machine can build this project, verify the blueprint, create the repository skeleton exactly as PROJECT-TREE.md says, and make the gate pass on an empty project.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §0 (lines 33-61) Document guide
- §15 (lines 2240-2342) Architecture and data flow
- §19 (lines 2544-3064) Build prompts (intro and order of work)
- §20.3 (lines 3106-3122) Open questions (already answered in docs/ledger/DECISIONS.md)

## Build (do every item, in order, as real working code)

1. TOOLCHAIN CHECK. Run each and record the exact versions in docs/ledger/DEPENDENCIES.md: `flutter --version`, `flutter doctor -v`, `dart --version`, `rustc --version`, `cargo --version`, `rustup target list --installed`, `java -version`, `adb version`, `docker --version`, `docker compose version`, `node --version`, `git --version`. Check the Android SDK has platform 36, build-tools and the NDK, and that these Rust targets are installed: aarch64-linux-android, armv7-linux-androideabi, x86_64-linux-android. Check an emulator exists (`emulator -list-avds`) or a phone is attached (`adb devices`). Anything missing: write the exact install steps into docs/runbooks/LOCAL-DEV.md and STOP and tell the owner (this is an allowed stop). Do not continue on a machine that cannot build.
2. VERIFY THE BLUEPRINT. `docs/blueprint/Sargoshi-Final-Blueprint-v5.md` must have 3,250 lines and sha256 0a8769238c4f43b2a16aa8a907db5b9aa0e6964a97f7a1fb7ff7f39cb4607312 (use `node -e` with crypto if `sha256sum` is missing). Verify that `docs/spec-data/registry.csv` has 350 rows. If anything differs, STOP.
3. GIT. `git init -b main` if needed. Do not add a remote and never push unless the owner tells you to (the owner's repository is MAbrar1/Sargoshi). Commit style: conventional commits, one slice per commit.
4. TREE. Create every directory that docs/ledger/PROJECT-TREE.md marks "from P00" and give each a README.md of 3 to 6 real lines (purpose, what may live here, what may not). Do not create directories that belong to later phases. Never leave an empty directory and never use .gitkeep.
5. LEDGER. Run `node tools/ledger.mjs init`. The row count must equal the sum printed by `node tools/ledger.mjs status`. Do not edit status.csv by hand, ever.
6. RUNBOOKS. Write docs/runbooks/LOCAL-DEV.md with the real commands you just ran (start compose services later; run gate; run tests). Write docs/quality/DEVICE-RUNS.md with its table header (`ID | device | android | command | result`).
7. CI. Confirm .github/workflows/gate.yml runs `node tools/gate.mjs`. Do not add a workflow for tools that do not exist yet.
8. Run `node tools/gate.mjs`. It must pass on the bare skeleton. Fix tools only if they are wrong (never to hide a violation), and say what you changed.

## Tests (required)

- No app tests yet. The gate itself is the test: break it on purpose (add a file containing "TODO" under apps/), see check-reality fail, remove the file, see it pass. Record that you did this in PROGRESS.md.

## Done only when ALL are true

- Every ledger row of P00 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P00` shows no open rows.
- `node tools/gate.mjs --phase P00 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- PROGRESS.md says CURRENT_PHASE: P01.
- PROGRESS.md updated and `CURRENT_PHASE: P01` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

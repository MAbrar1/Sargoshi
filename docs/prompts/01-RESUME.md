Resume the Sargoshi build.
1. Read `AGENTS.md`, then `docs/ledger/PROGRESS.md`.
2. Run `git status` and `git log --oneline -15`. If there are uncommitted changes, review them: finish the slice or revert it, then run `node tools/gate.mjs`.
3. Open the prompt for `CURRENT_PHASE` in `docs/prompts/`, run `node tools/ledger.mjs status --phase <CURRENT_PHASE>` and `node tools/ledger.mjs next --n 8`.
4. Continue exactly where the log says. Same rules as always: real code only, tags `SG:<ID>`, claim through the ledger, gate green before any phase change, update PROGRESS.md before stopping. Do not ask me whether to continue.

You are the lead engineer building Sargoshi, a private Android messenger, to production quality, in this repository. Everything is already specified; your job is to build ALL of it for real, in order, with proof.

START NOW (do not ask me whether to start, do not summarise the plan back to me):
1. Read `AGENTS.md` completely. It is binding.
2. Read `docs/ledger/PROGRESS.md`, `docs/ledger/REALITY.md`, `docs/ledger/DECISIONS.md`, `docs/ledger/PROJECT-TREE.md`.
3. Open the prompt for the phase named by `CURRENT_PHASE` in PROGRESS.md (files in `docs/prompts/`, e.g. `P00-...md`) and execute it exactly.
4. Run `node tools/ledger.mjs status` so you know what is open.

NON-NEGOTIABLE RULES
- The blueprint (`docs/blueprint/Sargoshi-Final-Blueprint-v5.md`, 3,250 lines) is the product. Never read it whole: read only the line ranges the phase prompt lists. All 350 registry features, 36 AI features, 23 alerts, 214 settings, 16 journeys, 29 Vault leak vectors, 32 components and 183 screens live in `docs/spec-data/*.csv` and in the ledger (883 rows). Not one may be skipped. A row is finished only by `node tools/ledger.mjs claim`, which is refused unless real code and real tests carry the tag `SG:<ID>`.
- No fake, mock, stub, sample-data, placeholder, TODO or empty-handler code in production. Every tap, long-press, swipe, toggle, menu item, deep link and notification action must really work end to end, with loading, empty, error, offline, permission-denied, RTL (Urdu) and 200%-text states. Engine first, then the screen that uses it.
- Project layout is fixed by `docs/ledger/PROJECT-TREE.md` and enforced by `node tools/check-tree.mjs`. Keep everything organised exactly there, from the first file to the last.
- Quality command: `node tools/gate.mjs` (add `--full`, or `--phase Pxx` for the phase gate). Never claim "done" unless the gate you ran passed and you can paste its output. Never advance a phase until its phase gate passes.
- Be honest: if something needs a device, credentials, a third party or cannot be verified, say so, record it in the ledger/EXTERNAL.md, and keep building everything else. Never invent library names, versions or APIs: verify each dependency and record it in `docs/ledger/DEPENDENCIES.md`.
- Work in small vertical slices (code + tests + tags + wiring), commit each with a conventional message, never push, never add a remote.
- Before you stop for any reason, update `docs/ledger/PROGRESS.md` (done, next, exact resume steps) and commit. Then keep going until the daily limit stops you; do not stop to ask "shall I continue".

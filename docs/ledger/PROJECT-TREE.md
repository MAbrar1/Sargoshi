# PROJECT-TREE

Enforced by `node tools/check-tree.mjs` (reads `tools/tree-spec.json`). A path marked "from Pxx" must exist once that phase is current. Directories not listed here do not exist yet. To add a directory: record the reason in DECISIONS.md, add it to BOTH this file and tools/tree-spec.json in the same commit.

## Rules

1. Dependency direction: `features/` -> `shared/`, `data/`, `core/`, `platform/`. Never the reverse, never feature -> feature (go through `data/` or `app/`). Checked in review and by `flutter analyze` import lints from P05.
2. Each feature folder (`lib/features/<name>/`) has `presentation/`, `application/`, `domain/` only. Name: snake_case.
3. Dart files snake_case; tests mirror lib/ path and end `_test.dart`. Rust crates are `sg-*`. Only `sg-ffi` may contain `unsafe`.
4. Every directory has at least one real file; every new top-level directory under apps/, core/, server/, infra/ needs a README.md of 3 to 6 lines. No `.gitkeep`, no empty folders, no backup/copy/tmp files.
5. Secrets never in the repo. Generated files end `.g.dart` or live in `generated/` and are reproducible by a script in tools/.
6. Documentation lives only in docs/ (plus README.md, AGENTS.md at root). No stray notes elsewhere.

## Required paths

| Path | Type | From | Purpose |
|---|---|---|---|
| `AGENTS.md` | file | P00 | Constitution read by Freebuff at every session start |
| `README.md` | file | P00 | Human entry point: what this repo is, how to run the gate |
| `.gitignore` | file | P00 | Ignore rules (secrets, build output, keystores) |
| `.gitattributes` | file | P00 | Line endings and binary markers |
| `.editorconfig` | file | P00 | Editor settings |
| `.codebuffignore` | file | P00 | Keeps the agent out of build output and the blueprint figures |
| `.github/workflows/gate.yml` | file | P00 | CI: runs node tools/gate.mjs |
| `tools` | dir | P00 | Node tooling: ledger, gate, checkers, generators (no dependencies) |
| `docs` | dir | P00 | All documentation |
| `docs/architecture` | dir | P00 | PROTOCOL, THREAT_MODEL, BACKUP-FORMAT, CALLS, V2-<ID> notes |
| `docs/blueprint` | dir | P00 | Read-only copy of Blueprint v5, figures, logo, design tokens |
| `docs/compliance` | dir | P00 | Licences, Play checklist, CSAE standards, privacy text sources |
| `docs/ledger` | dir | P00 | PROGRESS, DECISIONS, REALITY, EXTERNAL, DEPENDENCIES, PROJECT-TREE, status.csv |
| `docs/prompts` | dir | P00 | One prompt per phase. Read-only for the agent |
| `docs/quality` | dir | P00 | DEVICE-RUNS, PERFORMANCE, RISK-VERIFICATION, AI-EVAL, exported screens |
| `docs/runbooks` | dir | P00 | LOCAL-DEV, RELEASE, operations runbooks |
| `docs/spec-data` | dir | P00 | Machine-readable extracts of the blueprint (CSV). Read-only |
| `docs/ledger/PROGRESS.md` | file | P00 | Resume file: CURRENT_PHASE and the session log |
| `docs/ledger/ROADMAP.md` | file | P00 | Phase list with dependencies |
| `docs/ledger/DECISIONS.md` | file | P00 | Locked decisions and every decision made later |
| `docs/ledger/PROJECT-TREE.md` | file | P00 | This tree, enforced by tools/check-tree.mjs |
| `docs/ledger/REALITY.md` | file | P00 | The Reality Contract |
| `docs/ledger/EXTERNAL.md` | file | P00 | Everything that needs the owner, credentials or a third party |
| `docs/ledger/DEPENDENCIES.md` | file | P00 | Pinned toolchain and library versions with reasons |
| `docs/ledger/status.csv` | file | P00 | The ledger. Changed only by tools/ledger.mjs |
| `docs/quality/DEVICE-RUNS.md` | file | P00 | Device/emulator evidence table |
| `docs/runbooks/LOCAL-DEV.md` | file | P00 | Real commands to build, test and run locally |
| `apps` | dir | P00 | Client applications |
| `core` | dir | P00 | Shared Rust crypto and protocol workspace |
| `server` | dir | P00 | Rust gateway workspace and SQL migrations |
| `infra` | dir | P00 | Compose, container and observability configuration |
| `apps/mobile` | dir | P01 | Flutter Android app |
| `apps/mobile/pubspec.yaml` | file | P01 | Flutter manifest |
| `apps/mobile/lib` | dir | P01 | Dart source |
| `apps/mobile/lib/main.dart` | file | P01 | Entry point |
| `apps/mobile/lib/app` | dir | P01 | App widget, router, bootstrap, global providers |
| `apps/mobile/lib/core` | dir | P01 | Cross-cutting: theme, adaptive, tiers, bidi, logging, errors |
| `apps/mobile/lib/core/theme` | dir | P01 | Themes; generated/ holds sargoshi_tokens.g.dart |
| `apps/mobile/lib/shared` | dir | P01 | The 32 shared components, SgIcon, SgLogo, scaffolds |
| `apps/mobile/lib/l10n` | dir | P01 | app_en.arb, app_ur.arb and generated localisations |
| `apps/mobile/lib/features` | dir | P06 | One folder per feature: presentation/ application/ domain/ |
| `apps/mobile/lib/data` | dir | P05 | Drift database, DAOs, repositories, settings catalogue |
| `apps/mobile/lib/platform` | dir | P01 | Dart side of platform channels and FFI bindings |
| `apps/mobile/lib/debug` | dir | P01 | Debug-only gallery (compiled out of release) |
| `apps/mobile/assets` | dir | P01 | fonts/, brand/, no other binaries without a DECISIONS entry |
| `apps/mobile/android` | dir | P01 | Android host app |
| `apps/mobile/test` | dir | P01 | Unit, widget and golden tests mirroring lib/ |
| `core/crates` | dir | P02 | Rust crates sg-* |
| `Cargo.toml` | file | P02 | Workspace manifest for core/ and server/ |
| `rust-toolchain.toml` | file | P02 | Pinned Rust toolchain |
| `server/crates` | dir | P03 | Gateway crates |
| `server/migrations` | dir | P03 | Forward-only SQL migrations |
| `infra/compose` | dir | P03 | docker-compose.yml and service config |

## Allowed children (directories)

- `docs/`: `architecture`, `blueprint`, `compliance`, `ledger`, `prompts`, `quality`, `runbooks`, `spec-data`
- `docs/quality/`: `screens`
- `apps/`: `mobile`, `landing`
- `core/`: `crates`
- `server/`: `crates`, `migrations`
- `infra/`: `compose`, `docker`, `observability`, `scripts`
- `apps/mobile/`: `android`, `assets`, `lib`, `test`, `integration_test`, `tool`, `.dart_tool`, `build`, `.idea`
- `apps/mobile/android/`: `app`, `gradle`, `engine`, `.gradle`, `build`, `.idea`
- `apps/mobile/assets/`: `fonts`, `brand`, `models`
- `apps/mobile/lib/`: `app`, `core`, `data`, `features`, `l10n`, `platform`, `shared`, `debug`
- `apps/mobile/lib/core/`: `theme`, `adaptive`, `tiers`, `bidi`, `logging`, `errors`, `l10n`, `util`, `security`
- `core/crates/`: `sg-common`, `sg-crypto`, `sg-handshake`, `sg-ratchet`, `sg-group`, `sg-vault`, `sg-backup`, `sg-ffi`, `sg-testvectors`
- `server/crates/`: `sg-gateway`, `sg-common`, `sg-store`, `sg-auth`, `sg-relay`, `sg-push`, `sg-media`, `sg-calls`, `sg-jobs`, `sg-admin`

## Canonical layout (target at P23)

```
sargoshi/
├─ AGENTS.md  README.md  Cargo.toml  rust-toolchain.toml  deny.toml  rustfmt.toml  .gitignore  .editorconfig  .gitattributes  .codebuffignore
├─ .github/workflows/gate.yml
├─ tools/            ledger.mjs gate.mjs check-*.mjs gen-*.mjs lib.mjs tree-spec.json
├─ docs/
│  ├─ architecture/ blueprint/ compliance/ ledger/ prompts/ quality/ runbooks/ spec-data/
├─ apps/
│  ├─ mobile/
│  │  ├─ android/ (app/ engine/ = Kotlin native modules)
│  │  ├─ assets/ (fonts/ brand/ models/)
│  │  ├─ lib/ (app/ core/ data/ features/ l10n/ platform/ shared/ debug/)
│  │  └─ test/ integration_test/
│  └─ landing/  (static site, P19)
├─ core/crates/      sg-common sg-crypto sg-handshake sg-ratchet sg-group sg-vault sg-backup sg-ffi sg-testvectors
├─ server/crates/    sg-gateway sg-common sg-store sg-auth sg-relay sg-push sg-media sg-calls sg-jobs sg-admin
├─ server/migrations/
└─ infra/            compose/ docker/ observability/ scripts/
```

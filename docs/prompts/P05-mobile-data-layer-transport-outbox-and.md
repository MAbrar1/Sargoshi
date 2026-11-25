# P05 — Mobile data layer, transport, outbox and settings engine

Depends on: P02 P03. Ledger rows owned by this phase: 0 (infrastructure phase; its proof is the gate and the tests below).

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Give the app a real encrypted database, real key custody, a real transport and outbox, and a settings engine generated from settings.csv so none of the 214 settings can be missed.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §15.1 (lines 2257-2273) Stack, §15.2 (lines 2274-2277) Multi-device, §15.5 (lines 2310-2317) Ordering and idempotency
- §13 (lines 1838-2150) Settings (13.1 to 13.3) and spec-data/settings.csv, presets.csv
- §19.1 (lines 2558-2704) Prompt A: INTERFACES, SETTINGS ENGINE, HARD RULES (note: Prompt A allows Fakes; THIS project does not, see AGENTS.md)
- §8.3 (lines 1359-1379) for Keystore-wrapped key patterns

## Build (do every item, in order, as real working code)

1. Drift over SQLCipher with schema migrations and a migration test; the database key is generated on first run and wrapped by an Android Keystore key through a Kotlin channel (never stored in plaintext, no flutter_secure_storage for the DB key unless you can show it uses the Keystore; record the decision).
2. Domain and data layers must not import Flutter so their tests run with `dart test` on the host. Platform code sits behind interfaces with REAL Android implementations.
3. CryptoCore (real, over the Rust FFI), session store, TransportClient (WebSocket with reconnect, backoff with jitter, HTTPS fallback endpoint plumbing; long-polling is added in P19), OutboxService with client idempotency keys, per-chat sequence numbers, ordering by (server time, sender sequence), exponential backoff, "Not delivered" after 30 days (15.5). Receipts may arrive before messages.
4. Settings engine: write `tools/gen-settings.mjs` (supports `--check`) that turns docs/spec-data/settings.csv into `apps/mobile/lib/data/settings/settings_catalog.g.dart` (one SettingDef per row: id, group, options, default, scopes, feature ids) and the title and description ARB keys for en and ur. `SettingsResolver` resolves contact > chat > device > global > default and is persisted in the database. Provide search over title and description including Urdu (using the P01 normaliser), a changed-settings query, an overrides query, preset application with an exact diff and one undo (presets.csv), and a settings file export and import (JSON with schema version).
5. Feature flags client reading the signed config the gateway serves (server side comes in P19; until then flags come from the local catalog).
6. Two-device harness: a Dart integration test that starts two independent app data stacks (two database files, two key stores) against the real compose gateway and exchanges encrypted messages, including offline then online.

## Tests (required)

- Unit tests: migrations, resolver precedence for every scope, preset diff and undo, settings import validation, outbox ordering and idempotency, backoff. Property tests for ordering with duplicate and out-of-order delivery.
- The two-device harness test is mandatory.

## Done only when ALL are true

- Every ledger row of P05 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P05` shows no open rows.
- `node tools/gate.mjs --phase P05 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- The generator --check passes and the generated catalog contains exactly the number of rows in settings.csv (the test asserts equality).
- PROGRESS.md updated and `CURRENT_PHASE: P06` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

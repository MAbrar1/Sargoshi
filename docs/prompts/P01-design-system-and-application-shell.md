# P01 — Design system and application shell

Depends on: P00. Ledger rows owned by this phase: 36 (Registry features 2, Settings 2, Components 32)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Create the Flutter app and everything every later screen depends on: tokens, themes, fonts, icons, brand, adaptive layout, device tiers, localisation, bidirectional typography, logging, and all 32 components with a debug gallery and golden tests.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §6 (lines 382-862) Design language and UX system (all of 6.1 to 6.17)
- §13.1 (lines 1840-1843) Settings information architecture
- §16.3 (lines 2378-2434) Core copy table and §16.4 (lines 2435-2440) Every state is designed
- §18.6 (lines 2527-2539) Design gates
- docs/blueprint/design-tokens.json
- spec-data: colors.csv icons.csv components.csv scaffold-anatomy.csv chat-anatomy.csv screen-blueprints.csv copy-en.csv

## Build (do every item, in order, as real working code)

1. PROJECT. `flutter create --org app.sargoshi --project-name sargoshi --platforms android apps/mobile` (no iOS yet). Set minSdk 26, compileSdk and targetSdk 36 (if the Flutter template pins lower, override in android/app/build.gradle). Enable edge-to-edge and predictive back (`android:enableOnBackInvokedCallback="true"`). Put the application id in ONE place (gradle property). Manifest: `allowBackup="false"`, `dataExtractionRules` excluding everything for cloud backup and device transfer, `fullBackupContent` for old versions, no cleartext traffic. Layout of lib/ exactly as PROJECT-TREE.md.
2. TOKENS. Write `tools/gen-tokens.mjs` (Node, no dependencies, supports `--check`) that reads docs/blueprint/design-tokens.json and writes `apps/mobile/lib/core/theme/generated/sargoshi_tokens.g.dart` with SargoshiColors (ThemeExtension), SargoshiType, SargoshiSpace, SargoshiShape and SargoshiMotion. No colour, size or duration literal exists anywhere else (check-reality enforces it).
3. THEMES. Dark (default), Light (Parchment), AMOLED, Midnight Sky, High-contrast, plus System and Dynamic colour (use a maintained dynamic colour package, verify it). One set of token names for all. The Vault surface uses Vault Indigo and Vault Lilac (6.2 rules, including "never white text on #7C6CFF").
4. FONTS. Plus Jakarta Sans, Inter, JetBrains Mono and Noto Naskh Arabic must be downloaded from their official sources (Google Fonts repository releases), with their OFL licence files, into apps/mobile/assets/fonts, declared in pubspec.yaml, with sha256 and licence recorded in docs/compliance/LICENSES.md. Never load fonts at runtime. Per-run fallback: Inter for Latin runs, Naskh for Arabic-script runs (6.15).
5. ICONS. Use phosphor_flutter (verify the version and that every name in icons.csv exists). `SgIcon` enforces weights: regular default, fill when selected, duotone in empty states, bold under 18 dp, mirrors only the directional icons in RTL (6.4). A test walks icons.csv and asserts every icon resolves.
6. BRAND. Copy docs/blueprint/logo/* to apps/mobile/assets/brand. Build SgLogo (mark, horizontal and stacked lockups, light and dark) from the outlined SVGs. Build the Android adaptive launcher icon (background, foreground, monochrome layers on a 108 dp canvas), the notification icon (24 dp white silhouette), the Android 12 splash and the in-app startup animation (three arcs, 120 ms apart, under 1 second, never blocking). If you need an SVG to Android vector converter, pick one, record it in DECISIONS.md, and make it a script in tools/.
7. ADAPTIVE. WindowSizeClass (compact under 600, medium 600 to 839, expanded 840 and up), AdaptiveScaffold (bottom bar 80 dp or rail 72 dp), pane layout (list 340 dp, chat, info), foldable posture through Flutter display features, bubble width capped at 560 dp, 4/8/12 column grids with the margins and gutters in 6.11.
8. PERFORMANCE TIERS. A Kotlin platform channel returns low-RAM flag, memory class, total memory and system-AI availability; Dart maps it to T1, T2, T3 (6.6). A `PerformanceProfile` provider accepts a forced tier (the settings override arrives in P18). T1 uses the Lite UI: no blur, no shadows, 150 ms transitions, small caches. Glass blur is T3 only.
9. LOCALISATION. ARB files lib/l10n/app_en.arb (source) and app_ur.arb. Seed every row of copy-en.csv as a key. Urdu text you write is machine-drafted: add `"@@x-review": "needs native Urdu review"` metadata and list it in docs/ledger/EXTERNAL.md (a native Urdu reviewer is required by the blueprint). ICU plurals. Direction follows the first strong character per message; RTL mirrors layout, back/forward/send/undo/redo icons, swipe actions and rail position, but not clocks, play, checkmarks, the logo or phone numbers.
10. BIDI AND URDU TYPOGRAPHY (6.15). Bidirectional isolates for numbers, usernames and links; Eastern Arabic digits option; Urdu letter-variant normalisation function for search (ye, kaf, heh variants); line heights 1.7 to 1.8 for Naskh. Pure Dart, unit-tested.
11. LOGGING AND ERRORS. A redacting logger (no message text, usernames, file names, keys, tokens, paths) and a Result/AppError type. Nothing else may call print.
12. COMPONENTS. Build all 32 rows of components.csv with every state (default, pressed 8% overlay, focused 2 dp Glow ring, disabled 38%, error Coral, loading skeleton or ring), in LTR and RTL and at 200% text. Targets at least 48 dp. The PIN pad is a custom in-app widget, 3 by 4, keys 52 dp, gap 12, scramble option, no digit-leaking sounds. Add a debug-only gallery route (`/debug/gallery`, compiled out of release with kDebugMode) that shows every component.
13. SCAFFOLD. Implement the scaffold anatomy and the 11-part chat anatomy as reusable layout widgets (scaffold-anatomy.csv, chat-anatomy.csv). Do not build real screens yet.

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P01 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (2)

| ID | What | Notes |
|---|---|---|
| LOC-01 | English and Urdu with full right-to-left mirroring | bp line 1243 |
| NET-01 | Lite UI profile | bp line 1256 |

### Settings (2) — ids: STG-06, STL-01

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Components (32)

| ID | What | Notes |
|---|---|---|
| CMP-01 | Primary button |  |
| CMP-02 | Tonal button |  |
| CMP-03 | Ghost button |  |
| CMP-04 | Icon button |  |
| CMP-05 | FAB |  |
| CMP-06 | Chip |  |
| CMP-07 | Segmented control |  |
| CMP-08 | Text field |  |
| CMP-09 | Toggle |  |
| CMP-10 | Slider |  |
| CMP-11 | Checkbox and radio |  |
| CMP-12 | Avatar |  |
| CMP-13 | Badge |  |
| CMP-14 | List tile |  |
| CMP-15 | Card |  |
| CMP-16 | Chat bubble |  |
| CMP-17 | Rule badge |  |
| CMP-18 | Reaction bar |  |
| CMP-19 | Composer |  |
| CMP-20 | Top app bar |  |
| CMP-21 | Bottom navigation |  |
| CMP-22 | Navigation rail |  |
| CMP-23 | Bottom sheet |  |
| CMP-24 | Dialog |  |
| CMP-25 | Snackbar |  |
| CMP-26 | Banner |  |
| CMP-27 | Alert card |  |
| CMP-28 | PIN pad |  |
| CMP-29 | Stat ring |  |
| CMP-30 | Progress |  |
| CMP-31 | Skeleton |  |
| CMP-32 | Empty state |  |


## Tests (required)

- One widget test per component covering each state and a no-overflow check at textScaleFactor 2.0 in LTR and RTL.
- Golden tests of the gallery at widths 360, 412, 600, 840 and 1280, in LTR and RTL (helper `goldenMatrix`). Goldens guard against regressions; a text-only agent cannot judge beauty, so export the PNGs to docs/quality/screens/P01/ for the owner to review.
- Unit tests: tokens generator (--check), tier detection mapping, bidi isolation, Urdu normalisation, redacting logger (a test proves secrets never reach output), SgIcon weights and mirroring.
- Tag every component test with `SG:CMP-xx`, the layout tests with `SG:NET-01`, the localisation tests with `SG:LOC-01`.

## Done only when ALL are true

- Every ledger row of P01 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P01` shows no open rows.
- `node tools/gate.mjs --phase P01 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- `flutter run` starts on an emulator and shows the real startup animation, then the gallery in debug only. Release build (`flutter build apk --release --split-per-abi`) succeeds.
- PROGRESS.md updated and `CURRENT_PHASE: P02` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

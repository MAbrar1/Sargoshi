# P18 — Settings completion, appearance, accessibility, lockdown, Simple mode

Depends on: P17. Ledger rows owned by this phase: 98 (Registry features 28, Alerts 1, Settings 49, Journeys 2, Vault leak vectors 1, Screens 17)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Finish all settings UI: Settings home, 14 group pages, search, presets, overrides, security centre, appearance studio, accessibility suite, Lockdown, Simple mode, wellbeing, tours, shortcuts, tiles and widgets, local automation.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §13 (lines 1838-2150) Settings (all), spec-data: settings.csv presets.csv
- §7.16 (lines 1213-1238) SET, §7.8 (lines 1018-1042) PRV-10 PRV-17 PRV-18, §7.17 (lines 1239-1251) LOC-03 LOC-04 LOC-07, BOT-04
- §14.9 (lines 2225-2239) Lockdown row, §16.3 (lines 2378-2434) copy (Lockdown, Simple mode, Focus)
- §6.17 (lines 827-862) Figure 9 and 12 text, §8.6 (lines 1408-1441) vector 27
- §3 (lines 253-295) Journeys J13 and J16

## Build (do every item, in order, as real working code)

1. Every row of settings.csv has a visible, working control with its one-line explanation, a "changed" dot, scope handling (global, device, chat, contact) and a test that toggles it and asserts the real behaviour. A setting whose feature is DESIGN_ONLY (v2) is omitted and recorded in REALITY.md.
2. Settings home (search, security checkup card, Simple mode switch, preset chips, 14 groups in the fixed order), settings search with deep links and recent settings, presets with exact diff and one undo, overrides overview, settings file export/import included in backups, appearance studio (theme, 12 accent presets plus custom, text size, font choice including dyslexia-friendly, bubble style, corner radius, density, avatar shape, tabs reorder, start tab, rail or bar, performance profile override, sounds, haptics), optional Noto Nastaliq Urdu download with hash verification, alternative branded icons, theme packs import/export, accessibility suite (high contrast, colour-blind-safe palette, reduce motion, one-handed mode, screen-reader detail, read aloud, flash and vibration alerts, per-contact haptics, TalkBack custom actions, switch access, large print), Lockdown mode (exact list shown first, undo, security alerts stay on, ALR-19), quick lock gesture (shake, double power press via supported means, tile), privacy checkup, security centre, Simple mode (four big tiles, voice prompts, family set-up), digital wellbeing (local only), guided tour and demo mode with sample content that is clearly separated from real data, launcher shortcuts (never a Vault chat), Quick Settings tiles and privacy-safe widgets (counts only), local if-then automation (BOT-04), experiments flags, about and transparency (versions, audit status, licences, SBOM link, build hash).

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P18 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (28)

| ID | What | Notes |
|---|---|---|
| PRV-10 | Privacy checkup: a 5-step review of your settings | bp line 1031 |
| PRV-17 | Lockdown mode: one tap applies strict settings (no links or media from non-contacts, no previews, calls only from contacts and relayed, new linked devices paused). Sho... | bp line 1038 |
| PRV-18 | Quick lock gesture: shake, double power press or a tile locks the app and the Vault and clears the recents entry | bp line 1039 |
| SET-01 | Settings search with deep links and recent settings | bp line 1217 |
| SET-02 | Presets: Standard, Private, Maximum privacy, Low data, Battery saver. Each shows exactly what changes before you apply it | bp line 1218 |
| SET-03 | Per-chat and per-contact overrides, with one overview of everything you overrode | bp line 1219 |
| SET-04 | Settings file: export, import, and automatic inclusion in every backup | bp line 1220 |
| SET-05 | Appearance studio: theme, accent colour (12 plus custom), text size, font choice, bubble style, corner radius, density, avatar shape | bp line 1221 |
| SET-06 | Alternative branded app icons | bp line 1222 |
| SET-07 | Gestures and quick actions: remap swipes, double-tap, long-press menu order and the composer quick-action bar | bp line 1223 |
| SET-08 | Navigation: reorder or hide tabs, choose the start tab, rail or bar on large screens | bp line 1224 |
| SET-09 | Language and region: UI language, per-chat translation language, number style (Western or Eastern Arabic), Gregorian calendar with optional Hijri dates | bp line 1225 |
| SET-10 | Sounds and haptics: per-event sounds, ringtone, haptic strength | bp line 1226 |
| SET-11 | Performance profile: automatic device tier with manual override | bp line 1227 |
| SET-12 | Accessibility suite: high-contrast theme, dyslexia-friendly font, colour-blind-safe palette, one-handed mode, screen-reader verbosity | bp line 1228 |
| SET-13 | Quick Settings tiles and privacy-safe widgets (Hush, Lock Vault, Present screen). Widgets show counts, never message text | bp line 1229 |
| SET-14 | Security centre: devices, sessions, recovery key status, alert log, vault checkup and security checkup in one place | bp line 1230 |
| SET-15 | About and transparency: versions, protocol version, audit report, licences, SBOM, transparency report, build hash | bp line 1231 |
| SET-16 | Experiments: opt-in early features behind flags | bp line 1232 |
| SET-17 | Simple mode: larger text, four big tiles, voice prompts and fewer features for first-time and older users, set up by a family member | bp line 1233 |
| SET-18 | Digital wellbeing: local usage insights, daily limit nudges, bedtime. Nothing leaves the phone | bp line 1234 |
| SET-19 | Guided tour and demo mode with sample content, so the Vault and rules can be tried before real data goes in | bp line 1235 |
| SET-20 | Theme packs: export and import a theme file | bp line 1236 |
| SET-21 | Launcher shortcuts: New chat, Scan QR and Record voice note. Never a Vault chat | bp line 1237 |
| LOC-03 | Reduce-motion setting | bp line 1245 |
| LOC-04 | Read messages aloud with the system voice | bp line 1246 |
| LOC-07 | Hearing and vision suite: flash and vibration call alerts, per-contact haptic patterns, TalkBack custom actions on messages, switch access support, large-print reading... | bp line 1249 |
| BOT-04 | Rules on my phone: if-then automation (auto-reply, auto-archive, remind) that runs locally and gives no bot access to messages | bp line 1318 |

### Alerts (1)

| ID | What | Notes |
|---|---|---|
| ALR-19 | Lockdown mode is on | bp line 1741 |

### Settings (49) — ids: STB-18, STB-19, STB-20, STC-08, STC-10, STD-12, STE-08, STE-09, STE-10, STE-11, STF-10, STJ-09, STK-01, STK-02, STK-03, STK-04, STK-05, STK-06, STK-07, STK-08, STK-09, STK-10, STK-11, STK-12, STK-13, STK-14, STK-15, STK-16, STK-17, STK-18, STK-19, STK-20, STL-02, STL-03, STL-05, STM-01, STM-02, STM-03, STM-04, STM-05, STM-06, STM-08, STM-09, STM-10, STM-11, STN-02, STN-03, STN-04, STN-05

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Journeys (2)

| ID | What | Notes |
|---|---|---|
| JNY-13 | Lockdown |  |
| JNY-16 | Older user |  |

### Vault leak vectors (1)

| ID | What | Notes |
|---|---|---|
| LKV-27 | Launcher shortcuts and widgets (SET-13, SET-21) |  |

### Screens (17)

| ID | What | Notes |
|---|---|---|
| SCR-150 | Settings home |  |
| SCR-151 | Settings group pages (14) |  |
| SCR-152 | Settings search and changed settings |  |
| SCR-153 | Preset diff and undo |  |
| SCR-154 | Overrides overview |  |
| SCR-155 | Security centre |  |
| SCR-156 | Privacy checkup |  |
| SCR-157 | Lockdown mode |  |
| SCR-158 | Appearance studio |  |
| SCR-159 | Gestures, navigation, theme packs |  |
| SCR-160 | Simple mode |  |
| SCR-161 | Digital wellbeing |  |
| SCR-162 | Guided tour and demo mode |  |
| SCR-163 | Accessibility pages |  |
| SCR-164 | Alternative icons and shortcuts |  |
| SCR-165 | Rules on my phone |  |
| SCR-166 | About and transparency |  |


## Tests (required)

- One test per setting id (tag `SG:STx-nn`). Widget tests and goldens for every settings page. JNY-13 and JNY-16. Leak vector 27 (shortcuts and widgets) on device.

## Done only when ALL are true

- Every ledger row of P18 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P18` shows no open rows.
- `node tools/gate.mjs --phase P18 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- All P18 rows claimed (ST rows included).
- PROGRESS.md updated and `CURRENT_PHASE: P19` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

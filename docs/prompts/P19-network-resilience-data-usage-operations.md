# P19 — Network resilience, data usage, operations, growth

Depends on: P18. Ledger rows owned by this phase: 54 (Registry features 22, Alerts 6, Settings 15, Journeys 1, Screens 10)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Connection fallbacks, outage mode, proxy links, data monitor, update channel, admin tooling, feature flags, analytics opt-in, transparency, help centre, landing page, switch kit, group-move tracker.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §15.6 (lines 2318-2331) Outage, censorship and offline resilience, §15.7 (lines 2332-2338) Push fallback and distribution
- §7.18 (lines 1252-1269) NET (P19 rows), §7.19 (lines 1270-1285) ADM, §7.20 (lines 1286-1298) GRW (P19 rows), PRV-13
- §14.6 (lines 2204-2207) Incident response, §14.8 (lines 2212-2224) What the servers store
- §16.3 (lines 2378-2434) copy (Outage, Data usage)
- §3 (lines 253-295) Journey J12

## Build (do every item, in order, as real working code)

1. Transport fallbacks in the documented order (WebSocket 443, HTTPS long polling, optional QUIC), automatic return when healthy, outage mode with cause classes from real probes (says likely, never claims blocked), signed proxy links with fingerprint and an explicit tap (no domain fronting), SOCKS5/HTTP proxy support, data usage monitor with caps and ALR-23, background data and battery guard, network diagnostics with real measurements, storage manager by chat, verified update channel (signed manifest, monotonic version, APK certificate fingerprint shown) with ALR-21, "what Sargoshi knows about you" screen reading the real server data export.
2. Operations: abuse dashboard, support tooling, transparency report generator, feature flags service, content-free analytics (opt-in only, events listed in docs), service status banner, help and safety centre, in-app feedback with a redacted diagnostics bundle the person reviews before sending, what-is-new cards. Build the admin web UI as server/crates/sg-admin (server-rendered, admin authentication, immutable audit log of every access; no message content exists to show).
3. Growth: public username landing page in apps/landing (static, no tracking, safe install fallback, assetlinks.json for App Links), switch kit checklist and group-move announcement template, group-move tracker, share-to-Sargoshi target and Direct Share (Vault chats excluded).

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P19 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (22)

| ID | What | Notes |
|---|---|---|
| PRV-13 | "What Sargoshi knows about you": a screen listing exactly what our servers store | bp line 1034 |
| NET-04 | Network diagnostics in Settings > Advanced, real measurements only | bp line 1259 |
| NET-05 | Storage manager: clear media by chat | bp line 1260 |
| NET-07 | Connect through your own proxy | bp line 1262 |
| NET-09 | Connection resilience: transport fallbacks (WebSocket over 443, QUIC, long polling), signed shareable proxy links and bridge-list updates. No domain fronting | bp line 1264 |
| NET-10 | Outage mode: when the network is down, show the likely cause class, the retry plan and the options (proxy link, Nearby) | bp line 1265 |
| NET-11 | Data usage monitor: per feature (messages, media, calls, presenting, backup) and per network, with a monthly cap alert | bp line 1266 |
| NET-13 | Background data and battery guard: a budget for background sync with adaptive behaviour | bp line 1268 |
| ADM-01 | Abuse dashboard | bp line 1274 |
| ADM-02 | Feature flags | bp line 1275 |
| ADM-03 | Content-free analytics, opt-in | bp line 1276 |
| ADM-04 | Support tooling | bp line 1277 |
| ADM-05 | Transparency report generator | bp line 1278 |
| ADM-06 | Help and safety centre inside the app | bp line 1279 |
| ADM-07 | Service status banner | bp line 1280 |
| ADM-08 | Verified update channel for direct APK downloads: signed update manifest, in-app update check, signature fingerprint shown | bp line 1281 |
| ADM-10 | In-app feedback with a redacted diagnostics bundle the person reviews before sending | bp line 1283 |
| ADM-11 | What’s new cards with a switch to turn them off | bp line 1284 |
| GRW-04 | Switch kit: checklist to move your circle, and a group-move announcement template | bp line 1293 |
| GRW-05 | Share to Sargoshi from any app and Direct Share targets (Vault chats excluded) | bp line 1294 |
| GRW-07 | Public username page: a web landing page with a safe install fallback and no tracking | bp line 1296 |
| GRW-08 | Group move tracker: bulk-invite a group to a new Sargoshi group and see who has joined | bp line 1297 |

### Alerts (6)

| ID | What | Notes |
|---|---|---|
| ALR-10 | Storage low | bp line 1732 |
| ALR-11 | App too old for the protocol | bp line 1733 |
| ALR-13 | Moderator action on your account | bp line 1735 |
| ALR-21 | Update available from the verified channel (ADM-08) | bp line 1743 |
| ALR-22 | Network outage detected (NET-10) | bp line 1744 |
| ALR-23 | Data cap near (NET-11) | bp line 1745 |

### Settings (15) — ids: STB-17, STG-08, STG-09, STG-11, STG-12, STG-13, STG-16, STN-01, STN-06, STN-07, STN-08, STN-09, STN-10, STN-11, STN-12

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Journeys (1)

| ID | What | Notes |
|---|---|---|
| JNY-12 | Network outage |  |

### Screens (10)

| ID | What | Notes |
|---|---|---|
| SCR-167 | Data usage |  |
| SCR-168 | Network diagnostics and outage mode |  |
| SCR-169 | Proxy link opener |  |
| SCR-170 | Update available |  |
| SCR-171 | Help and safety centre |  |
| SCR-172 | Send feedback |  |
| SCR-173 | What is new cards |  |
| SCR-174 | Storage manager |  |
| SCR-175 | Switch kit and group move tracker |  |
| SCR-176 | What Sargoshi knows about you |  |


## Tests (required)

- Outage tests that really block the WebSocket then 443 then the network (use a controllable local proxy or firewall rule) and assert fallbacks and messages; signed-manifest tests including rollback rejection; admin audit-log tests; landing page link check; JNY-12.

## Done only when ALL are true

- Every ledger row of P19 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P19` shows no open rows.
- `node tools/gate.mjs --phase P19 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P19 row claimed. Counsel review for NET-09 listed in EXTERNAL.md.
- PROGRESS.md updated and `CURRENT_PHASE: P20` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

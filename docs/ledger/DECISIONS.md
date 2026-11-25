# DECISIONS

Locked by the owner brief or resolved from Blueprint section 20.3 (the owner asked the builder to pick the best option). Add new decisions at the bottom as `D-nn`. Never silently reverse one: supersede it with a new entry that says why.

| ID | Decision | Reason |
|---|---|---|
| D-01 | No fake, mock, stub or sample implementations in production code; engine-first phase order. Blueprint Prompt A "fakes" are overridden. | Owner requirement: everything must really work. |
| D-02 | Stack: Flutter/Dart (Riverpod, go_router, Drift + SQLCipher), Kotlin native modules, Rust core (Path C of 14.3) and Rust gateway (axum, Postgres, Redis Streams, S3-compatible store, LiveKit). | Blueprint 14 and 15. Rust is acceptable for the backend. |
| D-03 | Crypto is Path C: vetted primitive crates plus OpenMLS. No custom primitives. A qualified cryptographer must review docs/architecture/PROTOCOL.md before any public launch (risk R-01). | Blueprint 14.3. |
| D-04 | Launch scope: global from day one with Pakistan and Urdu as first-priority market; English and Urdu (RTL) at launch. | Blueprint 20.3 question 1; lowest rework. |
| D-05 | Distribution: Google Play first; direct signed APK as a second channel with an offline release key owned by the owner. | Blueprint 20.3. |
| D-06 | Communities and Simple mode are built in their blueprint phases; release gating is a release-flag decision at P23, not a reason to skip building. | Never skip any feature. |
| D-07 | Disguised app icon (VLT-17) is built, behind an explicit in-app warning about store-policy and recovery risk. | Blueprint 12 / 20.3; honest disclosure. |
| D-08 | Lockdown preset is visible in Security centre for everyone, with a confirmation prompt before activation. | Blueprint 14.9; discoverability for at-risk users. |
| D-09 | Logo direction in Blueprint is approved for the build; trademark search is an owner task (EXTERNAL.md). | Build is not blocked by it. |
| D-10 | Urdu copy is machine-drafted and tagged for native review; a native reviewer is an owner task (EXTERNAL.md). | Blueprint requires native review. |
| D-11 | Blueprint v2-labelled rows: attempt real implementation first (TRY_BUILD). Only if infeasible: DESIGN_ONLY or DEFERRED with `docs/architecture/V2-<ID>.md`. MVP and v1 rows can never be waived. | Never skip, but stay honest. |
| D-12 | Settings count is 214 in spec-data (blueprint text says 213). Build all 214 and mention the discrepancy in the final report. | Extraction is authoritative; fewer would be a skip. |
| D-13 | Gradle application id: `app.sargoshi`; minSdk 26; compileSdk/targetSdk 36. | Blueprint client baseline. |
| D-14 | Never push, add remotes or publish anything unless the owner says so. Repository: MAbrar1/Sargoshi. | Safety. |
| D-15 | The build sandbox has no Flutter, Dart, Android SDK/NDK or Docker and cannot download them (hosts blocked). The owner asked to continue regardless. Decision: P00 stays open (its toolchain item needs the owner's machine). The Rust track (P02 crypto core, then P03/P04 gateway) is built and tested here, because the roadmap lists P02 as depending only on P00. P01 and every Flutter, Kotlin and device phase wait for a machine with those tools. CURRENT_PHASE stays P00 because check-tree requires apps/mobile from P01 onward. | Reality Contract: nothing is claimed that was not run. Roadmap dependency column allows P02 before P01. |
| D-16 | In the sandbox, Postgres and Redis run natively from conda-forge instead of docker compose, and no S3-compatible store or LiveKit is available. Rows that need them are claimed only against what was really run; the compose files are still written and are verified on the owner's machine. | Honest verification path. |

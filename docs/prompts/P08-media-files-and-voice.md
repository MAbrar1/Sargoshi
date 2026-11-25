# P08 — Media, files and voice

Depends on: P07C. Ledger rows owned by this phase: 41 (Registry features 22, Settings 8, Screens 11)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Photos, video, documents up to 2 GB, voice notes, view-once, EXIF stripping, resumable uploads, galleries, scanner, editor, trimming, location, GIFs, stickers, video notes, smart formats.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §7.5 (lines 946-974) MED (P08 rows) and NET-02, NET-03, NET-12 in §7.18 (lines 1252-1269)
- §6.6 (lines 511-538) device tiers (thumbnail sizes, caps)
- §13.3 (lines 1863-2150) group G Media and storage
- §16.3 (lines 2378-2434) copy rows (Cloud sheet not here)

## Build (do every item, in order, as real working code)

1. Capture and pick (Photo Picker first), compose preview with quality choice and size-before/after (NET-12), strip location and camera data from every sent photo (on by default), upload encrypted blobs with resumable chunks and correct resume after process death, download with auto-download rules per network (settings group G), view-once media that cannot be reopened, voice notes with waveform, speed control, lock recording, pause and resume, trim, queue playback and background playback, documents up to 2 GB, per-chat media gallery with filters (photos, videos, files, links, voice), shared albums, GIFs through a privacy-preserving proxy (a small gateway route; the provider key is DONE_NEEDS_CREDENTIALS), stickers, location sharing, live location and arrival share with automatic end (ETA from a destination geofence; show locations via a `geo:` intent and an owner-configurable tile source so no tile provider is hard-wired), manual photo editor (crop, draw, text, blur faces by hand), document scanner with edge detection and multi-page PDF (verify ML Kit document scanner licensing and API, or use OpenCV via a maintained package), video trim and compress, video notes up to 60 seconds, smart formats (AVIF/WebP, HEVC/AV1 with fallback), data saver and image quality caps.
2. Tier-aware: thumbnails 160, 256 or 384 px; video caps 480p or 720p by tier.

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P08 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (22)

| ID | What | Notes |
|---|---|---|
| MED-01 | Photos and video | bp line 950 |
| MED-02 | Documents up to 2 GB, resumable | bp line 951 |
| MED-03 | Voice notes with waveform and speed control | bp line 952 |
| MED-04 | View-once photos, videos and voice notes | bp line 953 |
| MED-05 | Auto-download rules per network type | bp line 954 |
| MED-06 | Chat media gallery with filters: photos, videos, files, links, voice | bp line 955 |
| MED-07 | "Send in original quality" toggle | bp line 956 |
| MED-08 | Strip location and camera data (EXIF) from every sent photo, on by default | bp line 957 |
| MED-09 | GIFs through a privacy-preserving proxy | bp line 958 |
| MED-10 | Stickers | bp line 959 |
| MED-11 | Location sharing, then live location | bp line 960 |
| MED-12 | Photo editor: crop, draw, text, blur faces by hand (automatic blur is MED-23) | bp line 961 |
| MED-15 | Document scanner with edge detection and multi-page PDF, with "Scan to Vault" | bp line 964 |
| MED-16 | Trim and compress video before sending | bp line 965 |
| MED-17 | Shared albums inside a chat | bp line 966 |
| MED-18 | Video notes: round videos up to 60 seconds | bp line 967 |
| MED-19 | Voice-note upgrades: lock recording, pause and resume, trim, play consecutive notes as a queue, keep playing in the background | bp line 968 |
| MED-20 | Smart formats: AVIF or WebP images and HEVC or AV1 video where the phone supports them, with fallback, to cut data | bp line 969 |
| MED-22 | Arrival share: share live location with an ETA that ends by itself on arrival | bp line 971 |
| NET-02 | Data saver and image quality caps | bp line 1257 |
| NET-03 | Resumable uploads and sync that survives dropped connections | bp line 1258 |
| NET-12 | Preview size before send: shows the size before and after compression and lets you choose | bp line 1267 |

### Settings (8) — ids: STB-10, STG-01, STG-02, STG-03, STG-04, STG-05, STG-14, STG-15

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Screens (11)

| ID | What | Notes |
|---|---|---|
| SCR-062 | Camera |  |
| SCR-063 | Media compose preview and quality |  |
| SCR-064 | Photo editor |  |
| SCR-065 | Document scanner |  |
| SCR-066 | Video trim and compress |  |
| SCR-067 | Voice recorder and player |  |
| SCR-068 | Media viewer |  |
| SCR-069 | Shared albums |  |
| SCR-070 | Location and arrival share |  |
| SCR-071 | GIF and sticker picker |  |
| SCR-072 | Video note recorder |  |


## Tests (required)

- Tests: EXIF is absent in the bytes actually uploaded (assert on the ciphertext source, not the UI), interrupted upload resumes at the chunk, view-once cannot reopen, 2 GB path uses streaming with bounded memory (assert peak memory in a test with a large synthetic file), size caps by tier. Voice recorder instrumented test. Goldens for media screens.

## Done only when ALL are true

- Every ledger row of P08 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P08` shows no open rows.
- `node tools/gate.mjs --phase P08 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P08 row claimed. NET-02, NET-03, NET-12 claimed.
- PROGRESS.md updated and `CURRENT_PHASE: P09` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

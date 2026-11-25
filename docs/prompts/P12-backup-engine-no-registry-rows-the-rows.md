# P12 — Backup engine (no registry rows; the rows complete in P13)

Depends on: P10 P02. Ledger rows owned by this phase: 0 (infrastructure phase; its proof is the gate and the tests below).

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Build the backup engine: recovery key, chunker, manifests, encryption, deduplication, retention, garbage collection, destinations, scheduling, verification, restore planner and journal, drills. Exactly Prompt C milestones C8 to C10.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §12 (lines 1749-1837) Backup, restore, export and migration (all)
- §19.3 (lines 2860-2951) Prompt C: BACKUP ENGINE and Backup tests, V5 addendum items 2, 3, 6
- §7.14 (lines 1165-1188) BKP

## Build (do every item, in order, as real working code)

1. sg-backup crate: 24-word recovery key with optional passphrase (reuse P02), BMK by HKDF, snapshot = encrypted manifest + encrypted chunks, 1 to 5 MB message batches and 4 MiB media chunks, keyed-hash deduplication, per-chunk keys, monotonic manifest counter, retention 7 daily / 4 weekly / 6 monthly with reference-counting GC and a mark-and-sweep safety pass, included data exactly as 12.2 (no session keys; identity key wrapped under BMK; Vault only if enabled with its own key).
2. Destinations behind one trait (put, get, head, delete, list, resumable put): local folder through the Storage Access Framework, S3-compatible, WebDAV, Sargoshi cloud (the P04 /backup/blobs API), Google Drive app data (real REST client with a narrow scope; DONE_NEEDS_CREDENTIALS because OAuth verification is the owner's task).
3. WorkManager scheduling with the constraints from settings, expedited manual backups, long restores as foreground work, verification after upload, monthly restore drill sampling at least 20 chunks or 5%, restore planner (chats first, thumbnails, media in background), restore journal (resume at the chunk), one-chat merge that never overwrites newer messages, time travel into a separate view, low-storage mode, readiness score 40/30/20/10.
4. BKP-19 forecast and BKP-18 quiz logic live here as pure functions; their UI is P13.

## Tests (required)

- Backup then restore on a clean emulator equals the original database and media hashes; interrupt a restore at 50 random points and resume each time; corrupt one chunk and the drill fails; roll back to an older manifest and the app warns; chunk dedupe never merges different content; GC never deletes a referenced chunk (property tests).

## Done only when ALL are true

- Every ledger row of P12 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P12` shows no open rows.
- `node tools/gate.mjs --phase P12 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- `cargo test` and the instrumented backup tests pass. docs/architecture/BACKUP-FORMAT.md written.
- PROGRESS.md updated and `CURRENT_PHASE: P13` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

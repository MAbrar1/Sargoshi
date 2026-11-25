# P04 — Gateway services: media, push, devices, links, reports, abuse

Depends on: P03. Ledger rows owned by this phase: 0 (infrastructure phase; its proof is the gate and the tests below).

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Finish the server side that later phases rely on: encrypted media storage, push wake-ups, device linking and approval, contact links, burner usernames, reports and appeals, account deletion, abuse controls. Every endpoint has real integration tests.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §19.2 (lines 2705-2859) Prompt B: BACKEND (endpoints list), BACKEND ADDITIONS (v4), V5 BACKEND ADDENDUM, milestones B9 B10 B13 B14
- §14.5 (lines 2195-2203) Account takeover defences, §14.9 (lines 2225-2239) Additional controls
- §11.1 (lines 1644-1651) and 11.2 Push pipeline

## Build (do every item, in order, as real working code)

1. Media: resumable chunked upload of client-encrypted blobs to S3-compatible storage under random IDs, 2 GB cap, Range download, deletion 30 days after last reference or on account deletion.
2. Push: FCM HTTP v1 sender configured by a service account file path in the environment (real code; DONE_NEEDS_CREDENTIALS because the owner must create the Firebase project). Fixed payload {"v":1,"t":"wake"} (calls {"v":1,"t":"call"}, high priority), wake-ups coalesced to one per device per 2 seconds, APNs behind a trait.
3. Device linking and new-device approval (pending approvals, signed approve/deny, 24-hour fallback job with alerts), rotating contact links and QR tokens, burner usernames, remote wipe flag, device-link scam signals (never block approval), inactive-account job, age band storage (no date of birth), reports (labelled unverified) and appeals, account self-deletion removing queues, bundles and media, backup blob storage with quotas, scheduled and repeating sealed envelopes, check-in timer envelopes with heartbeat, region pinning as configuration, update manifest endpoint with an offline signing script (Ed25519, rollback rejected), signed bridge list.
4. Write down in docs/architecture/THREAT_MODEL.md what the server learns for each feature (check-in deadline, release times, call existence).

## Tests (required)

- Integration tests for every endpoint, including abuse limits, refresh-token reuse revoking a device, approval flow with the 24-hour path using a controllable clock, scheduled release, backup quota enforcement, deletion removing everything.
- Tag tests with the registry ids they serve (ACC-12, ACC-14, ACC-15, ACC-16, ACC-17, ACC-18, ACC-21, PER-02, MSG-11, MSG-27, SAF-11, BKP-02, ADM-08, NET-09...). These rows are claimed later in their own phases.

## Done only when ALL are true

- Every ledger row of P04 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P04` shows no open rows.
- `node tools/gate.mjs --phase P04 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- All server tests pass against compose. docs/ledger/EXTERNAL.md lists FCM and the SMS provider with exactly what the owner must supply.
- PROGRESS.md updated and `CURRENT_PHASE: P05` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

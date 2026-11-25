# P03 — Gateway foundation: accounts, keys, messages, queues

Depends on: P02. Ledger rows owned by this phase: 0 (infrastructure phase; its proof is the gate and the tests below).

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

Build the Rust gateway core with a real Postgres, Redis and object store running in docker compose: one-time codes, accounts, tokens, usernames, key bundles, registration lock, message relay over WebSocket with durable per-device queues.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §19.2 (lines 2705-2859) Prompt B: BACKEND (sg-gateway), Database, SECURITY CONTROLS, TESTS
- §15.1 (lines 2257-2273) Stack, §15.5 (lines 2310-2317) Ordering and idempotency rules
- §14.4 (lines 2181-2194) Abuse controls and §14.8 (lines 2212-2224) What the servers store
- §11.2 (lines 1652-1662) Delivery pipeline

## Build (do every item, in order, as real working code)

1. Crates server/crates/sg-store (sqlx PostgreSQL, Redis Streams, S3-compatible client trait with a MinIO implementation) and sg-gateway (axum, tokio, WebSocket). Migrations in server/migrations with a migration test.
2. infra/compose/docker-compose.yml with postgres, redis, minio and a local SMTP sink (Mailpit) so the one-time-code email path is REAL end to end in development. Healthchecks on all services.
3. Implement Prompt B milestones B7 and B8: OtpSender trait with an SMTP implementation and a generic HTTP SMS adapter configured only by environment variables (the real SMS provider is DONE_NEEDS_CREDENTIALS and goes in EXTERNAL.md); 6-digit codes stored as HMAC, 5-minute expiry, 5 attempts, 5 requests per hour per identifier and 10 per hour per IP; device-bound 15-minute access tokens and rotating 30-day refresh tokens (reuse revokes the device); usernames (3 to 32, lowercase, digits, underscore, start with a letter, case-insensitive unique, reserved words, 30 lookups per minute); key bundle upload and fetch (consuming one-time prekeys, rate-limited); registration lock check; POST /messages with idempotency keys and per-device envelopes; GET /ws with replay of pending entries in order, ack, XADD/XDEL queue, 30-day trim.
4. Everything configured by environment variables, Dockerfile runs as non-root with a read-only filesystem, structured logging with a redaction layer, request size limits, token-bucket rate limits by account, device and IP.

## Tests (required)

- Integration tests (against the real compose services): register two accounts, exchange 1,000 messages, kill and restart the gateway mid-conversation, assert no loss and no duplicates. A load smoke test labelled as a smoke test.
- A test proves no secret or plaintext type implements Display or Debug with content.

## Done only when ALL are true

- Every ledger row of P03 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P03` shows no open rows.
- `node tools/gate.mjs --phase P03 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- `docker compose up -d` then `cargo test --workspace` pass with the services running.
- PROGRESS.md updated and `CURRENT_PHASE: P04` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

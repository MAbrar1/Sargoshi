# ADR-002: Native network boundary

Status: PROPOSED for Stage 0A; production endpoint and pins are intentionally not invented.

## Decision

Use one Kotlin-owned network layer (OkHttp) for REST and WebSocket traffic. Flutter accesses it only through typed Pigeon APIs. Do not let Dart open sockets to the Sargoshi service. Enforce TLS, hostname verification, bounded timeouts, retry with jitter, system DNS by default, and an optional explicit user-selected proxy. Configure SPKI certificate pinning with a primary and backup pin, expiry, and a documented rotation procedure.

The production build must fail closed if the API host or valid pin set is absent or still a placeholder. No production host or pin is committed until supplied by the backend owner and validated through the rotation process. The `.example` domain may be used only in non-networking documentation/tests, never for a release.

## Rationale

A single native boundary keeps REST/WebSocket policy centralized and prevents UI features from accidentally creating unpinned network paths. Pinning is defense in depth, not a substitute for TLS validation or key management. No third-party destinations are allowed by default.

## Consequences

- Backend contract and host configuration are required before Stage 4 networking.
- Pigeon API design must avoid exposing raw sockets or unrestricted URL fetching.
- Pin rotation and backup-pin tests are release gates.
- Calls/media networking through LiveKit requires a separately reviewed exception/configuration; do not imply it is covered by the REST/WebSocket client.

## TODO(verify)

- Verify OkHttp/Android platform compatibility and pinning behavior against the chosen minimum API and current dependency versions.
- Obtain production API hostname and primary/backup SPKI hashes from the backend owner.
- Specify pin expiry, rotation ownership, emergency recovery, and test procedure.
- Review proxy semantics and ensure the explicit proxy setting does not silently change DNS behavior.
- Review LiveKit's network path and certificate/security controls independently.

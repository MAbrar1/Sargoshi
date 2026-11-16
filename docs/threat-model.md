# Sargoshi threat model (Stage 0A)

Status: initial design baseline; not a security audit. It describes intended protections and known limits, not verified implementation claims.

## Assets

- Identity and account keys, recovery material, vault keys, and session keys.
- Message and media plaintext, encrypted local database, search index, and notification state.
- Safety numbers, key-change state, call/share session keys, and user-selected privacy settings.
- User metadata: account ID, display name/handle, contact relationships, IP address, connection timing, message sizes, and device signals.

## Security and privacy objectives

- Keep message/media plaintext and long-term private keys out of the server and Dart UI layer.
- Make encryption, verification, key changes, hardware security, delivery, and AI availability statuses evidence-based. Missing measurements are unknown, not positive.
- Minimize local and network metadata, prevent accidental exposure through backups, logs, notifications, recents, screenshots, and requests-inbox media handling.
- Give users clear control over requests, reporting, blocking, recovery, account deletion, and data sharing.
- Fail closed for release when required security configuration (production host, certificate pins, real crypto provider, or signing configuration) is absent.

## Adversaries and abuse cases

- A network observer, malicious hotspot, or active network attacker attempting interception or traffic manipulation.
- A compromised or curious service operator observing the metadata available to the backend contract, including IP, timing, sizes, and opaque ciphertext.
- A malicious or compromised contact sending harmful content, changing keys, or attempting social engineering and scam links.
- A person with temporary physical access to an unlocked or locked device, including lock-screen, recents, clipboard, notification, and backup exposure.
- A malicious app or overlay attempting tapjacking or misuse of exported Android components/intents.
- A supply-chain compromise, vulnerable dependency, compromised build, or accidental inclusion of debug/fake crypto in release.
- A rooted/unlocked device, OS compromise, memory inspection, screen photography, or a modified client.

## Trust boundaries and controls required

1. **UI ↔ application/domain:** UI renders state and accepts user intent only. Security/health labels consume `Measured<T>` with source and measurement time. Crypto and network operations do not run in UI code.
2. **Flutter ↔ native platform:** typed, validated Pigeon/FFI boundaries. Validate all incoming intent/channel data; do not pass secret material through logs or persistent preferences. Minimize secret copies and clear buffers where feasible.
3. **Application ↔ local storage:** SQLCipher-backed Drift store with keys protected by Android Keystore; background-isolate handoff is a documented residual risk. Backups exclude application data. Vault data uses a separate key/store boundary.
4. **Client ↔ service/network:** one native networking path; TLS, hostname verification, SPKI pin set with backup/expiry/rotation procedure; no Dart API sockets or third-party destinations. Server sees network metadata and opaque blobs; client must not imply metadata concealment.
5. **Messaging/calls/share:** audited protocol implementations only; no custom cryptographic protocol. Key changes gate sending. Call frame and data-channel encryption must both be configured and tested before any positive UI claim. Screen capture requires per-session OS consent and teardown on stop.
6. **AI boundary:** on-device only, separately isolated module with no network dependency. Exclude Vault, Whisper, and disappearing-message content. Feature status is runtime-measured; unavailable is not presented as active.
7. **Build/release boundary:** locked dependencies, verification/audit/SBOM, manifest allow-list, ELF alignment, release fake-crypto exclusion, reproducible recorded toolchain, and Play signing. These controls are not all implemented yet.

## Explicit limitations

- A compromised OS/device, rooted device, malicious recipient, external camera, screenshots on modified systems, and coercion cannot be fully prevented by the app.
- Screen security, disappearing messages, and best-effort deletion reduce exposure but cannot guarantee that recipients did not copy or retain information.
- End-to-end encryption does not conceal all metadata from the service or network. The backend contract permits observation of IP, timing, and sizes.
- Play Integrity is an abuse signal, not a security boundary; device integrity signals must not be used to claim protection or silently deny access.
- Push delivery, OEM background behavior, and battery-optimization settings are not guaranteed. Delivery health must reflect measured observations.
- This document is not a penetration test, cryptographic review, privacy-law analysis, or proof of implementation.

## Stage 0A verification items

- TODO(verify): selected protocol/library versions, security review status, platform support, and license obligations.
- TODO(verify): production App Link/API domain, pin values, backup pin, expiry, and rotation drill.
- TODO(verify): SQLCipher/Drift integration and least-exposure key transfer to the database isolate.
- TODO(verify): Flutter↔Rust FFI generation/build behavior on arm64-v8a and armeabi-v7a and supported 16 KB page devices.
- TODO(verify): all Android version-dependent security behaviors in the supported API range.
- TODO(verify): legal compatibility of all dependencies and distribution obligations.

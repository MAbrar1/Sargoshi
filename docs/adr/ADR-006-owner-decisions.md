# ADR-006: Owner decisions for Stage 0B

Status: ACCEPTED PROVISIONALLY. The owner asked for "the best option" on each open decision, so these were chosen on their behalf. The owner can reverse any of them; items marked (reversible) are cheap to change before Stage 2.

| # | Decision | Choice | Reason | Reversible |
|---|---|---|---|---|
| 1 | Crypto and licence | libsignal (Signal protocol, PQXDH/hybrid) in the Rust core; app published under AGPL-3.0 with source available | libsignal is AGPL, so publishing source is the compatible path. It also supports the Trust screen's "source repository" audit row. | Yes, before Stage 2. Switching later means re-licensing or replacing the crypto core. |
| 2 | Domain | `sargoshi.example` placeholder, kept in one build config value | No domain was provided. A release build must fail while the placeholder is present. | Required before release |
| 3 | Groups | Cap 64 members, OpenMLS (Rust) | Matches the spec; OpenMLS is the maintained Rust MLS implementation. TODO(verify) maturity and licence at Stage 8. | Yes |
| 4 | Bundled AI model | Not shipped in v1. On-device AICore / ML Kit GenAI only, feature-detected | Protects the 45 MB budget and tier C phones. | Yes |
| 5 | armeabi-v7a | Kept, behind a size gate | Cheap 32-bit phones are an explicit target. If the Stage 0B/4 size spike breaks the budget, drop it by ADR. | Yes |

## Consequences
- Release CI must fail if the domain placeholder is present.
- Licence file and a source-offer page are Stage 10 deliverables.
- None of these choices has been legally reviewed. TODO(verify): licence compatibility of every dependency with AGPL-3.0.

## Update 2026-11-17
The owner told us to continue with a fake domain and replace it later. `AppConfig.domain` defaults to `sargoshi.example` and is overridden with `--dart-define=SARGOSHI_DOMAIN=...`. `tool/ci/check_release_ready.py --release` fails while the placeholder domain is the default or while no production `CryptoCore` exists; without `--release` it only lists blockers. The fake core lives in `lib/testing/` and only `crypto_provider.dart` may import it.

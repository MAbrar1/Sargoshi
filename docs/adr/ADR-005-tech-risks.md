# ADR-005: Technical risk register (Stage 0A)

Status: Active. Items remain open until evidence and owner decisions are recorded.

| Risk | Priority | Mitigation / gate | Verification needed |
|---|---|---|---|
| Crypto protocol/library or license is unsuitable | Critical | Select audited implementations; legal and security review; do not ship custom crypto | Protocol support, audit status, version, license obligations |
| Production domain and pin set are unknown | Critical | No release networking with placeholders; fail closed | Backend owner supplies host, primary/backup pins, expiry, rotation plan |
| SQLCipher/Drift native wiring or isolate handoff exposes keys | Critical | Prototype before user data; fail closed on encryption errors | Package support, memory/key handoff review, backup tests |
| Flutter/Rust binding or plugin native binaries fail ABI/page-size requirements | High | Build matrix and ELF checks for all ABIs and dependencies | 16 KB emulator/device validation and APK/AAB inspection |
| Scope is large relative to current skeleton | High | Deliver one stage at a time; no scope expansion; each stage has testable acceptance criteria | Stage reports and CI evidence |
| Android/OEM background, screen-capture, biometric, and permission behavior varies | High | Native API guards, measured fallback, device/API matrix | Version-specific behavior tests and OEM verification |
| Push/LiveKit expose more metadata than users expect | High | Honest metadata copy; minimize payload and destinations | Backend and LiveKit data-flow review |
| Dependency supply-chain compromise | High | Lockfiles, verification metadata, audit/OSV, SBOM, release review | CI enforcement and release artifact inspection |
| Fake crypto or debug configuration reaches release | Critical | Build-time flag/symbol exclusion and release artifact check | Negative CI test against release outputs |
| Requirements conflict or are not implementable on a target API | Medium/High | Stop, document conflict, ask owner; do not silently weaken behavior | API- and device-specific prototype |

## Stage 0A work boundaries

This register and threat model are planning artifacts only. They do not implement or verify security controls. Stage completion requires the named architecture decisions and owner choices to be recorded, plus a review of the unresolved items.

# P06 — Accounts, onboarding, app lock, devices

Depends on: P04 P05. Ledger rows owned by this phase: 75 (Registry features 22, Alerts 7, Settings 19, Journeys 4, Screens 23)

Before anything else: read AGENTS.md and docs/ledger/PROGRESS.md. Work rules there apply in full. This file is your scope.

## Goal

First user-facing slice: sign-up, username, age gate, Secure setup (app lock and recovery key), invites, linked devices, new-device approval, remote wipe, profile, passkeys, registration lock, account deletion. Real server, real keys.

## Read now (use sed -n with these line ranges; never read the blueprint whole)

Blueprint file: `docs/blueprint/Sargoshi-Final-Blueprint-v5.md`
- §7.1 (lines 867-892) Accounts and identity (ACC) and §7.20 (lines 1286-1298) for GRW-03
- §3 (lines 253-295) Journeys J1, J4, J5, J10, J11
- §16.3 (lines 2378-2434) copy rows (Welcome, Secure setup, Sign-up, Account, New device, Remote wipe, Device-link scam), §16.5 (lines 2441-2444) First-run flow
- §14.5 (lines 2195-2203) Account takeover defences, §14.9 (lines 2225-2239) table rows for ACC-16 and ACC-17
- §11.7 (lines 1712-1748) alerts ALR-01 03 04 07 12 18 20
- Figure 10 text (fig-10-onboarding.svg is plain XML if you need geometry)

## Build (do every item, in order, as real working code)

1. Build every screen in the screens.csv rows for P06 with the real flows. Secure setup has TWO cards now (app lock, recovery key); the Vault card is added by P10 because a control for a missing feature must not exist.
2. Recovery key: 24 words from the P02 core, shown once, printable Recovery Kit PDF with QR (generate with a real PDF package), key quiz of 3 random positions with 7, 30 and 90 day reminders (store only the pass date, never the words). ACC-13 signs in with the recovery key.
3. App lock (PIN or biometrics) separate from the Vault, auto-lock choices, lock gate over every route, hidden from recents (PRV-14, PRV-07 is in P07C).
4. Sign-up by email code works today through the SMTP path; the phone/SMS path is real code behind configuration (DONE_NEEDS_CREDENTIALS for the SMS provider).
5. Linked devices with QR linking, new-device approval sheet and the 24-hour path, device list with last active and rename, sign-out with remote wipe (the wipe actually destroys the database key, sessions, Vault keys when they exist, local files, then signs out; 10-minute undo for the sign-out part), device-link scam signal sheet (ALR-18).
6. Invites: rotating contact link and QR, deep links `sargoshi://` and an HTTPS link built from environment configuration `SG_PUBLIC_DOMAIN` (the domain is an owner decision listed in EXTERNAL.md; Android App Links verification needs the P19 landing page).
7. Profile with per-field visibility, passkeys (use the Android Credential Manager through a Kotlin channel; verify), several passkeys and FIDO2 keys, registration lock PIN, username change with 30-day cooldown, contact nicknames and private notes (device-only), multi-account switch, inactive account manager, data request (encrypted export file), delete account with the exact dialog text.
8. Alerts: implement the alert cards/sheets needed by these ALR ids using the P09 framework only if it exists; otherwise build the minimal real sheet and let P09 unify it (record in PROGRESS.md).

## Ledger rows you must close (nothing may stay open)

Use `node tools/ledger.mjs next --phase P06 --kind <KIND> --n 10` to read the full text of rows in batches. Each row ends in `claim` (with tags in code and tests), or for blueprint-v2 rows only, `waive` after a real attempt (D-11). Row lists:

### Registry features (22)

| ID | What | Notes |
|---|---|---|
| ACC-01 | Sign up with phone or email and a one-time code, then choose a unique username | bp line 871 |
| ACC-02 | Add contacts by username, QR code or invite link. No mandatory contact upload | bp line 872 |
| ACC-03 | Linked devices (start with 4) and QR sign-in for a new device | bp line 873 |
| ACC-04 | Device management: last active time, rename, remote sign-out | bp line 874 |
| ACC-05 | Delete account inside the app | bp line 875 |
| ACC-06 | Profile: name, photo, short bio, per-field visibility | bp line 876 |
| ACC-07 | Passkey sign-in | bp line 877 |
| ACC-08 | Multi-account switch | bp line 878 |
| ACC-09 | Registration lock PIN, so nobody can re-register your number or email | bp line 879 |
| ACC-10 | Username change with a 30-day cooldown | bp line 880 |
| ACC-11 | Contact nicknames and private notes, stored only on your devices | bp line 881 |
| ACC-12 | New-device approval: a new device needs approval from an existing device. If none is reachable, a 24-hour wait with alerts on every device | bp line 882 |
| ACC-13 | Sign in with the recovery key instead of a code | bp line 883 |
| ACC-14 | Rotating contact link and QR that revoke the old ones | bp line 884 |
| ACC-15 | Request a copy of the data our servers hold about you, as an encrypted file | bp line 885 |
| ACC-16 | Remote wipe: signing out a lost device from another device also queues a wipe of Sargoshi chats, keys and Vault keys on it, applied on its next connection. App lock an... | bp line 886 |
| ACC-17 | Device-link scam signals: an extra warning when a link request arrives during a call, right after a code request, or from an unusual context, with a plain reason | bp line 887 |
| ACC-18 | Inactive account manager: delete after 6, 12 or 24 months without use, with reminders on every device and by email | bp line 888 |
| ACC-20 | Several passkeys and hardware security keys (FIDO2) | bp line 890 |
| ACC-21 | Age assurance: date-of-birth gate with no ID upload, age-appropriate defaults, and a parent-managed route for pre-teens (SAF-12) | bp line 891 |
| PRV-14 | App lock for the whole app with PIN or biometrics, separate from the Vault | bp line 1035 |
| GRW-03 | Invite cards, links and QR with deep links that open the right chat after install | bp line 1292 |

### Alerts (7)

| ID | What | Notes |
|---|---|---|
| ALR-01 | A new device was linked | bp line 1723 |
| ALR-03 | A sign-in code was requested | bp line 1725 |
| ALR-04 | Wrong Vault or app PIN 3 times | bp line 1726 |
| ALR-07 | Registration lock or recovery key changed | bp line 1729 |
| ALR-12 | Invite or call link about to expire | bp line 1734 |
| ALR-18 | Suspicious device-link request (ACC-17) | bp line 1740 |
| ALR-20 | This device was signed out and wiped by you (ACC-16) | bp line 1742 |

### Settings (19) — ids: STA-01, STA-02, STA-03, STA-04, STA-07, STA-08, STA-09, STC-01, STC-02, STC-03, STC-04, STC-05, STC-06, STC-07, STC-11, STC-12, STC-13, STC-14, STC-15

Each setting must exist in the settings engine with its real default, be visible where the blueprint puts it, persist, and actually change the behaviour it names. Read text with `ledger next --kind ST`.

### Journeys (4)

| ID | What | Notes |
|---|---|---|
| JNY-01 | First run |  |
| JNY-05 | Lost phone |  |
| JNY-10 | Delete account |  |
| JNY-11 | Lost phone, remote wipe |  |

### Screens (23)

| ID | What | Notes |
|---|---|---|
| SCR-001 | Welcome (3 pages) |  |
| SCR-002 | Sign-up: phone or email |  |
| SCR-003 | Verify code |  |
| SCR-004 | Choose username |  |
| SCR-005 | Age gate (date of birth) |  |
| SCR-006 | Secure setup |  |
| SCR-007 | App lock setup and gate |  |
| SCR-008 | Recovery key: show, print kit, quiz |  |
| SCR-009 | Add a friend (QR, link, scan) |  |
| SCR-010 | Public profile preview /u/{username} |  |
| SCR-011 | Linked devices list |  |
| SCR-012 | Link a new device (QR) |  |
| SCR-013 | New-device approval sheet |  |
| SCR-014 | Device detail, rename, sign out, remote wipe |  |
| SCR-015 | Profile and field visibility |  |
| SCR-016 | Contact link and QR manager |  |
| SCR-017 | Registration lock PIN |  |
| SCR-018 | Passkeys and security keys |  |
| SCR-019 | Multi-account switcher |  |
| SCR-020 | Username change |  |
| SCR-021 | Inactive account manager |  |
| SCR-022 | Request my data |  |
| SCR-023 | Delete account |  |


## Tests (required)

- Journeys as integration tests against the real compose gateway: JNY-01, JNY-05, JNY-10, JNY-11 (tag the tests `SG:JNY-01` etc.). Widget tests for every screen state. Golden tests for every P06 screen at 360 and 840 dp, LTR and RTL (tag `SG:SCR-0xx`).
- Server-side abuse tests (code attempts, enumeration limits) already exist from P03/P04; add client tests for error states.

## Done only when ALL are true

- Every ledger row of P06 is DONE / DONE_NEEDS_CREDENTIALS, or (v2 only / external only) waived with the required document. `node tools/ledger.mjs status --phase P06` shows no open rows.
- `node tools/gate.mjs --phase P06 --full` passes, output pasted into your report.
- Every control and gesture of everything built here has been exercised by a real test; nothing is stubbed; no SG-ALLOW exception without a real reason.
- `docs/ledger/DEPENDENCIES.md`, DECISIONS.md, EXTERNAL.md, DEVICE-RUNS.md and docs/architecture notes are current.
- Every P06 row claimed with `node tools/ledger.mjs claim`. SMS-dependent rows use `--credentials` and are listed in EXTERNAL.md.
- PROGRESS.md updated and `CURRENT_PHASE: P07A` set; all work committed.

## If you run out of time or context

Update PROGRESS.md (done, next, exact resume command), commit, and stop. Do not leave half-finished files: finish or revert the slice you are in.

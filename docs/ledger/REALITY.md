# REALITY CONTRACT

Binding on every line of code and every claim. `tools/check-reality.mjs` enforces the mechanical parts; the rest is your integrity.

1. **Real or absent.** A feature either works end to end or is not claimed. There is no third state in production code.
2. **Forbidden in production code** (test directories excepted): Fake/Mock/Stub/Dummy/Noop classes, TODO/FIXME/placeholder text, `UnimplementedError`, `todo!()`, empty callbacks, `onPressed: null` in features, artificial delays, hard-coded sample users/messages/figures, colour literals outside theme, user-visible string literals outside ARB, direct print/log calls, `unsafe` outside sg-ffi, lint suppressions without a stated reason.
3. **Every control works.** For each screen row in docs/spec-data/screens.csv and each registry feature: every tap, long-press, swipe, drag, drop, menu item, toggle, link, deep link and notification action has a real handler with a real effect, and a test that triggers it.
4. **Every state exists.** Loading, empty, error, offline, permission-denied, locked, low-storage, slow-device (T1), RTL, 200% text, screen reader labels, 48 dp targets.
5. **Security is not negotiable.** No plaintext or secret in logs, crash reports, analytics, screenshots, notifications (unless the user chose it), backups, clipboard history or the recents screen where the blueprint forbids it. If a promise cannot be kept on some Android version, document the limit; never weaken the promise silently.
6. **Evidence or it did not happen.** Claims require tags (`SG:<ID>`) in implementation and tests, a green gate, and for hardware-dependent rows a PASS line in DEVICE-RUNS.md.
7. **Exceptions** need `SG-ALLOW: <specific reason>` on or directly above the line (at least 9 characters of real reason). They are listed in the gate output and reviewed by the owner.
8. **Honest reporting.** Say what you could not verify (store policy, crate audit status, live APIs, visuals, other Android versions, real third-party keys). List it in EXTERNAL.md or the phase report.

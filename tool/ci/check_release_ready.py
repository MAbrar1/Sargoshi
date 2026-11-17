#!/usr/bin/env python3
"""Release gate (ADR-006). Always lists blockers; fails only with --release.

Blockers:
  1. Placeholder domain still configured.
  2. No production CryptoCore implementation exists.
  3. The fake crypto core is imported outside its allow-list.
"""
import pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
LIB = ROOT / "lib"
FAKE_ALLOWED = {"lib/application/crypto/crypto_provider.dart"}
problems, blockers = [], []

# Hard rule, always enforced: the fake may only be imported from the allow-list.
for f in LIB.rglob("*.dart"):
    rel = f.relative_to(ROOT).as_posix()
    if rel.startswith("lib/testing/"):
        continue
    if re.search(r"import\s+['\"][^'\"]*fake_crypto_core\.dart['\"]", f.read_text()):
        if rel not in FAKE_ALLOWED:
            problems.append(f"{rel}: imports fake_crypto_core.dart")

# Release blockers (reported always, fatal with --release).
cfg = (LIB / "core/config/app_config.dart").read_text()
if "sargoshi.example" in cfg:
    blockers.append("placeholder domain sargoshi.example is still the default")
impls = [
    f.relative_to(ROOT).as_posix()
    for f in LIB.rglob("*.dart")
    if re.search(r"implements\s+CryptoCore\b", f.read_text())
    and not f.relative_to(ROOT).as_posix().startswith("lib/testing/")
]
if not impls:
    blockers.append("no production CryptoCore implementation exists")

for p in problems:
    print(f"ERROR: {p}")
for b in blockers:
    print(f"RELEASE BLOCKER: {b}")
if not problems and not blockers:
    print("Release gate: clear")
if problems or (blockers and "--release" in sys.argv):
    sys.exit(1)

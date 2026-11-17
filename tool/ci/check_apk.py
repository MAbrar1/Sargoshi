#!/usr/bin/env python3
"""APK checks: each per-ABI APK contains the native core for its own ABI, and
the arm64 APK stays inside the size budget (spec: 45 MB arm64).

usage: check_apk.py <apk> [...]
"""
import sys
import zipfile

BUDGET = {"arm64-v8a": 45 * 1024 * 1024}
LIB = "libsargoshi_ffi.so"
failed = False

for path in sys.argv[1:]:
    abi = next((a for a in ("arm64-v8a", "armeabi-v7a", "x86_64") if a in path), None)
    if abi is None:
        print(f"FAIL {path}: cannot tell ABI from name")
        failed = True
        continue
    with zipfile.ZipFile(path) as z:
        names = z.namelist()
        entry = f"lib/{abi}/{LIB}"
        if entry not in names:
            print(f"FAIL {path}: missing {entry}")
            failed = True
        else:
            info = z.getinfo(entry)
            print(f"ok   {path}: {entry} {info.file_size} B (stored {info.compress_size} B)")
        others = [n for n in names if n.endswith(LIB) and n != entry]
        if others:
            print(f"FAIL {path}: foreign ABI libs {others}")
            failed = True
    import os
    size = os.path.getsize(path)
    limit = BUDGET.get(abi)
    print(f"     {abi} APK {size / 1048576:.2f} MiB" + (f" (budget {limit / 1048576:.0f})" if limit else ""))
    if limit and size > limit:
        print(f"FAIL {path}: over budget")
        failed = True

sys.exit(1 if failed else 0)

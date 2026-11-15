#!/usr/bin/env python3
"""ARB sanity: valid JSON, no empty values, no duplicate keys."""
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]


def no_dupes(pairs):
    keys = [k for k, _ in pairs]
    dupes = {k for k in keys if keys.count(k) > 1}
    if dupes:
        raise ValueError(f"duplicate keys: {sorted(dupes)}")
    return dict(pairs)


def main() -> int:
    bad = 0
    files = sorted((ROOT / "lib/l10n").glob("*.arb"))
    if not files:
        print("no ARB files found")
        return 1
    for path in files:
        try:
            data = json.loads(path.read_text(), object_pairs_hook=no_dupes)
        except ValueError as e:
            print(f"{path.name}: {e}")
            bad += 1
            continue
        for k, v in data.items():
            if not k.startswith("@") and (not isinstance(v, str) or not v.strip()):
                print(f"{path.name}: empty or non-string value for {k}")
                bad += 1
    print("check_arb: OK" if not bad else f"check_arb: {bad} problem(s)")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())

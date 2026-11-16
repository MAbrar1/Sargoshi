#!/usr/bin/env python3
"""Fail if any banned marketing claim appears in source, ARB, docs or tests."""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
BANNED_FILE = ROOT / "tool/ci/banned_words.txt"
BANNED = [w.strip() for w in BANNED_FILE.read_text().splitlines() if w.strip()]
SCAN_DIRS = ["lib", "test", "android", "docs", "tool", "store", "pigeons", "rust"]
SCAN_EXT = {".dart", ".arb", ".kt", ".kts", ".xml", ".md", ".yaml", ".yml",
            ".py", ".txt", ".json", ".gradle", ".rs", ".toml"}
# This file must name the words to ban them.
SELF = pathlib.Path(__file__).resolve()


def main() -> int:
    pattern = re.compile("|".join(re.escape(w) for w in BANNED), re.I)
    hits = []
    files = [p for d in SCAN_DIRS for p in (ROOT / d).rglob("*") if p.is_file()]
    files += [p for p in ROOT.glob("*") if p.is_file()]
    for path in files:
        if path.resolve() in (SELF, BANNED_FILE.resolve()) or path.suffix not in SCAN_EXT:
            continue
        for n, line in enumerate(path.read_text(errors="ignore").splitlines(), 1):
            if pattern.search(line):
                hits.append(f"{path.relative_to(ROOT)}:{n}: {line.strip()[:80]}")
    if hits:
        print("Banned words found:\n" + "\n".join(hits))
        return 1
    print(f"banned_words: OK ({len(files)} files scanned)")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Forbid physical left/right in Dart layout code (use start/end)."""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
FORBIDDEN = re.compile(
    r"\b(EdgeInsets\.(only|fromLTRB)|Alignment\.(centerLeft|centerRight|"
    r"topLeft|topRight|bottomLeft|bottomRight)|Positioned\(|TextAlign\.(left|right)|"
    r"Radius\.circular.*topLeft|BorderRadius\.only|Border\.(only))\b|"
    r"\b(left|right)\s*:"
)


def main() -> int:
    hits = []
    for path in (ROOT / "lib").rglob("*.dart"):
        if "l10n" in path.parts:
            continue
        for n, line in enumerate(path.read_text().splitlines(), 1):
            code = line.split("//")[0]
            if FORBIDDEN.search(code):
                hits.append(f"{path.relative_to(ROOT)}:{n}: {line.strip()}")
    if hits:
        print("Use directional APIs (start/end):\n" + "\n".join(hits))
        return 1
    print("check_directional: OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())

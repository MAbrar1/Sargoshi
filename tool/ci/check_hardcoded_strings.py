#!/usr/bin/env python3
"""Fail on user-visible string literals in Dart UI code (spec: all strings in ARB).

Flags Text('...') and common label arguments (tooltip, semanticLabel, hintText,
labelText, label, title, semanticsLabel) given a literal. Skips lib/l10n and
generated files. Add `// l10n-ignore` on a line for a reviewed exception.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
TEXT = re.compile(r"\bText\(\s*(?:const\s+)?r?['\"]")
ARG = re.compile(
    r"\b(tooltip|semanticLabel|semanticsLabel|hintText|labelText|label|title)\s*:\s*r?['\"](?!['\"])"
)


def main(root: pathlib.Path = ROOT) -> int:
    hits = []
    for path in (root / "lib").rglob("*.dart"):
        parts = path.relative_to(root).parts
        if "l10n" in parts or path.name.endswith(".g.dart"):
            continue
        for n, line in enumerate(path.read_text().splitlines(), 1):
            code = line.split("//")[0] if "l10n-ignore" not in line else ""
            if code and (TEXT.search(code) or ARG.search(code)):
                hits.append(f"{path.relative_to(root)}:{n}: {line.strip()}")
    if hits:
        print("Move these strings into lib/l10n/app_en.arb:\n" + "\n".join(hits))
        return 1
    print("check_hardcoded_strings: OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""WCAG contrast check for every text/background pair, both themes.

Colours are parsed from lib/core/design/tokens.dart so there is one source.
Body text needs 4.5:1; large text and icons need 3:1.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
SRC = (ROOT / "lib/core/design/tokens.dart").read_text()


def tokens(cls: str) -> dict:
    block = re.search(rf"class {cls} \{{(.*?)\n\}}", SRC, re.S).group(1)
    return {
        m.group(1): m.group(2)
        for m in re.finditer(r"static const (\w+) = Color\(0xFF([0-9A-Fa-f]{6})\)", block)
    }


def lum(hex6: str) -> float:
    def ch(v: int) -> float:
        c = v / 255
        return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4
    r, g, b = (int(hex6[i:i + 2], 16) for i in (0, 2, 4))
    return 0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b)


def ratio(a: str, b: str) -> float:
    la, lb = sorted((lum(a), lum(b)), reverse=True)
    return (la + 0.05) / (lb + 0.05)


D, L = tokens("DarkTokens"), tokens("LightTokens")
TEXT, LARGE = 4.5, 3.0
# (theme, foreground, background, minimum, usage)
PAIRS = [
    ("dark", D["moon"], D["ink"], TEXT, "Moon on Ink"),
    ("dark", D["moon"], D["dusk"], TEXT, "Moon on Dusk"),
    ("dark", D["moon"], D["dusk2"], TEXT, "Moon on Dusk2"),
    ("dark", D["mist"], D["ink"], TEXT, "Mist on Ink"),
    ("dark", D["mist"], D["dusk"], TEXT, "Mist on Dusk"),
    ("dark", D["mist"], D["dusk2"], TEXT, "Mist on Dusk2"),
    ("dark", D["white"], D["emberStart"], TEXT, "White on Ember gradient start"),
    ("dark", D["white"], D["roseEnd"], TEXT, "White on Rose gradient end"),
    ("dark", D["ink"], D["amber"], TEXT, "Ink on Amber (sharing bar)"),
    ("dark", D["mint"], D["ink"], TEXT, "Mint on Ink"),
    ("dark", D["mint"], D["dusk"], TEXT, "Mint on Dusk"),
    ("dark", D["coral"], D["ink"], TEXT, "Coral on Ink"),
    ("dark", D["coral"], D["dusk"], TEXT, "Coral on Dusk"),
    ("dark", D["glow"], D["ink"], LARGE, "Glow icon on Ink"),
    ("dark", D["glow"], D["dusk"], LARGE, "Glow icon on Dusk"),
    ("light", L["text"], L["parchment"], TEXT, "Text on Parchment"),
    ("light", L["text"], L["paper"], TEXT, "Text on Paper"),
    ("light", L["text"], L["sand"], TEXT, "Text on Sand"),
    ("light", L["secondary"], L["parchment"], TEXT, "Secondary on Parchment"),
    ("light", L["secondary"], L["paper"], TEXT, "Secondary on Paper"),
    ("light", L["secondary"], L["sand"], TEXT, "Secondary on Sand"),
    ("light", L["emberDeep"], L["parchment"], TEXT, "Ember Deep on Parchment"),
    ("light", L["emberDeep"], L["paper"], TEXT, "Ember Deep on Paper"),
    ("light", L["mintDeep"], L["parchment"], TEXT, "Mint Deep on Parchment"),
    ("light", L["mintDeep"], L["paper"], TEXT, "Mint Deep on Paper"),
    ("light", L["amberDeep"], L["parchment"], TEXT, "Amber Deep on Parchment"),
    ("light", L["amberDeep"], L["paper"], TEXT, "Amber Deep on Paper"),
    ("light", L["coralDeep"], L["parchment"], TEXT, "Coral Deep on Parchment"),
    ("light", L["coralDeep"], L["paper"], TEXT, "Coral Deep on Paper"),
]


def main() -> int:
    failed = 0
    for theme, fg, bg, need, label in PAIRS:
        r = ratio(fg, bg)
        ok = r >= need
        failed += not ok
        print(f"{'PASS' if ok else 'FAIL'}  {theme:5} {label:34} {r:5.2f}:1 (need {need})")
    print(f"\n{len(PAIRS) - failed}/{len(PAIRS)} pairs pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())

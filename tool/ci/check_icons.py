#!/usr/bin/env python3
"""Check that the glyph font and the Dart constants agree (spec section 5.3).

Fails if a Dart constant points at a missing or empty glyph, if a glyph has no
constant, if codepoints collide, or if the font is not named SargoshiIcons.
Requires: pip install fonttools
"""
import pathlib
import re
import sys

from fontTools.ttLib import TTFont

ROOT = pathlib.Path(__file__).resolve().parents[2]
REQUIRED = {
    "whisperWave", "lantern", "lanternVerified", "lanternChanged", "keyRotate",
    "safetyGrid", "vaultDoor", "guidePointer", "shareGuard", "scamShield",
    "relayShield", "moonMute", "emberPin", "tickQueued", "tickSent",
    "tickDelivered", "tickOpened",
}


def camel(s: str) -> str:
    head, *rest = s.split("_")
    return head + "".join(w.title() for w in rest)


def main() -> int:
    font = TTFont(ROOT / "assets/fonts/SargoshiIcons.ttf")
    dart = (ROOT / "lib/core/design/sargoshi_icons.dart").read_text()
    problems = []

    if font["name"].getDebugName(1) != "SargoshiIcons":
        problems.append("font family name must be SargoshiIcons")
    if font["head"].unitsPerEm != 1000:
        problems.append("unitsPerEm must be 1000")

    consts = {n: int(h, 16) for n, h in re.findall(
        r"static const (\w+) = IconData\(0x([0-9a-fA-F]+)", dart)}
    cmap = font.getBestCmap()
    glyf = font["glyf"]
    by_name = {camel(g): cp for cp, g in cmap.items()}

    for name in sorted(REQUIRED - consts.keys()):
        problems.append(f"missing Dart constant {name}")
    for name, cp in consts.items():
        glyph = cmap.get(cp)
        if glyph is None:
            problems.append(f"{name}: codepoint U+{cp:04X} not in font")
        elif camel(glyph) != name:
            problems.append(f"{name}: maps to glyph {glyph}")
        elif glyf[glyph].numberOfContours == 0:
            problems.append(f"{name}: glyph is empty")
    for name in sorted(set(by_name) - consts.keys()):
        problems.append(f"glyph {name} has no Dart constant")
    if len(set(consts.values())) != len(consts):
        problems.append("duplicate codepoints in Dart constants")
    if "fontFamily: fontFamily" not in dart or "'SargoshiIcons'" not in dart:
        problems.append("Dart constants must use the SargoshiIcons family")

    if problems:
        print("check_icons FAILED:\n  " + "\n  ".join(problems))
        return 1
    print(f"check_icons: OK ({len(consts)} glyphs)")
    return 0


if __name__ == "__main__":
    sys.exit(main())

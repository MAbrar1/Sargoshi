#!/usr/bin/env python3
"""Subset the bundled fonts (spec section 5.2). Reproducible and checked.

usage: build_fonts.py <google/fonts checkout> <output dir>

Keeps Basic Latin, Latin-1, common punctuation and spaces. Fails if any font
lacks required glyphs, if a licence file is not the SIL OFL, or if the total
payload exceeds 250 KB.
"""
import hashlib
import io
import pathlib
import shutil
import sys

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

SRC = pathlib.Path(sys.argv[1])
OUT = pathlib.Path(sys.argv[2])
BUDGET = 250 * 1024

# U+00AD (soft hyphen) is invisible and ignored by text engines; many fonts omit it.
REQUIRED = list(range(0x20, 0x7F)) + [c for c in range(0xA0, 0x100) if c != 0xAD]
# Display font (Instrument Serif) only has to cover Basic Latin and accented
# letters; it is used for headlines, not running text.
LETTERS = [c for c in range(0xC0, 0x100) if c not in (0xD7, 0xF7)]
CORE = list(range(0x20, 0x7F)) + LETTERS
EXTRA = (
    list(range(0x2002, 0x200B))   # spaces incl. thin space (safety-number groups)
    + [0x202F]                     # narrow no-break space
    + list(range(0x2010, 0x2016))  # dashes
    + list(range(0x2018, 0x2020))  # quotes
    + [0x2020, 0x2021, 0x2022, 0x2026, 0x2030, 0x2039, 0x203A, 0x20AC, 0x2122, 0x2212]
)
FEATURES = ["kern", "liga", "calt", "tnum", "lnum", "case", "ccmp", "locl", "mark", "mkmk"]


def sha(p: pathlib.Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def reload(font: TTFont) -> TTFont:
    """Round-trip through bytes; the subsetter needs a freshly loaded font."""
    buf = io.BytesIO()
    font.save(buf)
    buf.seek(0)
    return TTFont(buf)


def pick(folder: str, pattern: str, exclude: str = "") -> pathlib.Path:
    hits = [p for p in (SRC / "ofl" / folder).glob(pattern) if not (exclude and exclude in p.name)]
    if len(hits) != 1:
        sys.exit(f"expected exactly one match for {folder}/{pattern}, got {hits}")
    return hits[0]


def subset_font(font: TTFont, name: str) -> pathlib.Path:
    opts = subset.Options()
    opts.layout_features = FEATURES
    opts.hinting = False
    opts.notdef_outline = True
    opts.name_IDs = [0, 1, 2, 3, 4, 5, 6, 13, 14]
    opts.name_languages = [0x409]
    opts.glyph_names = False
    opts.drop_tables += ["DSIG"]
    s = subset.Subsetter(opts)
    s.populate(unicodes=REQUIRED + EXTRA)
    s.subset(font)
    dest = OUT / name
    font.save(dest)
    return dest


def check(path: pathlib.Path, strict: bool) -> None:
    cmap = TTFont(path).getBestCmap()
    need = REQUIRED if strict else CORE
    missing = [hex(c) for c in need if c not in cmap]
    if missing:
        sys.exit(f"{path.name}: missing required glyphs {missing[:8]}")
    absent = [hex(c) for c in REQUIRED + EXTRA if c not in cmap]
    print(f"  note {path.name}: {len(absent)} of the wider set absent {absent[:12]}")


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    lic = OUT / "licenses"
    lic.mkdir(exist_ok=True)
    jobs = []

    # Inter: variable. Pin optical size at 14, keep weights 400-700.
    inter = pick("inter", "Inter*.ttf", exclude="Italic")
    f = TTFont(inter)
    f = reload(instancer.instantiateVariableFont(f, {"opsz": 14, "wght": (400, 700)}))
    jobs.append(("Inter-Variable.ttf", f, inter, "inter"))

    for style in ("Regular", "Italic"):
        p = pick("instrumentserif", f"InstrumentSerif-{style}.ttf")
        jobs.append((f"InstrumentSerif-{style}.ttf", TTFont(p), p, "instrumentserif"))

    mono = pick("jetbrainsmono", "JetBrainsMono*.ttf", exclude="Italic")
    f = reload(instancer.instantiateVariableFont(TTFont(mono), {"wght": 400}))
    jobs.append(("JetBrainsMono-Regular.ttf", f, mono, "jetbrainsmono"))

    total = 0
    for name, font, src, folder in jobs:
        out = subset_font(font, name)
        check(out, strict=not name.startswith('InstrumentSerif'))
        size = out.stat().st_size
        total += size
        print(f"{name}: {size} bytes  (source {src.name} sha256 {sha(src)})")
        ofl = SRC / "ofl" / folder / "OFL.txt"
        text = ofl.read_text(errors="ignore")
        if "SIL OPEN FONT LICENSE" not in text.upper():
            sys.exit(f"{folder}: OFL.txt does not look like the SIL OFL")
        shutil.copy(ofl, lic / f"{folder}-OFL.txt")
    print(f"TOTAL {total} bytes (budget {BUDGET})")
    if total > BUDGET:
        sys.exit("font payload over budget")
    return 0


if __name__ == "__main__":
    sys.exit(main())

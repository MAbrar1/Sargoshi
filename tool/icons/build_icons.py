#!/usr/bin/env python3
"""Build the Sargoshi glyph font (spec section 5.3).

Glyphs are drawn on a 24 dp grid with a 1.5 dp stroke, round caps and joins,
then outlined and compiled into ONE icon font plus a Dart constants file.

usage: build_icons.py <assets/fonts dir> <lib/core/design/sargoshi_icons.dart> [preview.png]
Requires: pip install fonttools shapely (pillow for the optional preview).
"""
import math
import pathlib
import sys

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from shapely import affinity
from shapely.geometry import LineString, MultiPoint, Point, Polygon
from shapely.geometry.polygon import orient
from shapely.ops import unary_union

STROKE = 1.5
EM = 1000
ASCENT, DESCENT = 850, -150
GRID = 24
FIRST_CODEPOINT = 0xE000


# ---- drawing primitives (SVG-style coordinates: y points down) ----------
def stroke(geom, w=STROKE):
    return geom.buffer(w / 2, cap_style="round", join_style="round", quad_segs=16)


def line(*pts):
    return stroke(LineString(pts))


def arc_pts(cx, cy, r, a0, a1, n=48):
    return [
        (cx + r * math.cos(math.radians(a0 + (a1 - a0) * i / n)),
         cy + r * math.sin(math.radians(a0 + (a1 - a0) * i / n)))
        for i in range(n + 1)
    ]


def arc(cx, cy, r, a0, a1):
    return stroke(LineString(arc_pts(cx, cy, r, a0, a1)))


def ring(cx, cy, r):
    return stroke(LineString(arc_pts(cx, cy, r, 0, 360, 96)))


def disc(cx, cy, r):
    return Point(cx, cy).buffer(r, quad_segs=16)


def closed(*pts):
    return stroke(LineString(list(pts) + [pts[0]]))


def rrect_pts(x, y, w, h, r):
    pts = []
    for cx, cy, a in ((x + w - r, y + r, -90), (x + w - r, y + h - r, 0),
                      (x + r, y + h - r, 90), (x + r, y + r, 180)):
        pts += arc_pts(cx, cy, r, a, a + 90, 12)
    return pts


def rrect(x, y, w, h, r):
    p = rrect_pts(x, y, w, h, r)
    return stroke(LineString(p + [p[0]]))


def filled_rrect(x, y, w, h, r):
    return Polygon(rrect_pts(x, y, w, h, r))


def quad(p0, p1, p2, n=24):
    return [((1 - t) ** 2 * p0[0] + 2 * (1 - t) * t * p1[0] + t * t * p2[0],
             (1 - t) ** 2 * p0[1] + 2 * (1 - t) * t * p1[1] + t * t * p2[1])
            for t in (i / n for i in range(n + 1))]


def arrowhead(p, d, length=3.4, spread=38):
    dx, dy = d
    out = []
    for s in (spread, -spread):
        a = math.radians(180 + s)
        rx = dx * math.cos(a) - dy * math.sin(a)
        ry = dx * math.sin(a) + dy * math.cos(a)
        out.append(line(p, (p[0] + rx * length, p[1] + ry * length)))
    return unary_union(out)


# ---- shared shapes -------------------------------------------------------
def shield_pts():
    right = quad((19.5, 10.5), (19.5, 17.5), (12, 21.2))
    left = quad((12, 21.2), (4.5, 17.5), (4.5, 10.5))
    return [(12, 2.8), (19.5, 5.6)] + right + left[1:] + [(4.5, 5.6)]


def shield():
    p = shield_pts()
    return stroke(LineString(p + [p[0]]))


def lantern_body():
    handle = arc(12, 8, 3.5, 180, 360)
    body = closed((8.5, 8), (15.5, 8), (17.5, 13), (15.5, 19), (8.5, 19), (6.5, 13))
    base = line((9.5, 21), (14.5, 21))
    return unary_union([handle, body, base])


def tick_check(x0, y0):
    return line((x0, y0), (x0 + 4.7, y0 + 4.7), (x0 + 14, y0 - 5.3))


# ---- the glyphs ----------------------------------------------------------
def build_glyphs():
    g = {}
    g["whisper_wave"] = unary_union(
        [line((x, 12 - h / 2), (x, 12 + h / 2))
         for x, h in ((3.5, 4), (7.75, 9), (12, 15), (16.25, 9), (20.5, 4))])
    g["lantern"] = unary_union([lantern_body(), disc(12, 13.5, 1.7)])
    g["lantern_verified"] = unary_union(
        [lantern_body(), line((9.6, 13.6), (11.4, 15.4), (14.6, 11.4))])
    g["lantern_changed"] = unary_union(
        [lantern_body(), line((12, 10.6), (12, 14.2)), disc(12, 16.6, 0.95)])

    p, d = (12 + 9 * math.cos(math.radians(300)), 12 + 9 * math.sin(math.radians(300))), (0.866, 0.5)
    g["key_rotate"] = unary_union([
        arc(12, 12, 9, 20, 300), arrowhead(p, d),
        ring(10.3, 10.8, 2.3), line((12.1, 12.6), (16, 16.5)), line((14.3, 14.8), (16.4, 12.7)),
    ])

    def finder(x, y):
        return unary_union([rrect(x, y, 7, 7, 1.6), filled_rrect(x + 2.3, y + 2.3, 2.4, 2.4, 0.5)])

    g["safety_grid"] = unary_union([
        finder(3, 3), finder(14, 3), finder(3, 14),
        filled_rrect(14.2, 14.2, 2.6, 2.6, 0.6), filled_rrect(18.4, 18.4, 2.6, 2.6, 0.6),
        filled_rrect(18.4, 14.2, 2.6, 2.6, 0.6), filled_rrect(14.2, 18.4, 2.6, 2.6, 0.6),
    ])

    g["vault_door"] = unary_union(
        [rrect(3, 3, 18, 18, 3.5), ring(12, 12, 4.4), disc(12, 12, 1.1)]
        + [line((12 + 4.4 * math.cos(math.radians(a)), 12 + 4.4 * math.sin(math.radians(a))),
                (12 + 7 * math.cos(math.radians(a)), 12 + 7 * math.sin(math.radians(a))))
           for a in (45, 135, 225, 315)])

    g["guide_pointer"] = closed((6, 3.8), (6, 18), (9.8, 14.6), (12.6, 20.4), (15.2, 19.2),
                                (12.5, 13.6), (17.8, 13.2))

    sp = [(12, 6.7), (15.4, 8), (15.4, 10.4), (12, 13.2), (8.6, 10.4), (8.6, 8)]
    g["share_guard"] = unary_union([
        rrect(2.5, 3.5, 19, 13.5, 2.5), line((12, 17), (12, 20.5)), line((8, 20.5), (16, 20.5)),
        stroke(LineString(sp + [sp[0]]), 1.3)])

    g["scam_shield"] = unary_union([shield(), line((12, 8), (12, 13)), disc(12, 16.2, 1.05)])
    g["relay_shield"] = unary_union([
        shield(), line((8.7, 14.2), (12, 9.4), (15.3, 14.2)),
        disc(8.7, 14.2, 1.35), disc(12, 9.4, 1.35), disc(15.3, 14.2, 1.35)])

    crescent = Point(11, 12).buffer(8, quad_segs=32).difference(Point(15.2, 9.3).buffer(6.6, quad_segs=32))
    g["moon_mute"] = unary_union([stroke(crescent.boundary), line((4, 20), (20, 4))])

    pin = MultiPoint(arc_pts(12, 9.4, 6.4, 0, 360, 72) + [(12, 21.3)]).convex_hull
    g["ember_pin"] = unary_union([stroke(pin.boundary), disc(12, 9.4, 2.1)])

    g["tick_queued"] = unary_union([ring(12, 12, 8.5), line((12, 7.2), (12, 12), (15.6, 14.2))])
    g["tick_sent"] = line((4.5, 12.5), (9.5, 17.5), (19.5, 7))
    g["tick_delivered"] = unary_union([
        line((2.2, 12.9), (6.4, 17.1), (14.2, 7.9)),
        line((10.3, 15.2), (11.8, 16.7), (20.8, 7.9)),
    ])
    upper, lower = quad((2.5, 12), (12, 3.2), (21.5, 12)), quad((21.5, 12), (12, 20.8), (2.5, 12))
    g["tick_opened"] = unary_union([stroke(LineString(upper + lower[1:])), disc(12, 12, 3)])
    return g


# ---- font output -----------------------------------------------------------
def to_font_coords(poly: Polygon) -> Polygon:
    k = EM / GRID
    poly = affinity.scale(poly, xfact=k, yfact=-k, origin=(0, 0))
    poly = affinity.translate(poly, 0, ASCENT)
    return orient(poly, sign=-1.0)  # TrueType: outer contours clockwise


def draw(geom) -> "object":
    pen = TTGlyphPen(None)
    polys = [geom] if geom.geom_type == "Polygon" else list(geom.geoms)
    for poly in polys:
        poly = to_font_coords(poly.simplify(0.02))
        for ringc in [poly.exterior, *poly.interiors]:
            pts = [(round(x), round(y)) for x, y in list(ringc.coords)[:-1]]
            dedup = [p for i, p in enumerate(pts) if i == 0 or p != pts[i - 1]]
            if len(dedup) < 3:
                continue
            pen.moveTo(dedup[0])
            for pt in dedup[1:]:
                pen.lineTo(pt)
            pen.closePath()
    return pen.glyph()


def camel(s: str) -> str:
    head, *rest = s.split("_")
    return head + "".join(w.title() for w in rest)


def main() -> int:
    out_dir, dart_path = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
    glyphs = build_glyphs()
    names = list(glyphs)
    fb = FontBuilder(EM, isTTF=True)
    order = [".notdef"] + names
    fb.setupGlyphOrder(order)
    fb.setupCharacterMap({FIRST_CODEPOINT + i: n for i, n in enumerate(names)})
    empty = TTGlyphPen(None).glyph()
    fb.setupGlyf({".notdef": empty, **{n: draw(g) for n, g in glyphs.items()}})
    fb.setupHorizontalMetrics({n: (EM, 0) for n in order})
    fb.setupHorizontalHeader(ascent=ASCENT, descent=DESCENT)
    fb.setupNameTable({"familyName": "SargoshiIcons", "styleName": "Regular",
                       "uniqueFontIdentifier": "SargoshiIcons-1.0",
                       "fullName": "SargoshiIcons Regular", "psName": "SargoshiIcons-Regular",
                       "version": "Version 1.0",
                       "licenseDescription": "Original artwork for Sargoshi, distributed under the app licence (AGPL-3.0)."})
    fb.setupOS2(sTypoAscender=ASCENT, sTypoDescender=DESCENT, usWinAscent=ASCENT, usWinDescent=-DESCENT)
    fb.setupPost()
    fb.font["head"].created = fb.font["head"].modified = 3_500_000_000  # fixed, so builds are deterministic
    out_dir.mkdir(parents=True, exist_ok=True)
    ttf = out_dir / "SargoshiIcons.ttf"
    fb.save(ttf)

    lines = [
        "// GENERATED by tool/icons/build_icons.py. Do not edit by hand.",
        "import 'package:flutter/widgets.dart' show IconData;",
        "",
        "/// Sargoshi glyphs (24 dp grid, 1.5 dp stroke, round caps and joins).",
        "abstract final class SargoshiIcons {",
        "  static const String fontFamily = 'SargoshiIcons';",
    ]
    for i, n in enumerate(names):
        lines.append(f"  static const {camel(n)} = IconData(0x{FIRST_CODEPOINT + i:04x}, fontFamily: fontFamily);")
    lines += ["}", ""]
    dart_path.write_text("\n".join(lines))
    print(f"{ttf.name}: {ttf.stat().st_size} bytes, {len(names)} glyphs")

    if len(sys.argv) > 3:
        from PIL import Image, ImageDraw, ImageFont
        cols, cell, size = 6, 150, 96
        rows = math.ceil(len(names) / cols)
        img = Image.new("RGB", (cols * cell, rows * cell), (11, 13, 26))
        d = ImageDraw.Draw(img)
        f = ImageFont.truetype(str(ttf), size)
        small = ImageFont.load_default()
        for i, n in enumerate(names):
            x, y = (i % cols) * cell, (i // cols) * cell
            d.rectangle([x + 6, y + 4, x + 6 + 100 + 28, y + 4 + 100 + 28], outline=(43, 49, 96))
            d.text((x + 6 + 14, y + 4 + 14 - 12), chr(FIRST_CODEPOINT + i), font=f, fill=(237, 235, 255))
            d.text((x + 8, y + cell - 18), n, font=small, fill=(167, 169, 199))
        img.save(sys.argv[3])
        # small-size strip: how the delivery ticks read at 14 px
        strip = Image.new("RGB", (4 * 60, 40), (11, 13, 26))
        sd = ImageDraw.Draw(strip)
        sf = ImageFont.truetype(str(ttf), 16)
        for j, n in enumerate(("tick_queued", "tick_sent", "tick_delivered", "tick_opened")):
            sd.text((10 + j * 60, 10), chr(FIRST_CODEPOINT + names.index(n)), font=sf, fill=(237, 235, 255))
        strip = strip.resize((strip.width * 4, strip.height * 4), Image.NEAREST)
        strip.save(sys.argv[3].replace(".png", "_small.png"))
    return 0


if __name__ == "__main__":
    sys.exit(main())

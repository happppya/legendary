#!/usr/bin/env python3
"""Generate the Legendary desktop icon set.

Draws a minimal "graph node" glyph (three linked nodes) on a dark rounded
badge, then emits the .ico and .png files the Tauri shell references from
tauri.conf.json. Pure PIL, no network. Run from this directory:

    python generate_icons.py
"""
from __future__ import annotations

from PIL import Image, ImageDraw

OUT = 512  # master canvas, downsample for each size

BG = (22, 26, 32, 255)
BG_EDGE = (47, 53, 64, 255)
EDGE = (122, 132, 148, 255)
NODE = (232, 230, 227, 255)
ACCENT = (224, 164, 88, 255)

ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]
PNG_SIZES = [32, 128, 256]  # 512 canvas is emitted as icon.png


def draw_master() -> Image.Image:
    img = Image.new("RGBA", (OUT, OUT), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    # Rounded badge so the app reads as a single tile on any taskbar theme.
    pad = 14
    radius = 96
    d.rounded_rectangle([pad, pad, OUT - pad, OUT - pad], radius=radius, fill=BG)
    d.rounded_rectangle(
        [pad + 3, pad + 3, OUT - pad - 3, OUT - pad - 3],
        radius=radius - 3,
        outline=BG_EDGE,
        width=6,
    )

    # Three nodes in a shallow triangle + connecting edges.
    pts = [(236, 150), (140, 342), (372, 342)]
    r = 78
    for a, b in ((pts[0], pts[1]), (pts[1], pts[2]), (pts[2], pts[0])):
        d.line([a, b], fill=EDGE, width=46)
    for i, (x, y) in enumerate(pts):
        fill = ACCENT if i == 0 else NODE
        d.ellipse([x - r, y - r, x + r, y + r], fill=fill)
        d.ellipse([x - r, y - r, x + r, y + r], outline=(0, 0, 0, 90), width=10)
    return img


def main() -> None:
    master = draw_master()
    master.save("icon.png", format="PNG")
    master.save(
        "icon.ico",
        format="ICO",
        sizes=[(s, s) for s in ICO_SIZES],
    )
    for s in PNG_SIZES:
        name = "128x128@2x.png" if s == 256 else f"{s}x{s}.png"
        master.resize((s, s), Image.LANCZOS).save(name, format="PNG")
    print("wrote icon.ico, icon.png, 32x32.png, 128x128.png, 128x128@2x.png")


if __name__ == "__main__":
    main()

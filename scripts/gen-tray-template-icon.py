#!/usr/bin/env python3
"""Generate apps/tokenTracer-host/icons/tray.png — the macOS menu-bar *template* glyph.

Template image format: pure black (0,0,0) glyph, information only in the alpha channel,
fully transparent background. macOS tints it for light/dark menu bars. A coin ring with a
bold 'T' (token), 44x44 px = 22 pt @2x.  Windows/Linux use icons/tray-color.png instead
(a black glyph on transparent is nearly invisible on a dark taskbar).

Usage: python3 scripts/gen-tray-template-icon.py   (needs Pillow)
"""
from pathlib import Path
from PIL import Image, ImageDraw

SIZE, SS = 44, 8                       # final px, supersample factor
S = SIZE * SS
img = Image.new("L", (S, S), 0)        # alpha mask: 255 = glyph
d = ImageDraw.Draw(img)

def px(v):                              # final-px units -> supersampled
    return round(v * SS)

# coin ring
m, ring = 3.0, 3.0
d.ellipse([px(m), px(m), S - px(m) - 1, S - px(m) - 1], fill=255)
d.ellipse([px(m + ring), px(m + ring), S - px(m + ring) - 1, S - px(m + ring) - 1], fill=0)
# bold T
cx = S // 2
d.rectangle([px(13), px(13), S - px(13) - 1, px(18.5)], fill=255)          # top bar
d.rectangle([cx - px(2.75), px(13), cx + px(2.75) - 1, px(32)], fill=255)  # stem

alpha = img.resize((SIZE, SIZE), Image.BOX)
out = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
out.putalpha(alpha)                     # RGB stays (0,0,0): no coloured halo
dest = Path(__file__).resolve().parent.parent / "apps/tokenTracer-host/icons/tray.png"
out.save(dest, optimize=True)
print("wrote", dest, out.size, out.mode)

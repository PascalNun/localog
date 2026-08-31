#!/usr/bin/env python3
"""Draw the bitmaps NSIS shows during a Windows install.

Run by hand, not by the build. The two images change about never, so the cost of
a Python toolchain in CI would buy nothing; what matters is that the files in
`src-tauri/installer` are explainable rather than two binaries nobody can
regenerate. Needs `pillow`, `fonttools` and `brotli`, which is why it is not a
step in `build-sidecars.sh`:

    python3 -m venv /tmp/installer-venv
    /tmp/installer-venv/bin/pip install pillow fonttools brotli
    /tmp/installer-venv/bin/python scripts/installer-images.py

The sizes are NSIS's, not ours: 164x314 for the welcome and finish panel, 150x57
for the header on every page between them. BMP3 with no alpha channel, because
that is the only thing the installer will display.

The header sits on white. The rest of that strip is drawn by NSIS and painting a
warm background there would put a visible rectangle in the middle of it, so only
the panel — which occupies its whole side of the dialog — gets the paper colour.
"""

from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
from fontTools.ttLib import TTFont
import tempfile

ROOT = Path(__file__).resolve().parent.parent
FONTS = ROOT / "node_modules/@fontsource/barlow/files"
OUT = ROOT / "src-tauri/installer"

PAPER = (251, 249, 245)     # --workspace
INK = (40, 38, 33)          # --ink
LINE = (221, 215, 206)      # --line
MUTED = (110, 105, 97)      # --muted
WHITE = (255, 255, 255)

# The mark from the application icon: seven bars, tallest just left of centre.
BARS = [0.34, 0.62, 0.88, 1.00, 0.54, 0.78, 0.38]


def barlow(weight: int, size: int) -> ImageFont.FreeTypeFont:
    """Barlow at a weight, unpacked from the woff2 the application itself uses."""
    font = TTFont(FONTS / f"barlow-latin-{weight}-normal.woff2")
    font.flavor = None
    with tempfile.NamedTemporaryFile(suffix=".ttf", delete=False) as handle:
        font.save(handle.name)
        return ImageFont.truetype(handle.name, size)


def waveform(draw: ImageDraw.ImageDraw, centre_x: int, centre_y: int,
             span: int, tallest: int, thickness: int) -> None:
    gap = span / (len(BARS) - 1)
    left = centre_x - span / 2
    for index, height in enumerate(BARS):
        x = left + index * gap
        half = tallest * height / 2
        draw.rounded_rectangle(
            [x - thickness / 2, centre_y - half, x + thickness / 2, centre_y + half],
            radius=thickness / 2,
            fill=INK,
        )


def panel() -> Image.Image:
    """The welcome and finish page's left side."""
    image = Image.new("RGB", (164, 314), PAPER)
    draw = ImageDraw.Draw(image)
    # A hairline where the panel meets the dialog, so the paper reads as a
    # deliberate surface rather than as a slightly-off white.
    draw.line([(163, 0), (163, 313)], fill=LINE)

    waveform(draw, centre_x=82, centre_y=132, span=82, tallest=62, thickness=5)

    name = barlow(600, 27)
    draw.text((82, 206), "LocaLog", font=name, fill=INK, anchor="mm")
    # Not "nothing leaves this machine", which would be a claim about the
    # network this cannot keep: models are downloaded. Meeting content is what
    # stays, and that is what the line says.
    note = barlow(400, 11)
    draw.text((82, 229), "Meetings stay on your machine", font=note, fill=MUTED, anchor="mm")
    return image


def header() -> Image.Image:
    """The strip along the top of every page in between."""
    image = Image.new("RGB", (150, 57), WHITE)
    draw = ImageDraw.Draw(image)
    waveform(draw, centre_x=28, centre_y=29, span=32, tallest=24, thickness=2.6)
    draw.text((54, 29), "LocaLog", font=barlow(600, 17), fill=INK, anchor="lm")
    return image


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    for name, image in (("sidebar", panel()), ("header", header())):
        path = OUT / f"{name}.bmp"
        # BMP3: no alpha channel, which is all NSIS reads.
        image.save(path, format="BMP")
        print(f"{path.relative_to(ROOT)}  {image.width}x{image.height}")


if __name__ == "__main__":
    main()

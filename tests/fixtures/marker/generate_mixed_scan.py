#!/usr/bin/env python3
"""Build the mixed native-text and scanned PDF used by the Marker fixture.

The page has real text-layer content plus a small embedded greyscale image that
contains scanned text.  The image is deliberately smaller than a full-page
scan: the worker must still choose OCR, because without OCR it cannot prove the
image contains no text.  Explicit ``no_ocr`` mode must reject this document
instead of returning a partial result.

Requires Pillow.  Run it inside the worker image, convert the resulting PDF
through ``MarkerConverter``, and replace ``mixed-scan-article.json`` with the
response:

    python3 tests/fixtures/marker/generate_mixed_scan.py /tmp/mixed-scan.pdf
"""

from __future__ import annotations

import zlib
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

PAGE_WIDTH = 612
PAGE_HEIGHT = 792
IMAGE_WIDTH = 720
IMAGE_HEIGHT = 300
PLACED_WIDTH = 360
PLACED_HEIGHT = 150
FONT_PATH = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
SCAN_LINES = ("Scanned text in a small region", "must still trigger OCR.")
IMAGE_OBJECT_ID = 5
FIRST_FORM_OBJECT_ID = 6
VECTOR_GLYPHS = {
    "A": ("01110", "10001", "10001", "11111", "10001", "10001", "10001"),
    "C": ("01111", "10000", "10000", "10000", "10000", "10000", "01111"),
    "E": ("11111", "10000", "10000", "11110", "10000", "10000", "11111"),
    "L": ("10000", "10000", "10000", "10000", "10000", "10000", "11111"),
    "O": ("01110", "10001", "10001", "10001", "10001", "10001", "01110"),
    "R": ("11110", "10001", "10001", "11110", "10100", "10010", "10001"),
    "T": ("11111", "00100", "00100", "00100", "00100", "00100", "00100"),
    "V": ("10001", "10001", "10001", "10001", "10001", "01010", "00100"),
    "X": ("10001", "10001", "01010", "00100", "01010", "10001", "10001"),
    " ": ("00000",) * 7,
}
VECTOR_LABEL = "VECTOR TEXT"
VECTOR_CELL = 8


def vector_lettering(label: str = VECTOR_LABEL) -> str:
    """Draw OCR-readable lettering using PDF path rectangles, not PDF text."""
    commands = []
    cursor = 72
    baseline = 540
    for character in label:
        glyph = VECTOR_GLYPHS[character]
        for row, cells in enumerate(glyph):
            for column, filled in enumerate(cells):
                if filled == "1":
                    x = cursor + column * VECTOR_CELL
                    y = baseline + (6 - row) * VECTOR_CELL
                    commands.append(
                        f"{x} {y} {VECTOR_CELL - 1} {VECTOR_CELL - 1} re"
                    )
        cursor += 6 * VECTOR_CELL
    return " ".join(commands) + " f\n"


def scan_image() -> Image.Image:
    image = Image.new("L", (IMAGE_WIDTH, IMAGE_HEIGHT), 255)
    draw = ImageDraw.Draw(image)
    font = ImageFont.truetype(FONT_PATH, 36)
    for index, line in enumerate(SCAN_LINES):
        draw.text((30, 70 + index * 90), line, fill=0, font=font)
    return image


def build_pdf(header: str, form_depth: int = 1) -> bytes:
    if form_depth < 1:
        raise ValueError("form_depth must be at least one")
    image = scan_image()
    pixels = zlib.compress(image.tobytes(), 6)
    content = (
        f"BT /F1 20 Tf 72 720 Td ({header}) Tj ET\n"
        "BT /F1 12 Tf 72 680 Td (This native paragraph is in the text layer.) Tj ET\n"
        + vector_lettering()
        + f"q {PLACED_WIDTH} 0 0 {PLACED_HEIGHT} 72 320 cm /X1 Do Q\n"
        "BT /F1 12 Tf 72 260 Td (Following native text stays outside the image.) Tj ET\n"
    ).encode("latin-1")
    objects = {
        1: b"<< /Type /Catalog /Pages 2 0 R >>",
        2: b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        3: (
            f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_WIDTH} {PAGE_HEIGHT}] "
            f"/Resources << /Font << /F1 {FIRST_FORM_OBJECT_ID + form_depth} 0 R >> "
            f"/XObject << /X1 {FIRST_FORM_OBJECT_ID} 0 R >> >> "
            "/Contents 4 0 R >>"
        ).encode(),
        4: b"<< /Length " + str(len(content)).encode() + b" >>\nstream\n" + content + b"\nendstream",
        IMAGE_OBJECT_ID: (
            b"<< /Type /XObject /Subtype /Image /Width "
            + str(image.width).encode()
            + b" /Height "
            + str(image.height).encode()
            + b" /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode /Length "
            + str(len(pixels)).encode()
            + b" >>\nstream\n"
            + pixels
            + b"\nendstream"
        ),
        FIRST_FORM_OBJECT_ID + form_depth: b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>",
    }
    for offset in range(form_depth):
        form_id = FIRST_FORM_OBJECT_ID + offset
        child_id = form_id + 1 if offset < form_depth - 1 else IMAGE_OBJECT_ID
        child_name = f"/X{offset + 2}" if offset < form_depth - 1 else "/Im"
        if offset < form_depth - 1:
            form_content = f"{child_name} Do".encode("ascii")
        else:
            form_content = b"/Im Do"
        form_dict = (
            f"<< /Type /XObject /Subtype /Form /BBox [0 0 {PLACED_WIDTH} {PLACED_HEIGHT}] "
            f"/Resources << /XObject << {child_name} {child_id} 0 R >> >> "
        ).encode()
        objects[form_id] = (
            form_dict
            + b"/Length "
            + str(len(form_content)).encode()
            + b" >>\nstream\n"
            + form_content
            + b"\nendstream"
        )

    output = bytearray(b"%PDF-1.4\n")
    offsets = [0] * (max(objects) + 1)
    for index, body in sorted(objects.items()):
        offsets[index] = len(output)
        output.extend(f"{index} 0 obj\n".encode() + body + b"\nendobj\n")
    xref = len(output)
    output.extend(f"xref\n0 {len(offsets)}\n0000000000 65535 f \n".encode())
    for offset in offsets[1:]:
        output.extend(f"{offset:010d} 00000 n \n".encode())
    output.extend(
        b"trailer\n<< /Size "
        + str(len(offsets)).encode()
        + b" /Root 1 0 R >>\nstartxref\n"
        + str(xref).encode()
        + b"\n%%EOF\n"
    )
    return bytes(output)


if __name__ == "__main__":
    import sys

    target = Path(sys.argv[1] if len(sys.argv) > 1 else "mixed-scan.pdf")
    form_depth = int(sys.argv[2]) if len(sys.argv) > 2 else 1
    target.write_bytes(build_pdf("Native header for mixed page", form_depth))
    print(target)

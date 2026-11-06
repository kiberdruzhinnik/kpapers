#!/usr/bin/env python3
"""Generate tiny native-text PDFs used only by the isolated E2E stack."""

from pathlib import Path
import os
import sys

ROOT = Path(os.environ.get("FIXTURE_ROOT", Path(__file__).parent / "fixtures")) / "current-month"


def pdf(text: str) -> bytes:
    escaped = text.replace("\\", "\\\\").replace("(", "\\(").replace(")", "\\)")
    stream = f"BT /F1 12 Tf 50 740 Td ({escaped}) Tj ET".encode("ascii")
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>",
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        b"<< /Length " + str(len(stream)).encode() + b" >>\nstream\n" + stream + b"\nendstream",
    ]
    output = bytearray(b"%PDF-1.4\n")
    offsets = [0]
    for index, body in enumerate(objects, 1):
        offsets.append(len(output))
        output.extend(f"{index} 0 obj\n".encode() + body + b"\nendobj\n")
    xref = len(output)
    output.extend(f"xref\n0 {len(offsets)}\n0000000000 65535 f \n".encode())
    for offset in offsets[1:]:
        output.extend(f"{offset:010d} 00000 n \n".encode())
    output.extend(f"trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode())
    return bytes(output)


def main() -> None:
    ROOT.mkdir(parents=True, exist_ok=True)
    changed = sys.argv[1] if len(sys.argv) > 1 else ""
    first = "Updated fixture article alpha" if changed == "article-alpha.pdf" else "Fixture article alpha"
    (ROOT / "article-alpha.pdf").write_bytes(pdf(first))
    (ROOT / "article-beta.pdf").write_bytes(pdf("Fixture article beta"))
    (ROOT / "club-promotion.pdf").write_bytes(pdf("Join the club today"))
    (ROOT / "ignored.txt").write_text("not a PDF", encoding="utf-8")
    (ROOT / "nested").mkdir(exist_ok=True)
    (ROOT / "nested" / "nested.pdf").write_bytes(pdf("Must not be listed"))


if __name__ == "__main__":
    main()

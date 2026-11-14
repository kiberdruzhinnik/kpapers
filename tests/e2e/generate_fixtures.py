#!/usr/bin/env python3
"""Generate tiny native-text PDFs used only by the isolated E2E stack."""

from pathlib import Path
import os
import sys

ROOT = Path(os.environ.get("FIXTURE_ROOT", Path(__file__).parent / "fixtures")) / "current-month"


def pdf_pages(page_texts: list[str | list[list[tuple[str, int, str]]]]) -> bytes:
    characters = {}
    for page in page_texts:
        if isinstance(page, str):
            lines = [[(line, 12, "F1")] for line in (page.splitlines() or [page])]
        else:
            lines = page
        for line in lines:
            for text, _, *_ in line:
                for char in text:
                    encoded = char.encode("cp1251")
                    if len(encoded) != 1:
                        raise ValueError(f"fixture character is not single-byte CP1251: {char!r}")
                    characters[encoded[0]] = char

    page_ids = [3 + 2 * index for index in range(len(page_texts))]
    content_ids = [page_id + 1 for page_id in page_ids]
    font_id = 3 + 2 * len(page_texts)
    cmap_id = font_id + 4
    objects = {
        1: b"<< /Type /Catalog /Pages 2 0 R >>",
        2: (f"<< /Type /Pages /Kids [{' '.join(f'{page_id} 0 R' for page_id in page_ids)}] "
            f"/Count {len(page_texts)} >>").encode(),
    }
    for page_id, content_id, page in zip(page_ids, content_ids, page_texts):
        objects[page_id] = (
            f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
            f"/Resources << /Font << /F1 {font_id} 0 R /F2 {font_id + 1} 0 R /F3 {font_id + 2} 0 R /F4 {font_id + 3} 0 R >> >> "
            f"/Contents {content_id} 0 R >>"
        ).encode()
        if isinstance(page, str):
            lines = [[(line, 12, "F1")] for line in (page.splitlines() or [page])]
        else:
            lines = page
        commands = ["BT"]
        for index, line in enumerate(lines):
            line_x = line[0][3] if line and len(line[0]) > 3 else 50
            commands.append(f"1 0 0 1 {line_x} {740 - index * 16} Tm")
            for text, size, font_name, *_ in line:
                commands.append(f"/{font_name} {size} Tf <{text.encode('cp1251').hex()}> Tj")
        commands.append("ET")
        stream = " ".join(commands).encode("ascii")
        objects[content_id] = b"<< /Length " + str(len(stream)).encode() + b" >>\nstream\n" + stream + b"\nendstream"

    cmap = [
        "/CIDInit /ProcSet findresource begin", "12 dict begin", "begincmap",
        "/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def",
        "/CMapName /Adobe-Identity-UCS def", "/CMapType 2 def",
        "1 begincodespacerange", "<00> <FF>", "endcodespacerange",
    ]
    entries = [f"<{code:02X}> <{ord(char):04X}>" for code, char in sorted(characters.items())]
    for offset in range(0, len(entries), 100):
        chunk = entries[offset:offset + 100]
        cmap.append(f"{len(chunk)} beginbfchar")
        cmap.extend(chunk)
        cmap.append("endbfchar")
    cmap.extend(["endcmap", "CMapName currentdict /CMap defineresource pop", "end", "end"])
    cmap_stream = "\n".join(cmap).encode("ascii")
    for offset, base_font in enumerate(("Helvetica", "Helvetica-Bold", "Helvetica-Oblique", "Helvetica-BoldOblique")):
        objects[font_id + offset] = f"<< /Type /Font /Subtype /Type1 /BaseFont /{base_font} /Encoding /WinAnsiEncoding /ToUnicode {cmap_id} 0 R >>".encode()
    objects[cmap_id] = b"<< /Length " + str(len(cmap_stream)).encode() + b" >>\nstream\n" + cmap_stream + b"\nendstream"

    output = bytearray(b"%PDF-1.4\n")
    offsets = [0] * (len(objects) + 1)
    for index, body in sorted(objects.items()):
        offsets[index] = len(output)
        output.extend(f"{index} 0 obj\n".encode() + body + b"\nendobj\n")
    xref = len(output)
    output.extend(f"xref\n0 {len(offsets)}\n0000000000 65535 f \n".encode())
    for offset in offsets[1:]:
        output.extend(f"{offset:010d} 00000 n \n".encode())
    output.extend(f"trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode())
    return bytes(output)


def pdf(text: str) -> bytes:
    return pdf_pages([text])


def main() -> None:
    ROOT.mkdir(parents=True, exist_ok=True)
    changed = sys.argv[1] if len(sys.argv) > 1 else ""
    ad = "skladchik.ORG\nТы еще\nне в Клубе?\nЧто такое Клуб «Складчик»?"
    first_page_ad = "КУПЛЕНО НА\n\nskladchik.ORG"
    headline_first_line = (
        "Обновлённый Дмитриев у Трампов, трудности у"
        if changed == "article-alpha.pdf"
        else "Дмитриев у Трампов, трудности у"
    )
    typography_page = [
        [("2", 12, "F1")],
        [("Древние индейцы делятся зелёными", 22.5, "F1")],
        [("технологиями", 22.5, "F1")],
        [("Сегодня для нас нет ничего актуальней добра.", 15, "F1")],
        [],
        [],
        [("06.11.2026", 14, "F2"), (" ", 14, "F1"), ("12:12", 14, "F2")],
        [(headline_first_line, 22, "F2")],
        [("танкеров, мегаполисы США у", 22, "F2")],
        [("социалистов", 22, "F2")],
        [("У Европы кризис жанра", 16, "F2")],
        [("Нижний уровень заголовка", 13, "F2")],
        [],
        [("Fixture article alpha wraps", 12, "F1")],
        [("across two lines.", 12, "F1")],
        [("Bold ", 12, "F2"), ("italic", 12, "F3"), (" both", 12, "F4"), (" styles.", 12, "F1")],
        [],
        [("это не инвестиционные рекомендации", 12, "F1")],
        [],
        [("Традиционные технические предуведомления:", 12, "F1", 70)],
        [],
        [("Для новых читателей: начинайте с этого текста.", 12, "F1", 70)],
        [],
        [("Продолжение технического предуведомления.", 12, "F1", 70)],
        [("Обычный текст после предуведомления.", 12, "F1", 50)],
        [],
        [("Колумнистка Bloomberg по этому поводу пишет:", 12, "F1")],
        [],
        [("«Техас — крупнейший ", 12, "F3"), ("красный", 12, "F4"), (" штат в стране. Он трижды голосовал за", 12, "F3")],
        [("Трампа; Трамп провёл здесь съезд всего два месяца назад.", 12, "F3")],
        [],
        [("Демократ Джеймс Таларико легко победил его со счётом 55% : 43%.", 12, "F3")],
    ]
    # The cover ad is not a content page, so article page labels start at 1.
    first_content_page = [[("1", 12, "F1")]]
    (ROOT / "article-alpha.pdf").write_bytes(pdf_pages([first_page_ad, first_content_page, typography_page, ad]))
    sponsr_page = [
        [("1", 12, "F1")],
        [("sponsr", 24, "F2")],
        [("CrimsonAlter – Analytics", 12, "F1")],
        [("27 февр 11:44", 10, "F1")],
        [("58 минут", 10, "F1")],
        [],
        [("это не инвестиционные рекомендации", 16, "F1")],
        [],
        [("Сразу к делу, коллеги.", 16, "F1")],
        [],
        [("Традиционные технические ", 16, "F1", 70), ("предуведомления:", 16, "F2", 70)],
        [],
        [("Для новых читателей: ", 16, "F1", 70), ("начинайте с этого текста.", 16, "F3", 70)],
        [],
        [("Продолжение ", 16, "F1", 70), ("технического предуведомления.", 16, "F4", 70)],
        [],
        [("На данном проекте периодически упоминается термин «старые деньги».", 16, "F1", 50)],
    ]
    (ROOT / "[Философское] «Старые деньги» — FAQ_ как правильно считать дивиденды, «бессмертные» портфели и стоицизм.pdf").write_bytes(pdf_pages([sponsr_page]))
    (ROOT / "club-promotion.pdf").write_bytes(pdf("Join the club today"))
    # A page with no text layer simulates a scanned-only PDF. Extraction must
    # report a failure instead of publishing empty content when OCR is disabled.
    (ROOT / "scanned-only.pdf").write_bytes(pdf_pages([""]))
    (ROOT / "ignored.txt").write_text("not a PDF", encoding="utf-8")
    (ROOT / "nested").mkdir(exist_ok=True)
    (ROOT / "nested" / "nested.pdf").write_bytes(pdf("Must not be listed"))


if __name__ == "__main__":
    main()

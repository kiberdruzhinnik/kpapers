# Design

## Context

`extract_pdf` obtains styled word metadata from `pdf_oxide`; `text_lines_to_styled_lines` preserves bold and italic flags, and the existing HTML and Markdown renderers serialize them as emphasis. `pdf_oxide` captures text render mode and line width in its internal text buffer but currently derives weight only from font metadata. In the reference PDF, visually bold text uses render mode 2 (fill and stroke) with a stroke width of 0.42857 points at 15-point text, or 0.68571 points at 24-point text: about 2.9% of font size. The existing PDF-ingestion requirement already covers preserving bold and italic styling. See proposal.md for motivation and scope.

## Goals / Non-Goals

**Goals:**
- Translate the reference PDF's narrow fill-and-stroke text treatment into bold metadata consumed by the existing renderers.
- Exercise the real PDF-to-HTML/Markdown path and retain separate unit coverage for the inference rule.

**Non-Goals:**
- Add raster-image OCR or visual-weight analysis.
- Snapshot the entire article output or test unrelated PDF extraction behavior.

## Decisions

- Keep font weight from font metadata as the primary signal. Also classify text as bold when its PDF text rendering mode fills and strokes the glyph (mode 2 or 6) and its positive stroke width is no more than 4% of the font size. The reference uses a 2.9% ratio. Larger outline strokes and other rendering modes do not trigger this rule.
- Apply the inference where `pdf_oxide` has both rendering mode and line width, so the resulting `FontWeight` continues through the existing span, word, and HTML/Markdown paths without changing public data structures.
- Store an unchanged copy of the supplied PDF under `tests/fixtures/pdf-bold-formatting/` and load it with `include_bytes!` from a dedicated integration test. Assert focused passages in HTML and Markdown, including a bold phrase inside italic text, and confirm ordinary text remains unwrapped.
- Keep `full_text` expectations limited to confirming the selected passages remain present as plain text. Do not add a spec delta because the existing PDF-ingestion requirement already defines the expected emphasis behavior.

## Risks / Trade-offs

- [A narrow outline effect may be mistaken for bold] → Limit the rule to fill-and-stroke modes with stroke widths at or below 4% of font size, then verify normal text in the reference remains unwrapped.
- [Some PDFs may use other synthetic-bold encodings] → Keep this rule conservative and add new evidence-backed cases separately.
- [A committed source PDF adds binary fixture data] → The reference file is about 183 KB; retain only this one fixture and avoid generated page images or full-output snapshots.

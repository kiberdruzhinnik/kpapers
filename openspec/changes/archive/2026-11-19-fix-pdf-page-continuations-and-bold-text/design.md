# Design

## Context

See `proposal.md` for the reported failures. `src/pdf_processing.rs` converts `pdf_oxide` text words into styled lines, detects paragraph boundaries across pages, and renders bold runs as `<strong>`. The existing continuation decision also compares line style, font size, and horizontal position, so identifying which signal rejects the supplied page boundary is part of implementation. The project already specifies general continuation and emphasis behavior in `pdf-ingestion`.

## Goals / Non-Goals

**Goals:**
- Preserve the paragraph across the reported page transition while continuing to distinguish genuine paragraph boundaries.
- Ensure bold style detected by the extraction pipeline survives to semantic HTML, including short bold runs between regular text.
- Add focused regression checks at the lowest layer that reproduces each failure.

**Non-Goals:**
- Change the extraction engine or enable OCR.
- Broaden paragraph joining for unrelated punctuation or layout patterns without evidence.
- Add the supplied full article PDF as a repository fixture.

## Decisions

- Diagnose the page transition using the extracted text, geometry, and style metadata from the supplied document, then fix the rejecting signal in the existing continuation/paragraph flow. Prefer a narrow correction to changing the general continuation threshold. Preserve existing paragraph-boundary protections and test both continuation and non-continuation cases.
- Trace bold metadata from `pdf_oxide` text words through `text_lines_to_styled_lines` and HTML serialization. If the bold flag is already present, correct run coalescing or rendering; if it is absent, address style detection at the extraction boundary or add a narrowly scoped fallback supported by evidence. Do not infer bold from word content.
- Add unit-level regression coverage for page continuation and inline bold. Where the provided PDF can be used locally during development, verify its output without checking the article into the repository.

## Risks / Trade-offs

- [Relaxing page-boundary heuristics may merge distinct paragraphs] → Keep style/layout checks where valid and add a counterexample asserting a real paragraph break remains.
- [The PDF engine may not expose reliable bold metadata for this font] → Confirm the metadata at the `TextWord` boundary before choosing whether the fix belongs in the renderer or extractor.
- [A synthetic regression may miss document-specific geometry] → Also check the supplied PDF locally when available, while keeping committed tests small and non-sensitive.

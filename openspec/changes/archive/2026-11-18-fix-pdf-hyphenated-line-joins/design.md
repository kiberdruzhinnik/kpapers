# Design

## Context

PDF text is converted into styled lines, then reconstructed as prose and HTML. In the supplied PDF, `pdf_oxide` emits each line-ending hyphen as a separate final word. Current line reconstruction joins the next lowercase line but retains that glyph, producing `пристрасти-лись` and `неискушен-ными`.

## Goals / Non-Goals

**Goals:**
- Remove discretionary hyphens from the reported line-wrap cases in extracted text and published HTML.
- Retain ordinary inter-word spaces and hyphens within visual lines.

**Non-Goals:**
- Change PDF engines, introduce OCR, or alter publication cleanup and cache behavior.
- Remove meaningful hyphens from words that are not split across lines.

## Decisions

- Test the representation emitted by `pdf_oxide`: a line ending in a standalone hyphen word followed by lowercase text on the next line.
- Remove the final hyphen from both the plain-text buffer and the styled HTML runs when joining a lowercase continuation. Leave hyphens within a visual line unchanged.
- Increment the extraction pipeline revision so cached documents are reprocessed with the corrected text instead of continuing to serve earlier output.
- Reuse `pdf-ingestion` behavior and tests instead of introducing a new capability; this change sharpens an existing PDF text reconstruction contract.

## Risks / Trade-offs

- [A meaningful compound split across visual lines will also lose its line-end hyphen] → Restrict the change to hyphens at the end of a visual line followed by lowercase text, and retain tests for in-line compounds and ordinary wrapped spaces.

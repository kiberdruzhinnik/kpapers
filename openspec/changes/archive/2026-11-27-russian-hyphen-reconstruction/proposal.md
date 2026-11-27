# Proposal

## Why

PDF text extraction can turn Russian lexical compounds into incorrect unhyphenated strings when the source PDF breaks a word at a visual line boundary. The current small list of recognized compounds does not generalize to Russian text, as shown by `во-вторых` becoming `вовторых`.

## What Changes

- Use Russian language data to distinguish lexical hyphens from discretionary word-wrap hyphens across Russian extracted text.
- Preserve lexical hyphens and join their components without spaces; remove discretionary wrap hyphens and join the word fragments.
- Cover the same reconstruction behavior in extracted text and published HTML, including line breaks within a page and across page boundaries.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `pdf-ingestion`: Generalize lexical versus discretionary hyphen reconstruction for Russian text.

## Impact

- `src/pdf_processing.rs`: existing line and page-boundary hyphen reconstruction and its tests.
- PDF-ingestion behavior and cached extracted text, whose existing extraction pipeline revision mechanism may need to invalidate stale results when reconstruction changes.
- A Russian lexical resource or dependency may be needed; its format, maintenance, and fallback behavior belong in the design.

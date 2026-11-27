# Proposal

## Why

The PDF pipeline preserves bold metadata when a document provides a bold font face, but the supplied PDF encodes visibly bold passages as filled and stroked glyphs using a light font face, so their emphasis is lost. The extractor should recognize the document's narrow-stroke bold treatment and protect it with a real-PDF regression test.

## What Changes

- Detect synthetic bold PDF text rendered with fill-and-stroke operators and a narrow stroke relative to its font size.
- Add the supplied 10-page PDF as a test fixture and verify bold and bold-italic output in HTML and Markdown.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. The existing `pdf-ingestion` requirement already requires preserving bold and italic styling.

## Impact

- PDF text style extraction in `vendor/pdf_oxide/src/extractors/text.rs`.
- Test fixture storage under `tests/fixtures/` and an integration test using `kpapers_api::pdf_processing::extract_pdf`.
- No new dependencies or public API changes are expected.

# Proposal

## Why

When a prose paragraph crosses a PDF page boundary and the next page begins with a parenthetical phrase, the extraction pipeline treats the page break as a paragraph break. This incorrectly splits the text and published HTML even though the PDF layout shows a single continuing paragraph.

## What Changes

- Recognize an opening parenthesis followed by lowercase prose as continuation evidence at a page boundary when existing layout and style checks agree.
- Preserve genuine paragraph boundaries and completed-sentence behavior.
- Add regression coverage for the reported Russian passage in normalized text and published HTML.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Join a paragraph across a page boundary when its continuation begins with a parenthetical phrase.

## Impact

- Affects page-boundary continuation detection in `src/pdf_processing.rs` and its PDF ingestion regression coverage.
- Changes cached extracted text and published HTML for affected PDFs; incrementing the processing revision will cause them to be reprocessed.
- No API or dependency changes are expected.

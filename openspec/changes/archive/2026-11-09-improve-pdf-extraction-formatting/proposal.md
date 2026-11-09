# Proposal

## Why

Some PDFs are published with standalone page numbers left in the article body, while ordinary prose is incorrectly promoted to `<h2>` or `<h3>`. This damages readability and document structure in the RSS output, including for the supplied Russian article.

## What Changes

- Remove standalone page labels that match their one-based content-page position, including when leading boundary advertisement pages shift the printed numbering or PDF text extraction places the label after other lines on that page.
- Make heading recognition conservative enough that body paragraphs remain paragraphs when font size or spacing alone resembles a heading.
- Preserve genuine document titles and section hierarchy, wrapped headings, and inline bold and italic styling.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Refine page-label removal and semantic heading recognition requirements to prevent page-number artifacts and false heading classifications.

## Impact

- `src/pdf_processing.rs`: page-level line filtering and heading classification.
- PDF-ingestion specification and extraction-focused tests.
- Existing extracted-content cache fingerprinting may need a revision so documents are reprocessed with updated formatting behavior.
- No API or dependency changes are expected.

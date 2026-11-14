# Proposal

## Why

PDF review shows the article title and introductory paragraph as separate visual blocks, but Miniflux displays them together as one unformatted line. This makes published articles harder to read and means the RSS path does not reliably preserve the document structure visible in the source PDF.

## What Changes

- Preserve meaningful title, lead, and paragraph boundaries in the HTML delivered through RSS.
- Ensure the published entry content renders those blocks as distinct elements in Miniflux.
- Add regression coverage for the reported PDF layout and RSS publication path.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Preserve the semantic distinction between the document title, lead paragraph, and body paragraphs in extracted HTML.
- `rss-publishing`: Deliver structured article HTML in a form that Miniflux displays with title, lead, and body blocks kept distinct.

## Impact

- PDF text layout analysis and cached extracted HTML in `src/pdf_processing.rs`.
- RSS item serialization in `src/rss.rs`.
- PDF extraction and Miniflux end-to-end regression coverage.
- Existing cached extraction may need regeneration if extraction behavior changes.

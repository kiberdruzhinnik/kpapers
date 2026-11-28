# Proposal

## Why

PDF bold recognition now works in the parser, but existing entries can still publish stale cached HTML. The supplied March 2025 export contains no bold markers in any of its 31 Markdown files, and RSS reads the same cached HTML; forcing an entry to parse must refresh that cached content so both outputs show the corrected formatting.

## What Changes

- Make extraction cache invalidation account for parser revisions so unchanged PDFs are reprocessed when formatting behavior changes.
- Ensure a forced parse updates the cached extracted text and HTML, and verify refreshed content reaches RSS and feed Markdown ZIP exports.
- Normalize cached article headings so RSS and Markdown exports contain one H1 followed by a valid H2/H3 hierarchy.
- Add regression coverage using the existing March 10 reference PDF for refreshed bold HTML, Markdown, RSS output, and heading structure while preserving normal text.

## Capabilities

### New Capabilities

- `feed-markdown-export`: Export cached feed content as Markdown while preserving PDF-derived emphasis and a single hierarchical document heading structure.

### Modified Capabilities

- `pdf-ingestion`: Require parser revision changes and forced parsing to refresh cached extraction results for unchanged source PDFs.
- `rss-publishing`: Require refreshed emphasis and normalized single-H1 heading hierarchy in RSS content.

## Impact

- PDF extraction cache revision and forced reprocessing in `src/pdf_processing.rs` and `src/feed_api.rs`.
- RSS publication in `src/rss.rs` and Markdown ZIP export in `src/export.rs`.
- Regression tests using `tests/fixtures/pdf-bold-formatting/dark-enlighteners-feudalism-again.pdf`.

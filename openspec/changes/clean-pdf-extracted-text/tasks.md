# Tasks

## 1. Filter promotional pages and page labels

- [x] 1.1 Add final-page Skladchik recognition and omit only a matching final page; add unit coverage for matching and non-matching final pages and verify with `cargo test`.
- [x] 1.2 Remove only a standalone page-position number at the start of each original PDF page; add unit coverage for matching labels and numbers in body text and verify with `cargo test`.

## 2. Reflow extracted paragraphs

- [x] 2.1 Normalize wrapped lines to spaces while preserving blank-line paragraph and page boundaries; add coverage for wrapped prose, separate paragraphs, and standalone headings or list items, and verify with `cargo test`.
- [x] 2.2 Verify sanitized HTML contains the reflowed text in distinct `<p>` elements without changing HTML escaping; add or update coverage and verify with `cargo test`.

## 3. Reprocess cached PDFs

- [x] 3.1 Add an extraction-pipeline revision to the cached-content fingerprint comparison and stored key; verify old source-only cache fingerprints miss once and the revised key is reusable on subsequent unchanged refreshes.
- [x] 3.2 Extend the end-to-end feed coverage to verify extracted RSS content excludes the matching final ad page and page labels and contains reflowed paragraphs; verify with the project's e2e workflow.

## 4. Detect and render PDF typography

- [x] 4.1 Replace flattened `pdf-extract` output with `pdfium-render` character extraction retaining Unicode, position, font size, font name, weight, and italic metadata; add unit coverage for bold, italic, combined, and missing style metadata and verify with `cargo test`.
- [x] 4.2 Identify standalone headings and assign `<h1>` through `<h4>` from document title, relative typography, placement, and spacing; add unit coverage for title, section, nested heading, and ordinary prose cases and verify with `cargo test`.
- [x] 4.3 Render bold, italic, and combined-style runs as `<strong>` and `<em>` while preserving text escaping and safe-markup filtering; add unit coverage and verify with `cargo test`.
- [x] 4.4 Extend generated PDF fixtures and end-to-end RSS assertions to cover heading hierarchy and bold, italic, and combined styling, plus wrapped text ordering; verify with the project's e2e workflow.

## 5. Refresh cached typography markup

- [x] 5.1 Advance the extraction-pipeline revision from `pdf-text-v1` to `pdf-text-v2`; verify existing v1 cache entries miss once and v2 entries are reused on subsequent unchanged refreshes.
- [x] 5.2 Pin compatible `pdfium-render` and Pdfium runtime versions, package the native library in the API runtime container, and verify the container starts and extracts a fixture PDF.

## 6. Refine wrapped heading and styled-run grouping

- [x] 6.1 Group visual continuation lines of a document title into one logical `<h1>` with spaces between lines; add unit coverage for wrapped titles and ensure continuation lines are not emitted as separate headings.
- [x] 6.2 Coalesce adjacent text runs with identical bold/italic styles within each paragraph while preserving whitespace, nested style transitions, and paragraph boundaries; add unit and end-to-end coverage for a continuous italic quote with an embedded bold phrase and a separate following paragraph.

## 7. Normalize headings in dated headline materials

- [x] 7.1 Merge adjacent same-style runs separated only by whitespace inside headings; add unit and E2E coverage that renders a bold date and time as one `<strong>` span.
- [x] 7.2 Group visual continuation lines of a top-level headline into one `<h2>` with spaces between lines; add unit and E2E coverage for the supplied three-line headline.
- [x] 7.3 Stop emitting `<h4>` elements and render heading candidates below `<h3>` as paragraphs; add unit and E2E coverage.
- [x] 7.4 Advance the extraction-pipeline revision from `pdf-text-v2` to `pdf-text-v3`; verify existing v2 cache entries miss once and v3 entries are reused on subsequent unchanged refreshes.

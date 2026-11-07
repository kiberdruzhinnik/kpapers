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
- [x] 4.2 Identify standalone headings and assign `<h1>` through `<h3>` from document title, relative typography, placement, and spacing; add unit coverage for title, section, nested heading, and ordinary prose cases and verify with `cargo test`.
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

## 8. Remove boundary-page ads and Sponsr publication chrome

- [x] 8.1 Recognize and omit a Skladchik ad on the first page using the “куплено на” and `skladchik.ORG` signals, while retaining the existing final-page pattern and preserving ordinary pages; add unit and E2E coverage.
- [x] 8.2 Omit recognizable Sponsr masthead, byline, date/time, and reading-duration metadata before an identifiable article title; preserve the title and body and add unit and E2E coverage.
- [x] 8.3 Advance the extraction-pipeline revision from `pdf-text-v3` to `pdf-text-v4`; verify existing v3 cache entries miss once and v4 entries are reused on subsequent unchanged refreshes.

## 9. Correct Sponsr headline boundary detection

- [x] 9.1 Ensure Sponsr extraction omits the exact `27 февр 11:44` date/time and `58 минут` reading duration and begins published HTML at the complete `[Философское] ...` headline; add a regression PDF fixture and E2E assertions for the first heading, removed metadata, and preserved body.
- [x] 9.2 Advance the extraction-pipeline revision from `pdf-text-v4` to `pdf-text-v5`; verify existing v4 cache entries miss once and v5 entries are reused on subsequent unchanged refreshes.
- [x] 9.3 Test the supplied Mail.ru Cloud PDF as an optional local E2E reference; recover its image-only headline from the filename when the text layer omits it, remove its date/time and duration, and filter XML-invalid PDF control characters.
- [x] 9.4 Advance the extraction-pipeline revision from `pdf-text-v5` to `pdf-text-v6` so cached documents regenerate with the live-PDF correction.
- [x] 9.5 Run the live-reference E2E against the second Mail.ru folder and cover date-prefixed article filenames, first-page publication metadata removal, and filename-derived headline restoration; advance the cache revision to v7.

## 10. Preserve explicit emphasis and indented notice blocks

- [x] 10.1 Parse matched literal Markdown `**...**` spans in extracted PDF text as `<strong>`, merge with PDF-derived bold styling, preserve unmatched markers as text, and add unit and E2E coverage for `**это не инвестиционные рекомендации**`.
- [x] 10.2 Detect the “Традиционные технические предуведомления:” heading and its consistently indented continuation region, render the region as one `<blockquote>`, and leave following ordinary-margin text outside it; add unit and E2E coverage.
- [x] 10.3 Advance the extraction-pipeline revision from `pdf-text-v7` to `pdf-text-v8`; verify existing v7 cache entries miss once and v8 entries are reused on subsequent unchanged refreshes.

## 11. Preserve Sponsr article block layout

- [x] 11.1 Add a generated regression PDF matching the supplied Sponsr layout and assert the filename-recovered headline is the first `<h1>`, the bold disclaimer and lead/body text remain paragraphs, the indented notice is one `<blockquote>`, and following text stays outside the quote; verify no ordinary body block is promoted to `<h2>` or `<h3>`.
- [x] 11.2 Correct Sponsr-specific heading classification so a recovered filename headline is authoritative and font size alone does not promote following body blocks; preserve real section headings and existing emphasis and quote boundaries, with unit and E2E coverage.
- [x] 11.3 Advance the extraction-pipeline revision from `pdf-text-v8` to `pdf-text-v9`; verify existing v8 cache entries miss once and v9 entries are reused on subsequent unchanged refreshes.

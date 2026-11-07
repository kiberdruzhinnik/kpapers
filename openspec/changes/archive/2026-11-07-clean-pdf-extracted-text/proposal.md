# Proposal

## Why

PDFs in the shared source folder include Skladchik promotional pages at the beginning or end, publication mastheads and metadata before article titles, and page labels in extracted text. Line wrapping also leaves unnecessary breaks inside paragraphs, reducing readability in the published feed. Plain text extraction also loses visual hierarchy and emphasis, so headings, bold text, and italic text are not represented semantically in the published HTML.

## What Changes

- Remove a recognized Skladchik promotional page when it appears at the beginning or end of a PDF.
- Remove recognizable Sponsr masthead, byline, date/time (including forms such as `27 февр 11:44`), and reading-duration metadata (such as `58 минут`) before the article title, so the published content starts at the headline.
- When a Sponsr PDF text layer omits an image-only headline, use its identifiable `[Философское]` filename headline and remove the date/time and reading duration that precede the article body.
- Filter PDF text characters forbidden in XML 1.0 so extracted content remains valid in RSS.
- Omit standalone page labels extracted at the start of their corresponding pages.
- Join wrapped lines within paragraphs while preserving paragraph boundaries.
- Detect headings and represent their hierarchy with semantic `<h1>` through `<h3>` tags, combining visual continuation lines into one logical heading and keeping lower-level text as paragraphs.
- Preserve bold and italic emphasis with semantic `<strong>` and `<em>` tags, merging same-style runs and whitespace-only separators into one span within paragraphs and headings.
- Interpret explicit Markdown bold markers such as `**это не инвестиционные рекомендации**` as semantic `<strong>` text, removing the marker characters from published content.
- Detect visually indented notice sections such as “Традиционные технические предуведомления” and render the complete indented region as a semantic `<blockquote>` while preserving its internal paragraphs and emphasis.
- For Sponsr materials with a recovered `[Философское]` filename headline, keep the headline as the first `<h1>` and preserve the following bold disclaimer, paragraphs, indented notice, and body text as their corresponding semantic blocks instead of promoting ordinary body text to headings.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: define cleanup of promotional pages, page labels, and wrapped paragraph lines in extracted PDF text.

## Impact

PDF text extraction, typography detection, and normalization in the ingestion pipeline, plus sanitized semantic HTML content published in RSS items. The extraction pipeline revision must advance to v9 so cached content is regenerated with the corrected Sponsr block layout.

# Proposal

## Why

PDFs in the shared source folder include a repeated promotional final page and page labels in extracted text. Line wrapping also leaves unnecessary breaks inside paragraphs, reducing readability in the published feed. Plain text extraction also loses visual hierarchy and emphasis, so headings, bold text, and italic text are not represented semantically in the published HTML.

## What Changes

- Remove a matching Skladchik promotional page when it is the final PDF page.
- Omit standalone page labels extracted at the start of their corresponding pages.
- Join wrapped lines within paragraphs while preserving paragraph boundaries.
- Detect headings and represent their hierarchy with semantic `<h1>` through `<h3>` tags, combining visual continuation lines into one logical heading and keeping lower-level text as paragraphs.
- Preserve bold and italic emphasis with semantic `<strong>` and `<em>` tags, merging same-style runs and whitespace-only separators into one span within paragraphs and headings.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: define cleanup of promotional pages, page labels, and wrapped paragraph lines in extracted PDF text.

## Impact

PDF text extraction, typography detection, and normalization in the ingestion pipeline, plus sanitized semantic HTML content published in RSS items. The extraction pipeline revision must advance so cached content is regenerated with the new markup.

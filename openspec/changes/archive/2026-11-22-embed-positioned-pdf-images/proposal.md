# Proposal

## Why

PDF articles can contain images that carry context missing from extracted text, but the current pipeline discards them. Readers should see those images near their original reading positions in RSS HTML and Markdown exports without relying on separate hosted files.

## What Changes

- Extract embedded PDF images with their page and position, and place them near surrounding text in the article reading order.
- Decode JPEG 2000 (`/JPXDecode`) PDF images before converting them to web output, including the format used by the supplied PDF.
- Convert images to a supported web format, cap width at 1200 px, and use libcaesium with lossy quality 80 and metadata removal.
- Embed processed images inline in sanitized publication HTML and Markdown exports; do not create external image files.
- Reprocess cached PDFs when the image extraction and rendering pipeline changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: extract, position, optimize, cache, and safely embed PDF images in processed content.
- `rss-publishing`: publish the inline images as part of safe RSS article HTML.
- `feed-management`: include inline images in Markdown ZIP exports.

## Impact

- PDF extraction and HTML/Markdown rendering, safe HTML policy, cache identity, cached content storage, RSS publication, and Markdown ZIP export.
- Add libcaesium, enable JPEG 2000 decoding in the PDF image extractor, and account for any image decoding/format conversion build requirements needed to process PDF image formats such as JPEG 2000.
- Inline image data increases cached content and export sizes; implementation needs bounded image processing and clear failure behavior.

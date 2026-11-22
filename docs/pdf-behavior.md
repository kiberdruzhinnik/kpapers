# PDF extraction behavior

The application extracts each eligible PDF page with the Rust `pdf_oxide` crate, including JPEG 2000 (`/JPXDecode`) images, then applies publication cleanup and safe HTML rendering. Embedded images are extracted with page bounds and inserted near their vertical reading position. Images are decoded to JPEG, compressed with libcaesium at quality 80 with metadata removed, and resized to at most 1200 pixels wide without enlargement. RSS and Markdown exports use inline `data:image/jpeg` URIs; no separate image files are generated. Scanned-only PDFs without extractable text fail rather than publishing empty or partial content.

The publication pipeline removes recognized boundary advertisements, printed page labels, and Sponsr publication chrome; applies filename-based title recovery and promotional classification; and emits escaped HTML with the application's fixed tag policy. Raw HTML and unsafe links from extracted content are not published.

Cache identity includes the `pdf_oxide` engine identity and processing revision. A change to either causes the next refresh to process the source PDF again. If conversion fails, the last successful cached content remains available.

## Limits and failures

Files larger than `MAX_PDF_BYTES` (25 MiB by default), invalid PDFs, and PDFs without usable text are recorded as extraction failures. Image processing is separately bounded to 100 images per PDF, 20,000 pixels per source dimension, 8 million decoded pixels per image, 32 million decoded pixels per PDF, and 8 MiB of inline image markup per PDF. Images over these limits or images that fail decoding or compression are omitted with a warning; text and other eligible images remain available. The month review retains extraction errors, while other PDFs continue processing. Operators can retry extraction after correcting the source. The promotion decision remains visible; ambiguous files default to publish and the operator can choose publish, skip, or automatic.

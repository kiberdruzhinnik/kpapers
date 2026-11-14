# PDF extraction behavior

The application extracts each eligible PDF page with the default plain-text API in the Rust `pdf_oxide` crate, then applies the existing publication cleanup and safe HTML pipeline. Image embedding and OCR are disabled for feed extraction. Scanned-only PDFs without extractable text fail rather than publishing empty or partial content.

The publication pipeline removes recognized boundary advertisements, printed page labels, and Sponsr publication chrome; applies filename-based title recovery and promotional classification; and emits escaped HTML with the application's fixed tag policy. Raw HTML and unsafe links from extracted content are not published.

Cache identity includes the `pdf_oxide` engine identity and processing revision. A change to either causes the next refresh to process the source PDF again. If conversion fails, the last successful cached content remains available.

## Limits and failures

Files larger than `MAX_PDF_BYTES` (25 MiB by default), invalid PDFs, and PDFs without usable text are recorded as extraction failures. The month review retains the error state and message, while other PDFs continue processing. Operators can retry extraction after correcting the source. The promotion decision remains visible; ambiguous files default to publish and the operator can choose publish, skip, or automatic.

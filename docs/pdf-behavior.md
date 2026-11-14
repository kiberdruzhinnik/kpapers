# PDF extraction behavior

Each feed has an effective extraction engine: **Pdfium** or **Marker**. Pdfium is the default and requires no extra service. Marker is optional, runs in a private worker container, and is selected globally with `PDF_EXTRACTION_ENGINE=marker` or per feed in the feed form. A feed override wins over the global default; an inherit setting follows the default. Existing deployments and existing feeds therefore remain on Pdfium after upgrade.

Both engines share the same publication pipeline after extraction:

- boundary advertisement pages are removed;
- printed page labels and Sponsr publication chrome are stripped;
- article title recovery, including filename-based Sponsr titles, is applied;
- promotional classification uses the first content page;
- paragraphs, headings and emphasis are emitted as escaped HTML with a fixed tag policy;
- unsafe elements, attributes, links, images and source markup are never published.

The cache identity includes the engine and the processing revision. Changing either value causes the document to be reprocessed on its next refresh, even when the source PDF fingerprint is unchanged.

## Pdfium

Pdfium reads the native text layer from in-memory bytes, keeps page boundaries, and uses character positions and font information for line grouping, heading detection and paragraph joining. Files larger than `MAX_PDF_BYTES` (25 MiB by default), invalid PDFs, and PDFs with no usable native text are recorded as extraction failures. Pdfium does not perform OCR.

## Marker

Marker runs in the optional `marker` Compose profile. The API sends the bounded PDF body to the worker, which loads its models once and returns a page-aware structure of lines and emphasis runs. Marker can recover text from scans, while Pdfium cannot. The worker is private to the deployment network and is not a public endpoint.

Marker's output is not trusted as HTML. The worker reduces it to plain text, bold/italic flags, line size and position hints, and the API applies the same cleanup and escaping rules as Pdfium. Fast mode can merge an indented region into one paragraph; the worker restores its source line and paragraph boundaries and mixed emphasis from the PDF text layer, so notice structure and styling survive and following body text stays outside the quote. A fully native PDF uses its text layer instead of OCR only when no page contains an embedded image; a page with native text and even a small scanned region, or an image-only page, still uses OCR. Explicit OCR-disabled mode rejects an incomplete text layer rather than returning partial content. A Marker failure is reported as an extraction failure; the API does **not** silently fall back to Pdfium. Previously cached successful content remains available. See [Marker extraction](marker-evaluation.md) for the contract, pinned versions, evaluation notes and resource requirements.

## Operator-visible failures

Files that exceed the configured limit, fail to download, are malformed, or contain no usable text remain in the month review with extraction status `failure` and a last-error message. The month action reports how many extractions failed and continues processing the remaining PDFs. A failed attempt leaves previously cached text and HTML available. The automatic promotion decision remains visible; ambiguous files default to publish and the operator can choose publish, skip, or automatic.

If Marker is selected but the worker is not running or is misconfigured, affected documents fail with a worker error. Start the worker and retry, or switch the feed (or the global default) back to Pdfium and refresh. Switching engines is a deliberate operator action, not an automatic fallback.

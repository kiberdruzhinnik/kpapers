# PDF extraction behavior

The API uses `pdf-extract` 0.12.1 to read native PDF text from in-memory bytes. It keeps page-separated extraction so the first page can drive promotional classification while the complete text is cached. Files larger than `MAX_PDF_BYTES` (25 MiB by default), invalid PDFs, and PDFs with no usable native text are recorded as extraction failures. OCR is not included.

## Sample evaluation

The supplied public folders were downloaded with `curl` and extracted through the API:

| Source sample | Pages | Extracted characters | Observed output |
| --- | ---: | ---: | --- |
| Yandex Disk October, first root PDF | 10 | 13,958 | Russian title and body text extracted; title and paragraphs are readable. |
| Mail Cloud November, `2026-11-02 [skladchik.org].pdf` | 5 | 8,460 | Russian date header, article title, and body extracted in readable order. |
| Mail Cloud November, `Если вы еще не в Клубе.pdf` | 1 | 495 | First-page text identifies the registration pitch and supports promotional classification. |

The Yandex sample begins with a page number before its title. The Mail article starts with a date and time header. These samples confirm native text extraction across both providers, but do not establish reliable reading order for every column, table, or unusual font encoding. The parser follows the PDF text content order; image-only scans have no usable text and need OCR outside this release.

Text cleanup removes NUL and replacement characters, collapses horizontal whitespace, and preserves line breaks. RSS HTML is generated from escaped text in paragraph tags, with no source markup or attributes carried through. Ambiguous promotion cases remain publishable by default and can be overridden in the operator review screen.

## Operator-visible failures

Files that exceed the configured limit, fail to download, are malformed, or contain no usable native text remain in the month review with extraction status `failure` and a last-error message. The month action reports how many extractions failed and continues processing the remaining PDFs. A failed attempt leaves previously cached text and HTML available. The automatic promotion decision remains visible; ambiguous files default to publish and the operator can choose publish, skip, or automatic.

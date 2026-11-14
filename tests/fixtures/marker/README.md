# Marker response fixtures

These JSON documents are real output from the pinned worker (`marker-pdf`
2.2.0, Python 3.12, CPU `fast` mode) for the synthetic PDFs produced by
`tests/e2e/generate_fixtures.py`, plus a generated image-only scanned page.
They exercise native text, an empty page, a scanned page, a page with both a
native header and a scanned body, boundary advertisements, and Sponsr
publication chrome. The native-article fixture also covers the worker's
text-layer line and emphasis recovery, including a mixed bold/italic line. The
Sponsr fixture includes the indented multi-paragraph notice with mixed emphasis
from the generated PDF, so it covers splitting Marker's merged `ListGroup`
output while preserving style boundaries. The mixed-scan fixture proves that a
page with native text, outlined vector lettering and a small embedded scanned
region uses OCR, retaining the native text, vector lettering and scanned text;
explicit `no_ocr` mode rejects the same document. The Marker E2E suite converts
this PDF with the pinned runtime and checks all three kinds of text. The worker
also uses OCR when it encounters a real path object. `mixed-scan-nested.pdf`
places its scan 40 Form XObjects deep. The worker fails closed because Pdfium's
bounded object traversal cannot establish whether deeper forms contain
additional images.

Regenerate them by running the optional worker profile and posting each PDF to
`/extract`. `mixed-scan-article.json` comes from the PDF built by
`generate_mixed_scan.py` (Pillow required); pass `40` as the second argument to
generate the nested-form case. The other PDFs come from
`tests/e2e/generate_fixtures.py` or an image-only scanned page. Do not add
private or operator documents.

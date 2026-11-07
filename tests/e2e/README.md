# E2E tests

Run the standard disposable Docker stack with generated PDF fixtures:

```sh
bash scripts/e2e.sh
```

To use a local PDF as the Sponsr regression document, set `E2E_REFERENCE_PDF` to the PDF path. The script substitutes that file for the synthetic Sponsr fixture and runs the same RSS and Miniflux checks. If the local temporary filename differs from the document's original filename, set `E2E_REFERENCE_FILENAME` to the original name so filename-derived headings are tested accurately. It does not download or store the reference document:

```sh
E2E_REFERENCE_PDF="/path/to/[Философское] article [sklad.pdf" bash scripts/e2e.sh
```

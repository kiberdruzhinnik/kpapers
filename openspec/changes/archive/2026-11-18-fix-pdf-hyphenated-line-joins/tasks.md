# Tasks

## 1. Reproduce the extraction failure

- [x] 1.1 Inspect the supplied PDF's `pdf_oxide` line and word representation around the reported words, and record the observed line-ending hyphen representation and expected joined spelling in the spec.

## 2. Correct line reconstruction

- [x] 2.1 Add a regression test asserting the reported dehyphenated words in extracted text and HTML, and verify it fails with the current reconstruction behavior.
- [x] 2.2 Remove discretionary line-ending hyphens during lowercase continuation joins, preserve documented compounds, bump the processing revision to reprocess cached PDFs, and verify focused and existing PDF processing tests pass.

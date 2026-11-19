# Tasks

## 1. Preserve lexical hyphens across wrapped lines

- [x] 1.1 Update the shared PDF line-join logic to preserve lexical hyphens across same-page and page-boundary wraps; add regression cases for `какой-либо`, `государств-членов`, `Украина-Россия`, `Косиняк-Камыш`, and `Эр-Рияде`, and verify extracted text and HTML preserve each compound while existing discretionary-hyphen cases still dehyphenate.
- [x] 1.2 Increment the PDF extraction pipeline revision and verify the cache fingerprint changes and the focused PDF processing tests pass.

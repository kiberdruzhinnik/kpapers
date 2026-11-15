# Tasks

## 1. Recognize parenthetical page continuations

- [x] 1.1 Update page-boundary continuation detection to accept an opening parenthesis followed by lowercase prose when existing style and layout checks agree; verify continuation and completed-sentence boundary cases.
- [x] 1.2 Add regression coverage using the reported Russian passage and assert the exact normalized text and single-paragraph published HTML.

## 2. Refresh cached output

- [x] 2.1 Increment the PDF processing revision so unchanged source PDFs are reprocessed; verify the cache fingerprint changes for an unchanged source fingerprint.

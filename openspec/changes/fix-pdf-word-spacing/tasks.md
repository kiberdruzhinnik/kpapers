# Tasks

## 1. Reconstruct word separators

- [x] 1.1 Replace unconditional spaces between PDF words with text and bounding-box based separator reconstruction; verify unit cases preserve real word gaps and join hyphenated compounds.
- [x] 1.2 Keep commas and closing quotation marks adjacent to preceding text across styled runs; verify extracted text and HTML, including italic spans.
- [x] 1.3 Add regression fixtures through the PDF word-line adapter for both reported Russian passages and assert exact normalized text and published HTML.

## 2. Refresh cached output

- [x] 2.1 Increment the PDF processing revision and verify the cache fingerprint changes for unchanged source fingerprints.
- [x] 2.2 Run the relevant PDF processing test suite and verify existing paragraph, heading, and emphasis behavior remains intact.

# Tasks

## 1. Sentence continuation boundary rule

- [x] 1.1 Add a shared paragraph-boundary decision for large vertical gaps that uses punctuation, lowercase continuation, alignment, and compatible font/style cues; verify focused unit cases cover the supplied comma-to-lowercase pattern and reject a completed sentence followed by a paragraph gap.
- [x] 1.2 Use the shared decision for extracted plain text and published HTML while preserving ordinary gaps, explicit paragraph breaks, and page-boundary handling; verify line-level regression cases produce matching paragraph grouping in both outputs.
- [x] 1.3 Add negative regression cases for mismatched alignment or styling and confirm headings and notice blocks retain their current grouping behavior in the relevant unit tests.

## 2. Cached output compatibility

- [x] 2.1 Increment the PDF extraction pipeline revision if the behavior changes cached extracted text or HTML, and verify the cache fingerprint changes for an unchanged source fingerprint.
- [x] 2.2 Run the focused PDF processing tests and the relevant repository validation checks; verify the sample continuation joins while genuine large-gap paragraph boundaries remain separate.

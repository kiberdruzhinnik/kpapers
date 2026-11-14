# Tasks

## 1. Recognize the honorific continuation

- [x] 1.1 Extend the shared large-gap continuation logic to recognize a line ending in the standalone abbreviation `ув.` followed by uppercase continuation text, while retaining compatible typography and alignment checks; verify with focused tests for both reported passages and an ordinary period boundary.
- [x] 1.2 Verify extracted text and generated HTML keep both reported continuations in single paragraphs, and preserve genuine paragraph boundaries, with the relevant PDF processing test suite.

## 2. Regenerate cached content

- [x] 2.1 Increment the PDF processing pipeline revision and verify cache fingerprint tests show prior extracted content is invalidated on refresh.

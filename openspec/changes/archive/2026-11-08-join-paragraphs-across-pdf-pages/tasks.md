# Tasks

## 1. Preserve page-boundary paragraph continuity

- [x] 1.1 Track page-start line context and implement conservative continuation detection using text and layout evidence; add regression coverage for the 2026-11-04 sentence and verify it renders as one paragraph.
- [x] 1.2 Apply the same boundary decision to extracted `full_text` and sanitized HTML; verify regression coverage preserves true paragraph boundaries across pages and within pages.

## 2. Refresh cached extraction output

- [x] 2.1 Increment the extraction pipeline revision and verify the cache fingerprint changes for the same source fingerprint.
- [x] 2.2 Run the PDF processing unit suite and verify the existing extraction, heading, page-label, and paragraph-boundary cases continue to pass.

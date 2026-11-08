# Proposal

## Why

When a prose paragraph continues across a PDF page boundary, the current extraction pipeline treats the page transition as a paragraph boundary. The 2026-11-04 PDF therefore splits the sentence «Вот такой-то псих с накаченной на него поверх системой ложных представлений о себе — это третий вариант возможного гражданина вашей утопии» between two published paragraphs.

## What Changes

- Preserve clear paragraph continuations across PDF page boundaries in extracted text and published HTML.
- Keep genuine paragraph boundaries at page transitions, and continue to preserve page labels and heading structure.
- Invalidate previously cached PDF output so affected documents are reprocessed with the corrected behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Preserve prose paragraphs that continue across page boundaries while retaining actual paragraph boundaries.

## Impact

- `src/pdf_processing.rs`: page joining and HTML paragraph assembly, plus the extraction pipeline cache revision.
- `openspec/specs/pdf-ingestion/spec.md`: paragraph joining behavior and scenarios.
- Existing cached extracted text and HTML will be regenerated when the cache revision changes.

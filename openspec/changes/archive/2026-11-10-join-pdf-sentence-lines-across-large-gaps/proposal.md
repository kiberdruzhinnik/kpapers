# Proposal

## Why

PDF extraction currently treats a sufficiently large vertical gap as a paragraph boundary even when the next line continues the same sentence. In the supplied 20240702 PDF, this splits a sentence after “зелёным спасителям,” and publishes the continuation as a separate paragraph.

## What Changes

- Preserve sentence continuations across large vertical gaps when the line content and layout provide sufficient evidence that the text is one paragraph.
- Keep genuine paragraph boundaries intact and cover both cases with regression examples.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Paragraph grouping must account for sentence continuation cues when a large visual gap would otherwise split prose.

## Impact

- Affects PDF line-to-paragraph grouping and the extracted HTML output in `src/pdf_processing.rs`.
- Requires regression coverage for the supplied page-6 sentence and for genuine paragraph boundaries; no API or dependency changes are expected.

# Proposal

## Why

PDF extraction currently splits a sentence at a page boundary in the supplied Russian article, leaving “потому что” and “любой вопрос...” in separate paragraphs. The article also contains visually bold text that is not represented as bold in the parsed output, reducing fidelity to the source.

## What Changes

- Keep prose continuous across a page boundary when text and layout indicate that the next page continues the preceding sentence, including the supplied case.
- Preserve bold emphasis in extracted and published PDF text, including when bold words occur within otherwise regular prose.
- Add regression coverage for the reported page transition and bold styling behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Clarify and enforce page-boundary paragraph continuation and rendering of bold text as semantic emphasis.

## Impact

The PDF text-line conversion and paragraph/HTML rendering path in `src/pdf_processing.rs`, its regression tests, and the `pdf-ingestion` behavior specification. The vendored `pdf_oxide` extraction result may need investigation if it does not provide reliable font-style metadata for the example.

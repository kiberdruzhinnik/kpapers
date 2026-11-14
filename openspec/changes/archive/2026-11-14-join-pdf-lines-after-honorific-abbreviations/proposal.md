# Proposal

## Why

The PDF paragraph classifier can split a wrapped sentence after the Russian honorific abbreviation `ув.` when the following line begins with a capitalized name or noun. In Miniflux this appears as an unintended paragraph gap in the RSS article, interrupting the sentence.

## What Changes

- Treat a line ending in `ув.` as a possible continuation when the next line begins with an uppercase character and the existing layout cues support continuity.
- Preserve genuine paragraph boundaries and avoid treating arbitrary sentence-ending periods as continuation signals.
- Cover both extracted text and published HTML, which RSS serves from the cached HTML.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Clarify that wrapped lines following the honorific abbreviation `ув.` remain in one paragraph when text and layout indicate continuation, including across large vertical gaps.

## Impact

- PDF paragraph classification and its extracted text/HTML output in `src/pdf_processing.rs`.
- RSS item content consumes the resulting cached HTML in `src/rss.rs`; no RSS API or schema change is expected.
- PDF ingestion specification and regression coverage for the two reported Russian passages.

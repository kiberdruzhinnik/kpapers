# Proposal

## Why

PDF text extraction currently inserts a space between every extracted word, which can alter correctly extracted Russian text by splitting compounds, separating punctuation, and adding whitespace before closing quotation marks. This produces visibly incorrect cached text and published HTML for otherwise valid PDFs.

## What Changes

- Reconstruct spaces between extracted PDF words using their text and layout instead of inserting a space unconditionally.
- Preserve punctuation attachment, hyphenated compounds, and quote boundaries while retaining genuine inter-word spaces.
- Add regression coverage for the reported passages in extracted text and published HTML.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Specify faithful reconstruction of word spacing and punctuation from PDF text runs.

## Impact

- Affects `src/pdf_processing.rs` where `pdf_oxide` word blocks are converted into styled lines.
- Changes the extracted text and safe HTML produced for affected PDFs. The pipeline revision should change so cached documents are reprocessed.
- No API or dependency changes are expected.

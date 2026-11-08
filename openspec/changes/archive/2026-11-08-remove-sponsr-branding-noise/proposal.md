# Proposal

## Why

Some processed PDFs expose Sponsr branding as stray body paragraphs in the published feed, including `Кримсон Аналитика – CrimsonAnalytics`. This publication chrome is not part of the article and reduces feed readability; the existing first-page cleanup does not consistently account for this branding form or placement.

## What Changes

- Recognize and omit the reported Sponsr branding from extracted text and published HTML wherever it appears as standalone publication chrome.
- Preserve article text that merely mentions the same words as part of a sentence or a larger content block.
- Add regression coverage for the reported PDF and for preservation of legitimate article mentions.
- Advance the PDF extraction cache revision so previously cached entries are regenerated with the cleanup.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Require omission of the reported standalone Sponsr brand line from extracted and published content without removing ordinary article mentions.

## Impact

The PDF extraction and normalization path in `src/pdf_processing.rs`, the extraction cache fingerprint, and regression fixtures or tests for PDF ingestion. RSS output changes only for entries containing the recognized standalone branding line; no API or database schema change is expected.

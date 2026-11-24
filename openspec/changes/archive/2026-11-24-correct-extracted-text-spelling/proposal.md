# Proposal

## Why

PDF text extraction sometimes produces misspellings and damaged word forms, while nearly all source text is Russian or English. Correcting only reliable errors improves feed readability while keeping the original wording visible beside every correction.

## What Changes

- Add entirely local Russian and English correction to the PDF text-processing pipeline.
- Automatically apply a correction only when evaluation demonstrates at least 85% precision for that correction class, or when that exact extracted form and replacement were explicitly reviewed and approved; otherwise preserve the extracted text unchanged.
- Include the original text inline beside every applied correction in the published content, using a language-appropriate annotation.
- Retain the original form in the annotation so readers can inspect what was changed.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Correct high-confidence Russian and English extraction errors locally and annotate corrections in published text.

## Impact

The PDF processing pipeline and its cache fingerprint will change. Cached documents will need reprocessing so corrected text and annotations appear in RSS and exports. The correction implementation and dictionaries must run locally and must not send extracted content to external services.

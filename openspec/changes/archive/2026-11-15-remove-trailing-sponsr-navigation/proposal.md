# Proposal

## Why

PDF extraction has two cleanup defects: Sponsr PDFs can append navigation and related-article material after the investment disclaimer, and line-wrapped hyphenated words can gain an unwanted space in extracted text and feed HTML. Both leave publication chrome or malformed prose in otherwise readable article content.

## What Changes

- Preserve the investment disclaimer and remove the trailing Sponsr navigation and related-content material that follows it.
- Preserve hyphenated compounds when a PDF line break separates the hyphen from the rest of a word, including the reported `объяснения-` / `мастерключа` wrap.
- Keep the existing handling of dedicated Skladchik advertisement pages.
- Invalidate cached extracted content so existing documents receive both processing fixes on their next extraction.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Omit post-disclaimer Sponsr navigation and related-content material, and preserve hyphenated compounds across line breaks in extracted text and HTML.

## Impact

The PDF post-processing pipeline and its cache revision in `src/pdf_processing.rs`, plus the `pdf-ingestion` specification and extraction regression coverage. Regression coverage will reproduce the reported line wrap through the relevant extraction path. No API or database schema change is expected.

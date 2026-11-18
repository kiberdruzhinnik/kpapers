# Proposal

## Why

The supplied PDF uses line-end hyphens to split `пристрастились` and `неискушенными`; retaining those glyphs in reconstructed text leaves the words incorrectly hyphenated. Removing the discretionary hyphens restores the intended text in extracted content and published HTML.

## What Changes

- Ensure extracted PDF text and published HTML remove discretionary line-ending hyphens when joining lowercase continuations.
- Add regression coverage for both reported words from the supplied PDF and retain ordinary wrapped-word spacing.
- Bump the processing revision so cached extractions are regenerated with the corrected text.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Require removal of discretionary line-end hyphens while retaining in-line hyphenation.

## Impact

- Affects PDF line reconstruction, cache identity, and HTML generation in `src/pdf_processing.rs`.
- Adds regression coverage to the existing PDF processing tests; no API or dependency changes are expected.

# Proposal

## Why

PDF text extraction can lose conventional spaces at word boundaries when PDF geometry reports a narrow gap, including before parenthetical phrases, between Latin and Cyrillic text, between letters and digits, and around quoted phrases. It also mishandles some CrimsonAnalytics publication headers and long wrapped headlines, leaving header text in the article and failing to emit the headline as an `<h1>`.

## What Changes

- Reconstruct conventional spaces at prose word boundaries when geometry alone misses the separator, including before opening parentheses and guillemets, after closing quotes, and between different scripts or letters and digits.
- Preserve lexical hyphens in compounds split across visual lines.
- Recognize the supplied CrimsonAnalytics publication header and remove its branding and publication metadata before the article title.
- Preserve a long wrapped article title as one semantic `<h1>` while keeping its lead and article body separate.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Require conventional prose spacing across parenthetical, quoted, script, and letter-digit boundaries; preservation of lexical hyphens across line wraps; and correct cleanup and semantic rendering of long wrapped titles from the CrimsonAnalytics publication header.

## Impact

PDF line reconstruction, first-page publication-header cleanup, heading detection, and regression coverage in `src/pdf_processing.rs`; no API or dependency changes are expected.

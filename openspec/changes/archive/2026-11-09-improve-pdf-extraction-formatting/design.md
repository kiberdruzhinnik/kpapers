# Design

## Context

See proposal.md for motivation and `specs/pdf-ingestion/spec.md` for the behavior contract. PDF extraction already retains per-line font size, bold/italic runs, vertical gaps, and horizontal position. Heading detection currently takes the smallest line size as its body baseline and treats nearly every short line above that threshold as a heading candidate. Page-label removal currently checks only the first non-empty line of a page.

## Goals / Non-Goals

**Goals:**
- Use page identity and line content to remove standalone matching page labels independently of extraction order.
- Reduce false heading positives while preserving titles, real section hierarchy, wrapped headings, and inline emphasis.
- Ensure cached extraction is regenerated when formatting behavior changes.

**Non-Goals:**
- OCR or recovery of text that is absent from the PDF text layer.
- General document classification or language-specific NLP to decide whether a sentence is a heading.
- Changes to RSS endpoints, sanitization policy, or provider ingestion.

## Decisions

### Identify page labels from each page's lines

Compare each standalone numeric line with that page's one-based position among the pages remaining after leading boundary advertisement pages are removed, and remove a match wherever it occurs in the extracted line sequence. Keep text that shares a line with the number and standalone numbers that do not match the content-page position. Defer label removal until after boundary advertisement pages are removed so a printed sequence that starts at `1` on the first content page continues to match even when an unnumbered advertisement or cover page precedes it.

Alternative considered: remove numeric lines only at the beginning or end of extracted page text. This is simpler but misses labels when PDFium returns a footer out of visual order or mixes it into the text sequence. Comparing against the original source page index keeps removal narrower but misses documents with a leading unnumbered advertisement page, as in the supplied article, so content-page position is the reliable identity. A genuinely meaningful standalone number equal to its content-page position remains a risk and should be covered in review against representative documents.

### Establish a representative body-text baseline and require corroborating heading cues

Use the dominant body text-size cluster rather than the smallest line size as the baseline. Treat size as one cue, and consider available emphasis and layout evidence such as bold styling, isolation from surrounding paragraphs, indentation/alignment, and hierarchy relative to other heading candidates. A short line or a larger font by itself is insufficient. Keep body paragraphs intact and preserve their bold/italic runs. Keep wrapped-line merging limited to adjacent lines that share heading-like typography and compatible layout.

Alternative considered: classify every line above a fixed size ratio as a heading. The existing behavior demonstrates that this over-promotes prose, especially when small labels lower the baseline. Language-specific sentence classification was rejected because it adds fragility and is unnecessary for the supplied examples.

### Invalidate cached output when the formatter changes

Increment the extraction pipeline revision used in the cache fingerprint so existing documents are reprocessed and their stored text and HTML reflect the new rules. No database schema change is needed.

Alternative considered: leave old cached entries until source files change. That would make the fix appear inconsistent and would require operators to force a retry per document.

## Risks / Trade-offs

- [A meaningful standalone number equals its PDF page index] -> Remove only exact, standalone matches; inspect representative output and retain any evidence that disambiguates page furniture from body content.
- [A genuinely short, visually understated heading is treated as prose] -> Use multiple cues rather than a single strict threshold, and include title, section, subsection, and ordinary-prose fixtures in regression coverage.
- [Pipeline revision causes a one-time reprocessing load] -> Reuse the existing bounded extraction path and verify cache invalidation is limited to the revision change.

## Migration Plan

Deploy the formatter and increment its pipeline revision. Existing source fingerprints will then produce new cache fingerprints on refresh or extraction retry; no persistent data migration is required. Rollback consists of restoring the previous application version, which will use its prior revision and cached-output behavior.

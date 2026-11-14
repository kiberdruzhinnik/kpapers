# Design

## Context

See proposal.md for the user-visible problem. `large_gap_starts_paragraph` currently recognizes large-gap sentence continuations only when the prior line ends in a comma, semicolon, or colon and the next line begins lowercase. Both text and HTML paragraph assembly use this classifier. RSS publishes the cached HTML. Cache identity includes `EXTRACTION_PIPELINE_REVISION`.

## Goals / Non-Goals

**Goals:**
- Preserve the two reported wrapped sentences as single paragraphs in extracted text and RSS HTML.
- Keep the continuation rule narrow and require the existing compatible text layout cues.
- Ensure cached content is regenerated under the updated paragraph logic.

**Non-Goals:**
- General sentence reconstruction after arbitrary periods.
- Changing RSS markup or Miniflux rendering.
- Broad changes to paragraph-gap thresholds or PDF heading detection.

## Decisions

- Extend the shared large-gap continuation predicate to recognize a prior line ending in the standalone honorific abbreviation `ув.` followed by an uppercase character on the next line. Retain compatible font size, emphasis style, and horizontal alignment checks; this handles the capitalized continuation while limiting false joins.
  - **Alternative considered:** Treat any period followed by uppercase text as a continuation. Rejected because it can merge real sentence and paragraph boundaries.
  - **Alternative considered:** Lower the large-gap threshold globally. Rejected because it would affect unrelated paragraph boundaries throughout PDFs.
- Use the shared predicate so extracted text and generated HTML apply the same paragraph decision.
- Increment the PDF processing pipeline revision so the next refresh regenerates cached text and HTML for documents processed under the old rule.

## Risks / Trade-offs

- [A genuine new paragraph immediately following `ув.` could be joined when typography and alignment also match] → Keep the abbreviation-specific rule dependent on compatible size, style, and alignment, and cover ordinary period boundaries with regression cases.
- [Existing feeds may keep old paragraph markup until refreshed] → Change the processing revision so the next refresh bypasses the old extraction cache.

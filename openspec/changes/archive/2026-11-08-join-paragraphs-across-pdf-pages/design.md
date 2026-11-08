# Design

## Context

See `proposal.md` for the reported PDF and motivation. `extract_pdf` currently joins pages with a blank line for `full_text`, while `lines_to_html` flushes the current paragraph unconditionally at each page boundary. The line model retains text, font size, style, and horizontal position, but its vertical gap is measured only within a page.

## Goals / Non-Goals

**Goals:**
- Preserve a paragraph across a page boundary when neighboring text and layout indicate that the sentence continues.
- Keep actual paragraph starts, headings, and page separation behavior intact when the boundary represents a real break.
- Ensure previously cached output is regenerated after the extraction behavior changes.

**Non-Goals:**
- Reconstruct arbitrary PDF reading order or infer paragraph structure across columns, tables, or unrelated blocks.
- Change heading, emphasis, advertisement, or Sponsr preamble handling.

## Decisions

### Detect continuation using content and layout evidence

Mark page-start lines so the HTML renderer can evaluate a page transition without treating it as an ordinary within-page line gap. Continue an open body paragraph only when the boundary is a strong continuation: the prior body line is incomplete, the next line begins as a continuation rather than as a heading, and font/style and horizontal position are compatible. Otherwise close the paragraph at the page boundary. Keep heading recognition independent and do not join heading blocks to body paragraphs.

This conservative rule addresses the supplied case, where page 6 ends with “это третий вариант” and page 7 starts with the lowercase continuation “возможного гражданина…”. It avoids blindly merging paragraphs whenever a page ends. An unconditional carry-over was considered but rejected because a page can end after a complete paragraph and the next page can start a new one.

### Use the same boundary decision for extracted text and HTML

Build page-separated extracted text from the same paragraph-boundary decisions used by HTML, so `full_text` and published content do not disagree about whether a paragraph continued. Page labels remain stripped before joining. Preserve paragraph separation where the boundary is classified as a real break.

### Invalidate cached output with the extraction revision

Increment `EXTRACTION_PIPELINE_REVISION`; the processed-content fingerprint includes this value, so the next normal or forced extraction refreshes cached text and HTML without a database migration. This follows the existing cache invalidation mechanism.

## Risks / Trade-offs

- [Conservative signals can miss continuations that start with a capital letter or change typography] → Use multiple available text and layout signals, and cover the supplied lowercase continuation as a regression case; avoid a blanket page merge.
- [A page boundary can coincide with a genuine paragraph break that resembles a continuation] → Require compatible layout and incomplete prior text, and retain a separate-paragraph scenario in regression coverage.
- [Revision invalidation causes PDFs to be processed again] → Reuse the existing extraction/cache flow; no source files need to be downloaded until normal refresh processing detects the revised pipeline fingerprint.

## Migration Plan

Increment the extraction pipeline revision and rely on the existing fingerprint comparison to rebuild cached text and HTML on the next extraction refresh. A forced document refresh remains available for immediate reprocessing. Rolling back the code restores the previous revision behavior; newly generated cache rows will be refreshed again when the earlier revision is used.

# Design

## Context

See proposal.md for the motivation and spec delta for observable behavior. The current `pdf-extract` path returns plain text per page, which loses font and layout information needed to identify headings and emphasis. Normalization preserves line breaks, and cached output is reused when its stored fingerprint equals the source fingerprint.

## Goals / Non-Goals

**Goals:**
- Clean extracted pages before joining them into the cached text and safe HTML.
- Ensure the new cleanup is applied to previously cached PDFs without requiring operators to manually locate each document.
- Preserve ordinary content, page boundaries, and paragraph boundaries.
- Preserve document hierarchy and bold/italic emphasis as semantic HTML.

**Non-Goals:**
- Modify source PDF files or provider storage.
- Remove promotional content from anywhere other than a recognized final page.
- Reformat lists or other distinct content beyond joining wrapped lines within paragraphs.
- Infer styling for image-only text or reconstruct arbitrary visual design such as color and font family.

## Decisions

- Process extracted text page by page. Use a normalized copy of the final page to recognize the Skladchik ad from its distinctive domain and prompt text, and omit that page only when the recognition signals are present together. Keep the original PDF page count as the source page count; the extracted text and HTML omit the ad page.
- Strip a page label only from the first non-empty line of a page, and only when that line contains the page's one-based position in the original PDF as a standalone integer. This avoids deleting numbers from article content and keeps page indexing stable when an ad page is removed.
- Treat blank lines as paragraph boundaries and join ordinary line breaks inside each paragraph with spaces. Keep pages separated by paragraph boundaries before generating safe HTML, so the end of one page is not accidentally joined to the start of the next.
- Replace the flattened-text extraction path with `pdfium-render` character-level extraction. Retain each character's Unicode value, position, font size, font name, weight, and italic metadata, then group characters into text runs and lines in application code. This avoids implementing PDF font decoding and content-stream interpretation ourselves.
- Infer bold from font weight/style metadata and italic from the font's italic metadata; do not infer emphasis from the words themselves. When metadata is missing or ambiguous, preserve the text without adding emphasis markup.
- Classify heading lines using relative font size, placement, and spacing in the document. Join visual continuation lines belonging to the same logical heading with spaces before assigning its level: `<h1>` for the document title, `<h2>` for top-level headlines, and `<h3>` for nested headings. Do not emit `<h4>`; candidates below `<h3>` remain paragraph content. Keep ordinary or uncertain text as paragraphs and retain existing line reflow within paragraph bodies.
- Before HTML serialization, coalesce adjacent text runs with identical bold and italic styles within paragraphs and headings. When a whitespace-only run separates same-style runs, fold that whitespace into the styled span so date and time such as `06.11.2026 12:12` share one `<strong>` element. Emit bold and italic text as `<strong>` and `<em>`, nesting both tags when both styles apply; preserve style transitions and block boundaries. Continue escaping text and allow only the semantic tags supported by the safe HTML output.
- Pin the `pdfium-render` crate and matching Pdfium runtime binary version. Add the native Pdfium library to the API container runtime image and verify the application starts and extracts text in the container. Treat the Pdfium binary as an explicit runtime dependency that must be updated with security and compatibility fixes.
- Advance the extraction-pipeline revision from `pdf-text-v2` to `pdf-text-v3` in the cached-content fingerprint comparison and stored cache key. Existing v2 cache rows will miss once and be regenerated; subsequent unchanged refreshes can reuse v3 output. This avoids a database migration or manual force refresh.

## Risks / Trade-offs

- [Ad text extraction differs across PDFs] → Normalize case, whitespace, and `ё`/`е` for recognition, require the distinctive signals together, and check only the final page.
- [A genuine opening line is a number matching the page index] → Remove it only when it is the first non-empty line and exactly matches that page's one-based index.
- [Overjoining structured content] → Preserve blank-line boundaries and test headings, lists, and prose wrapping with representative extracted pages.
- [Wrapped title lines are misclassified as separate headings] → Use typography and layout continuity to group title lines before assigning heading levels, and verify a multi-line title remains one `<h1>`.
- [Fragmented extraction runs create noisy emphasis markup] → Coalesce adjacent runs with identical styles within paragraphs and headings, folding whitespace-only separators into the styled span while preserving block boundaries and nested style changes.
- [Wrapped headlines and lower-level headings are misrepresented] → Group visual continuation lines before assigning levels, cap semantic heading output at `<h3>`, and retain lower-level candidates as paragraphs.
- [Pipeline revision is not advanced when cleanup behavior changes later] → Keep the revision adjacent to the processing logic and include a task to verify cache invalidation for the existing cache format.
- [Embedded font metadata is missing or inconsistent] → Fall back to unmarked text for uncertain emphasis and use typography and layout together for heading detection; cover common font descriptors in fixtures.
- [Relative typography heuristics misclassify body text as headings] → Require standalone-line and layout evidence, test title and nested heading fixtures, and retain ordinary prose as paragraphs when evidence is weak.
- [Character order differs from visual reading order on complex pages] → Group characters by geometry and validate ordering against representative multi-column and wrapped-text fixtures; keep the supported reading-order assumptions documented.
- [Pdfium runtime library is missing or mismatched in deployment] → Pin the crate and binary versions together, package the binary in the runtime image, and verify startup and extraction in the container build.

## Migration Plan

No schema migration is needed. On the first extraction after deployment, the v3 pipeline revision makes existing v2 cached fingerprints miss, so normal processing rebuilds extracted text and HTML with the updated heading and emphasis markup. Subsequent unchanged refreshes reuse the v3 cache. Rollback restores the prior pipeline revision; rows written by v3 will be reprocessed by the old code rather than reused as matching fingerprints.

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
- Classify standalone heading lines using relative font size, placement, and spacing in the document. Assign `<h1>` to the document title, `<h2>` to top-level sections, and `<h3>`/`<h4>` to nested levels. PDFs may not contain semantic heading labels, so keep ordinary or uncertain text as paragraphs and retain the existing line reflow within paragraph bodies.
- Emit detected bold and italic runs as `<strong>` and `<em>`, nesting both tags when both styles apply. Continue escaping text and allow only the semantic tags supported by the safe HTML output.
- Pin the `pdfium-render` crate and matching Pdfium runtime binary version. Add the native Pdfium library to the API container runtime image and verify the application starts and extracts text in the container. Treat the Pdfium binary as an explicit runtime dependency that must be updated with security and compatibility fixes.
- Advance the extraction-pipeline revision from `pdf-text-v1` to `pdf-text-v2` in the cached-content fingerprint comparison and stored cache key. Existing v1 cache rows will miss once and be regenerated; subsequent unchanged refreshes can reuse v2 output. This avoids a database migration or manual force refresh.

## Risks / Trade-offs

- [Ad text extraction differs across PDFs] → Normalize case, whitespace, and `ё`/`е` for recognition, require the distinctive signals together, and check only the final page.
- [A genuine opening line is a number matching the page index] → Remove it only when it is the first non-empty line and exactly matches that page's one-based index.
- [Overjoining structured content] → Preserve blank-line boundaries and test headings, lists, and prose wrapping with representative extracted pages.
- [Pipeline revision is not advanced when cleanup behavior changes later] → Keep the revision adjacent to the processing logic and include a task to verify cache invalidation for the existing cache format.
- [Embedded font metadata is missing or inconsistent] → Fall back to unmarked text for uncertain emphasis and use typography and layout together for heading detection; cover common font descriptors in fixtures.
- [Relative typography heuristics misclassify body text as headings] → Require standalone-line and layout evidence, test title and nested heading fixtures, and retain ordinary prose as paragraphs when evidence is weak.
- [Character order differs from visual reading order on complex pages] → Group characters by geometry and validate ordering against representative multi-column and wrapped-text fixtures; keep the supported reading-order assumptions documented.
- [Pdfium runtime library is missing or mismatched in deployment] → Pin the crate and binary versions together, package the binary in the runtime image, and verify startup and extraction in the container build.

## Migration Plan

No schema migration is needed. On the first extraction after deployment, the v2 pipeline revision makes existing cached fingerprints miss, so normal processing rebuilds extracted text and HTML with semantic typography. Subsequent unchanged refreshes reuse the v2 cache. Rollback restores the prior pipeline revision; rows written by v2 will be reprocessed by the old code rather than reused as matching fingerprints.

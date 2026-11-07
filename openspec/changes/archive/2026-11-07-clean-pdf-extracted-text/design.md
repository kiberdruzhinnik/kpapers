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
- Remove promotional content only from a recognized first or final page.
- Reformat lists or other distinct content beyond joining wrapped lines within paragraphs.
- Infer styling for image-only text or reconstruct arbitrary visual design such as color and font family.

## Decisions

- Process extracted text page by page. Recognize the existing Club advertisement by its distinctive domain and prompt text, and recognize the first-page advertisement when the first non-empty text begins with “куплено на” and the same page contains `skladchik.ORG`. Apply these narrow patterns only to the first and final PDF pages, preserving other boundary pages and all interior pages. Keep the original PDF page count as the source page count; extracted text and HTML omit recognized ad pages.
- For documents using the Sponsr publication layout, identify the masthead and the article headline that follows the byline and publication metadata. Locate the headline as a logical heading, including when it wraps across multiple visual lines, rather than relying on a single-line length threshold. Omit preceding metadata such as localized date/time strings and reading durations when the title is identifiable. If the first page contains a publication timestamp and its text layer omits an image-only `[Философское]` headline, use the corresponding identifiable filename title, remove the first metadata heading, and preserve the remaining article body. Preserve leading text when the layout and title are ambiguous. Filter XML 1.0 forbidden control characters before serialization. The regression fixture SHALL assert the exact first heading and absence of date/duration metadata.
- Strip a page label only from the first non-empty line of a page, and only when that line contains the page's one-based position in the original PDF as a standalone integer. This avoids deleting numbers from article content and keeps page indexing stable when an ad page is removed.
- Treat blank lines as paragraph boundaries and join ordinary line breaks inside each paragraph with spaces. Keep pages separated by paragraph boundaries before generating safe HTML, so the end of one page is not accidentally joined to the start of the next.
- Replace the flattened-text extraction path with `pdfium-render` character-level extraction. Retain each character's Unicode value, position, font size, font name, weight, and italic metadata, then group characters into text runs and lines in application code. This avoids implementing PDF font decoding and content-stream interpretation ourselves.
- Infer bold from font weight/style metadata and italic from the font's italic metadata; do not infer emphasis from the words themselves. When metadata is missing or ambiguous, preserve the text without adding emphasis markup.
- Classify heading lines using relative font size, placement, and spacing in the document. Join visual continuation lines belonging to the same logical heading with spaces before assigning its level: `<h1>` for the document title, `<h2>` for top-level headlines, and `<h3>` for nested headings. Do not emit `<h4>`; candidates below `<h3>` remain paragraph content. Keep ordinary or uncertain text as paragraphs and retain existing line reflow within paragraph bodies.
- For Sponsr materials whose image-only `[Философское]` headline is recovered from the filename, treat that headline as the authoritative first `<h1>`. Classify later blocks independently: font size alone is insufficient to promote body text to `<h2>` or `<h3>`. Keep the bold disclaimer in its paragraph, retain ordinary lead and article text as paragraphs, and preserve the indented notice as one blockquote. Only assign later heading levels when standalone typography and layout provide heading evidence.
- Before HTML serialization, coalesce adjacent text runs with identical bold and italic styles within paragraphs and headings. When a whitespace-only run separates same-style runs, fold that whitespace into the styled span so date and time such as `06.11.2026 12:12` share one `<strong>` element. Emit bold and italic text as `<strong>` and `<em>`, nesting both tags when both styles apply; preserve style transitions and block boundaries. Continue escaping text and allow only the semantic tags supported by the safe HTML output.
- Parse matched literal Markdown `**...**` spans in extracted text before HTML escaping and emit their contents as `<strong>`. Merge this explicit emphasis with PDF-derived font emphasis so the output contains semantic markup without literal delimiter characters or redundant nested `<strong>` tags; leave unmatched delimiter characters as escaped text.
- Use the left positions of text lines together with the “Традиционные технические предуведомления:” marker to identify the associated indented notice region. Emit the heading and its visually grouped contents inside one `<blockquote>`, preserving paragraphs, headings, and inline styles, and close the blockquote when text returns to the ordinary body margin.
- Pin the `pdfium-render` crate and matching Pdfium runtime binary version. Add the native Pdfium library to the API container runtime image and verify the application starts and extracts text in the container. Treat the Pdfium binary as an explicit runtime dependency that must be updated with security and compatibility fixes.
- Advance the extraction-pipeline revision from `pdf-text-v8` to `pdf-text-v9` in the cached-content fingerprint comparison and stored cache key. Existing v8 cache rows will miss once and be regenerated; subsequent unchanged refreshes can reuse v9 output. This avoids a database migration or manual force refresh.

## Risks / Trade-offs

- [Ad text extraction differs across PDFs] → Normalize case, whitespace, and `ё`/`е` for recognition, require each pattern's distinctive signals together, and check only the first and final pages.
- [A boundary-page ad is missed or article text is removed] → Recognize the first-page “куплено на” plus domain pattern and existing Club-ad pattern only on the first or final page; preserve pages that do not match a complete pattern.
- [Sponsr preamble cleanup removes article content] → Require both the recognizable Sponsr masthead and a distinct article title, remove only preceding publication chrome, and preserve all content when either signal is ambiguous.
- [Sponsr date or duration leaks into published text] → Test the exact localized `27 февр 11:44` and `58 минут` strings and require the headline to be the first HTML block when the Sponsr layout is identifiable.
- [A genuine opening line is a number matching the page index] → Remove it only when it is the first non-empty line and exactly matches that page's one-based index.
- [Overjoining structured content] → Preserve blank-line boundaries and test headings, lists, and prose wrapping with representative extracted pages.
- [Wrapped title lines are misclassified as separate headings] → Use typography and layout continuity to group title lines before assigning heading levels, and verify a multi-line title remains one `<h1>`.
- [Fragmented extraction runs create noisy emphasis markup] → Coalesce adjacent runs with identical styles within paragraphs and headings, folding whitespace-only separators into the styled span while preserving block boundaries and nested style changes.
- [Markdown markers become visible or duplicate font emphasis] → Parse only matched `**...**` spans before escaping, merge marker-derived and font-derived bold state, and preserve unmatched marker characters as text.
- [Indented notice content is grouped incorrectly] → Require the known notice heading and consistent left-position indentation, and end the blockquote when the text returns to the body margin; test the heading, multiple notice paragraphs, and following body text.
- [Wrapped headlines and lower-level headings are misrepresented] → Group visual continuation lines before assigning levels, cap semantic heading output at `<h3>`, and retain lower-level candidates as paragraphs.
- [Pipeline revision is not advanced when cleanup behavior changes later] → Keep the revision adjacent to the processing logic and include a task to verify cache invalidation for the existing cache format.
- [Embedded font metadata is missing or inconsistent] → Fall back to unmarked text for uncertain emphasis and use typography and layout together for heading detection; cover common font descriptors in fixtures.
- [Relative typography heuristics misclassify body text as headings] → Require standalone-line and layout evidence, test title and nested heading fixtures, and retain ordinary prose as paragraphs when evidence is weak.
- [Sponsr body text inherits heading tags from relative font-size heuristics] → Treat a recovered filename headline as the authoritative `<h1>`, classify following blocks independently, and test the bold disclaimer, lead paragraph, quote block, and subsequent body text against the supplied layout.
- [Character order differs from visual reading order on complex pages] → Group characters by geometry and validate ordering against representative multi-column and wrapped-text fixtures; keep the supported reading-order assumptions documented.
- [Pdfium runtime library is missing or mismatched in deployment] → Pin the crate and binary versions together, package the binary in the runtime image, and verify startup and extraction in the container build.

## Migration Plan

No schema migration is needed. On the first extraction after deployment, the v9 pipeline revision makes existing v8 cached fingerprints miss, so normal processing rebuilds extracted text and HTML with the corrected Sponsr block layout, explicit Markdown emphasis, and blockquote detection. Subsequent unchanged refreshes reuse the v9 cache. Rollback restores the prior pipeline revision; rows written by v9 will be reprocessed by the old code rather than reused as matching fingerprints.

# Design

## Context

See proposal.md for the reported feed behavior. PDF text is assembled page by page into positioned lines in `src/pdf_processing.rs`. Existing Sponsr cleanup is focused on the first page and filename-title repair; the latter removes only leading blocks containing `crimsonalter`. Cached extracted text and HTML share a versioned processing fingerprint, currently `pdf-text-v9`.

## Goals / Non-Goals

**Goals:**
- Remove the reported branding line before both plain-text and HTML output are assembled.
- Keep cleanup narrow enough to preserve article mentions and unrelated recurring headers or footers.
- Regenerate previously cached content after the behavior changes.

**Non-Goals:**
- General header/footer detection or removal of other publication branding.
- Changing article layout, promotion classification, feed APIs, or database schema.

## Decisions

- Normalize each extracted line for comparison by trimming, collapsing whitespace, lowercasing, and normalizing dash variants; remove only a full-line match for `Кримсон Аналитика – CrimsonAnalytics`. Apply this check to every page after line extraction and before deriving `first_page`, `full_text`, and HTML. This catches a standalone paragraph even when PDF character runs split the text or vary spacing, while a sentence containing the phrase does not match. A broader heuristic based on repeated position or font would risk removing real article content and is not needed for this known defect.
- Keep the existing Sponsr masthead and title-repair behavior separate. The reported brand line should be filtered as a standalone line regardless of page position, while title recovery remains limited to its existing filename-based case.
- Add a regression using the supplied PDF if a native extraction runtime is available in the implementation environment. Also add deterministic line-level coverage for removal from a later page, preservation of neighboring lines and article sentences, and agreement between plain text and HTML. If the attachment cannot be kept as a repository fixture, create the smallest representative fixture from its extracted page text and retain a document-level regression through the project’s existing fixture path.
- Advance the processing cache fingerprint from `pdf-text-v9` to `pdf-text-v10`. This makes the regular refresh/extraction path rebuild cached outputs without a database migration or manual per-document repair.

## Risks / Trade-offs

- [Typography or Unicode extraction changes the spelling or punctuation of the brand line] → Normalize whitespace and dash variants for exact full-line matching, and base regression coverage on the actual PDF output when available.
- [An article contains the branding line as an intentionally standalone mention] → The current specification treats the exact standalone line as publication chrome; preserve occurrences embedded in larger article blocks and keep this limitation explicit.
- [A new fingerprint causes all existing cached PDFs to be reprocessed once] → Use the existing scheduled refresh/cache-miss behavior; subsequent unchanged refreshes reuse the new version.

## Migration Plan

No database migration is required. Deploying `pdf-text-v10` causes cached v9 fingerprints to miss and be rebuilt through normal ingestion. Rollback to v9 causes v10 entries to miss and be reprocessed by the previous pipeline.

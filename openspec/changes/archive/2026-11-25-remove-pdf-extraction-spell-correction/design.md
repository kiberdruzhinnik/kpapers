# Design

## Context

See `proposal.md` for motivation and `specs/pdf-ingestion/spec.md` for the behavior being removed. The correction pass currently runs during PDF finalization, before text, HTML, and Markdown are generated. Published RSS and Markdown exports consume cached HTML. PDF cache identity uses a monotonically versioned processing revision; the current revision is `35`.

## Goals / Non-Goals

**Goals:**

- Return PDF extraction to preserving the source text without automatic spelling edits or correction annotations.
- Remove correction-only runtime and distribution assets.
- Regenerate cached output that may contain annotations.

**Non-Goals:**

- Change general PDF text reconstruction, line joining, or lexical-hyphen handling.
- Change browser form spell-check settings or vendored `pdf_oxide` form-field APIs.
- Remove or rewrite unrelated extraction, publication, or caching behavior.

## Decisions

1. **Remove correction-specific processing only.** Delete the `spell_correction` module and its invocation. Remove the exact `по-` plus `своему,` reconstruction that inserts a correction annotation, while preserving the general PDF line-wrap and hyphen reconstruction logic.

2. **Remove correction-only package and asset wiring.** Remove `spellbook` from the application and fuzz manifests and lockfiles, the bundled Hunspell data and notices, correction documentation and fixtures, correction-only tests, fuzz target module imports, and Docker copies of `assets/`. The repository has no other source references to those bundled assets. Leave similarly named browser and vendored-library features alone.

3. **Advance the processing revision from 35 to 36.** Do not reset it to an older revision: `35` identifies cached output already produced with corrections, and a fresh value ensures those entries are regenerated rather than mistaken for output from a previous pipeline. The existing general requirement to reprocess on pipeline revision changes remains the cache contract.

4. **Let refresh semantics govern stale output.** On the next successful refresh, the new revision causes extraction and publication to be rebuilt without annotations. If the PDF cannot be fetched or processed, existing behavior preserves the last successful cached entry, so its previous annotations can remain until a later successful refresh.

## Risks / Trade-offs

- [A failed refresh can leave previously corrected content visible] → Preserve the existing last-known-good cache behavior; the new revision regenerates the entry when a refresh succeeds.
- [Removing the exact-form line-wrap repair may change reconstruction for that particular source layout] → Remove only the correction-specific special case and retain general line-wrap and hyphen handling.
- [Dictionary removal touches large tracked assets and two dependency lockfiles] → Verify both manifests and lockfiles no longer include `spellbook`, and ensure the container builds no longer need the removed assets.

## Migration Plan

Remove the correction stage and correction-only resources, then increment the PDF processing revision to `36`. No database migration is needed. Cached entries are rebuilt from their source PDFs on the next successful refresh; failed refreshes continue to retain the last successful content.

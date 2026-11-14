# Design

## Context

See proposal.md for motivation and the spec deltas for the content contract. PDF extraction already emits semantic `<h1>` through `<h3>` headings and `<p>` paragraphs, and the RSS renderer copies cached HTML into both `description` and `content:encoded`. The Miniflux end-to-end test checks the RSS markup, but its reader API assertion currently checks only that article text is present. The supplied screenshots therefore do not yet identify whether block structure is lost during extraction, RSS serialization, Miniflux ingestion, or rendering.

## Goals / Non-Goals

**Goals:**
- Keep document title, lead, and body paragraphs as distinct semantic blocks from extraction through Miniflux ingestion.
- Locate the stage where structure is lost before choosing the narrowest correction.
- Reuse existing PDF extraction and Miniflux end-to-end test infrastructure.

**Non-Goals:**
- Change article wording, feed item titles, or PDF download links.
- Introduce reader-specific presentation styles or change Miniflux itself.
- Change the safe HTML tag allowlist.

## Decisions

### Trace the HTML through the existing publication path

Use a regression case with a visually distinct title, lead paragraph, and body paragraphs. Compare the generated/cached HTML, the parsed RSS `description` and `content:encoded`, and the Miniflux entry API `content`. This identifies whether the defect is in PDF block classification, RSS serialization, or Miniflux ingestion. Correct the first stage that drops or flattens the required blocks. If the Miniflux API retains the blocks but the UI combines them, record that as a reader presentation issue rather than weakening the feed content contract.

Alternative considered: Change RSS serialization immediately. The current code already publishes the cached HTML verbatim in both RSS content fields, so a serializer-only change without checking those boundaries may not address the observed behavior.

### Cover both extraction and reader-visible output

Add a focused PDF extraction regression for a title followed by a lead and body paragraphs, then extend the Miniflux end-to-end assertions to verify those elements survive ingestion. The current end-to-end fixture already checks safe RSS tags and semantic heading markup; strengthen its reader API check to assert block structure as well as text.

Alternative considered: Test only the XML. That would not detect a parser or ingestion transformation between RSS and Miniflux's stored entry content.

### Reprocess cached extraction when extraction output changes

If the correction changes extraction or formatting output, increment the existing extraction pipeline revision so previously cached HTML is regenerated on refresh or extraction retry. If the correction is limited to RSS serialization and cached HTML remains valid, no cache invalidation is needed.

Alternative considered: Always bump the revision. This would trigger unnecessary document reprocessing for a feed-only correction.

## Risks / Trade-offs

- [The Miniflux API may preserve HTML while its UI renders it unexpectedly] → Inspect the API content and the rendered entry separately; keep the feed's semantic blocks intact and avoid adding Miniflux-specific markup without evidence.
- [A stronger fixture may assume a specific block classification] → Use visual cues representative of the supplied document and assert the externally required semantic boundaries rather than internal font thresholds.
- [Extraction changes can cause a one-time refresh workload] → Bump the cache revision only when extraction output changes.

## Migration Plan

No schema migration is expected. If extraction behavior changes, deploy with an incremented processing revision and let the existing refresh/retry flow regenerate cached HTML. If only RSS output changes, deploy without reprocessing cached documents. Roll back by restoring the previous application version.

# Design

## Context

See `proposal.md` for motivation and `specs/feed-management/spec.md` for the behavior contract. Overview already counts documents needing review beneath active feeds, years, and months. The API lists documents only by month, and PDF review's current URL path represents its feed/year/month selection.

## Goals / Non-Goals

**Goals:**
- Use one review predicate for queue results and the active Overview count.
- Keep global queue selection directly addressable while preserving existing month browsing URLs.
- Make each queue row identifiable by feed, year, and month and retain publication decision controls.

**Non-Goals:**
- Change automatic classification, extraction, or publication eligibility rules.
- Change archive, restore, or refresh behavior.
- Add a new document-level archive state; archive status is inherited from its feed, year, and month.

## Decisions

- **Represent global queue selection in the PDF review query string.** Use `/manage/pdf-review?queue=needs-review` for active documents and `/manage/pdf-review?queue=archived` for archived documents. This preserves existing path-based feed/year/month selection and lets Overview link directly to the active queue. The ordinary `/manage/pdf-review` view continues to provide month browsing.
- **Add a management API query for global review documents.** Return document review fields together with feed display name, year, month number, and source scope activity. Use the existing predicate: extraction status is pending or failure, or promotional classification is outside `publish` and `promotional`. The active queue requires feed, year, and month all active; the archived queue includes a document when any of those scopes is inactive. The Overview count remains restricted to the active queue and uses the same predicate.
- **Keep archive membership derived from current hierarchy state.** Do not store a queue assignment on a document. Archiving moves matching attention items into the archived queue; restoring all ancestors moves them back to the active queue without a data migration.
- **Keep the existing month review flow intact.** Queue tabs/modes supplement the feed/year/month selectors. The global rows show source context and use the existing publication decision behavior. Extraction retry and source access continue to follow their current active-scope API constraints; restoring the archived scope is required for those operations.

## Risks / Trade-offs

- [Global queue scans may grow with the document table] → Filter in SQL by the review predicate and archive state, and add or adjust indexes only if query evidence requires it.
- [Queue and Overview totals could diverge if their predicates drift] → Share the same SQL condition or a common query fragment and verify both against active and archived fixtures.
- [Archived items cannot currently use source-open or extraction-retry operations because those endpoints require active scopes] → Show review decisions in the archived queue and communicate that restoring the scope is required before retrying extraction or opening its source.

## Migration Plan

No data migration is required. Deploy the management API and UI together; old PDF review paths continue to represent existing feed/year/month selections. Rollback removes the queue modes and global-list API while preserving stored document and archive state.

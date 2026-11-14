# Proposal

## Why

Operators currently have to choose a feed, year, and month before they can find PDFs that need a decision. Overview reports an active attention count but its review link opens the unfiltered chooser, while issues in archived sources should remain accessible without distracting from active publishing work.

## What Changes

- Add global “Needs review” and “Archived” queues to PDF review, separating documents by whether their feed, year, or month is archived.
- Show documents needing attention in the active queue across active feeds, years, and months, with source context and the existing review controls.
- Provide access to a separate queue for documents needing attention under archived scopes.
- Make Overview’s attention count and shortcut refer only to the active queue, and link directly to that queue when active documents need attention.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `feed-management`: Define global active and archived PDF review queues and direct Overview access to active documents needing attention.

## Impact

- Management API needs a global document listing that includes feed, year, and month context and classifies each result by scope activity.
- Overview summary and PDF review navigation in `src/feed_api.rs`, `web/src/lib/api.ts`, and `web/src/App.tsx` / `web/src/components/documents/PdfReview.tsx` will change.
- Existing month-scoped PDF review and archive, restore, publication decision, and extraction behavior need to remain coherent with the queues.
- `openspec/specs/feed-management/spec.md` will receive a delta specification.

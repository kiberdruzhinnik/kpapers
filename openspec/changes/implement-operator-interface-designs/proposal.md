# Proposal

## Why

The current operator screens already implement the required workflows, but their presentation does not yet consistently reflect the latest reviewed design drafts. Applying the approved visual direction will make PDF decisions, proxy routing, and feed/source management easier to scan without changing their behavior.

## What Changes

- Apply the current PDF review design to the review queues and source-browsing view.
- Reshape global proxy settings around the current settings design, including a distinct explanation of routing precedence.
- Align the feed list and feed detail/source-management screens with their current designs, including the per-feed proxy section.
- Preserve existing routes, data, validation, loading/error states, decisions, and destructive-action confirmations.
- Treat the current design drafts as approved implementation references:
  - PDF review v3: https://p.superdesign.dev/draft/9ef922b6-59aa-4221-a321-b9bfffc130d4
  - Settings v4: https://p.superdesign.dev/draft/019c063d-9171-4d5e-a8cc-bb5cef44889d
  - Feeds list v3: https://p.superdesign.dev/draft/f4af5ba3-6858-4482-bc91-62d3b825eee0
  - Feed detail v2: https://p.superdesign.dev/draft/d677b336-17bc-4d93-9e1c-aa98bf9b6974

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. Existing feed-management, pdf-ingestion, and fetch-proxy-configuration specifications already cover the observable behavior. This change is limited to presentation and explicitly preserves those contracts; spec deltas are skipped.

## Impact

- UI: `web/src/components/documents/PdfReview.tsx`, `web/src/components/settings/ProxySettings.tsx`, `web/src/components/feeds/FeedManagement.tsx`, `web/src/components/feeds/FeedProxySettings.tsx`, and `web/src/components/feeds/HierarchyManagement.tsx`.
- Reuses the existing React, Tailwind CSS v4, shadcn/Radix, and Lucide stack and shared dashboard shell.
- No API, persistence, dependency, or backend changes.
- Verification should extend existing PDF review, feed management, hierarchy, and app-level coverage for responsive presentation while retaining current behavior.

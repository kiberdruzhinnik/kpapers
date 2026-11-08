# Proposal

## Why

The operator dashboard loses the current view and PDF review selection on reload, while its Overview displays placeholder dashes for useful workspace counts. The sidebar and footer also contain controls and copy that take space without helping operators navigate or understand their workspace.

## What Changes

- Make the desktop sidebar collapsible and remember whether it is expanded or collapsed across reloads.
- Remove the refresh scheduler banner from the sidebar and the operator desk branding and schedule message from the footer.
- Make the Kpapers mark and name navigate to Overview.
- Give Overview, feed management and feed selection, and PDF review selections stable URLs that load directly and survive browser reloads.
- Show the number of configured source folders and PDFs awaiting a publication decision on Overview.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `feed-management`: Update the operator dashboard requirements for persistent navigation, directly addressable feed and PDF review views, and meaningful Overview counts.

## Impact

- Dashboard layout and navigation in `web/src/App.tsx`.
- Feed selection and PDF review state in `web/src/components/feeds/FeedManagement.tsx` and `web/src/components/documents/PdfReview.tsx`.
- Management API usage, and possibly a summary endpoint, to provide aggregate Overview counts.
- Existing `feed-management` capability specification.

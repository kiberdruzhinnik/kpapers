# Proposal

## Why

The selected feed page currently gives year groups, month entries, and source details too much visual weight, making routine settings difficult to scan. The approved feed detail direction keeps the hierarchy compact while leaving source metadata and operational actions available when needed.

## What Changes

- Present source folders as a single section with restrained, compact year groups.
- Render configured months as concise rows with status, PDF review, and an on-demand source-details disclosure.
- Keep source URLs, refresh metadata, and existing month actions within the disclosure, with destructive actions clearly separated.
- Preserve independent year expansion preferences, the feed list, feed identity, RSS controls, proxy settings, and existing action outcomes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `feed-management`: Specify the compact year/month hierarchy and the information and controls revealed by month source details.

## Impact

- Feed detail source-folder presentation in `web/src/components/feeds/HierarchyManagement.tsx`.
- Related hierarchy and feed management UI tests.
- The approved feed detail reference in `docs/design/DESIGN.md`.
- No API, storage, refresh, RSS, or proxy behavior changes.

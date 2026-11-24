# Proposal

## Why

PDF review currently uses a wide underlined button row that looks different from the compact segmented tabs on Feeds. Removing the redundant page-level back button will make PDF review fit the shared dashboard navigation more consistently.

## What Changes

- Restyle the PDF review mode selector to match the Feeds Active/Archived segmented tabs.
- Remove the PDF review page's back-to-feeds button; dashboard navigation remains available through the shared shell.
- Preserve the Needs review, Archived, and Browse by source modes, their behavior, and existing review URLs.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `feed-management`: PDF review presents its modes using the shared segmented-tab treatment and does not show a page-level back button.

## Impact

The PDF review page component and its UI tests are affected. No API, data, route, or dependency changes are expected.

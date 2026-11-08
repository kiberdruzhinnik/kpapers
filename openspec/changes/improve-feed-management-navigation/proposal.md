# Proposal

## Why

Feed management and public RSS currently share `/feeds/<id>`, so opening or reloading a selected feed through Nginx reaches RSS instead of the dashboard. Always-expanded years, raw refresh timestamps, and an embedded creation form also make the management view harder to browse.

## What Changes

- **BREAKING**: Move all dashboard views under `/manage/`: Overview at `/manage/`, feed list and selection at `/manage/feeds` and `/manage/feeds/<id>`, and PDF review at `/manage/pdf-review` with its existing selection suffixes. Reserve `/feeds/<id>` for existing public RSS subscriptions.
- Give feed creation its own protected `/manage/feeds/new` page, reached from Overview and Feeds, with success navigation to the created feed and Cancel navigation to the feed list.
- Remove the embedded creation form from the feed management view while retaining existing fields and validation.
- Make years independently collapsible, initially collapsed, with browser-local preferences remembered per feed and year.
- Render month last-attempt and next-refresh timestamps in the browser's local timezone as `YYYY-MM-DD HH:mm:ss`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `feed-management`: Move all dashboard routes under `/manage/`, separate dashboard and RSS routes, add a dedicated creation page, and remember year expansion preferences.
- `refresh-scheduling`: Specify local-time presentation of month refresh timestamps.

## Impact

Primarily `web/src/App.tsx`, feed management components, timestamp formatting, and associated component and browser tests. Nginx's existing SPA fallback must serve the new management paths while `/feeds/` continues proxying RSS. Management API URLs, public RSS URLs, persisted timestamp values, scheduler behavior, and database schema remain unchanged. No new dependency is expected.

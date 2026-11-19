# Proposal

## Why

Older configured months currently receive recurring automatic refreshes even though only the current month needs to stay continuously updated. Stopping those background scans reduces unnecessary provider traffic and processing while preserving on-demand refresh for historical data.

## What Changes

- Limit scheduled automatic refresh to the configured month matching the API server's current UTC year and month.
- Retain the feed-level interval override and current-month default interval for that eligible month.
- Preserve manual month, year, and feed refresh operations for older months.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `refresh-scheduling`: Automatic refresh applies only to the current UTC calendar month; older months remain manually refreshable.

## Impact

- `src/scheduler.rs`: automatic due-month selection and scheduling behavior.
- `docs/refresh-and-cache.md`: scheduling defaults and environment-variable documentation.
- `openspec/specs/refresh-scheduling/spec.md`: current-month automatic scheduling requirement.
- Existing manual refresh API and controls remain available.

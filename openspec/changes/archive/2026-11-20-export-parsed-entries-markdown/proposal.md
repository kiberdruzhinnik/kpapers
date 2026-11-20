# Proposal

## Why

Operators need a portable copy of publishable parsed entries for offline reading, backup, or reuse. Exporting from the feed hierarchy should preserve the same feed/year/month organization used to manage and review those entries.

## What Changes

- Add ZIP export actions at feed, year, and month scope.
- Include only entries with successful extraction that are currently eligible for publication, honoring promotional overrides.
- Store each exported entry as a Markdown file inside the preserved feed/year/month folder structure.
- Scope a feed export to all its years and months, a year export to all its months, and a month export to that month alone.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `feed-management`: add protected Markdown ZIP exports for feed, year, and month scopes.

## Impact

- Protected management API and operator dashboard export actions.
- Cached extracted content in SQLite and ZIP response generation.
- Existing feed publication eligibility rules determine which entries are included.

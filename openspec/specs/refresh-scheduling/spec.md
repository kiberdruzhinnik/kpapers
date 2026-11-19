# refresh-scheduling Specification

## Purpose

Keeps feed entries current through automatic refreshes, prioritizes the current calendar month, and lets the operator request immediate refetches at useful scopes.

## Requirements

### Requirement: Refresh the current month more frequently
The system SHALL schedule automatic refreshes only for the configured month matching the API server's current UTC calendar year and month. The current month SHALL use the configured per-feed interval override when present, otherwise the current-month default interval.

#### Scenario: Current month has a source
- **WHEN** the scheduler selects sources for automatic refresh
- **THEN** it schedules the configured source for the current UTC month using its feed override or the current-month default interval

#### Scenario: Older month has a source
- **WHEN** the scheduler selects sources for automatic refresh
- **THEN** it does not schedule configured months earlier or later than the current UTC calendar month, regardless of a feed interval override

#### Scenario: Current month changes
- **WHEN** the API server's UTC calendar advances into a configured month
- **THEN** that month becomes eligible for automatic refresh using its feed override or the current-month default interval, and the previous month is no longer eligible

### Requirement: Allow per-feed refresh interval overrides
The system SHALL let the operator configure an automatic refresh interval override for each feed.

#### Scenario: Feed override is configured
- **WHEN** a feed has a refresh interval override
- **THEN** the scheduler uses that interval for the feed's automatic refreshes

#### Scenario: Feed uses default schedule
- **WHEN** a feed has no override
- **THEN** the scheduler applies the system defaults, including current-month priority

### Requirement: Force refetch at feed, year, or month scope
The system SHALL let the operator request an immediate forced refetch of a whole feed, a year, or a month, bypassing normal schedule and unchanged-source cache checks.

#### Scenario: Force a month refetch
- **WHEN** the operator requests a forced refetch for a month
- **THEN** the system immediately re-lists and re-downloads eligible PDFs for that month and refreshes their extracted content

#### Scenario: Force a year refetch
- **WHEN** the operator requests a forced refetch for a year
- **THEN** the system forces a refetch for every configured month in that year

#### Scenario: Force a feed refetch
- **WHEN** the operator requests a forced refetch for a feed
- **THEN** the system forces a refetch for every configured year and month in that feed

### Requirement: Report refresh state
The system SHALL expose the last refresh time, next scheduled refresh time, and latest refresh result for each configured month. The dashboard SHALL display last-attempt and next-refresh timestamps in the browser's local timezone as zero-padded `YYYY-MM-DD HH:mm:ss` using a 24-hour clock. Missing last-attempt and next-refresh values SHALL display “Never” and “Pending” respectively.

#### Scenario: Operator reviews refresh status
- **WHEN** the operator views a feed's refresh status
- **THEN** each configured month shows its last attempt, next scheduled time, and success or failure result

#### Scenario: Convert timestamps into local time
- **WHEN** a browser using Europe/Moscow displays a month timestamp representing `2026-11-08T18:09:15Z`
- **THEN** the dashboard displays `2026-11-08 21:09:15`

#### Scenario: Use the offset applicable to the timestamp
- **WHEN** the browser's local timezone observes daylight-saving changes
- **THEN** each timestamp is displayed using the local offset applicable to that timestamp's instant

#### Scenario: Display missing timestamps
- **WHEN** a month has no last-attempt or next-refresh timestamp
- **THEN** the dashboard displays “Never” for the missing last attempt and “Pending” for the missing next refresh

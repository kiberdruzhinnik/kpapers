# Spec Delta

## Purpose

Keeps feed entries current through automatic refreshes, prioritizes the current calendar month, and lets the operator request immediate refetches at useful scopes.

## ADDED Requirements

### Requirement: Refresh the current month more frequently
The system SHALL schedule the current calendar month for more frequent automatic refresh than older configured months.

#### Scenario: Current month has a source
- **WHEN** the scheduler selects sources for automatic refresh
- **THEN** it gives the current month a shorter refresh interval than older months

#### Scenario: Current month changes
- **WHEN** the calendar advances into a configured month
- **THEN** that month receives the current-month refresh priority

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
The system SHALL expose the last refresh time, next scheduled refresh time, and latest refresh result for each configured month.

#### Scenario: Operator reviews refresh status
- **WHEN** the operator views a feed's refresh status
- **THEN** each configured month shows its last attempt, next scheduled time, and success or failure result

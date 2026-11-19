# Spec Delta

## MODIFIED Requirements

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

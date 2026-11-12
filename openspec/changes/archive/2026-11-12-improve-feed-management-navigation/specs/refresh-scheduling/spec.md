# Spec Delta

## MODIFIED Requirements

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

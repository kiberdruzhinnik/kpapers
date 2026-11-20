# Spec Delta

## ADDED Requirements

### Requirement: Export publishable entries as Markdown ZIPs
The protected operator interface SHALL export successfully parsed entries that are currently eligible for publication as Markdown files in a ZIP archive.

#### Scenario: Require operator authentication
- **WHEN** an unauthenticated client requests an export
- **THEN** the system rejects the request and returns no archive data

#### Scenario: Export a feed
- **WHEN** an operator exports a feed
- **THEN** the ZIP includes eligible entries from every year and month in that feed, preserving feed/year/month folders

#### Scenario: Export a year
- **WHEN** an operator exports a year
- **THEN** the ZIP includes eligible entries from every month in that year, preserving feed/year/month folders

#### Scenario: Export a month
- **WHEN** an operator exports a month
- **THEN** the ZIP includes eligible entries from only that month, preserving feed/year/month folders

#### Scenario: Filter entries by publication readiness
- **WHEN** a scoped export is generated
- **THEN** it includes only entries with successful extraction whose effective publication decision is publish, honoring operator overrides

#### Scenario: Export a scope with no eligible entries
- **WHEN** an operator exports a scope with no eligible entries
- **THEN** the export succeeds with a valid ZIP containing no Markdown entry files

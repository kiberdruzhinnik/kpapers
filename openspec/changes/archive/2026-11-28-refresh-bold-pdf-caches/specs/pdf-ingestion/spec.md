# Spec Delta

## MODIFIED Requirements

### Requirement: Cache source and processed PDF data
The system SHALL cache each discovered file's provider metadata, source content fingerprint, extraction pipeline revision, extracted text, and sanitized HTML for reuse between refreshes. Cached extraction SHALL be reused only when both the source fingerprint and extraction pipeline revision match, unless an operator forces parsing.

#### Scenario: Source file is unchanged
- **WHEN** a scheduled refresh sees the same source fingerprint and extraction pipeline revision as the cached version
- **THEN** the system reuses the cached extraction without downloading and parsing the PDF again

#### Scenario: Source file changes
- **WHEN** a refresh detects a changed source fingerprint
- **THEN** the system downloads and processes the changed PDF and updates its cached version

#### Scenario: Extraction pipeline changes for an unchanged PDF
- **WHEN** a refresh sees an unchanged source fingerprint but a newer extraction pipeline revision
- **THEN** the system downloads and processes the PDF again and updates its cached extracted text and sanitized HTML

#### Scenario: Operator forces parsing of an unchanged PDF
- **WHEN** an operator forces parsing for a PDF whose source fingerprint has not changed
- **THEN** the system bypasses cached extraction, processes the PDF, and replaces the cached extracted text and sanitized HTML after successful processing

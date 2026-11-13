# Spec Delta

## ADDED Requirements

### Requirement: Select a PDF extraction engine
The system SHALL support Pdfium and Marker as PDF extraction engines and SHALL use the configured global default unless a feed overrides it. Existing deployments SHALL default to Pdfium.

#### Scenario: Marker is selected
- **WHEN** Marker is the effective engine for a feed
- **THEN** Marker extracts every eligible PDF in that feed, including PDFs with native text

#### Scenario: Pdfium is selected
- **WHEN** Pdfium is the effective engine for a feed
- **THEN** Pdfium extracts its eligible PDFs

### Requirement: Apply publication processing to every engine
The system SHALL apply the existing PDF cleanup, promotional classification, and safe HTML requirements to output from either extraction engine.

#### Scenario: Marker encounters recognized publication chrome
- **WHEN** Marker extracts a PDF containing recognized boundary advertisements or Sponsr publication chrome
- **THEN** the same applicable cleanup rules remove that content while preserving article content

#### Scenario: Marker output contains unsafe markup
- **WHEN** Marker output includes markup outside the permitted HTML policy
- **THEN** published HTML excludes unsafe elements and attributes

### Requirement: Reprocess content when the effective engine changes
The system SHALL include the effective extraction engine and processing revision in cache identity and reprocess affected documents on their next refresh, even if their source fingerprint is unchanged.

#### Scenario: Feed changes engines
- **WHEN** a cached document is refreshed after its effective engine changes
- **THEN** the system extracts it with the newly selected engine instead of reusing the previous engine's cache

#### Scenario: Extraction fails
- **WHEN** the selected engine fails or is unavailable
- **THEN** the system records the failure and retains the last successfully processed content without silently switching engines

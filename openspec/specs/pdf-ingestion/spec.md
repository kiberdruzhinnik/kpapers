# pdf-ingestion Specification

## Purpose

Discovers eligible PDFs in supported public cloud folders, extracts and safely formats their text, and caches source and processed data for feed publication.

## Requirements

### Requirement: Discover only root-level PDFs
The system SHALL list only PDF files directly inside a configured month folder and SHALL NOT traverse subfolders.

#### Scenario: Folder contains a subfolder
- **WHEN** a refresh lists a month folder containing PDFs and a subfolder
- **THEN** only the direct PDF files are considered and the subfolder is ignored

#### Scenario: Folder contains non-PDF files
- **WHEN** a refresh lists a direct child that is not a PDF
- **THEN** the child is not ingested as a feed entry

### Requirement: Extract and sanitize PDF text
The system SHALL extract available text from each eligible PDF and expose it only as HTML composed of an allowlist of basic structural and emphasis tags without executable content or unsafe attributes.

#### Scenario: Text PDF is processed
- **WHEN** an eligible PDF contains extractable text
- **THEN** the system produces readable HTML using only the configured safe tag allowlist

#### Scenario: PDF has no extractable text
- **WHEN** text extraction yields no usable text or fails
- **THEN** the system records an ingestion error and does not publish an empty or unsafe entry

### Requirement: Cache source and processed PDF data
The system SHALL cache each discovered file's provider metadata, content fingerprint, extracted text, and sanitized HTML for reuse between refreshes.

#### Scenario: Source file is unchanged
- **WHEN** a scheduled refresh sees the same source fingerprint as the cached version
- **THEN** the system reuses the cached extraction without downloading and parsing the PDF again

#### Scenario: Source file changes
- **WHEN** a refresh detects a changed source fingerprint
- **THEN** the system downloads and processes the changed PDF and updates its cached version

### Requirement: Preserve cached entries on refresh failure
The system SHALL retain the last successfully processed version of an entry when a later provider request or extraction attempt fails.

#### Scenario: Provider is unavailable
- **WHEN** a refresh cannot list or download a source file
- **THEN** the system keeps the last successful cached entry and records the refresh failure

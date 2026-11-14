# Spec Delta

## ADDED Requirements

### Requirement: Extract eligible PDFs with pdf_oxide default text extraction
The system SHALL use the Rust `pdf_oxide` default plain-text extraction path for every eligible PDF, without OCR or image embedding, and SHALL NOT switch to another extraction engine when conversion fails.

#### Scenario: Convert an eligible multi-page PDF
- **WHEN** an eligible PDF is refreshed
- **THEN** `pdf_oxide` extracts each page with `PdfDocument::extract_text_lines`, preserving available text, run styling, and page reading order for the existing publication pipeline

#### Scenario: Conversion fails
- **WHEN** `pdf_oxide` cannot open or extract text from a PDF
- **THEN** the system records an ingestion error and preserves the last successfully processed content

#### Scenario: Scanned-only PDF has no extractable text
- **WHEN** a PDF contains only scanned pages and OCR is not enabled
- **THEN** the system reports extraction failure rather than publishing empty or incomplete output

### Requirement: Apply publication processing to extracted text
The system SHALL apply PDF cleanup, promotional classification, publication HTML generation, and safe HTML requirements to text produced by `pdf_oxide`.

#### Scenario: Conversion encounters recognized publication chrome
- **WHEN** `pdf_oxide` extracts text from a PDF containing recognized boundary advertisements or Sponsr publication chrome
- **THEN** the applicable cleanup rules remove that content while preserving article content

#### Scenario: Converted content contains unsafe markup
- **WHEN** extracted text contains strings that resemble HTML or content that could form markup outside the permitted HTML policy
- **THEN** published HTML excludes unsafe elements and attributes


### Requirement: Reprocess content when the processing pipeline changes
The system SHALL include the `pdf_oxide` engine identity and processing revision in cache identity and reprocess affected documents on their next refresh when the processing revision changes, even if their source fingerprint is unchanged.

#### Scenario: Processing revision changes
- **WHEN** a cached document is refreshed after the PDF processing revision changes
- **THEN** the system converts it with `pdf_oxide` instead of reusing content produced by an earlier pipeline revision

#### Scenario: Conversion fails
- **WHEN** `pdf_oxide` fails or is unavailable
- **THEN** the system records the failure and continues publishing the last successfully processed content without silently switching engines

#### Scenario: Cached content is reused after a failed refresh
- **WHEN** a later refresh reuses a matching successful cache after a conversion failure
- **THEN** the cached entry remains available in RSS and the document's recovery state reflects the successful reuse

## REMOVED Requirements

### Requirement: Select a PDF extraction engine
**Reason**: Supporting multiple selectable engines is no longer required; the system now uses `pdf_oxide` for every feed.
**Migration**: Remove global and per-feed engine selection and remove Pdfium-specific configuration.

### Requirement: Apply publication processing to every engine
**Reason**: The multi-engine contract is obsolete now that `pdf_oxide` is the only extraction path.
**Migration**: Apply the same cleanup, promotional classification, safe HTML, and notice formatting behavior to `pdf_oxide` extracted text under the replacement single-engine requirement.

### Requirement: Prevent incomplete OCR-disabled output
**Reason**: This requirement governs Marker conversion modes, which are no longer supported.
**Migration**: Existing Marker configuration is removed; `pdf_oxide` extracts available native PDF text without OCR. Scanned-only PDFs without extractable text remain ingestion failures.

### Requirement: Recover worker capacity after a conversion timeout
**Reason**: This requirement governs the separate Marker worker, which is removed.
**Migration**: Remove the Marker worker and its timeout configuration from deployments. `pdf_oxide` conversion runs in the API process.

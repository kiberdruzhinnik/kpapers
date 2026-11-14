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

#### Scenario: Marker extracts an indented Sponsr notice
- **WHEN** a Sponsr PDF contains an indented notice region with multiple paragraphs
- **THEN** published HTML groups the notice in one blockquote, preserves its paragraph boundaries, and keeps subsequent body text outside it

#### Scenario: An indented notice contains mixed emphasis
- **WHEN** an indented notice contains multiple paragraphs with mixed emphasis
- **THEN** published HTML preserves the paragraph boundaries and emphasis within one blockquote

### Requirement: Reprocess content when the effective engine changes
The system SHALL include the effective extraction engine and processing revision in cache identity and reprocess affected documents on their next refresh, even if their source fingerprint is unchanged.

#### Scenario: Feed changes engines
- **WHEN** a cached document is refreshed after its effective engine changes
- **THEN** the system extracts it with the newly selected engine instead of reusing the previous engine's cache

#### Scenario: Extraction fails
- **WHEN** the selected engine fails or is unavailable
- **THEN** the system records the failure and continues publishing the last successfully processed content without silently switching engines

#### Scenario: Cached content is reused after a failed refresh
- **WHEN** a later refresh reuses a matching successful cache after an extraction failure
- **THEN** the cached entry remains available in RSS and the document's recovery state reflects the successful reuse

### Requirement: Prevent incomplete OCR-disabled output
The system SHALL NOT replace successful cached content with an OCR-disabled conversion result when scanned content may have been omitted, whether OCR-disabled mode is initially selected or used in a retry. The system SHALL use OCR unless completeness of the native text layer can be established, and SHALL report extraction failure if conversion cannot complete safely.

#### Scenario: Marker retries a mixed document without OCR
- **WHEN** Marker conversion fails and a retry without OCR cannot establish that all native-text and scanned pages were extracted
- **THEN** the system reports a failure instead of publishing or caching the partial result

#### Scenario: A page combines native text and scanned content
- **WHEN** a PDF page contains both native text and scanned content and completeness of its native text layer cannot be established
- **THEN** the system uses OCR and reports a failure rather than publishing incomplete content if extraction cannot complete safely

#### Scenario: A small image contains scanned text
- **WHEN** a PDF page contains native text and a small image that may contain scanned text
- **THEN** image size alone does not establish completeness of the native text layer and the system uses OCR unless completeness is established

#### Scenario: Visible text is drawn as vector paths
- **WHEN** a PDF page contains native text and visible lettering drawn as vector paths whose text is absent from the native text layer
- **THEN** the system uses OCR or reports a failure without publishing or caching partial content unless completeness can be established

#### Scenario: An image is nested within PDF forms
- **WHEN** a PDF page contains native text and an image nested beyond an object traversal limit
- **THEN** the system detects the image or treats the native text layer as unverifiable and uses OCR or reports a failure without publishing partial content

#### Scenario: Automatic text-layer selection could skip scanned content
- **WHEN** Marker encounters a mixed native-text and scanned page whose text layer cannot be established as complete
- **THEN** the system uses a conversion path that performs OCR on the scanned content or reports a failure without publishing partial content, even if Marker's automatic mode would skip OCR

#### Scenario: OCR-disabled mode is explicitly configured
- **WHEN** OCR-disabled worker mode is configured for a PDF whose native text-layer completeness cannot be established
- **THEN** the system uses OCR or reports a failure without publishing or caching partial content

### Requirement: Recover worker capacity after a conversion timeout
The system SHALL stop timed-out conversion work or restart the worker so that a timed-out conversion cannot permanently occupy extraction capacity.

#### Scenario: A conversion exceeds its execution limit
- **WHEN** a Marker conversion exceeds its execution time limit
- **THEN** the request fails and the worker recovers capacity to process a subsequent document

#### Scenario: The client disconnects before a timeout response
- **WHEN** a Marker conversion exceeds its execution time limit after the client disconnects
- **THEN** the worker still stops the conversion or restarts and recovers capacity to process a subsequent document

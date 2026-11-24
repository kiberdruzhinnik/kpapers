# Spec Delta

## ADDED Requirements

### Requirement: Correct high-confidence Russian and English extraction errors locally
The system SHALL check extracted Russian and English text for spelling or word-form errors using only local processing. It SHALL automatically apply a correction only for a correction class whose measured precision is at least 85%, or for an exact extracted form and replacement explicitly reviewed and approved; otherwise, it SHALL preserve the extracted text unchanged.

#### Scenario: An exact reviewed form is approved
- **WHEN** extracted text matches an explicitly reviewed and approved exact form
- **THEN** the system applies its approved correction locally and leaves other forms unchanged

#### Scenario: A Russian correction meets the precision threshold
- **WHEN** extracted Russian text contains an error covered by a correction class validated at 85% precision or higher
- **THEN** the system applies the correction locally

#### Scenario: An English correction meets the precision threshold
- **WHEN** extracted English text contains an error covered by a correction class validated at 85% precision or higher
- **THEN** the system applies the correction locally

#### Scenario: A correction class does not meet the precision threshold
- **WHEN** a proposed correction belongs to a class with measured precision below 85% or without a validated precision result
- **THEN** the system leaves the extracted text unchanged

#### Scenario: Extracted text contains other languages or mixed-language content
- **WHEN** a word cannot be confidently handled as Russian or English
- **THEN** the system preserves it without correction

#### Scenario: Correction runs without external services
- **WHEN** the system processes extracted PDF text for correction
- **THEN** it performs spelling checks and corrections locally and sends no extracted text to an external service

#### Scenario: An exact reviewed form is approved
- **WHEN** extracted text matches an explicitly reviewed and approved exact form
- **THEN** the system applies its approved correction locally and leaves other forms unchanged

### Requirement: Annotate applied corrections in published text
The system SHALL place the original text immediately after each applied correction in a bracketed, language-appropriate annotation, and SHALL include that annotation in published content.

#### Scenario: Publish an English correction
- **WHEN** the system applies a correction to English text
- **THEN** published text contains the corrected form followed by `[corrected from: original_text]`

#### Scenario: Publish a Russian correction
- **WHEN** the system applies a correction to Russian text
- **THEN** published text contains the corrected form followed by `[исправлено: original_text]`

#### Scenario: No correction is applied
- **WHEN** extracted text contains no eligible correction
- **THEN** published text contains no correction annotation for that text

### Requirement: Reprocess cached content when local correction processing changes
The system SHALL include local correction processing in PDF cache identity and SHALL reprocess cached documents on their next refresh when its behavior or correction data changes.

#### Scenario: Correction processing revision changes
- **WHEN** a cached document is refreshed after the correction processing revision changes while its source PDF is unchanged
- **THEN** the system regenerates its corrected extracted text and published HTML instead of reusing the earlier processing result

#### Scenario: Corrected content is reused from cache
- **WHEN** a refresh sees matching source and processing fingerprints
- **THEN** the system reuses the cached corrected text and its published annotations without repeating PDF extraction

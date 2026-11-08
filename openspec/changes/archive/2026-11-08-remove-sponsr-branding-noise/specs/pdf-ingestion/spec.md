# Spec Delta

## ADDED Requirements

### Requirement: Omit standalone Sponsr branding from extracted PDF content
The system SHALL omit the standalone branding line `Кримсон Аналитика – CrimsonAnalytics` from extracted text and published HTML wherever it appears as publication chrome in a PDF.

#### Scenario: Branding appears as its own line on a later page
- **WHEN** an eligible PDF contains the branding line as a standalone line on any page, including after the first page
- **THEN** the line is absent from cached extracted text and published HTML while surrounding article content is preserved

#### Scenario: Branding is part of an article sentence
- **WHEN** an eligible PDF mentions the same branding text as part of a larger article sentence or paragraph
- **THEN** the article text is preserved in extracted text and published HTML

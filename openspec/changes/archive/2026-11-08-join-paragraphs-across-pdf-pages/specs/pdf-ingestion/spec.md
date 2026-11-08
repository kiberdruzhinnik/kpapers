# Spec Delta

## MODIFIED Requirements

### Requirement: Join wrapped lines within paragraphs
The system SHALL join line-wrapped text within a paragraph using spaces while preserving paragraph boundaries, including when a paragraph continues across adjacent PDF pages.

#### Scenario: Prose wraps across extracted lines
- **WHEN** a paragraph is extracted as multiple lines separated by line breaks
- **THEN** those lines are joined into continuous prose with spaces and the paragraph remains a single HTML paragraph

#### Scenario: Separate paragraphs are extracted
- **WHEN** paragraphs are separated in the extracted text
- **THEN** each paragraph remains distinct in the resulting HTML

#### Scenario: A prose paragraph continues across a page boundary
- **WHEN** a page ends mid-sentence and the next page begins with its continuation
- **THEN** the text on both pages is joined with whitespace in one paragraph in extracted text and published HTML

#### Scenario: A paragraph boundary falls at a page boundary
- **WHEN** the PDF layout indicates that the preceding paragraph ends and a new paragraph begins on the next page
- **THEN** the paragraphs remain distinct in extracted text and published HTML

# Spec Delta

## ADDED Requirements

### Requirement: Join parenthetical continuations across page boundaries
The system SHALL keep a prose paragraph continuous across a PDF page boundary when the next page begins with a parenthetical continuation and the text and layout indicate that it continues the preceding paragraph.

#### Scenario: Page continuation begins with a parenthetical phrase
- **WHEN** a page ends mid-sentence and the next page begins with an opening parenthesis followed by lowercase prose, with compatible styling and alignment
- **THEN** both pages' text is joined with whitespace in one paragraph in extracted text and published HTML

#### Scenario: Parenthetical continuation follows a completed sentence
- **WHEN** a page ends with a completed sentence and the next page begins a parenthetical phrase as a new paragraph
- **THEN** the paragraphs remain distinct in extracted text and published HTML

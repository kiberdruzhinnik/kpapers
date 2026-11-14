# Spec Delta

## MODIFIED Requirements

### Requirement: Join wrapped lines within paragraphs
The system SHALL join line-wrapped text within a paragraph using spaces while preserving paragraph boundaries, including when a paragraph continues across adjacent PDF pages. A large vertical gap alone SHALL NOT split a sentence when the text and layout indicate that the next line continues it. The honorific abbreviation `ув.` followed by an uppercase continuation SHALL be treated as continuation evidence when layout and styling are compatible.

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

#### Scenario: A sentence continues after a large vertical gap
- **WHEN** a line ends with continuation punctuation such as a comma and the following line begins as a grammatical continuation, with compatible text styling and alignment despite a large vertical gap
- **THEN** both lines remain in one paragraph in extracted text and published HTML

#### Scenario: A wrapped line continues after the honorific abbreviation
- **WHEN** a line ends with `ув.` and the following line begins with an uppercase continuation, as in `... экспресса» ув.` followed by `Александр Романов ...`, with compatible styling and alignment
- **THEN** both lines remain in one paragraph in extracted text and published HTML, even if a large vertical gap would otherwise suggest a paragraph boundary

#### Scenario: A wrapped line continues with the capitalized word after the honorific abbreviation
- **WHEN** a line ends with `ув.` and the following line begins `Собеседника.` as in `... стартового тезиса ув.` followed by `Собеседника. Идея ...`, with compatible styling and alignment
- **THEN** both lines remain in one paragraph in extracted text and published HTML

#### Scenario: An ordinary sentence-ending period does not imply continuation
- **WHEN** a line ends with a period that is not the recognized honorific abbreviation and the PDF layout indicates a paragraph boundary before the next line
- **THEN** the paragraphs remain distinct in extracted text and published HTML

#### Scenario: A large gap marks a genuine paragraph boundary
- **WHEN** the PDF layout indicates that a large vertical gap separates complete prose paragraphs and no continuation evidence applies
- **THEN** the paragraphs remain distinct in extracted text and published HTML

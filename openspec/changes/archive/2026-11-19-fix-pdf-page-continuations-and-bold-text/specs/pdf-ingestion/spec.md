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

#### Scenario: A lowercase sentence continuation follows a page-ending conjunction
- **WHEN** a page ends with `потому что` and the next page begins with `любой вопрос есть вопрос добра и зла`, as in the supplied PDF, with layout indicating the same paragraph
- **THEN** the extracted text and published HTML contain `потому что любой вопрос есть вопрос добра и зла` in one paragraph

### Requirement: Preserve bold and italic PDF styling
The system SHALL represent bold text with `<strong>` and italic text with `<em>`, including both tags when a text run has both styles, while leaving normally styled text unwrapped. Adjacent text runs with the same style SHALL be merged into continuous semantic tags within a paragraph or heading, including when separated only by a whitespace run; spaces and actual block boundaries SHALL be preserved.

#### Scenario: Render bold and italic runs
- **WHEN** a PDF text run uses bold or italic styling
- **THEN** the corresponding text is wrapped in `<strong>` or `<em>` in the resulting HTML

#### Scenario: Render text with both styles
- **WHEN** a PDF text run uses both bold and italic styling
- **THEN** the resulting HTML nests `<strong>` and `<em>` around that text

#### Scenario: Preserve normal text
- **WHEN** a PDF text run has neither bold nor italic styling
- **THEN** the resulting HTML does not add emphasis tags around it

#### Scenario: Merge adjacent runs with the same style
- **WHEN** consecutive runs within one paragraph use the same italic style
- **THEN** their text is rendered inside one continuous `<em>` element with intervening spaces preserved

#### Scenario: Merge same-style runs inside a heading
- **WHEN** date and time text inside one heading are emitted as adjacent bold runs separated only by whitespace
- **THEN** both values are rendered inside one `<strong>` element with one space between them

#### Scenario: Preserve nested style changes and paragraph boundaries
- **WHEN** a bold run appears inside an italic paragraph and the document has a paragraph break before the next italic text
- **THEN** bold is nested inside the continuous italic span, and the next paragraph remains a separate paragraph

#### Scenario: Interpret Markdown bold markers in extracted PDF text
- **WHEN** extracted text contains a matched Markdown bold span such as `**это не инвестиционные рекомендации**`
- **THEN** the marker characters are removed and the enclosed text is emitted inside `<strong>` tags, combined with any PDF font styling without duplicate or fragmented emphasis tags

#### Scenario: Preserve unmatched Markdown marker characters
- **WHEN** extracted text contains unmatched `**` marker characters
- **THEN** the unmatched characters remain ordinary escaped text and do not cause the surrounding content to be dropped or incorrectly emphasized

#### Scenario: Keep an explicit bold disclaimer in a paragraph
- **WHEN** a Sponsr article contains the matched literal Markdown disclaimer `**это не инвестиционные рекомендации**` after its title
- **THEN** the markers are removed and the disclaimer is rendered as `<p><strong>это не инвестиционные рекомендации</strong></p>`, not as a heading

#### Scenario: Preserve a bold word within regular prose
- **WHEN** a PDF paragraph contains a word set in bold between normally styled words
- **THEN** the word is emitted inside `<strong>` while surrounding text remains unwrapped and in the original reading order

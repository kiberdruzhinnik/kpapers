# Spec Delta

## MODIFIED Requirements

### Requirement: Preserve bold and italic PDF styling
The system SHALL represent bold text with `<strong>` and italic text with `<em>`, including both tags when a text run has both styles, while leaving normally styled text unwrapped. Adjacent text runs with the same style SHALL be merged into continuous semantic tags within a paragraph or heading, including when separated only by a whitespace run; spaces and actual block boundaries SHALL be preserved. For the literal investment disclaimer `**это не инвестиционные рекомендации**`, the marker characters SHALL remain visible inside its generated `<strong>` element.

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

#### Scenario: Preserve literal markers in the investment disclaimer
- **WHEN** extracted text contains the literal investment disclaimer `**это не инвестиционные рекомендации**`
- **THEN** the full literal span, including both pairs of asterisks, is emitted inside `<strong>` tags without duplicate or fragmented emphasis tags

#### Scenario: Preserve unmatched Markdown marker characters
- **WHEN** extracted text contains unmatched `**` marker characters
- **THEN** the unmatched characters remain ordinary escaped text and do not cause the surrounding content to be dropped or incorrectly emphasized

#### Scenario: Keep an explicit bold disclaimer in a paragraph
- **WHEN** a PDF contains the literal disclaimer `**это не инвестиционные рекомендации**` after its title
- **THEN** it is rendered as `<p><strong>**это не инвестиционные рекомендации**</strong></p>`, not as a heading, with the marker characters preserved

#### Scenario: Preserve a bold word within regular prose
- **WHEN** a PDF paragraph contains a word set in bold between normally styled words
- **THEN** the word is emitted inside `<strong>` while surrounding text remains unwrapped and in the original reading order

## ADDED Requirements

### Requirement: Preserve the complete wrapped CrimsonAnalytics headline
The system SHALL combine the visual lines of the supplied CrimsonAnalytics headline into one `<h1>` and keep the following literal Markdown disclaimer in its own bold paragraph.

#### Scenario: Parse the supplied Coffee with a Banker headline and disclaimer
- **WHEN** the PDF contains the headline beginning `[Кофе с банкиром] Последствия поражения Трампа` and ending `заработках на деклассированных элементах и спонсорах Демпартии.` across wrapped lines, followed by `**это не инвестиционные рекомендации**`
- **THEN** published HTML begins with the complete headline in one `<h1>`, followed by `<p><strong>**это не инвестиционные рекомендации**</strong></p>`

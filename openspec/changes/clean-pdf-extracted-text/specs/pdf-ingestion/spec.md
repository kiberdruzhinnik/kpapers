# Spec Delta

## ADDED Requirements

### Requirement: Remove a recognized promotional final page
The system SHALL omit the final page from extracted text when that page contains the recognizable Skladchik promotional content.

#### Scenario: PDF ends with the Skladchik advertisement
- **WHEN** the final page contains the `skladchik.ORG`, “Ты еще не в Клубе?”, and “Что такое Клуб «Складчик»?” promotional text
- **THEN** the final page is excluded from the extracted text and published HTML

#### Scenario: PDF final page is not the advertisement
- **WHEN** the final page does not contain the recognizable Skladchik promotional content
- **THEN** the final page remains in the extracted text and published HTML

### Requirement: Omit page labels from extracted text
The system SHALL omit a standalone numeric page label at the beginning of a page when it matches that page's position in the PDF.

#### Scenario: Page starts with its printed page number
- **WHEN** the first non-empty extracted line of a page is only that page's one-based page number
- **THEN** the number is omitted from extracted text

#### Scenario: A number appears in page content
- **WHEN** a number appears after page content has started or does not match the page's position
- **THEN** the number is preserved

### Requirement: Join wrapped lines within paragraphs
The system SHALL join line-wrapped text within a paragraph using spaces while preserving paragraph boundaries.

#### Scenario: Prose wraps across extracted lines
- **WHEN** a paragraph is extracted as multiple lines separated by line breaks
- **THEN** those lines are joined into continuous prose with spaces and the paragraph remains a single HTML paragraph

#### Scenario: Separate paragraphs are extracted
- **WHEN** paragraphs are separated in the extracted text
- **THEN** each paragraph remains distinct in the resulting HTML

### Requirement: Render PDF headings with semantic levels
The system SHALL identify headings from PDF typography and layout and render them using semantic heading tags from `<h1>` through `<h4>` according to their document hierarchy. The document title uses `<h1>`, top-level sections use `<h2>`, and nested headings use `<h3>` or `<h4>` as appropriate. Ordinary prose remains paragraph content.

#### Scenario: Render document title and section hierarchy
- **WHEN** a PDF contains a title and visually distinct section and subsection headings
- **THEN** the title is rendered as `<h1>`, sections as `<h2>`, and nested headings at the matching `<h3>` or `<h4>` level

#### Scenario: Preserve ordinary prose as paragraphs
- **WHEN** extracted text is ordinary body prose rather than a heading
- **THEN** it remains rendered as paragraph content and is not promoted to a heading

### Requirement: Preserve bold and italic PDF styling
The system SHALL represent bold text with `<strong>` and italic text with `<em>`, including both tags when a text run has both styles, while leaving normally styled text unwrapped.

#### Scenario: Render bold and italic runs
- **WHEN** a PDF text run uses bold or italic styling
- **THEN** the corresponding text is wrapped in `<strong>` or `<em>` in the resulting HTML

#### Scenario: Render text with both styles
- **WHEN** a PDF text run uses both bold and italic styling
- **THEN** the resulting HTML nests `<strong>` and `<em>` around that text

#### Scenario: Preserve normal text
- **WHEN** a PDF text run has neither bold nor italic styling
- **THEN** the resulting HTML does not add emphasis tags around it

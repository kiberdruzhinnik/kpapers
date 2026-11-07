# pdf-ingestion Specification

## Purpose

Discovers eligible PDFs in supported public cloud folders, extracts and safely formats their text, and caches source and processed data for feed publication.

## Requirements

### Requirement: Discover only root-level PDFs
The system SHALL list only PDF files directly inside a configured month folder and SHALL NOT traverse subfolders.

#### Scenario: Folder contains a subfolder
- **WHEN** a refresh lists a month folder containing PDFs and a subfolder
- **THEN** only the direct PDF files are considered and the subfolder is ignored

#### Scenario: Folder contains non-PDF files
- **WHEN** a refresh lists a direct child that is not a PDF
- **THEN** the child is not ingested as a feed entry

### Requirement: Extract and sanitize PDF text
The system SHALL extract available text from each eligible PDF and expose it only as HTML composed of an allowlist of basic structural and emphasis tags without executable content or unsafe attributes.

#### Scenario: Text PDF is processed
- **WHEN** an eligible PDF contains extractable text
- **THEN** the system produces readable HTML using only the configured safe tag allowlist

#### Scenario: PDF has no extractable text
- **WHEN** text extraction yields no usable text or fails
- **THEN** the system records an ingestion error and does not publish an empty or unsafe entry

### Requirement: Cache source and processed PDF data
The system SHALL cache each discovered file's provider metadata, content fingerprint, extracted text, and sanitized HTML for reuse between refreshes.

#### Scenario: Source file is unchanged
- **WHEN** a scheduled refresh sees the same source fingerprint as the cached version
- **THEN** the system reuses the cached extraction without downloading and parsing the PDF again

#### Scenario: Source file changes
- **WHEN** a refresh detects a changed source fingerprint
- **THEN** the system downloads and processes the changed PDF and updates its cached version

### Requirement: Preserve cached entries on refresh failure
The system SHALL retain the last successfully processed version of an entry when a later provider request or extraction attempt fails.

#### Scenario: Provider is unavailable
- **WHEN** a refresh cannot list or download a source file
- **THEN** the system keeps the last successful cached entry and records the refresh failure

### Requirement: Remove a recognized boundary-page advertisement
The system SHALL omit a dedicated Skladchik promotional page from extracted text when it appears as the first or final PDF page. Recognition SHALL support both the Club advertisement and the “куплено на skladchik.ORG” advertisement.

#### Scenario: PDF begins with the purchased-on Skladchik advertisement
- **WHEN** the first page begins with “куплено на” and contains `skladchik.ORG` as the advertisement's destination
- **THEN** the first page is excluded from extracted text and published HTML

#### Scenario: PDF ends with the Skladchik Club advertisement
- **WHEN** the final page contains the `skladchik.ORG`, “Ты еще не в Клубе?”, and “Что такое Клуб «Складчик»?” promotional text
- **THEN** the final page is excluded from the extracted text and published HTML

#### Scenario: A boundary page contains ordinary article text
- **WHEN** a first or final page mentions `skladchik.ORG` without matching either recognized advertisement pattern
- **THEN** the page remains in extracted text and published HTML

### Requirement: Omit Sponsr publication preamble
The system SHALL omit recognizable Sponsr publication chrome and article metadata that precede an identifiable article title, so published content begins with the article title. The article body SHALL remain intact.

#### Scenario: Sponsr metadata precedes the article title
- **WHEN** a PDF contains a Sponsr masthead, author/byline, publication date/time, or reading duration before an identifiable article title
- **THEN** the preamble is omitted, the article title is the first published content, and the article body is preserved

#### Scenario: Remove the Sponsr date and reading duration shown in the publication header
- **WHEN** a Sponsr PDF contains `27 февр 11:44` and a reading duration before the headline `[Философское] «Старые деньги» — FAQ: как правильно считать дивиденды, «бессмертные» портфели и стоицизм`
- **THEN** the published HTML begins with that headline as its first heading, omits both metadata values and the preceding Sponsr chrome, and preserves the content following the headline

#### Scenario: Sponsr PDF title is present in the filename but not the text layer
- **WHEN** the PDF's first extracted heading contains publication date/time and duration, its text layer omits the article title, and its filename contains the `[Философское]` headline
- **THEN** the filename headline is emitted as the first `<h1>`, the publication metadata is removed from extracted text, the article body remains, and XML-invalid control characters do not break RSS

#### Scenario: No recognizable Sponsr preamble and title are present
- **WHEN** a PDF does not contain recognizable Sponsr publication chrome followed by an identifiable article title
- **THEN** leading text is preserved

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
The system SHALL identify headings from PDF typography and layout and render them using semantic heading tags from `<h1>` through `<h3>` according to their document hierarchy. The document title uses `<h1>`, top-level headings use `<h2>`, and nested headings use `<h3>`. Visual continuation lines of one logical heading SHALL be combined with whitespace between words. Text that would otherwise be classified below `<h3>` SHALL remain paragraph content.

#### Scenario: Render document title and supported section hierarchy
- **WHEN** a PDF contains a title and visually distinct section and subsection headings
- **THEN** the title is rendered as `<h1>`, sections as `<h2>`, and nested headings as `<h3>`

#### Scenario: Preserve ordinary prose as paragraphs
- **WHEN** extracted text is ordinary body prose rather than a heading
- **THEN** it remains rendered as paragraph content and is not promoted to a heading

#### Scenario: Keep a wrapped document title as one heading
- **WHEN** the document title wraps across multiple PDF lines but remains one logical title
- **THEN** all title text is rendered in one `<h1>` with spaces between the wrapped lines, not split into separate heading levels

#### Scenario: Keep a wrapped section headline as one heading
- **WHEN** a top-level headline wraps across multiple PDF lines but remains one logical heading
- **THEN** all headline text is rendered in one `<h2>` with spaces between the wrapped lines

#### Scenario: Keep lower-level heading candidates as paragraphs
- **WHEN** typography and layout would classify a line below the supported `<h3>` level
- **THEN** its text is rendered as a paragraph and no `<h4>` element is emitted

#### Scenario: Keep Sponsr body blocks distinct after a recovered filename headline
- **WHEN** a Sponsr PDF has an image-only `[Философское]` headline recovered from its filename, followed by a bold disclaimer, ordinary paragraphs, an indented notice, and more article text
- **THEN** the headline is the first `<h1>`, the disclaimer remains a paragraph with `<strong>`, ordinary text remains in `<p>` elements, the notice is grouped in one `<blockquote>`, and body text is not promoted to `<h2>` or `<h3>` based on font size alone

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

### Requirement: Render visually indented notice regions as blockquotes
The system SHALL identify a notice heading such as “Традиционные технические предуведомления:” and its visually indented continuation region from PDF text position and layout. It SHALL wrap that region in a `<blockquote>`, preserve the heading and paragraph boundaries inside it, and end the blockquote when the text returns to the ordinary body margin. It SHALL preserve ordinary body text outside the indented region.

#### Scenario: Render the technical notice and its indented contents as a blockquote
- **WHEN** “Традиционные технические предуведомления:” begins a visually indented region followed by notice paragraphs
- **THEN** the heading and indented notice paragraphs are contained in one `<blockquote>` with their paragraph and inline emphasis markup preserved

#### Scenario: End a blockquote when ordinary body text resumes
- **WHEN** text following an indented notice region returns to the ordinary body margin
- **THEN** the `<blockquote>` ends before that text, which remains outside the blockquote as ordinary document content

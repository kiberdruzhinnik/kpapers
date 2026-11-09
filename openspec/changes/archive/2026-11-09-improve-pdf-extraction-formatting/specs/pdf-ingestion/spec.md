# Spec Delta

## MODIFIED Requirements

### Requirement: Omit page labels from extracted text
The system SHALL omit a standalone numeric page label when it matches that page's one-based position among the pages remaining after leading boundary advertisement pages are removed, regardless of where the label appears in the extracted line order. Numbers that are part of content or do not match the page position SHALL be preserved.

#### Scenario: Page starts with its printed page number
- **WHEN** the first non-empty extracted line of a page is only that page's one-based content-page number
- **THEN** the number is omitted from extracted text

#### Scenario: Page number follows extracted page content
- **WHEN** a standalone line containing only the page's one-based content-page number appears after other extracted lines on that page
- **THEN** the number is omitted from extracted text and published HTML

#### Scenario: Leading advertisement page shifts printed numbering
- **WHEN** a PDF begins with a boundary advertisement page that is removed and the first remaining content page prints `1`
- **THEN** that `1` and later matching standalone page numbers are omitted from extracted text and published HTML

#### Scenario: A number appears in page content
- **WHEN** a number appears as part of a line with other content or does not match the page's content-page position
- **THEN** the number is preserved

### Requirement: Render PDF headings with semantic levels
The system SHALL identify document titles and section headings from corroborating PDF typography and layout cues and render them using semantic heading tags from `<h1>` through `<h3>` according to their document hierarchy. Font size or spacing alone SHALL NOT cause ordinary prose to be classified as a heading. The document title uses `<h1>`, top-level headings use `<h2>`, and nested headings use `<h3>`. Visual continuation lines of one logical heading SHALL be combined with whitespace between words. Text that would otherwise be classified below `<h3>` SHALL remain paragraph content.

#### Scenario: Render document title and supported section hierarchy
- **WHEN** a PDF contains a document title and visually distinct section and subsection headings supported by typography and layout cues
- **THEN** the title is rendered as `<h1>`, sections as `<h2>`, and nested headings as `<h3>`

#### Scenario: Preserve ordinary prose as paragraphs
- **WHEN** extracted text is ordinary body prose rather than a heading
- **THEN** it remains rendered as paragraph content and is not promoted to a heading

#### Scenario: Keep visually prominent prose as paragraphs
- **WHEN** a body paragraph is larger or more widely spaced than surrounding text but has no corroborating heading cues
- **THEN** the paragraph remains in a `<p>` element and is not promoted to `<h2>` or `<h3>`

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

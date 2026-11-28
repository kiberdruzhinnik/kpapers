# feed-markdown-export Specification

## Purpose

Allows users to download feed, year, or month content as Markdown files in a ZIP archive, preserving formatting already extracted from each PDF.

## Requirements

### Requirement: Preserve PDF emphasis in feed Markdown exports
The system SHALL convert cached sanitized HTML to Markdown while preserving semantic bold and italic emphasis in the exported files.

#### Scenario: Export cached bold text
- **WHEN** a successfully processed PDF has bold text in its cached sanitized HTML and the user exports a scope containing that PDF
- **THEN** its Markdown file represents the bold passage with `**` markers

#### Scenario: Export cached italic text
- **WHEN** a successfully processed PDF has italic text in its cached sanitized HTML and the user exports a scope containing that PDF
- **THEN** its Markdown file represents the italic passage with Markdown emphasis markers

#### Scenario: Export normally styled text
- **WHEN** a successfully processed PDF has normally styled text in its cached sanitized HTML and the user exports a scope containing that PDF
- **THEN** its Markdown file leaves that text unwrapped by emphasis markers

### Requirement: Export one document title with hierarchical headings
The system SHALL produce exactly one H1 in each Markdown file, using the first content H1 when present and the filename-derived title otherwise. Remaining headings SHALL use H2 or H3 levels without skipping a level in the document hierarchy.

#### Scenario: Content contains a title heading
- **WHEN** cached article HTML contains one or more H1 headings
- **THEN** the first content H1 is the sole H1 in the Markdown file and additional headings are normalized to H2 or H3

#### Scenario: Content has no title heading
- **WHEN** cached article HTML contains no H1 heading
- **THEN** the filename-derived title is the sole H1 in the Markdown file

#### Scenario: A source heading skips a level
- **WHEN** a source heading would create an H1-to-H3 or H2-to-H4 jump in the exported document
- **THEN** the exported heading is normalized so no heading level is skipped

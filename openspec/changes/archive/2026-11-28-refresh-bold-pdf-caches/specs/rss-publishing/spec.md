# Spec Delta

## MODIFIED Requirements

### Requirement: Include safe extracted content in RSS
The system SHALL include sanitized extracted HTML as the RSS item's description or content, including emphasis from the latest successful PDF extraction.

#### Scenario: Subscriber reads an article
- **WHEN** an RSS reader retrieves a feed containing a processed PDF
- **THEN** the item exposes its safe HTML content

#### Scenario: Publish emphasis after a forced reparse
- **WHEN** a PDF with bold text is successfully forced through the current extraction pipeline
- **THEN** the RSS item contains the refreshed `<strong>` markup for that text

## ADDED Requirements

### Requirement: Publish one hierarchical title heading in RSS content
The system SHALL publish article content with exactly one H1, using the first content H1 when present and the filename-derived title otherwise. Remaining headings SHALL use H2 or H3 levels without skipping a level.

#### Scenario: Article content has a title heading
- **WHEN** cached article HTML contains one or more H1 headings
- **THEN** RSS content contains the first content H1 as its sole H1 and normalizes remaining headings to H2 or H3

#### Scenario: Article content has no title heading
- **WHEN** cached article HTML contains no H1 heading
- **THEN** RSS content contains the filename-derived title as its sole H1

#### Scenario: Article headings skip a level
- **WHEN** an article heading would create an H1-to-H3 or H2-to-H4 jump
- **THEN** RSS content normalizes it so no heading level is skipped

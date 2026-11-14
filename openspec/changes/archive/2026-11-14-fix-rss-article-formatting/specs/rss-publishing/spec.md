# Spec Delta

## ADDED Requirements

### Requirement: Preserve article block structure in RSS content
The system SHALL publish extracted article HTML with semantic block boundaries intact so RSS readers can render titles, introductory paragraphs, and body paragraphs as separate blocks.

#### Scenario: Miniflux displays a structured article
- **WHEN** Miniflux ingests an RSS item whose PDF has a title, introductory paragraph, and body paragraphs
- **THEN** the item content presents the title, introduction, and body paragraphs as separate blocks in their original reading order

#### Scenario: RSS item contains safe structured content
- **WHEN** an RSS reader reads the item's description or encoded content
- **THEN** it receives the safe structured HTML for the article with paragraph and heading boundaries preserved

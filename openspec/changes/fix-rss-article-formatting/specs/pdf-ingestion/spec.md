# Spec Delta

## ADDED Requirements

### Requirement: Preserve title and lead as distinct content blocks
The system SHALL preserve a visually distinct document title and introductory lead as separate semantic blocks in extracted HTML, followed by body content in its own paragraphs.

#### Scenario: PDF title is followed by an introductory lead
- **WHEN** a PDF presents a visually distinct title followed by an introductory paragraph and body paragraphs
- **THEN** extracted HTML represents the title, introductory paragraph, and body paragraphs as distinct elements in the same reading order

#### Scenario: Title wraps across visual lines
- **WHEN** a document title spans multiple visual lines before an introductory paragraph
- **THEN** the title lines are combined into one title element and the introductory paragraph remains a separate element

# Spec Delta

## ADDED Requirements

### Requirement: Publish inline PDF images in RSS article content
RSS items SHALL include the safe inline images present in processed article HTML, in the same reading order and position as the processed content.

#### Scenario: Subscriber reads an article with images
- **WHEN** an eligible processed PDF contains successfully extracted images
- **THEN** the RSS item's description and encoded content include those images as inline data URIs near their associated article text

#### Scenario: Article image cannot be embedded safely
- **WHEN** an image is omitted during processing because it cannot be safely converted or embedded
- **THEN** the RSS item publishes the remaining article content without an external image reference

# Spec Delta

## ADDED Requirements

### Requirement: Include inline PDF images in Markdown exports
Markdown ZIP exports SHALL include successfully processed PDF images inline near their associated article text, without depending on external image files.

#### Scenario: Export an article with images
- **WHEN** an eligible exported entry contains successfully processed images
- **THEN** its Markdown file embeds each image as inline image data near the corresponding text in reading order

#### Scenario: Export image processing omitted an image
- **WHEN** an image could not be safely converted or embedded during PDF processing
- **THEN** the Markdown file includes the remaining article content without a broken or external image reference

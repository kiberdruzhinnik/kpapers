# Spec Delta

## ADDED Requirements

### Requirement: Embed optimized PDF images near their original reading positions
The system SHALL include supported embedded PDF images in processed article HTML near the text at their original page positions, preserving page reading order.

#### Scenario: Article contains positioned images
- **WHEN** an eligible PDF contains extractable text and embedded images with page positions
- **THEN** processed HTML includes each successfully processed image near the corresponding text in page reading order

#### Scenario: Image position cannot be determined
- **WHEN** an embedded image has no usable page position
- **THEN** the image is placed after the text for its page and before content from later pages

### Requirement: Optimize inline PDF images for web publication
The system SHALL convert embedded images to a browser-supported lossy format, limit output width to 1200 pixels, compress at quality 80, and remove image metadata before embedding.

#### Scenario: Image exceeds the width limit
- **WHEN** an extracted image is wider than 1200 pixels
- **THEN** the embedded image is resized to no more than 1200 pixels wide while preserving its aspect ratio

#### Scenario: Image already fits the width limit
- **WHEN** an extracted image is 1200 pixels wide or narrower
- **THEN** its width is not increased and it is compressed at quality 80 with metadata removed

#### Scenario: Source image format is not supported for web output
- **WHEN** an extracted image uses a format that the selected web output format cannot encode directly
- **THEN** the system decodes and converts it before lossy compression and embedding

#### Scenario: PDF image uses JPEG 2000 encoding
- **WHEN** an eligible PDF contains an embedded image using the `/JPXDecode` filter
- **THEN** the system decodes the image and processes it through the same sizing, compression, and inline embedding requirements

### Requirement: Keep embedded image markup safe and bounded
The system SHALL publish image data only through validated inline data URIs in the allowed image format, without arbitrary attributes or externally loaded image URLs.

#### Scenario: Processed image is embedded
- **WHEN** processed HTML includes an image
- **THEN** the image source is an inline data URI for the selected image format and the markup contains no executable content or unsafe attributes

#### Scenario: Image processing fails
- **WHEN** an embedded image cannot be decoded, converted, or compressed
- **THEN** the system omits that image and continues publishing the successfully extracted article text and other images

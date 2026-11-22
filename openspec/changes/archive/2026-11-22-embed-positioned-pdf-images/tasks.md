# Tasks

## 1. Image Processing Dependencies

- [x] 1.1 Add libcaesium and the in-memory image encoding support; verify the API container and CI toolchain can build the native JPEG codec dependency.
- [x] 1.2 Enable the vendored `pdf_oxide` crate's `jpeg2000` feature and verify the supplied PDF yields both extractable images.

## 2. PDF Image Extraction and Placement

- [x] 2.1 Extract images per page with bounding boxes and merge image events into the text reading order; verify images before, between, after, and without usable text bounds are placed in the expected page order.
- [x] 2.2 Decode PDF image formats, convert to JPEG, and configure libcaesium for quality 80, metadata removal, and a 1200 px maximum width without upscaling; verify dimensions, aspect ratio, output format, and metadata on representative images including both extracted images from the supplied JPEG 2000 PDF.
- [x] 2.3 Set and document image count, source dimension, decoded pixel, and total inline payload limits against the PDF input cap; verify over-limit or failed images are skipped while text and other images remain available.
- [x] 2.4 Render positioned images as constrained inline data URI elements in processed HTML and update extraction fixtures/documentation; verify safe markup and approximate placement on the supplied PDF.

## 3. Cache and Publication Integration

- [x] 3.1 Update the generated HTML policy to allow only the selected inline image data URI form and required fixed sizing; verify remote URLs, unsafe attributes, and other data URI formats remain excluded.
- [x] 3.2 Bump the extraction pipeline revision and verify an unchanged source PDF is reprocessed once, with the successful cached HTML retained if later processing fails.
- [x] 3.3 Verify RSS description and encoded content include inline images in article order, and update RSS behavior documentation if required.
- [x] 3.4 Preserve safe inline image elements when converting cached HTML to Markdown and export that HTML-derived Markdown in ZIP entries; verify images remain near article text without external files.

## 4. Integration Verification

- [x] 4.1 Run the relevant Rust checks and the Miniflux end-to-end flow with an image-bearing PDF; verify published images render inline and exports contain matching image placement.

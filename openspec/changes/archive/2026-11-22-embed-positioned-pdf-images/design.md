# Design

## Context

PDF text extraction currently keeps page-local line positions through HTML block generation, but image extraction is disabled. `pdf_oxide` exposes per-page images with bounding boxes and decoded image data. The pipeline caches extracted text and sanitized HTML; RSS serves that HTML, while Markdown ZIP export currently reads only extracted text. See proposal.md for motivation and the spec deltas for observable behavior.

## Goals / Non-Goals

**Goals:**
- Preserve page and vertical reading order when placing images among extracted text.
- Reuse one processed inline image representation for cached HTML, RSS, and Markdown ZIP export.
- Keep image processing bounded and skip an individual image on processing failure without failing the document.

**Non-Goals:**
- OCR or image understanding, captions generated from image contents, or external image hosting.
- Reproducing precise page layout, columns, or floating-image wraps in reflowable HTML.

## Decisions

- Extract images page by page and retain their page index and bounding box alongside text lines. Sort page content in reading order and insert an image block near the first text block at or below its vertical position. If there is no usable box, place it after that page's text. This keeps approximate placement while preserving the existing reflowed article structure; absolute CSS positioning would not survive RSS or Markdown rendering consistently.
- Enable the vendored `pdf_oxide` crate's `jpeg2000` feature, which provides `/JPXDecode` support through its `hayro-jpeg2000` dependency. The supplied PDF uses this filter; with the feature disabled, its image extraction returns no image even though the page content references each image. Verify the feature by extracting both images from the supplied PDF before applying the normal dimension checks.
- Decode each extracted PDF image and encode a JPEG intermediate in memory, then use the `libcaesium` Rust crate (package `libcaesium`, crate `caesium`) for final lossy compression. JPEG 2000 input is decoded before compression because libcaesium does not list it as an input format. Configure JPEG quality to 80, disable metadata retention, and set the width parameter to 1200 only when the decoded image exceeds that width; leave height unconstrained so aspect ratio is preserved and smaller images are not enlarged. JPEG is selected for broad RSS/browser compatibility; transparent pixels, if any, are composited to white during JPEG conversion.
- Embed the optimized JPEG bytes as `data:image/jpeg;base64,...` in a constrained `<img>` element. Expand the generated-content allowlist only for this exact image source form and a fixed responsive style or width attribute; do not accept arbitrary source markup or remote URLs. The existing Base64 dependency can encode the bytes.
- Keep cached `sanitized_html` as the canonical content containing both text and images; do not add a separate binary-image table or image files. Adapt Markdown conversion to preserve safe inline image elements as raw HTML in the Markdown body, and have ZIP export convert the cached HTML instead of the text-only cache column. This avoids another copy of base64 data in the database while keeping image order consistent.
- Include the image processing behavior in the extraction cache revision so existing documents are reprocessed and old text-only cache entries are replaced on refresh.
- Bound image extraction by source dimensions, image count, decoded pixel count, and total encoded inline bytes per PDF. Skip images that exceed limits or fail decoding/compression, log a document-scoped warning, and continue with text and other valid images. The implementation task will set concrete limits against the existing 25 MiB PDF input cap and document the resulting resource budget.

## Risks / Trade-offs

- Inline base64 increases HTML, RSS, Markdown, and database sizes → enforce per-image and per-document processing limits and avoid storing duplicate image payloads.
- Some RSS clients restrict data URIs → retain the image in feed content as specified; validate behavior with the project's Miniflux end-to-end path and document client limitations if observed.
- JPEG conversion can alter transparency and fine line art → composite transparency against white, use quality 80 as requested, and visually inspect the supplied PDF's images during implementation.
- An image's bounding box may overlap text or have coordinate orientation differences → normalize coordinates to the extractor's page convention and test placement against images before, between, and after text blocks.
- libcaesium adds native codec build work and raises the minimum Rust version → confirm its `mozjpeg-sys` build in the container and CI toolchains before deployment; the current upstream manifest declares Rust 1.90.
- JPEG 2000 decoding adds code and build requirements to PDF processing → keep decoding behind the extractor's feature and verify the supplied JPX images can be decoded within the existing pixel limits.

## Migration Plan

No schema migration is planned. Bumping the extraction pipeline revision causes existing cached documents to be reprocessed on refresh. If image extraction or compression fails for a document, preserve the last successful cached HTML according to the existing refresh failure behavior. Rollback removes inline-image processing and bumps/reverts the pipeline revision so newly processed content is regenerated under the deployed implementation.

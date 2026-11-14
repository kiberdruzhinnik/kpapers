# Proposal

## Why

Marker adds a Python model worker and a second extraction path that is no longer wanted. Pdfium also requires a native runtime bundled into the API image. The project should use one Rust PDF text extraction implementation, `pdf_oxide`, and the TrueNAS deployment should no longer require either extraction service or native Pdfium runtime.

## What Changes

- Remove Marker extraction, its worker service, dependencies, configuration, and feed-level engine selection; use `pdf_oxide` default text extraction as the only PDF text extraction path.
- Remove Pdfium bindings, native runtime packaging, and Pdfium-specific configuration from the API build and deployment.
- Update the TrueNAS Compose example and `.env.example` to deploy only the API and web images with `pdf_oxide` extraction.
- Update deployment documentation, CI, and PDF-ingestion and feed-management requirements to reflect the single-engine behavior.
- **BREAKING** Existing feeds configured to use Marker or Pdfium will use `pdf_oxide`; deployments must stop configuring or running the Marker worker and remove Pdfium runtime configuration.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: require `pdf_oxide` as the sole supported extraction engine and retain the shared cleanup, cache, and failure behavior for that path.
- `feed-management`: remove operator selection of a per-feed extraction engine.
- `container-deployment`: publish only API and web images and make the TrueNAS example use the `pdf_oxide` implementation without Pdfium native runtime packaging.

## Impact

Affected areas include Rust extraction and text-to-publication rendering, feed configuration, database migrations for stored engine overrides, web feed-management controls, Marker worker source and fixtures, Pdfium bindings and native runtime packaging, CI and container publishing, Compose files, and deployment documentation. `pdf_oxide` default text extraction becomes the sole PDF text extraction implementation.

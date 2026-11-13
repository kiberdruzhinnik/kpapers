# Proposal

## Why

Feeds need an alternative PDF extraction engine that can use Marker while retaining the existing publication cleanup and safety behavior. Operators need to select it globally or for individual feeds.

## What Changes

- Add Marker as a selectable backend extractor, alongside the existing Pdfium extractor.
- Provide a global default and per-feed inherit, Pdfium, or Marker selection; Marker handles every PDF when selected.
- Preserve existing cleanup, classification, sanitization, and last-successful-content behavior for both extractors.
- Reprocess cached documents on a subsequent refresh when their effective extractor changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Select extraction engines and preserve processing and cache behavior across them.
- `feed-management`: Configure a feed's extraction engine override.

## Impact

The Rust API, SQLite feed configuration and cache identity, management UI, refresh pipeline, deployment configuration, and tests change. A Python Marker runtime and its model dependencies are added; deployment and model licensing must be documented.

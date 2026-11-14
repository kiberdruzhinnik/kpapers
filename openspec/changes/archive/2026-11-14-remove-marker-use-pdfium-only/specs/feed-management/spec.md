# Spec Delta

## REMOVED Requirements

### Requirement: Configure feed extraction engine
**Reason**: `pdf_oxide` is the only supported extraction engine, so a per-feed choice or inheritance setting no longer changes system behavior.
**Migration**: Remove the feed engine selector and persisted engine override. Migrate existing feeds with Marker or Pdfium overrides to the single `pdf_oxide` behavior, and reprocess their cached content with `pdf_oxide` on refresh.

# Proposal

## Why

Authors publish article PDFs in monthly folders on cloud drives, but readers need a reliable RSS feed and the operator currently has to find, extract, and republish each document manually. This change creates a single-operator web application that discovers eligible PDFs, extracts and caches their text, and exposes stable RSS feeds with controllable refresh behavior.

## What Changes

- Add an operator web interface and Rust API for managing author feeds, years, optional months, and cloud folder links.
- Let the operator reversibly archive or permanently delete feeds, years, and months, with deletion of descendants when a parent is permanently deleted.
- Keep management forms and action controls consistently aligned across feed, year, and month workflows, including a readable, clearly grouped month-detail layout on narrow and wide screens.
- Give each feed a persistent ID independent of its display name and public URL slug.
- Discover PDF files only at the linked folder's root; do not traverse subfolders. Publish each non-promotional PDF as its own RSS item.
- Extract text from PDFs, convert it to a restricted set of safe HTML tags, and cache the source metadata and processed content.
- Refresh the current month more frequently, allow per-feed refresh interval overrides, and provide force-refetch controls for a feed, year, or month.
- Expose public RSS endpoints with stable channel and item identifiers, and link each item through a stable application URL that redirects to the exact provider PDF download rather than a cloud folder or file-view page.
- Provide Dockerfiles for separate web and Rust API containers, plus Docker Compose configuration for building, running, health-checking, and persisting the application.
- Provide paired local quickstart and cleanup scripts that generate isolated non-production settings, start the Compose stack for human testing on all host interfaces, and remove only that test stack and its data.
- Add automated end-to-end coverage that exercises public feeds through Miniflux as an RSS reader.

## Capabilities

### New Capabilities

- `feed-management`: Operator-managed feeds, years, month links, stable feed identity, and promotional-content overrides.
- `pdf-ingestion`: Root-only cloud folder listing, PDF discovery, promotional PDF exclusion, text extraction, safe HTML conversion, and caching.
- `rss-publishing`: Public RSS channels and stable identifiers for PDF entries.
- `refresh-scheduling`: Current-month refresh priority, per-feed intervals, and scoped force refetch.
- `local-quickstart`: Isolated, non-production environment generation and lifecycle scripts for local human testing.

### Modified Capabilities

None.

## Impact

- Greenfield Rust API and background refresh worker, plus a web interface for the single operator.
- Containerized deployment with separate web and API services, Docker Compose orchestration, and persistent storage for SQLite and cached content.
- Cloud provider integrations for Yandex Disk and Mail Cloud public links.
- Persistent storage for feed hierarchy, source file metadata, extracted content, cache state, and refresh schedules.
- Public RSS endpoints consumed by third-party readers.
- PDF parsing and HTML sanitization dependencies; provider-specific folder listing and download behavior must be isolated behind adapters.

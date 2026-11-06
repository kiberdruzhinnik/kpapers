# Tasks

## 1. Application foundation

- [x] 1.1 Create the Rust API and web client scaffold with separate multi-stage Dockerfiles; verify both images build successfully.
- [x] 1.2 Add SQLite configuration, migrations, and repository boundaries for feeds, hierarchy, source documents, cache, and refresh jobs; verify a fresh database initializes and migrations are repeatable.
- [x] 1.3 Add single-operator authentication and separate management routes from public RSS routes; verify management requests require a session while public routes remain readable.

## 2. Operator feed management

- [x] 2.1 Implement feed creation, editing, deactivation, and persistent UUID-based RSS URLs; verify changing a feed name does not change its ID or URL.
- [x] 2.2 Implement optional years and operator-created month links; verify a year may omit any month and each month can store a supported public folder link.
- [x] 2.3 Implement source-file review and persistent promotional include/exclude overrides; verify an operator can reverse an automatic classification.
- [x] 2.4 Document the operator's feed and month management workflow; verify the documented workflow matches the web client.
- [x] 2.5 Implement reversible archive and restore for feeds, years, and months; verify archived scopes and descendants are excluded from management, refresh, and RSS, then return with IDs and data intact after restoration.
- [x] 2.6 Implement confirmed permanent deletion for feeds, years, and months with descendant cleanup; verify deleted data and RSS items are removed and feed IDs are never reused.
- [x] 2.7 Align feed, year, and month forms and action controls consistently across desktop and narrow layouts; verify controls wrap without overflow and buttons do not stretch unpredictably.
- [x] 2.8 Update the operator guide to explain archive, restore, and permanent deletion; verify the guidance matches the UI.
- [x] 2.9 Refine linked-month layout so the month name, source link, refresh metadata, and action group are clearly separated and readable; verify controls wrap or stack without overlap or status-text clipping on desktop and narrow screens.

## 3. Cloud provider adapters

- [x] 3.1 Implement the Yandex Disk public-folder adapter for direct-child listing, metadata, and downloads; verify the supplied October and November examples list their direct PDFs and do not descend into folders.
- [x] 3.2 Implement the Mail Cloud public-folder adapter for root listing and dispatcher-based downloads; verify the supplied Mail examples list only root PDFs and ignore nested folders.
- [x] 3.3 Validate source URLs, redirect destinations, download size, and timeouts against provider allowlists; verify unsupported hosts and unsafe redirects are rejected.

## 4. PDF processing and caching

- [x] 4.1 Evaluate Rust-compatible PDF text extractors against representative supplied PDFs and document the selected parser and observed encoding/layout limits; verify it extracts native text from both providers' samples.
- [x] 4.2 Implement bounded PDF extraction, whitespace/encoding cleanup, and allowlist-based HTML generation; verify generated content contains only allowed tags and has no attributes or executable markup.
- [x] 4.3 Implement promotional classification from filenames and extracted first-page text, default ambiguous files to publish, and apply operator overrides; verify the standalone club-promotion PDF is excluded while article PDFs remain publishable.
- [x] 4.4 Implement source fingerprint caching and preserve-last-good behavior; verify unchanged files reuse cached extraction and failed refreshes keep the previous successful content.
- [x] 4.5 Document supported PDF behavior and how extraction or classification failures appear to the operator; verify the guidance matches the UI status and error messages.

## 5. RSS publishing

- [x] 5.1 Implement RSS 2.0 output and stable channel self URLs from feed UUIDs; verify a feed rename leaves its public URL unchanged.
- [x] 5.2 Implement one item per eligible PDF with persistent document GUIDs, sanitized content, source link, and publication-date fallback; verify a changed source file updates the existing item without changing its GUID.
- [x] 5.3 Add a stable public application download route per document, resolve a fresh provider-issued download URL for its exact source path, and redirect without proxying bytes; verify Mail Cloud `weblink` and Yandex `?path=` viewer URLs are not used as RSS item links.
- [x] 5.4 Validate the stable redirects against the supplied Mail Cloud and Yandex Disk examples with `curl`; follow each redirect and confirm it returns the existing PDF bytes, including a `%PDF-` signature, for the exact named file.

## 6. Refresh scheduling and controls

- [x] 6.1 Implement persisted scheduled refresh jobs with configurable defaults of one hour for the current month and 24 hours for older months; verify the current month is scheduled more frequently and advances with the calendar.
- [x] 6.2 Implement per-feed interval overrides and display last attempt, next refresh, and latest result; verify the configured override is used for that feed.
- [x] 6.3 Implement immediate forced refetch at feed, year, and month scope, bypassing normal schedule and fingerprint checks; verify each scope queues only its configured descendants.
- [x] 6.4 Document refresh defaults, per-feed overrides, force-refetch controls, and cache backup/restore; verify the documented settings map to runtime configuration.

## 7. Containerized operation

- [x] 7.1 Connect operator actions, provider adapters, PDF processing, cache, scheduler, and RSS routes; verify a configured month produces public RSS items from its root-level non-promotional PDFs.
- [x] 7.2 Add Docker Compose orchestration for the web and API containers, health checks, private service networking, runtime configuration, and a named persistent SQLite/cache volume; verify `docker compose up --build` starts both healthy services and RSS is reachable through the web entry point.
- [x] 7.3 Add deployment and database backup instructions for the single-instance Compose deployment; verify application data survives container recreation and can be restored from backup using the documented steps.
- [x] 7.4 Add paired quickstart and cleanup scripts for a non-production Compose project bound to all host interfaces, with generated operator credentials and a separate data volume; verify login and RSS work, cleanup removes quickstart resources, and the normal deployment and data remain untouched.

## 8. End-to-end RSS reader verification

- [x] 8.1 Add an opt-in end-to-end test environment that runs the application and Miniflux with dedicated disposable configuration, storage, and cleanup; verify it does not alter the normal or quickstart Compose deployments.
- [x] 8.2 Add deterministic PDF/provider fixtures and an end-to-end scenario that creates a feed and month through the operator interface, ingests eligible PDFs, excludes promotional PDFs, subscribes Miniflux to the stable RSS URL, and verifies the expected entries, safe content, and source links.
- [x] 8.3 Verify in Miniflux that a re-fetch preserves each source item's GUID without producing duplicates, and that changed content for the same provider file updates the existing entry; verify the test can trigger and await refresh with bounded timeouts.
- [x] 8.4 Verify in Miniflux that each entry links to a stable application download route, then follow it and confirm the redirect targets that entry's individual PDF download rather than a viewer page or shared cloud folder.

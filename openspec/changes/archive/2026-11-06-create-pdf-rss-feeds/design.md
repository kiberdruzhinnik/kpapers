# Design

## Context

This repository has an OpenSpec root but no application code or existing capability specs. The operator is the sole manager; RSS readers are public consumers. Inspection of the supplied links showed root folders with multiple PDFs, nested folders that must be ignored, text-bearing PDFs, and a standalone promotional PDF. Native text extraction worked on samples but showed encoding and spacing artifacts, so parser quality and text cleanup need explicit attention.

See `proposal.md` for motivation and the capability specs for externally visible behavior.

## Goals / Non-Goals

**Goals:**
- Keep operator management, public RSS reads, and scheduled ingestion in a small Rust service with a separate web client.
- Isolate Yandex Disk and Mail Cloud behavior so provider changes do not affect feed or RSS identity.
- Keep cached entries available through transient cloud or parsing failures.
- Make the supported PDF extraction and HTML sanitization behavior bounded and reviewable.

**Non-Goals:**
- User accounts, author self-service, or multiple operator roles.
- Recursing through cloud folders, importing non-PDF files, or OCR for image-only PDFs in the first version.
- Modifying or uploading source documents to cloud drives.
- Republishing or proxying the original PDF bytes to RSS readers.

## Decisions

### Rust API, web client, and containers

Use a Rust HTTP API with the background scheduler in the same process, plus a browser client for operator workflows. Build separate multi-stage Docker images for the API and web client. The `api` container runs the Rust HTTP service and scheduler; the `web` container serves static client assets and reverse-proxies management API and RSS paths to the API over a private Compose network. This provides the requested API/web split while keeping the scheduler single-instance.

Provide a Docker Compose file that builds and starts both containers, waits for API health before marking the web service ready, exposes the web entry point, and mounts a named persistent volume for the SQLite database and cached content. Run containers as non-root users and pass configuration through environment variables; keep operator credentials and other secrets out of the image and Compose source. Do not add a separate database container while SQLite is the selected store. Run one API replica because its embedded scheduler and SQLite access are designed for a single instance.

### Local human-test quickstart

Provide `scripts/quickstart.sh` and `scripts/quickstart-down.sh` for a disposable non-production run. The start script checks for Docker Compose and OpenSSL, uses Docker directly when permitted or `sudo docker` when required, creates a mode-600 `.env.quickstart` with a random operator password and signing key, hashes the password using the API image helper, and starts the stack with the fixed `kpapers-quickstart` Compose project and its separate named volume. Use a dedicated local port and bind the test web entry point to `0.0.0.0` so it is reachable through any host interface; print the local URL and generated operator credentials after startup. Do not read or modify `.env` or the default Compose project.

The cleanup script targets only the fixed quickstart project and `.env.quickstart`. It stops and removes that project's containers and network, deletes its named volume and local images, and removes the generated environment file. Its output must make clear that quickstart database contents are deleted. The scripts must never accept an alternate project or volume name that could point cleanup at a normal deployment.

### Persistent storage and identifiers

Use SQLite with WAL mode for feeds, years, optional months, source-file records, extracted text/HTML, refresh state, and operator overrides. The single-operator workload is small enough to avoid a separate database service; a single backend instance also avoids competing scheduler workers. Mount the database and cached content under the same named Docker volume and back them up together.

### Operator removal and interface layout

Support both reversible archive/restore and permanent deletion for feeds, years, and months. Archiving preserves identifiers, source links, cached documents, and refresh history while suppressing the archived scope and its descendants from active management, scheduled refresh, and RSS. Restoring returns the retained data to active use. Permanent deletion is a separate, clearly labeled action that requires confirmation and removes the selected scope and descendants, including source documents, cached content, and refresh jobs. Deleting a feed ends its RSS channel; creating a new feed always assigns a new ID.

Use shared form and action styles across feed creation/editing, year creation, and month linking. Keep fields and labels aligned, size action buttons to their content instead of stretching them across flex rows, and allow controls to wrap on narrow screens without horizontal overflow.

Give the month-linking form its own labeled section, visually separate from year actions and existing month cards. In each month card, show the month name first, then the source link in a readable line that can wrap or truncate safely, followed by refresh metadata in a compact but legible block. Put month actions in a distinct, consistently ordered action group below the metadata; keep destructive deletion visually distinct from routine actions. At narrow widths, stack or wrap these groups without allowing buttons to overlap, obscure status text, or force metadata into a narrow column. The month name, source, status, and actions must remain scannable without relying on position alone.

Generate and persist an opaque UUID for every feed; derive the permanent public RSS path from that ID, never from the editable name. Persist a separate UUID for each discovered document. Match Yandex files using the provider resource ID and Mail files using their root-relative weblink; use provider hashes/revisions as change fingerprints, not as the item ID. If a provider file is renamed and no stable provider ID is available, treat it as a new source unless the operator explicitly reconciles it.

### Provider adapters and root-only discovery

Define a provider adapter boundary for validating a public link, listing direct children, returning file metadata, and downloading a file. The Yandex adapter uses the public-resource listing/download APIs. The Mail adapter encapsulates the public-folder listing and dispatcher-based download flow observed for the supplied examples; it is less stable because the public API surface is not clearly documented. A provider's `weblink` or a Yandex URL with a `path` query may be a folder/file-view page and must not be assumed to download the PDF. For every source document, retain the provider path and resolve a fresh provider-issued direct-download URL when a public download link is requested. Paginate only the configured folder's direct listing if required by the provider; ignore directory entries and never request their contents.

Accept only HTTPS public links on the supported provider hosts. Do not accept arbitrary download URLs from operators or trust a provider-supplied redirect without validating the destination host. Apply request timeouts and a configurable maximum PDF size before parsing. Keep provider response details and ephemeral download URLs out of logs.

### PDF processing, promotion filtering, and cache

Store source metadata and a content fingerprint before processing. If the fingerprint is unchanged, reuse cached text and HTML. For a changed file, download to bounded temporary storage, extract native text, normalize common encoding/line-break issues, and regenerate sanitized HTML. Keep the last successful cached version if the new download or extraction fails.

Put PDF extraction behind an interface so a Rust-compatible parser can be selected after evaluating it against the supplied samples. Do not make OCR part of the initial pipeline; mark files with no usable native text as failed and visible to the operator. The samples suggest text extraction is feasible, but observed NUL/replacement characters and spacing loss mean output quality must be checked before publishing.

Classify promotional documents using filename, extracted title, and first-page text signals. Exclude clearly promotional documents by default; do not classify an article as promotional merely because it contains a banner, links, or subscription references. Persist the classification and allow the operator to force include or exclude. Default ambiguous cases to publish, matching the operator's rule to publish all articles and skip promotional PDFs.

Generate HTML from extracted text using paragraph and heading heuristics, then enforce a small allowlist (for example `p`, `br`, `h1`-`h4`, `strong`, `em`, `ul`, `ol`, `li`, and `blockquote`). Strip all attributes and disallowed markup; represent URLs as text rather than active links. Store extracted plain text separately so HTML can be regenerated if formatting rules change.

### RSS identity and item metadata

Expose RSS 2.0 at a stable URL derived from the feed UUID, with an Atom self link using the same URL. Use each persisted document UUID as its non-permalink item GUID. A content change for the same provider file updates the existing item and retains its GUID. Prefer a publication date parsed from the filename or document text; fall back to provider modification time, then discovery time. Set each item's link to a stable application download route keyed by the persisted document UUID. On request, the route must look up the exact provider file using its stored provider identity and root-relative path, obtain a fresh provider-issued direct-download URL, validate the destination host, and return a redirect without proxying PDF bytes. Do not publish Mail Cloud `weblink` or Yandex `?path=` viewer URLs as the item's link. Route this public download path through the web reverse proxy. Validate the redirect against the supplied Mail and Yandex samples by following it and confirming the response is a PDF, including its PDF signature; a successful response from a viewer page is not sufficient.

### Scheduling and forced refresh

Run due refresh jobs inside the single backend process and persist job state so a restart does not lose operator requests. Use initial configurable defaults of one hour for the current month and 24 hours for older months. A per-feed interval override replaces these defaults for that feed. Forced refreshes enqueue work immediately at feed, year, or month scope and bypass fingerprint checks; they do not remove the old cached item while the replacement is processed. Show queued/running/completed/failed status to the operator.

### Operator access and public access

Require a single operator login for management APIs and pages; keep RSS reads unauthenticated. Store password hashes rather than plaintext credentials and protect browser sessions with secure, HTTP-only cookies. Keep public feed endpoints read-only and avoid exposing cloud access tokens or provider download URLs through the management API.

### End-to-end verification with Miniflux

Add an opt-in, isolated end-to-end test environment that starts the application and Miniflux with disposable configuration and storage. Keep Miniflux out of the normal production and quickstart Compose stacks. The test should drive the operator workflow against the running web/API application, use deterministic local PDF fixtures (including multiple eligible documents and a promotional document), wait for ingestion, then subscribe Miniflux to the resulting stable public RSS URL. Verify channel discovery, eligible item count, safe item content and source links, stable GUIDs without duplicate entries after re-fetch, and updated content for a changed fixture with the same provider identity. Ensure test cleanup removes only the dedicated end-to-end resources. Avoid depending on live cloud providers or timing-sensitive scheduled refreshes; trigger refreshes explicitly and wait with bounded timeouts.

## Risks / Trade-offs

- [Mail Cloud changes its undocumented public listing or download behavior] -> Keep the adapter isolated, record provider errors, and retain the last good cache; validate the adapter against real shared folders during implementation.
- [PDF text extraction yields corrupted or misordered text] -> Compare parser output with the supplied samples, normalize only safe whitespace/encoding artifacts, and surface low-quality or failed extraction for operator review.
- [Promotional classifier excludes an article or publishes an advert] -> Make the decision visible and reversible per document; default ambiguous files to publication.
- [A malicious PDF or provider redirect attacks the service] -> Allowlist provider and downloader hosts, validate redirects, cap file size and processing time, and sanitize generated HTML.
- [SQLite becomes a bottleneck if refresh concurrency or feed readership grows] -> Keep data access behind repository boundaries and move to PostgreSQL only if measured load or multi-instance operation requires it.

## Migration Plan

There is no existing application data to migrate. Configure the single operator credential and runtime settings, then use Docker Compose to build and start the web and API containers with the persistent data volume. Create feeds and month links, then run initial ingestion. Roll back by stopping the Compose services and restoring the database and cache backup into the named volume; public RSS endpoints become unavailable during rollback.

## Open Questions

- Exact default refresh intervals and maximum PDF size can be adjusted from configuration after operating the first feeds; the initial values above are starting defaults.
- The extraction library should be selected after the implementation spike compares output against representative Yandex and Mail PDFs.

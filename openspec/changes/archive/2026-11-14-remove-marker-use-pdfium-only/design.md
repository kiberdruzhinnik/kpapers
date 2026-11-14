# Design

## Context

The API currently selects Pdfium or Marker globally and per feed. Marker runs as a separate Python service and is included in local Compose, CI, image publishing, and the TrueNAS stack. Pdfium is integrated through `pdfium-render` and requires a bundled native runtime. This change removes both existing engines and uses the Rust `pdf_oxide` crate directly in the API. See proposal.md for motivation and the spec deltas for the behavior contract.

## Goals / Non-Goals

**Goals:**
- Keep one extraction path, `pdf_oxide`, for all feeds and deployments.
- Remove Marker-specific runtime, deployment, operator, and CI configuration.
- Remove Pdfium bindings and native runtime packaging from the API image.
- Migrate persisted per-feed engine choices so startup and refresh remain compatible with existing databases.
- Ensure Marker- and Pdfium-produced cached content is reprocessed by `pdf_oxide` while retaining valid cache entries produced by the current pipeline when their identity matches.
- Make the TrueNAS stack pull and run only the API and web images with no separate extraction service or Pdfium runtime.

**Non-Goals:**
- Add OCR. Scanned-only PDFs without a usable text layer continue to be reported as extraction failures.
- Change PDF cleanup, layout reconstruction, HTML sanitization, or feed publication behavior beyond adapting them to `pdf_oxide` output and removing engine-specific paths.

## Decisions

### Use `pdf_oxide` as the only extraction implementation

Remove engine choice from extraction settings and feed configuration. Route ingestion through `PdfDocument::extract_text_lines`, which keeps each line's reading order, font size, bold/italic runs, and geometry while still using `pdf_oxide`'s default text extraction without OCR or image embedding. Treat the extracted lines as extractor output, then apply the existing publication cleanup, text-to-HTML rendering, and HTML sanitization pipeline. Package no Pdfium native runtime in the API image. Verify that page boundaries and reading order meet the application’s supported publication behavior.

Alternative considered: keep Pdfium as the only engine. That retains a native library runtime dependency and does not meet the requested Rust implementation choice.

### Migrate per-feed engine state and cache entries

Add an ordered database migration that removes the `extraction_engine` feed override after upgrading existing databases. Update row models and API payloads with the migration. Treat the API contract change as breaking: clients must stop sending or displaying per-feed engine values. Existing Marker and Pdfium cache fingerprints must not be reused as `pdf_oxide` output; the next refresh reprocesses those documents. Preserve successful cache reuse for entries produced by the current `pdf_oxide` pipeline. Do not silently fall back to another engine when extraction fails.

Alternative considered: retain the column and ignore its values. That leaves obsolete state in the data model and permits misleading API responses, so remove it through a forward migration instead.

### Remove Marker and Pdfium deployment and publishing artifacts

Remove the Marker worker source/runtime, Rust HTTP client and response adapters, Marker-only test fixtures and E2E flow, compose profile and test compose file, Marker CI job, and GHCR image matrix entry. Remove `pdfium-render`, Pdfium binding code, native runtime downloads, and related image packages. Keep `pdf_oxide` in the API image. Update operator and deployment documentation so no Marker setup, model volume, worker URL, Pdfium runtime, or engine selection remains.

Alternative considered: keep Pdfium as an optional external component. That preserves a native runtime dependency and a second engine contract the change is intended to remove.

### Correct the TrueNAS stack to match the remaining services

Update `deploy/truenas/compose.yaml` and `.env.example` to run only API and web and use `pdf_oxide` in the API process. Remove Marker image, worker address, worker limits, model volume, and Pdfium-specific runtime configuration. Keep the existing API data bind mount at `./kpapers_data:/data`, `568:568` runtime identity, colocated `.env`, API health dependency, and external Nginx Proxy Manager network. Keep the documented directory ownership setup because it is needed for the TrueNAS dataset bind mount.

Alternative considered: move the TrueNAS database to a named Docker volume. The current deployment is designed around an operator-visible app directory and documents its bind-mount permissions; changing extraction libraries does not require changing that storage contract.

## Risks / Trade-offs

- `pdf_oxide` default text extraction returns plain text and does not expose Markdown heading or emphasis markup → Verify text ordering, page boundaries, cleanup, and safe HTML using representative fixtures.
- Scanned-only PDFs remain unsupported without OCR → Keep extraction error behavior and state clearly in operator documentation that the application extracts available text-layer content without OCR.
- Existing deployments can still carry Marker/Pdfium configuration → Update the TrueNAS example and general deployment docs, remove obsolete variables, and reject unsupported engine values if engine configuration remains.
- Removing a feed column requires a safe SQLite table migration → Preserve every other feed field and validate the migration against an existing database with both Marker and Pdfium overrides.
- A deployment may keep an old Marker container running after upgrade → Remove it from Compose and document that the updated stack no longer needs it; the service has no persistent application data beyond its disposable model cache.
- Removing Pdfium's native runtime may change container build and runtime requirements → Verify the production API image builds and runs with `pdf_oxide` without system Pdfium libraries.

## Migration Plan

1. Add the database migration and simplify feed models, API payloads, and UI so feeds no longer store or expose engine selection.
2. Route refreshes through `pdf_oxide`, adapt and test its output, and remove Marker and Pdfium runtimes, tests, fixtures, CI, Compose, and container-publishing configuration.
3. Update TrueNAS and documented deployment configuration; remove Marker and Pdfium runtime settings while retaining the existing data directory and permissions setup.
4. Upgrade existing deployments with the API migration. Remove any old Marker service and model volume and Pdfium runtime configuration when recreating the stack.
5. On the next refresh, Marker- and Pdfium-fingerprinted cached documents are processed with `pdf_oxide`. If a deployment must roll back, restore the prior image and its pre-upgrade database backup because the new schema removes the feed engine column.

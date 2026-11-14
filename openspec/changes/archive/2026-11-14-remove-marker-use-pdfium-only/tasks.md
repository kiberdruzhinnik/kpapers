## 1. Remove per-feed engine selection

- [x] 1.1 Add a forward SQLite migration that removes the feed `extraction_engine` override while preserving all other feed data; verify migration against databases containing both `marker` and `pdfium` overrides.
- [x] 1.2 Remove feed engine fields and validation from Rust models and API inputs/outputs; verify API and database tests cover creation, update, and serialization without an engine field.
- [x] 1.3 Remove the per-feed engine selector and its client types from the web UI; verify feed-management tests confirm the selector is absent and existing feed operations still work.

## 2. Use pdf_oxide as the only extraction path

- [x] 2.1 Route every refresh through the Rust `pdf_oxide` crate and remove Marker and Pdfium clients, bindings, response conversion, settings, native runtime packaging, and engine-selection branches; verify Rust tests cover default text-line extraction output, extraction failures, page reading order, and the shared cleanup and safe text-to-HTML pipeline.
- [x] 2.2 Ensure old Marker and Pdfium cache fingerprints cannot be reused for `pdf_oxide` and current `pdf_oxide` cache entries remain reusable; verify refresh tests show old-engine cached documents are reprocessed and failed extraction preserves the last successful content.
- [x] 2.3 Remove Marker- and Pdfium-specific unit and end-to-end fixtures and coverage, and document that `pdf_oxide` extracts available text-layer content without OCR; verify the remaining PDF end-to-end flow covers multi-page text extraction, cleanup, and failure for scanned-only input without OCR.

## 3. Remove Marker and Pdfium runtime/build integration

- [x] 3.1 Remove Marker worker files and Python dependencies, Pdfium bindings and native runtime files, Marker Compose profiles, Marker-specific E2E Compose configuration, and obsolete worker scripts; verify repository searches and Compose configuration contain no active Marker or Pdfium runtime references.
- [x] 3.2 Remove Marker CI jobs and its GHCR image build while retaining API and web publishing; verify workflow configuration has only those two image targets and expected publishing tags.
- [x] 3.3 Remove Marker and Pdfium settings and setup instructions from general operator and deployment documentation; verify documented environment variables match the remaining application configuration.

## 4. Fix the TrueNAS deployment for pdf_oxide

- [x] 4.1 Update `deploy/truenas/compose.yaml` and `.env.example` to run only API and web, use the in-process `pdf_oxide` implementation, and remove Marker image, worker settings, model volume, and Pdfium runtime configuration; verify `docker compose config` resolves the deployment with a populated test `.env`.
- [x] 4.2 Update the TrueNAS guide for the two-image stack and `pdf_oxide` behavior while retaining database bind-mount ownership, `568:568`, `.env`, and Nginx Proxy Manager instructions; verify each documented command and path against the deployment files.

## 5. Verify upgrade and deployment integration

- [x] 5.1 Verify a production-style database upgrade removes feed engine overrides without losing feed data and the API starts only after migrations complete.
- [x] 5.2 Build and run the updated TrueNAS stack with API and web images; verify web health and API routing, database persistence across container recreation, and successful `pdf_oxide` default text extraction without OCR or a Pdfium native runtime.

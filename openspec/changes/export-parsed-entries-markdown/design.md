# Design

## Context

The operator API already runs behind session middleware. Cached content stores each document's extracted text, sanitized HTML, and extraction status. RSS publication selects active hierarchy scopes, cached documents, successful content, and documents whose effective promotion decision is publish. The dashboard loads feed, year, and month data in the existing hierarchy.

## Goals / Non-Goals

**Goals:**
- Reuse the publication eligibility rule for export inclusion.
- Return downloadable ZIP archives at feed, year, and month scope.
- Preserve `<feed>/<year>/<month>/<entry>.md` paths inside each archive.
- Generate exports from cached extraction results without contacting source providers or parsing PDFs again.

**Non-Goals:**
- Export PDFs or source-folder files.
- Include archived scopes, failed or pending extractions, or entries excluded from publication.
- Add a standalone bulk-export workflow unrelated to the feed hierarchy.

## Decisions

- Add protected management endpoints for feed, year, and month exports. Keep them under the existing management router so session authentication remains consistent.
- Require a valid operator session for every export request; unauthenticated requests are rejected by the existing management session middleware before archive data is generated or returned.
- Select export rows by scope and apply the same effective publication decision as RSS, along with successful extraction and active feed/year/month constraints. This includes operator-included promotional classifications and excludes operator-excluded publish classifications.
- Build ZIP files in memory from `cached_content.extracted_text`. Store the extracted text as the Markdown body with the document title as a heading; derive the title from the source filename without its PDF extension.
- Use stable, sanitized path components for feed display name, year, and month. Use the PDF's safe basename with a `.md` extension for the entry; resolve duplicate generated names deterministically so entries do not overwrite one another.
- Name the ZIP download from the selected feed, year, or month scope. Empty scopes return a valid ZIP with no entry files.
- Add export controls to the existing feed detail, year, and month action groups. The client downloads the binary response as a file and reports request failures through the existing dashboard feedback pattern.
- Add a ZIP generation dependency if the current dependency set does not provide ZIP support; no database migration is needed.

## Risks / Trade-offs

- Large feeds can require substantial memory while assembling an in-memory archive → keep the initial implementation bounded to the selected scope and use only cached content; consider streaming if actual archive sizes demand it.
- Unsafe or duplicate filenames can produce invalid or overwritten ZIP entries → normalize archive path components and make duplicate entry names unique.
- Eligibility drift between RSS and export could produce surprising contents → centralize or directly share the effective publication predicate where practical, and cover override cases in API tests.

## Migration Plan

No data migration is required. Deploy the API and dashboard together; rollback removes the export controls and routes without changing stored data.

# Design

## Context

The provider modules already validate supported Mail Cloud and Yandex Disk public links, list direct child PDFs, download files with HTTPS/public-address and redirect checks, and accept the configured proxy. `pdf_processing::extract_pdf` performs the publication cleanup and currently returns normalized text and semantic safe HTML. The application binary is an API server; its modules are not yet exposed as a reusable library. The API uses a 25 MiB default PDF limit configurable through `MAX_PDF_BYTES`.

## Goals / Non-Goals

**Goals:**

- Reuse provider validation, download protections, and PDF cleanup rather than duplicate them in a separate implementation.
- Produce readable Markdown files from the cleaned semantic extraction, retaining document headings, emphasis, and blockquotes where the PDF processor identifies them.
- Provide the same CLI behavior on a host and in a standalone container with a mounted output directory.
- Publish one extractor image tag that resolves to `linux/amd64` and `linux/arm64` variants.

**Non-Goals:**

- Recursively scan subfolders, ingest non-PDF files, or use OCR for scanned PDFs.
- Store the source PDFs in the output directory; downloaded bytes are processed in memory and released after each file.
- Change web application ingestion, database, or RSS behavior.

## Decisions

1. **Expose shared application modules for a second binary.** Put reusable provider, proxy, and PDF processing modules behind the crate library surface, then add a dedicated `kpapers-extract` binary. This keeps the API and CLI on the same implementations. A separate implementation would duplicate provider URL validation and download safeguards; turning the API into a CLI subcommand would unnecessarily couple batch operation to server startup behavior.

2. **Use a positional URL and output directory interface.** The command accepts `<PUBLIC_FOLDER_URL> <OUTPUT_DIR>`, creates the output directory (including missing parents), and writes one `.md` file per direct child PDF. Host and container invocations use the same arguments; the container documents a host mount at `/output`.

3. **Render Markdown from the cleaned extraction structure.** Add Markdown output alongside the existing `full_text` and HTML outputs, using the same page cleanup, title repair, and semantic block decisions. The renderer maps identified titles/headings, bold and italic runs, and blockquotes into Markdown. This avoids converting HTML through a second parser and prevents the CLI from bypassing publication cleanup. Scanned-only files remain errors because the current processor does not perform OCR.

4. **Write safe, deterministic names and only Markdown artifacts.** Derive each destination name from the provider filename, replace path separators and unsafe path components, and add a deterministic disambiguator if sanitized names collide. Write the Markdown result in the destination folder; do not persist the source PDF. A later run may replace the Markdown file for the same source name so reruns update generated content.

5. **Continue after per-file failures.** Treat source-folder listing and output-directory setup failures as fatal. For each listed PDF, report its filename and a sanitized reason, continue with the next item, and return a nonzero process status if any file failed. Emit a final count summary. This allows partial folders to produce useful results without hiding failures.

6. **Keep runtime configuration aligned with ingestion.** Honor `MAX_PDF_BYTES` with the existing default of 25 MiB and `KPAPERS_PROXY_URL` for provider requests. Reject malformed proxy configuration before fetching. Preserve the current safe download controls, including destination and redirect validation and response-size enforcement.

7. **Build a dedicated minimal runtime image and publish a multi-platform manifest.** Add an extractor image using the Rust binary and CA certificates, without database volumes, API health checks, or web assets. Extend the existing GHCR workflow to scan the `amd64` build and publish the extractor's shared tags for both Linux architectures, leaving existing API and web image behavior intact. Run the extractor as a non-root user and document the output mount's write-permission requirement.

## Risks / Trade-offs

- **Markdown may not represent every nuance of PDF layout** → Use the same semantic headings, emphasis, and blockquote classifications as the current HTML path; keep extraction content and cleanup shared.
- **A source may contain many or very large PDFs** → Enforce the existing per-file byte limit, process files sequentially, and avoid retaining downloaded PDFs after conversion.
- **Container output permissions vary by host** → Run as a non-root user and document running with a matching host UID/GID or granting write access to the mounted directory.
- **Multi-platform builds can expose architecture-specific build failures** → Build both platforms in CI before publishing tags and keep the existing image scan on the Linux amd64 runtime image.

## Migration Plan

No database or deployment migration is needed. Add the extractor image to the publishing workflow and publish its existing default-branch, commit, and version tags after the normal build and scan steps succeed. Rollback consists of using the previous extractor image tag; the API and web images remain independently usable.

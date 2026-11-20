# Tasks

## 1. Shared Markdown extraction

- [x] 1.1 Expose the provider, proxy, and PDF-processing modules through a reusable crate library while preserving API behavior; verify existing Rust checks pass.
- [x] 1.2 Add Markdown rendering to the cleaned PDF extraction result, preserving title repair, heading levels, bold and italic emphasis, blockquotes, and publication cleanup; add Rust tests for representative formatting and verify them with the Rust test suite.

## 2. Standalone extractor CLI

- [x] 2.1 Add the `kpapers-extract <PUBLIC_FOLDER_URL> <OUTPUT_DIR>` command with supported URL validation, output-directory creation, proxy and `MAX_PDF_BYTES` configuration, and sequential provider listing/download/extraction; add tests for argument validation, output creation, naming safety, partial failures, and exit status, then run them.
- [x] 2.2 Write safe deterministic Markdown filenames, prevent output paths from escaping the destination, and report per-file failures plus a completion summary; verify collision, traversal-name, and mixed-success scenarios in CLI tests.
- [x] 2.3 Document host usage, supported public links, top-level-only PDF selection, output naming, proxy and size-limit settings, failure reporting, and that source PDFs are not retained; verify documented examples match the CLI help and argument syntax.

## 3. Container and release publishing

- [x] 3.1 Add a standalone extractor Docker image with CA certificates, a non-root runtime, and the extractor as its entrypoint; verify a local container invocation writes Markdown through a host-mounted output directory.
- [ ] 3.2 Extend the GHCR workflow to scan the extractor image and publish its tags as a `linux/amd64` and `linux/arm64` multi-platform image; verify the workflow builds both architectures and preserves the existing API and web image tags.
- [x] 3.3 Document the published image name, multi-architecture support, example Docker invocation, and output-directory permission requirements; verify the example uses the image's actual entrypoint and mount path.

## 4. Integration verification

- [x] 4.1 Run the repository Rust checks and validate the OpenSpec change; verify the CLI and API binaries build together and all change artifacts pass `openspec validate`.

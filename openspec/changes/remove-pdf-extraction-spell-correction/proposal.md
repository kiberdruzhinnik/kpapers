# Proposal

## Why

PDF extraction should preserve the wording recovered from the source document. The local correction pass adds edits and visible annotations that are no longer wanted, along with dictionary data and dependencies used only for those edits.

## What Changes

- Remove automatic spelling and word-form correction from the PDF extraction pipeline, including the reviewed Russian exceptions, their published annotations, and the one exact-form line-wrap repair used only by that correction.
- Remove correction-only dependencies, bundled dictionaries and notices, documentation, fixtures, tests, fuzz-target wiring, and Docker asset copies.
- Reprocess cached PDFs after the processing pipeline changes so previously cached correction annotations disappear on the next successful refresh.
- Leave browser form spell-check settings and vendored `pdf_oxide` spell-check APIs untouched.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Remove extracted-text corrections and annotations; retain general cache invalidation when PDF processing behavior changes.

## Impact

Affected areas include `src/pdf_processing.rs`, `src/spell_correction.rs`, `src/lib.rs`, feed and export tests, `Cargo.toml` and lockfiles, `fuzz/`, `assets/spellcheck/`, `docs/spell-correction.md`, and both Rust Dockerfiles. The PDF processing revision must advance monotonically to invalidate cached output that may contain correction annotations.

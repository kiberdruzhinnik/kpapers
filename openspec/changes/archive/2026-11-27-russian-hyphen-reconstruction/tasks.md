# Tasks

## 1. Russian lexical resource

- [x] 1.1 Select a redistributable Russian wordform lexicon with lexical and concatenated-form lookup, document its source, version, license, and size, and verify the chosen data can be loaded offline by the project.
- [x] 1.2 Add the versioned lexicon data and a focused lookup/classification abstraction; verify lexical, concatenated, ambiguous, and unknown candidate lookups with unit tests.

## 2. Hyphen reconstruction

- [x] 2.1 Replace the hard-coded Russian compound exceptions with lexical-data classification shared by same-page and cross-page reconstruction; verify `во-вторых` is preserved and discretionary fragments such as `пристрасти-` / `лись` are joined as `пристрастились` in extracted text and HTML.
- [x] 2.2 Add coverage for unlisted Russian lexical compounds and discretionary splits, ambiguous and unknown candidates, and page-boundary cases; verify non-Russian reconstruction retains its existing behavior.
- [x] 2.3 Increment `EXTRACTION_PIPELINE_REVISION` and verify cached PDF text is regenerated under the new revision.

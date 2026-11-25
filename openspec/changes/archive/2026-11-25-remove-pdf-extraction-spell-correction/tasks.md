# Tasks

## 1. Remove PDF text correction behavior

- [x] 1.1 Remove the correction pass and correction-only `по-`/`своему` line-wrap rewrite from PDF finalization, preserving general line reconstruction; update PDF extraction tests and verify the relevant `pdf_processing` tests pass.
- [x] 1.2 Remove correction-specific assertions and fixtures from feed and export tests, replacing them with checks that extracted and published text preserves its source spelling; verify the feed and export test suites pass.

## 2. Remove correction-only dependencies and resources

- [x] 2.1 Remove `spellbook` from application and fuzz manifests, regenerate both lockfiles, delete the correction module, dictionary assets/notices, correction docs and fixtures, and remove fuzz-target imports; verify neither lockfile nor source manifests retain `spellbook` and the fuzz targets compile.
- [x] 2.2 Remove correction-only `assets/` copies from API and extractor Dockerfiles; verify both container builds succeed without the bundled dictionary directory.

## 3. Refresh cached output without annotations

- [x] 3.1 Advance the PDF processing revision to a new value and update its fingerprint test; verify the fingerprint differs from the current revision 35 so cached corrected output is reprocessed.
- [x] 3.2 Verify a cached PDF is regenerated without correction annotations on the next successful refresh while a failed refresh continues to retain the last successful entry; run the relevant cache and refresh integration checks.

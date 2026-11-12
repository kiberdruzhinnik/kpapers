# Tasks

## 1. Establish a test coverage map and backend test foundations

- [x] 1.1 Add a maintained behavior-to-test map for Rust modules, frontend workflows, and existing end-to-end contracts; verify every production area in `src/` and `web/src/` is mapped to an existing or planned test layer.
- [x] 1.2 Add Rust unit tests for authentication/session parsing, feed and hierarchy validation, provider source/download URL validation, RSS date/XML formatting, and scheduler interval/date logic; verify with `cargo test` and explicit malformed/boundary input cases.
- [x] 1.3 Add deterministic provider payload parsing tests for Yandex Disk and Mail Cloud response shapes, including missing, malformed, and unexpected fields; verify fixtures parse or fail without network access.

## 2. Cover stateful backend behavior

- [x] 2.1 Add isolated SQLite integration tests for repository and management API behavior, including feed/year/month lifecycle, archival/restoration/deletion, document overrides, and error responses; verify tests use disposable databases and pass under `cargo test`.
- [x] 2.2 Add scheduler and refresh integration tests for due selection, current-month priority, interval overrides, forced scopes, and failure preservation; verify repeatable outcomes with a controlled clock or fixed timestamps.
- [x] 2.3 Add backend route tests for malformed JSON, path/query values, unauthenticated requests, and representative valid requests; verify status codes, response bodies, and persisted state at each tested boundary.

## 3. Exercise PDF and transformation inputs

- [x] 3.1 Add property-based tests for PDF text normalization, safe HTML generation, markup escaping, XML-safe text handling, and promotion classification; verify generated cases preserve the documented output invariants.
- [x] 3.2 Add a coverage-guided PDF byte fuzz target that invokes the production extraction path with an explicit byte limit and pinned Pdfium runtime; verify valid, truncated, malformed, and oversized inputs return safely within configured time and memory bounds.
- [x] 3.3 Add fuzz targets for extracted-text transformations and user/provider parsing boundaries, including request JSON, source/download URLs, provider JSON payloads, and filename/date parsing; verify each target reaches production parsing logic and has a bounded smoke command.
- [x] 3.4 Seed fuzz corpora from generated PDF fixtures, malformed/minimal documents, and representative valid/invalid request and provider payloads; turn every discovered failure into a minimized regression corpus entry or deterministic test and verify it reproduces through the regression suite.

## 4. Close frontend and integration behavior gaps

- [x] 4.1 Review the behavior-to-test map against React forms, API request handling, dashboard navigation, and error states; add component tests for uncovered user-input boundaries and verify with `npm test` in `web/`.
- [x] 4.2 Extend browser and RSS contract coverage only where the map identifies a missing cross-layer behavior; verify with `npm run test:e2e` in `web/` and `bash scripts/e2e.sh` without relying on live provider services.

## 5. Integrate deterministic and fuzz checks in CI

- [x] 5.1 Add a Rust CI job for formatting, compilation, unit/property/integration tests, and any required Pdfium test runtime; verify the workflow runs on pull requests and fails when a Rust test fails.
- [x] 5.2 Add a separate bounded fuzz smoke job for parser and PDF targets with explicit per-target timeouts and crash-artifact upload; verify a CI run executes each target for its configured short duration and preserves failures for reproduction.
- [x] 5.3 Add a scheduled or manually triggered longer fuzz workflow with corpus and crash artifact retention; verify it is independent of the pull-request gate and documents how to reproduce a reported crash locally.
- [x] 5.4 Run the complete deterministic CI command set and validate the change's coverage map against the final test inventory; verify all existing frontend, browser, Miniflux, and new Rust checks pass.

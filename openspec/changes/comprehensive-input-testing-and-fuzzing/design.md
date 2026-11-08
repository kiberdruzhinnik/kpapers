# Design

## Context

See proposal.md for the motivation and scope. Existing checks include focused PDF transformation unit tests, web component tests, Playwright operator journeys, and a Miniflux RSS integration test. Rust tests are concentrated in `pdf_processing.rs`; CI does not run `cargo test` or fuzz targets. PDF extraction uses Pdfium loaded as a native library, while user input and provider data enter through Axum JSON/path/query handlers, URL validators, provider JSON deserialization, and configuration parsing.

## Goals / Non-Goals

**Goals:**

- Make documented backend and frontend behavior traceable to deterministic automated tests.
- Exercise malformed and adversarial inputs at parser and validation boundaries, including PDFium-backed PDF parsing.
- Keep regular CI checks deterministic and bounded while enabling longer fuzz campaigns.
- Verify behavior and security invariants at module boundaries and through representative end-to-end journeys.

**Non-Goals:**

- Requiring a one-test-per-function or arbitrary line-coverage threshold.
- Replacing the existing Playwright and Miniflux end-to-end suites.
- Changing product behavior or attempting exhaustive enumeration of all possible input strings.

## Decisions

### Organize tests around behavior and trust boundaries

Create a behavior-to-test map covering the Rust modules and frontend workflows. Add unit tests for pure logic and validation, integration tests for database-backed API and repository behavior, and retain browser/E2E tests for user-visible workflows. Each mapped behavior must have a relevant test at the narrowest useful level; tests should assert outcomes and invariants rather than implementation details.

Alternative considered: require coverage for every function. That creates brittle tests without ensuring meaningful behavior coverage and does not focus attention on inputs that cross trust boundaries.

### Combine property tests and coverage-guided fuzzing

Use property-based tests for stable invariants that should hold across generated inputs, such as safe HTML output, XML-safe RSS construction, valid/invalid URL classification, and normalization behavior. Add coverage-guided fuzz targets for parsers and validation entry points where arbitrary malformed inputs may reveal panics, hangs, or unexpected acceptance. Include HTTP JSON/request parsing, provider URL and JSON shapes, PDF bytes, and extracted-text transformation inputs. Keep target adapters small and exercise production parsing functions directly.

Alternative considered: use fuzzing alone. Fuzzing is valuable for finding unanticipated failures but does not reliably prove specific invariants or behavior; property and example-based tests provide repeatable regression checks.

### Bound fuzz execution and isolate native PDF parsing

Set input-size limits and execution timeouts for fuzz targets. The PDF target must call the production PDF extraction path with an explicit maximum byte size and run in an environment containing the same pinned Pdfium runtime expected by the API. Seed it with the existing generated PDF fixtures and malformed/minimal files; preserve minimized regressions as corpus entries or ordinary regression tests. CI runs a short, fixed-duration smoke campaign, while longer campaigns run on a schedule or manually and retain crash artifacts for reproduction.

Alternative considered: fuzz PDF parsing only in an unbounded or ad hoc local process. That is difficult to reproduce and can consume unbounded CI resources. A separate fuzz job or image provides runtime isolation while still testing the real parser path.

### Gate changes on deterministic tests; keep fuzz CI bounded

Add Rust test execution to CI alongside existing frontend, browser, and Miniflux checks. Run deterministic unit, property, and integration tests on pull requests. Run bounded fuzz smoke tests as a separate job with explicit timeouts and failure artifact collection; longer campaigns should not make each pull request nondeterministic or excessively slow. A discovered failure becomes a deterministic regression test before the corresponding fix is considered complete.

Alternative considered: run long fuzz campaigns on every pull request. This increases latency and makes the gate less predictable; scheduled longer campaigns offer broader exploration without slowing routine feedback.

## Risks / Trade-offs

- [Native PDFium behavior may differ between developer and CI environments] → Use the pinned PDFium build and exercise the production `extract_pdf` entry point in a consistent Linux environment.
- [Fuzzing may find resource-exhaustion cases or native crashes] → Apply byte/time limits, isolate the fuzz process, and retain minimized inputs for regression tests.
- [Test scope may grow faster than useful signal] → Track coverage by externally meaningful behavior and input boundary, prioritizing security-sensitive validation and state transitions over line-count targets.
- [External provider APIs are unstable and unsuitable for deterministic tests] → Parse serialized provider fixtures locally and use controlled network mocks only for HTTP behavior; keep the Miniflux contract stack isolated as it is today.

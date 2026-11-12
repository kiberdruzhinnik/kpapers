# Proposal

## Why

Test coverage is concentrated in PDF text processing and the web UI, leaving backend validation, authentication, provider parsing, persistence, and scheduling without comparable automated checks. A systematic test suite will catch regressions across the application and exercise malformed user and provider input, including hostile or corrupt PDFs, before those inputs reach production paths.

## What Changes

- Establish a behavior-to-test map for the backend, web application, and end-to-end workflows.
- Add unit and integration tests for uncovered backend modules and their stateful interactions.
- Add property-based tests for invariants at validation, formatting, and serialization boundaries.
- Add fuzz targets for untrusted request and provider parsing inputs and PDF byte/text processing, with bounded resource use.
- Run the deterministic test suite and bounded fuzz smoke checks in CI; run longer fuzz campaigns separately.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This is a testing and development-tooling change; it does not change product behavior.

## Impact

Affected areas include Rust modules for authentication, feed API, providers, repositories, RSS, scheduling, PDF processing, and database setup; web form and API tests; Rust and frontend development dependencies; CI workflows; and test/fuzz corpus configuration. No production API or user-visible behavior is intended to change.

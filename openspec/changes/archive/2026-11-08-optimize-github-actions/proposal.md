# Proposal

## Why

Pull requests already run the project's quality, security, test, fuzz, and integration checks, but the same checks run again after merge to `main`. Container builds also overlap between the UI workflow and the container workflow. Aligning workflows with the PR and release lifecycle will reduce repeated CI work while preserving pre-merge validation and image publication.

## What Changes

- Run code-quality and security scans, Rust and Node checks, bounded fuzz smoke tests, and UI/integration tests on pull requests targeting `main`.
- Stop rerunning those validation workflows on pushes to `main` after merge.
- Keep API and web image builds and scans on pull requests without publishing them.
- Publish API and web container images on pushes to `main` and version-tag pushes, retaining the existing `latest`, short SHA, and release tag conventions as applicable.
- Remove the redundant production web-image build from the UI test workflow; retain the production image build and scan in the container workflow.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This change adjusts CI workflow triggers and job placement without changing application behavior or a user-facing contract. Specs are intentionally skipped.

## Impact

Affected files are the GitHub Actions workflows under `.github/workflows/`, particularly `code-quality.yml`, `fuzz-smoke.yml`, `rust-tests.yml`, `ui-tests.yml`, and `publish-containers.yml`. Scheduled extended fuzzing remains separate. No application code, runtime dependencies, or container interfaces change.

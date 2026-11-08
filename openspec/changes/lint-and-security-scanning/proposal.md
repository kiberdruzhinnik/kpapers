# Proposal

## Why

The repository already runs Rust formatting/tests and frontend linting, but Rust linting and security scans are not part of its pull-request checks. Adding consistent lint and security feedback will expose code-quality issues and vulnerable dependencies, source patterns, or container layers before changes are published.

## What Changes

- Add Rust Clippy, ShellCheck, and Ruff to CI for the maintained Rust, shell, and Python code, and establish a reviewed path from current warnings to enforced lint gates.
- Run Semgrep Community Edition against supported application source on pull requests and pushes to `main`.
- Run Trivy filesystem/configuration scans for repository dependencies and configuration, and scan the API and web container images built for CI.
- Initially publish scan findings without blocking merges; review the baseline and document explicit severity and gating thresholds before making security findings a required check.
- Keep the existing Oxlint, formatting, deterministic test, browser, RSS contract, and fuzz workflows in place.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This is a CI and developer-tooling change with no application behavior contract changes; the change opts out of specs.

## Impact

- GitHub Actions workflows under `.github/workflows/` and the current container build workflows.
- Rust lint configuration and existing Rust warning baseline; `web/` continues using its configured Oxlint checks; shell scripts and Python E2E tooling receive dedicated lint checks.
- Semgrep and Trivy CI tooling and their finding output. No application runtime dependency or API changes are expected.

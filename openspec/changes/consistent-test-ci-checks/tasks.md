# Tasks

## 1. Define canonical local validation commands

- [x] 1.1 Inventory every test and quality command, workflow trigger, and unique coverage area; record a check map that identifies duplicate work and verify every existing required suite has one intended owner.
- [x] 1.2 Add or adjust suite entry points so frontend lint, component tests, and production build run without compiling TypeScript twice; verify the documented frontend command sequence passes and targeted typecheck remains available.
- [x] 1.3 Define canonical Rust, quality/security, fuzz-smoke, long-fuzz, browser E2E, RSS contract, and production-image check commands; verify each entry point invokes its intended underlying tools once and preserves its existing trigger and failure-reporting behavior.

## 2. Consolidate integration setup and stabilize browser tests

- [x] 2.1 Refactor the two Compose integration scripts or add a shared orchestrator to reuse compatible API/web image builds while keeping separate projects, fixture data, configuration overlays, and cleanup; verify both suites pass independently and cleanup runs after success and failure.
- [x] 2.2 Make browser URL-selection tests wait for the relevant navigation/state transition and isolate shared browser state; verify the URL assertions remain strict and repeated browser runs pass without retries or weakened expectations.
- [x] 2.3 Update `docs/testing.md` and relevant script help with prerequisites, canonical commands, coverage, CI ownership, and triggers; verify every documented command exists and matches the check map.

## 3. Align GitHub Actions with the validation map

- [x] 3.1 Consolidate or reorganize pull request workflows so each canonical suite runs once, frontend build/type checking is not repeated, and independent runtime suites can still run in parallel; verify the workflow configuration references only the canonical commands.
- [x] 3.2 Reuse production container build outputs for integration checks and image scans where the artifact is identical and practical, while retaining release-only publishing and platform requirements; verify PR scans target the intended production images and release jobs publish only after successful scans.
- [x] 3.3 Keep PR fuzz smoke and scheduled/manual long fuzz distinct, and align report-only code/image security checks with documented triggers; verify all prior unique coverage and failure artifacts remain present.
- [x] 3.4 Run the full canonical local validation sequence and validate the resulting GitHub Actions workflows; verify every check-map row is covered once per intended trigger and `docs/testing.md` agrees with workflow behavior.

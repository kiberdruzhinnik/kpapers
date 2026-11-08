# Design

## Context

See proposal.md - Why. Pull request checks currently span Rust, frontend, browser E2E, RSS contract, fuzz smoke, code quality, and container image workflows. Frontend `build` runs `tsc -b`, duplicating the separate `typecheck` step. The browser E2E and Miniflux contract scripts each build the same API and web services with separate Compose overlays and isolated fixtures. The container workflow separately builds production images for image scanning. Weekly/manual fuzzing is a different duration and trigger from PR smoke fuzzing.

## Goals / Non-Goals

**Goals:**

- Give contributors and CI one documented entry point for each suite, with CI invoking the same commands documented for local use.
- Run each validation check once per applicable pull request while retaining all unique coverage and trigger-specific checks.
- Remove redundant frontend compilation and reuse compatible container build outputs where this can be done without weakening image validation or test isolation.
- Make failures attributable to the suite that owns them and keep test assertions meaningful and deterministic.

**Non-Goals:**

- Change application behavior or reduce coverage to shorten CI.
- Combine independent integration tests into one mutable environment when their configuration or fixture state differs.
- Remove scheduled/manual long fuzzing or report-only security scans solely because they do not block pull requests.

## Decisions

1. **Define a canonical validation map and entry points.** Keep suite-level commands close to the technology they run, then document the canonical commands and have Actions invoke those same entry points. A small orchestration layer may group commands for local full validation, but it must not hide which suite failed. A new generalized test framework was considered and rejected because it would duplicate npm, Cargo, Docker Compose, and cargo-fuzz behavior.

2. **Treat overlapping work separately from overlapping coverage.** Remove the standalone frontend typecheck from the full CI sequence because `npm run build` already runs `tsc -b`; retain the targeted typecheck script for fast local feedback. Keep component tests, Playwright browser journeys, Miniflux publication contract tests, Rust tests, fuzz checks, and quality/security scans where they cover different behavior. Align each workflow's trigger and documented scope with what it actually runs.

3. **Consolidate pull request validation ownership and share setup/build work where practical.** Organize required PR checks so there is a clear owner for each suite and shared Node/Rust setup is performed once per job. Evaluate whether the two Compose suites can use one prepared API/web build while keeping distinct Compose projects, fixtures, and cleanup. For production image scans, reuse a built image produced by the PR validation path when feasible; if job isolation requires artifact transfer, weigh transfer cost and complexity against a duplicate build. Keep release publishing on successful non-PR runs and preserve architecture/platform requirements.

4. **Fix unstable tests at their synchronization boundary.** For stateful browser tests, make navigation and selection completion observable before asserting URL state, and ensure tests that share browser storage or application state are isolated. Do not add retries, loosen expected URLs, or make assertions conditional as a substitute for determinism.

5. **Make documentation derive from executable checks.** Update `docs/testing.md` to list the canonical command, prerequisites, coverage, and CI trigger for each suite. Remove or clearly scope alternate commands that bypass the supported setup scripts, and ensure README or workflow references point to the same command names.

## Risks / Trade-offs

- **[Combining workflow ownership can make a single job slower or reduce parallelism]** → Keep independent jobs for unrelated runtimes, but eliminate repeated steps within a suite and use clear job names and summaries.
- **[Sharing container builds can couple E2E, image scanning, and release packaging]** → Reuse only byte-identical production image outputs; retain separate test configuration overlays and verify both architecture and image tags before publishing.
- **[Changing test orchestration can leave cleanup behind after failure or cancellation]** → Preserve traps and isolated Compose project names, and validate cleanup on both success and failure paths.
- **[Removing the separate typecheck CI step could obscure its result]** → Name the build step to make its TypeScript check explicit and keep `npm run typecheck` available for targeted local use.

## Migration Plan

1. Implement canonical suite commands and remove duplicate CI invocations while preserving check coverage and triggers.
2. Update local testing documentation and workflow names to match the resulting check map.
3. Run the canonical local commands and validate workflow syntax/behavior before merging. Roll back the workflow/script consolidation as a unit if any required suite is no longer exercised.

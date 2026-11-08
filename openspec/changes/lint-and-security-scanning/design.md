# Design

## Context

See proposal.md for motivation and scope. The repository runs Rust formatting/tests in `rust-tests.yml`, frontend Oxlint/typecheck/tests/build in `ui-tests.yml`, and builds API and web images in `publish-containers.yml`. Clippy is not currently installed or run in CI. A local Clippy check found existing warnings across production modules and a small number in tests; the warning baseline must be reviewed and resolved before warnings can be denied. The maintained shell scripts and Python E2E tooling have no configured linter. No Semgrep or Trivy workflow/configuration currently exists.

## Goals / Non-Goals

**Goals:**

- Make Rust lint results repeatable in CI and keep both production and fuzz code covered.
- Lint the maintained shell scripts and Python test tooling as well as Rust and the already-linted frontend.
- Give pull requests Semgrep source-analysis results and Trivy dependency/configuration and image-scan results.
- Keep initial security findings visible without failing merges until the repository has reviewed the baseline and chosen explicit gates.
- Avoid publishing an image before its corresponding image scan has completed.

**Non-Goals:**

- Replacing Oxlint or the existing test, browser, RSS contract, and fuzz workflows.
- Adding runtime dependencies or changing application behavior.
- Defining a vulnerability severity policy without reviewing initial findings.

## Decisions

### Enforce Clippy after resolving the baseline

Add the Clippy component to the Rust toolchain setup and run Clippy for the main crate and fuzz workspace. Review and correct the existing warnings, then make warnings fail CI with `-D warnings`. Keep formatting and test jobs separate or clearly named so failures identify the relevant check. This is preferable to a permanently advisory lint job, which would let new warnings accumulate.

### Add shell and Python lint checks for maintained helpers

Run ShellCheck on the scripts under `scripts/` and Ruff on the Python E2E helpers. Keep configuration limited to maintained project code, not generated artifacts or temporary directories. This closes the gap for the non-Rust, non-frontend code without introducing runtime dependencies.

### Run Semgrep Community Edition in pull-request and main CI

Run a full source scan on pull requests and pushes to `main` using Semgrep CE community rules and repository-owned configuration where needed. Restrict the scan to maintained application source and relevant scripts/configuration; exclude generated/build output and fuzz corpora. Store findings in the workflow output/summary. CE avoids requiring an account token and keeps the workflow self-contained. Semgrep AppSec Platform's token-backed reporting is an alternative, but is not required for this scope.

### Run Trivy filesystem and image scans with report-only findings initially

Scan the repository for vulnerable dependencies, secrets, and configuration issues, including the Rust, fuzz, and web lockfiles and Dockerfiles. Scan both production images on pull requests before publication and on publishing builds before an image is pushed. Prefer sharing each build output with Trivy over compiling/building the same image twice. Keep scan execution failures visible as workflow failures, but initially do not fail solely because findings exist. Record the observed baseline and decide severity, fixability, ignore, and merge-gate policy after review.

Alternative considered: immediately fail on all findings or a chosen severity. There is no reviewed baseline or agreed threshold yet, so this could block unrelated changes or encourage broad ignores before findings are understood.

## Risks / Trade-offs

- [Existing Clippy warnings span production and test code] → Resolve the known baseline before enabling `-D warnings`, and run Clippy for both manifests.
- [Semgrep or Trivy can report false positives or pre-existing findings] → Begin in report-only mode, review findings, and use narrow documented suppressions only when justified.
- [Scanning images can duplicate expensive Docker builds or scan only after publication] → Integrate scanning with the current build outputs and ensure the scan runs before the publish step.
- [Tool/action versions or remote rule sets can change scan results] → Pin scanner/action versions and review upgrades deliberately; keep scanner configuration in the repository.

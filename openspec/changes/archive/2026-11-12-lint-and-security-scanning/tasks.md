# Tasks

## 1. Establish lint coverage

- [x] 1.1 Add Clippy to the Rust toolchain checks and resolve the existing warning baseline in the main crate and fuzz workspace; verify `cargo clippy --all-targets --locked -- -D warnings` and `cargo clippy --manifest-path fuzz/Cargo.toml --all-targets --locked -- -D warnings` pass.
- [x] 1.2 Add ShellCheck for maintained scripts under `scripts/` and Ruff for `tests/e2e/`; pin or otherwise reproducibly select tool versions, configure only justified exclusions, and verify both commands pass against their maintained source paths.
- [x] 1.3 Keep the frontend Oxlint check and add the new lint commands to pull-request and `main` CI; verify each lint job runs on both event types and reports failures with a distinct job/step name.

## 2. Add Semgrep source scanning

- [x] 2.1 Configure a Semgrep Community Edition scan for maintained application source, shell scripts, and Python test tooling on pull requests and pushes to `main`; exclude generated output, dependency directories, and fuzz corpora, and verify the workflow emits a readable report without requiring a Semgrep account secret.

## 3. Add Trivy repository and image scanning

- [x] 3.1 Add a Trivy filesystem/configuration scan covering the Cargo, fuzz, and npm lockfiles, Dockerfiles, and repository configuration on pull requests and pushes to `main`; initially report findings without failing solely on vulnerability findings and verify the report is retained in workflow output or an artifact.
- [x] 3.2 Integrate Trivy scans for both production images into the existing image build/publish path, scanning the exact image output before it is pushed; verify pull-request builds are scanned and a simulated/real scanner failure prevents image publication.
- [x] 3.3 Document the initial Semgrep and Trivy findings baseline, justified suppressions, and the explicit severity/fixability policy needed before security findings become merge-blocking; verify the policy is available alongside the scanner configuration.

## 4. Verify CI integration

- [x] 4.1 Validate workflow YAML and scanner configuration, then run the Rust, web, shell, and Python lint commands plus Semgrep and Trivy scans locally or in CI; verify the existing formatting, test, browser, RSS contract, and fuzz workflows remain configured and no temporary/generated files are scanned.

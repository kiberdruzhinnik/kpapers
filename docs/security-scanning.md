# Lint and security scan policy

## CI checks

Pull requests to `main` and pushes to `main` run:

- Clippy with warnings denied for the API and fuzz workspace, Rust formatting, and Rust tests.
- ShellCheck v0.11.0 for `scripts/*.sh` and Ruff 0.15.10 for `tests/e2e/`.
- Semgrep Community Edition 1.183.0 for maintained code in `src/`, `web/src/`, `scripts/`, and `tests/e2e/`. The scan report is attached to the workflow run.
- Trivy 0.76.2 for dependency, secret, and configuration scans of the repository and for vulnerability/secret scans of the built API and web images. JSON reports are attached to the workflow run.

Trivy skips generated build output, installed dependencies, fuzz corpora/artifacts, and the temporary `tmp/` directory. Semgrep is given only maintained source paths. Scanner findings are report-only at present; scanner execution errors fail the job. Image publication occurs only after the image scan step succeeds.

## Initial baseline

Baseline recorded on 2026-11-08 using Semgrep CE 1.183.0 and Trivy 0.76.2:

- Semgrep: zero findings after replacing a generic `Object.assign` in a frontend test mock with explicit field assignments.
- Trivy filesystem scan: one medium vulnerability, `CVE-2026-118764`, for `minimatch` 10.2.6 in `web/package-lock.json`; Trivy reports 11.0.1 as the fixed version.
- Trivy configuration scan: `DS-0002` (high, no explicit non-root `USER`) in `web/Dockerfile` and `web/Dockerfile.e2e`; `DS-0026` (low, no `HEALTHCHECK`) in `web/Dockerfile.e2e`.
- No secrets were reported by the filesystem scan.
- Production image scans: API image reported 217 OS vulnerabilities (87 low, 95 medium, 34 high, and 1 critical); web image reported one medium vulnerability, `CVE-2026-4738`, in Alpine `zlib` 1.3.2-r1 (fixed in 1.4-r0). Neither image scan reported secrets. These image counts are a point-in-time baseline and include OS packages from the base image.
- No findings have been suppressed or added to an ignore list.

This is a dated observation, not a permanent allowlist. Re-run the scanners to obtain current findings.

## Merge-gating policy

Current policy: security findings do not fail merges while the baseline is being reviewed. Tool download, scan, configuration, or report-generation failures do fail their workflow job; a scanner that did not run successfully must not be treated as a clean scan.

Before enabling finding-based merge gates, review the current reports and adopt this proposed threshold:

- Confirmed exposed secrets block immediately, regardless of severity label.
- Confirmed critical or high vulnerabilities with an available fix block; medium and lower findings are tracked for remediation and do not block by default.
- Confirmed high or critical Docker/IaC misconfigurations block; lower severities are tracked for remediation.
- Semgrep findings block only after a maintainer confirms the issue in application code and marks the rule result high confidence. Findings in tests or fixtures require the same review and may be suppressed narrowly when the behavior is demonstrably safe.

Any exception must name the finding/rule, explain the impact assessment, identify an owner, and include a review or expiry date. Keep ignores scoped to an individual finding; do not suppress a whole scanner, rule family, package, or severity band to make CI pass. Revisit this policy and the recorded baseline before changing report-only scans into required merge checks.

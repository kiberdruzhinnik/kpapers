# Tasks

## 1. Logging foundation

- [x] 1.1 Add and initialize structured tracing output to stdout before API initialization, and verify startup and initialization failures produce parseable log records.
- [x] 1.2 Replace scheduler stderr messages with structured error events containing safe operation context, and verify scheduler failure cases are covered without logging sensitive values.

## 2. HTTP request logging and operator guidance

- [x] 2.1 Add completion logging for API requests with method, path, status, and duration while excluding query strings, headers, cookies, and bodies; verify with route/middleware tests including credential-bearing requests.
- [x] 2.2 Document how to view API stdout logs with `docker compose logs api` in the TrueNAS deployment guide, and verify the documented command matches the deployment files.
- [x] 2.3 Verify the complete API log stream uses stdout and contains no passwords, session credentials, signing keys, provider credentials, or secret-bearing URLs.

## 3. Verification follow-up

- [x] 3.1 Correct startup error termination and verify an initialization failure exits nonzero, produces a structured stdout event, and does not duplicate the error as plain text on stderr.
- [x] 3.2 Extend provider-failure coverage with an error containing a credential or secret-bearing URL and verify the captured logs omit the sensitive values.

## 4. Diagnostic and redaction follow-up

- [x] 4.1 Ensure provider error summaries exclude credentials separated from their labels by whitespace, and verify captured logs omit values such as `Authorization: Bearer secret-value` as well as secret-bearing URLs.
- [x] 4.2 Restore useful diagnostics for failing `--healthcheck` and `--hash-password` commands, and verify their nonzero exit status and diagnostics with process tests.

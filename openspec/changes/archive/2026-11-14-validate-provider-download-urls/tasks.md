# Tasks

## 1. Enforce download destination validation

- [x] 1.1 Validate the Yandex Disk download URL with the Yandex allowlist before issuing the HTTP request; verify with an async regression test that a disallowed URL is rejected without a network request.
- [x] 1.2 Validate the Mail Cloud download URL with the Mail Cloud allowlist before issuing the HTTP request; verify with an async regression test that a disallowed URL is rejected without a network request.
- [x] 1.3 Extend URL validation tests for both provider identities to cover allowed HTTPS hosts and rejection of unapproved hosts, non-HTTPS schemes, unexpected ports, and embedded credentials; verify with the focused Rust test filter.

## 2. Verify ingestion behavior

- [x] 2.1 Verify a rejected provider download URL follows the existing extraction and refresh failure path and does not replace last-known-good cached content; verify with focused extraction/refresh tests.
- [x] 2.2 Run the relevant Rust test suite and `openspec validate validate-provider-download-urls --strict`; resolve any failures before marking the change complete.

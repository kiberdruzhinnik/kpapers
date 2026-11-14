# Proposal

## Why

PDF ingestion trusts download destinations returned by provider APIs and fetches them directly. This leaves the service able to make requests to unintended destinations if provider response metadata is influenced or compromised, despite existing provider host validation for RSS redirects.

## What Changes

- Require each provider-supplied PDF download URL to satisfy that provider's HTTPS host allowlist before the service fetches it.
- Apply the check at the download request boundary for both Yandex Disk and Mail Cloud.
- Reject invalid destinations as ingestion failures, preserving existing cached content through the current refresh failure behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: constrain initial PDF download destinations to the configured provider's HTTPS host allowlist.

## Impact

- Affects `src/providers/yandex_disk.rs`, `src/providers/mail_cloud.rs`, and the shared URL validation in `src/providers/mod.rs`.
- No API or database changes. Provider responses that previously supplied an out-of-allowlist URL will now fail ingestion instead of being fetched.
- Adds focused tests for accepted and rejected provider download URLs and confirms both provider download paths enforce validation.

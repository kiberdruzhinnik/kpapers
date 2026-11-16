# Proposal

## Why

Provider-issued download URLs can use public hosts outside the current provider-specific allowlists, causing legitimate PDFs to fail ingestion. When extraction fails, operators also need the affected filename and concrete failure reason to decide what needs attention.

## What Changes

- Allow HTTPS downloads from any public host while rejecting destinations on private or otherwise non-public networks, including redirect destinations.
- Keep configured public folder input URLs restricted to `cloud.mail.ru` and `disk.yandex.ru` with their existing path and validation rules.
- Preserve actionable per-file failure details and show them where operators review PDFs.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: change download destination policy and require actionable per-file extraction failure reporting.

## Impact

Changes affect provider download URL validation and redirect handling, refresh error aggregation, document error persistence, and PDF review UI. Configured public folder input URLs remain limited to `cloud.mail.ru` and `disk.yandex.ru`.

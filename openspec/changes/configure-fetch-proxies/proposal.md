# Proposal

## Why

Operators need refresh traffic to use a chosen HTTP or SOCKS5 proxy, either for all feeds or for a specific feed. Today provider discovery and PDF downloads construct direct clients, so network routing cannot be configured from the dashboard or deployment environment.

## What Changes

- Add protected global proxy settings in the management UI and per-feed proxy selection.
- Support per-feed **Inherit**, **Direct**, and **Custom proxy** modes. A custom feed proxy takes precedence over the global proxy.
- Resolve proxy configuration in this order: feed override, GUI global setting, `KPAPERS_PROXY_URL`, then direct connection.
- Route both provider discovery and PDF downloads through the effective proxy.
- Support HTTP and SOCKS5 proxy URLs, including credentials; do not return credentials in API responses or expose them in logs/errors.
- Preserve existing source URL checks, redirect limits, download size limits, and refresh outcomes when requests use a proxy.

## Capabilities

### New Capabilities
- `fetch-proxy-configuration`: Configure global and per-feed proxy routing for provider discovery and article downloads, with an environment fallback.

### Modified Capabilities

## Impact

- Rust provider clients, download client construction, refresh-job handling, management API, and SQLite migrations.
- Dashboard navigation, global settings UI, and feed detail forms.
- Deployment environment documentation and configuration examples.
- HTTP/SOCKS5 client support may require a Reqwest feature or dependency change.

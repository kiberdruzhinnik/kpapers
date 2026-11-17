# Design

## Context

See `proposal.md` for motivation and `specs/fetch-proxy-configuration/spec.md` for the behavior contract. Refresh processing currently passes only a feed or month identity to provider code. Yandex and Mail Cloud create their own Reqwest clients; PDF downloads use a separate client that disables proxies and pins a locally resolved public address. Management endpoints are authenticated. SQLite migrations run before the scheduler starts.

## Goals / Non-Goals

**Goals:**

- Resolve one proxy policy for each feed and apply it consistently to provider discovery and PDF downloads.
- Keep proxy configuration available to both interactive and scheduled refreshes.
- Avoid proxy credential disclosure in API output, logs, and error text.
- Preserve source URL validation, redirect checks and limits, and response size limits.

**Non-Goals:**

- Proxying RSS readers' requests to Kpapers or changing public RSS behavior.
- Supporting per-month or per-request proxy overrides.
- Encrypting the SQLite database or implementing a general secret-management service.

## Decisions

### Represent feed override state explicitly

Add a feed proxy mode with `inherit`, `direct`, and `custom` values, plus a nullable custom proxy URL. `inherit` is the default for existing and newly created feeds. `direct` intentionally bypasses both GUI and environment defaults. `custom` requires a valid URL. This is preferable to interpreting a missing URL as both inherit and direct, which cannot express the agreed bypass behavior.

### Store the global operator setting separately from deployment defaults

Add a singleton/keyed application-settings row for the GUI global proxy URL. Resolve the effective value at the point a refresh request is made: feed custom/direct mode first, then the GUI value, then `KPAPERS_PROXY_URL`, then direct. Read the current setting for each operation so later changes apply to subsequent discovery and download requests. Validate the environment URL during startup and fail startup with a credential-free error if it is malformed; silently connecting directly could violate the operator's expected route.

### Carry feed context into provider operations

Resolve the feed ID from the month/document for all discovery and download entry points, including scheduled jobs, manual discovery, extraction retries, and force refresh. Pass a single resolved proxy policy into provider listing and PDF download functions rather than letting providers independently read environment or database settings. Keep fixture provider behavior local and unchanged.

### Build explicit clients for each proxy policy

Use a shared client/configuration path for provider API and download clients, with HTTP and SOCKS5 support. Disable implicit Reqwest environment proxy discovery so only the specified precedence controls routing. For direct mode, explicitly disable proxies. Keep provider redirect policy restrictions and download redirect handling; validate each download/redirect URL before sending it through the selected proxy, and retain the current redirect count, timeout, and size limits.

The existing downloader pins destination DNS results for direct connections. A proxy may perform its own destination resolution, so application-side URL shape and public-address checks must still run before each request, while the configured proxy is treated as an operator-trusted network component. Do not let proxy configuration make a provider-supplied private or non-HTTPS download destination acceptable.

### Make saved proxy credentials write-only

Persist GUI and per-feed proxy URLs in SQLite, consistent with the current local single-operator deployment model. Management reads return only whether a proxy is configured and a sanitized endpoint with user information removed. Write requests can set or replace a full URL; explicit clear actions remove the saved value. Forms do not prefill credentials, and proxy client/build errors must be sanitized before they reach refresh status, HTTP responses, or logs. This avoids exposing stored credentials to browser state while preserving the current database-at-rest model.

### Add a protected settings view and feed controls

Add a Settings destination under `/manage/settings` and a focused global proxy form. Add a proxy mode selector and conditional custom URL field to the existing feed detail edit workflow. The UI explains that Inherit follows global GUI settings, then the environment, then direct; Direct bypasses inherited proxies. Keep save/clear feedback and validation consistent with existing management forms.

### Update deployment and operator documentation

Document `KPAPERS_PROXY_URL` as the deployment fallback and show supported URL forms without real credentials. Explain precedence, direct overrides, write-only credential handling, and that the operator must trust configured proxies.

## Risks / Trade-offs

- [A configured proxy can resolve a destination differently from the API host] → Continue local URL and public-address validation before every request and redirect, and document that configured proxies are trusted network components.
- [Credentials remain readable by anyone with access to the SQLite file] → Keep them out of responses and logs, preserve existing volume permission guidance, and call out the storage model in deployment documentation; database encryption is out of scope.
- [A proxy outage can fail a refresh across many feed documents] → Keep failures attached to the existing refresh result and error reporting, with URL credentials removed.
- [Global changes during a long-running refresh could otherwise produce inconsistent routing] → Resolve configuration per provider/download operation so subsequent requests use current settings; an individual request uses one immutable client configuration.

## Migration Plan

1. Add an ordered SQLite migration for global proxy settings and feed proxy mode/URL, defaulting existing feeds to `inherit`.
2. Deploy the API and web application; migration runs before the scheduler starts. With no configured proxy, existing feeds continue to connect directly.
3. Operators may configure `KPAPERS_PROXY_URL`, the GUI global setting, or feed overrides after upgrade.
4. Rollback by stopping the deployment and restoring the pre-upgrade database backup with the previous application image. No data conversion is needed beyond the additive migration.

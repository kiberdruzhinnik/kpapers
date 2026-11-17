# Tasks

## 1. Persist and validate proxy settings

- [x] 1.1 Add a SQLite migration for global proxy configuration and feed proxy modes/URLs, default existing feeds to `inherit`, and verify migrations preserve current feed data.
- [x] 1.2 Add shared proxy URL parsing and validation for HTTP and SOCKS5, including credential-bearing URLs; verify accepted and rejected schemes, addresses, and credential-free validation errors.
- [x] 1.3 Add authenticated management API operations for reading, setting, replacing, and clearing the global proxy and feed proxy mode; verify values are persisted, credentials are omitted from read responses, and invalid updates leave saved values unchanged.
- [x] 1.4 Validate `KPAPERS_PROXY_URL` at startup and document its precedence in deployment examples; verify valid values load and malformed values fail startup without exposing credentials.

## 2. Apply effective proxy settings to refresh traffic

- [x] 2.1 Implement effective proxy resolution for custom, direct, inherited global, environment, and direct fallback modes; verify each precedence branch and that direct mode disables implicit Reqwest environment proxies.
- [x] 2.2 Pass the feed's effective proxy policy through scheduled refresh, manual discovery, extraction retry, and force refresh paths; verify both provider discovery and PDF download requests use the selected route.
- [x] 2.3 Configure Yandex and Mail Cloud clients for HTTP and SOCKS5 proxy support while preserving provider host and redirect restrictions; verify provider requests use proxy authentication when configured.
- [x] 2.4 Configure the PDF downloader to use the effective proxy while validating each original and redirected destination, retaining redirect limits, timeouts, and size limits; verify unsafe destinations and oversized responses are rejected through proxied paths.
- [x] 2.5 Sanitize proxy credentials from client construction failures, network errors, refresh status, and logs; verify representative failures never contain proxy user information.

## 3. Add operator proxy controls

- [x] 3.1 Add a protected `/manage/settings` view and navigation entry with global proxy set, replace, and clear controls; verify direct navigation, authentication, validation feedback, and write-only credential behavior.
- [x] 3.2 Add Inherit, Direct, and Custom proxy controls to feed detail management; verify saved modes persist, custom URL validation works, and inherited behavior is explained in the UI.
- [x] 3.3 Add component and browser coverage for global and per-feed settings, including error retention and the three-mode precedence behavior; verify no credentials appear in rendered read state.

## 4. Document operation and verify integration

- [x] 4.1 Update operator, refresh, and deployment documentation with `KPAPERS_PROXY_URL`, precedence, supported proxy URL formats, trust and credential storage notes; verify examples contain no real credentials and match runtime behavior.
- [x] 4.2 Run the Rust and web test suites plus the existing direct integration refresh flow; verify HTTP and SOCKS5 proxy authentication, direct routing, credential redaction, and consistent scheduled/manual proxy resolution.

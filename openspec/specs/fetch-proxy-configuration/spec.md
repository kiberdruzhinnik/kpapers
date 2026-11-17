# Fetch Proxy Configuration Specification

## Purpose

Lets operators route provider discovery and article downloads through HTTP or SOCKS5 proxies, using global settings or feed-specific choices with a deployment environment fallback.

## Requirements

### Requirement: Resolve the effective proxy for each feed
The system SHALL select a feed's effective proxy from its explicit feed setting, the GUI global setting, the `KPAPERS_PROXY_URL` environment variable, or a direct connection, in that order.

#### Scenario: Use a feed-specific proxy
- **WHEN** a feed is configured with a custom proxy
- **THEN** provider discovery and PDF downloads for that feed use the custom proxy

#### Scenario: Use the global GUI proxy
- **WHEN** a feed inherits proxy settings and a global proxy is configured in the GUI
- **THEN** provider discovery and PDF downloads for that feed use the global proxy

#### Scenario: Fall back to the environment proxy
- **WHEN** a feed inherits settings, no GUI global proxy is configured, and `KPAPERS_PROXY_URL` contains a valid proxy URL
- **THEN** provider discovery and PDF downloads use the environment proxy

#### Scenario: Reject an invalid environment proxy
- **WHEN** the service starts with `KPAPERS_PROXY_URL` set to an unsupported or malformed proxy URL
- **THEN** the service does not start and reports a credential-free configuration error

#### Scenario: Connect directly when no proxy is configured
- **WHEN** a feed inherits settings and neither the GUI global setting nor `KPAPERS_PROXY_URL` configures a proxy
- **THEN** provider discovery and PDF downloads connect directly

#### Scenario: Explicitly bypass inherited proxies
- **WHEN** a feed is configured for direct connections
- **THEN** provider discovery and PDF downloads for that feed connect directly, even when a global or environment proxy is configured

### Requirement: Manage the global proxy in the operator interface
The protected operator interface SHALL let an operator set, replace, and clear the global HTTP or SOCKS5 proxy URL.

#### Scenario: Save a global proxy
- **WHEN** an authenticated operator saves a valid HTTP or SOCKS5 proxy URL in global settings
- **THEN** the system persists it and uses it for feeds that inherit proxy settings

#### Scenario: Clear the global proxy
- **WHEN** an authenticated operator clears the GUI global proxy setting
- **THEN** feeds that inherit proxy settings use `KPAPERS_PROXY_URL` when valid, or connect directly when it is absent

#### Scenario: Reject an invalid global proxy
- **WHEN** an operator submits a proxy URL with an unsupported scheme or invalid address
- **THEN** the system rejects the change, explains the validation error, and retains the previously saved setting

### Requirement: Configure proxy behavior per feed
The protected feed management interface SHALL let an operator choose **Inherit**, **Direct**, or **Custom proxy** for each feed.

#### Scenario: Inherit proxy settings
- **WHEN** an operator selects Inherit for a feed
- **THEN** the feed uses the global GUI proxy, then the environment proxy, then a direct connection according to the effective-proxy precedence

#### Scenario: Configure a custom feed proxy
- **WHEN** an operator selects Custom proxy and saves a valid HTTP or SOCKS5 proxy URL
- **THEN** the feed uses that proxy for provider discovery and PDF downloads

#### Scenario: Reject an invalid feed proxy
- **WHEN** an operator submits an invalid custom proxy URL for a feed
- **THEN** the system rejects the change, explains the validation error, and retains the previously saved feed setting

### Requirement: Route refresh requests through the effective proxy
The system SHALL use the effective proxy for provider listing and PDF download requests made by scheduled, manual, and forced feed refreshes.

#### Scenario: Refresh a feed through its effective proxy
- **WHEN** a scheduled or operator-requested refresh processes a feed with an effective proxy
- **THEN** provider discovery and article PDF downloads use that proxy for the feed's configured sources

#### Scenario: Apply proxy changes to subsequent requests
- **WHEN** an operator changes a global or feed proxy setting
- **THEN** subsequent provider discovery and PDF download requests use the newly effective proxy

### Requirement: Protect proxy credentials and retain fetch safeguards
The system SHALL support credentials in HTTP and SOCKS5 proxy URLs while keeping credentials out of management API responses, logs, and user-facing error messages, and SHALL retain existing source URL, redirect, and download size protections.

#### Scenario: Save an authenticated proxy
- **WHEN** an operator saves a valid proxy URL containing a username and password
- **THEN** the system uses those credentials to authenticate with the proxy and does not return them in API responses or expose them in logs or errors

#### Scenario: Follow a redirect through a proxy
- **WHEN** a proxied provider download redirects to another URL
- **THEN** the system applies its existing redirect limit and destination validation before following the redirect through the effective proxy

#### Scenario: Reject an unsafe download destination
- **WHEN** a provider supplies a download URL or redirect that violates existing download URL protections
- **THEN** the system rejects the destination even when a proxy is configured

#### Scenario: Enforce download size limits through a proxy
- **WHEN** a proxied response exceeds the configured PDF size limit
- **THEN** the system stops reading the response and records the existing download size failure

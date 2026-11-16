# Design

## Context

The provider adapters receive direct download URLs and use a shared HTTP client. Initial download URL validation currently checks a provider-specific hostname allowlist; redirect handling applies a similar host check. Extraction failures are stored on each source document and rendered in PDF review, while scheduled refresh code currently replaces document-level failures with a generic summary.

See `proposal.md` for motivation and `specs/pdf-ingestion/spec.md` for the behavior contract.

## Goals / Non-Goals

**Goals:**

- Accept HTTPS download URLs on any public host while preventing connections to private or special-use network addresses.
- Apply address checks to every redirect without allowing DNS resolution to change between validation and connection.
- Keep document-level error causes visible in both PDF review and scheduled refresh results.

**Non-Goals:**

- Changing the supported public folder input hosts (`cloud.mail.ru` and `disk.yandex.ru`), their URL paths, or their existing validation rules.
- Changing download size limits, timeouts, PDF extraction, or cache retention behavior.
- Reporting provider-issued signed URLs or credentials in user-facing errors or logs.

## Decisions

### Validate resolved addresses, then connect to the validated address

Keep HTTPS as a required scheme and remove the provider-specific host allowlist for file downloads. Resolve the destination hostname, reject the request if any returned address is non-public, and connect using a validated address while preserving the original hostname for TLS certificate validation and SNI. Treat IPv4-mapped IPv6 addresses according to their embedded IPv4 address. Reject loopback, private, link-local, unspecified, multicast, shared-address, and other special-use ranges, not just RFC1918 IPv4 and IPv6 ULA.

This avoids relying on a hostname-only check, which does not prevent public names from resolving to internal services. It also avoids a second unchecked DNS lookup after validation. Keep the existing timeout and response-size controls.

### Validate each redirect as a new destination

Follow redirects explicitly or through a redirect mechanism that can run the same HTTPS and resolved-address checks before each connection. A redirect must not bypass address validation, even when the original host was public. Retain the existing redirect count bound. Reject non-HTTPS redirect targets.

### Preserve detailed document failures and summarize them by filename

Continue storing the per-document failure in the existing `last_error` field and rendering it in the review UI. Make the error identify the filename, processing stage, and safe underlying reason where available. When a scheduled refresh encounters failed documents, collect their stored errors and include filename-specific details in the refresh summary instead of replacing them with a generic extraction error.

Sanitize error text before persistence or presentation: do not include URL userinfo, signed query parameters, or provider dispatcher secrets. Keep low-level diagnostics available in structured server logs only when they can be recorded without those secrets.

## Risks / Trade-offs

- [Address classification can miss newly designated special-use ranges] → Centralize the classification logic and cover IPv4, IPv6, and mapped-address cases in tests.
- [DNS behavior differs across hosts and test environments] → Isolate resolution and connection pinning behind a small helper so it can be validated with controlled resolvers.
- [More detailed errors can expose provider secrets] → Redact URL credentials and query values before storing or displaying errors; prefer stable failure categories and safe reason text.
- [Refresh summaries can become long when many PDFs fail] → Keep each filename and reason, and cap the aggregate summary length without truncating the per-document `last_error` value shown in review.

## Migration Plan

No database migration is required. Existing `last_error` values remain readable; the next failed attempt stores the more specific safe message, and a successful retry clears it as today. Deploy the address-validation and reporting changes together so new public download hosts do not weaken redirect checks.

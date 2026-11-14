# Design

## Context

The Yandex Disk adapter copies `item.file` or a download API `href` into `RemotePdf.download_url`. The Mail Cloud adapter builds a download URL from the dispatcher response. Both adapters later pass that value to `reqwest::Client::get` without validating the initial destination. The shared `validate_download_url` helper already checks HTTPS, port, embedded credentials, and provider host allowlists, and the RSS redirect path uses it. The existing HTTP clients also constrain redirect destinations.

## Goals / Non-Goals

**Goals:**
- Enforce the provider host allowlist immediately before each PDF download request.
- Keep Yandex Disk and Mail Cloud validation consistent with the existing destination policy.
- Preserve existing extraction failure and cached-content recovery behavior when a URL is rejected.

**Non-Goals:**
- Change which provider folder URLs operators may configure.
- Change provider API endpoints, database records, PDF parsing, download-size limits, or redirect policy.
- Add DNS/IP-range filtering beyond the existing provider host allowlists.

## Decisions

- Validate the initial URL inside each provider's `download` function, before constructing or issuing the download request. This places the security check at the network sink and protects all current and future callers; validating only while listing files would rely on every caller preserving that earlier check.
- Reuse `validate_download_url` with the canonical provider names `yandex_disk` and `mail_cloud`. The Yandex and Mail Cloud host allowlists already exist there and match the host families used by their redirect policies. Do not widen either allowlist as part of this change.
- Return the existing provider download error path when validation fails. `extract_document` already turns provider failures into extraction failures, and refresh processing retains previously cached content on a failed refresh.
- Add focused tests for accepted and rejected URL forms for both provider identities, and ensure each adapter calls validation before `client.get`. Avoid a new HTTP mocking dependency unless implementation reveals it is needed to test the request boundary reliably.

Alternatives considered:
- Validate only when constructing `RemotePdf`: rejected because the download functions are the actual request boundary and may gain callers that bypass discovery.
- Replace redirect policy with validation at the initial URL only: rejected because redirects are separate destinations and are already constrained by the client policy.
- Introduce IP-range filtering: deferred because the scan finding is about unchecked provider-supplied hosts, and this change should preserve the established provider domain policy.

## Risks / Trade-offs

- A provider may add a legitimate CDN hostname that is not currently allowlisted → ingestion will fail safely until that provider host is reviewed and explicitly added.
- The validation helper and redirect policy maintain related host lists → focused tests should detect accidental disagreement; broader deduplication is outside this change.
- A hostname allowlist does not independently constrain DNS resolution for an allowed provider domain → deployment network controls remain a separate defense.

## Migration Plan

No database migration or cache invalidation is needed. Deploy the updated API normally. If a provider returns a newly introduced legitimate download hostname, review it and update the allowlist in a separate, evidence-backed change. Rollback is restoring the previous API image; stored feed and cache data remain compatible.

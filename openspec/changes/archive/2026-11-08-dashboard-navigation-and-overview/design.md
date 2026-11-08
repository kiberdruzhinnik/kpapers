# Design

## Context

See `proposal.md` for motivation and `specs/feed-management/spec.md` for the behavior contract. The dashboard currently stores its section, selected feed, and PDF review selection in React state. React Router is already a dependency. The production Nginx configuration serves `index.html` for unknown frontend paths, so dashboard deep links can reach the SPA without a proxy change. Management APIs are authenticated, and the database stores active state for feeds, years, and months plus extraction and classification state for source documents.

## Goals / Non-Goals

**Goals:**

- Make the URL the source of truth for dashboard section and selection state.
- Add a low-cost authenticated summary query for the two Overview counts.
- Keep the sidebar preference local to the browser.

**Non-Goals:**

- Add a persistent “reviewed” workflow or a new database classification state.
- Change refresh scheduling, feed publication, or authentication behavior.
- Change mobile navigation into a persistent sidebar.

## Decisions

- **Use React Router browser routes.** Map `/` to Overview, `/feeds` to the feed list, `/feeds/:feedId` to a selected feed, `/pdf-review` to the review chooser, `/pdf-review/:feedId` to a selected feed, `/pdf-review/:feedId/:year` to a selected year, and `/pdf-review/:feedId/:year/:monthId` to a selected month. Keep view changes and feed/year/month selection in route navigation so browser history, reloads, and copied links restore the same view. React Router is already installed; hash URLs would avoid server routing configuration but are less conventional, and the deployed Nginx SPA fallback already supports browser routes.
- **Resolve stale selections in the destination view.** Load feeds and the selected hierarchy as today. If an ID or hierarchy element no longer exists, show a clear unavailable-selection message and navigate to the nearest usable list or chooser route. This avoids leaving the URL and rendered selection inconsistent.
- **Persist the sidebar preference in local storage.** Store a small expanded/collapsed value, default to expanded when no preference exists, and use the same preference for desktop navigation. The mobile drawer remains transient and closes after navigation.
- **Return aggregate Overview counts from one management endpoint.** Add an authenticated `GET /api/management/overview` response with `source_folder_count` and `needs_review_count`, calculated in SQL rather than fetching documents one month at a time. Count configured months joined to active feeds, years, and months. Count documents requiring attention when extraction is pending or failed, or the classification is unknown; an operator override does not erase a processing failure. The current database constraint allows only `publish` and `promotional` classifications, so the unknown-classification clause is defensive and does not require a schema migration.
- **Represent count loading and failure separately.** Show a loading state while the summary request is in flight, numeric values including zero on success, and an explicit unavailable state on failure. Do not use an em dash as a substitute for missing data.

## Risks / Trade-offs

- [Direct routes depend on SPA fallback behavior in each hosting environment] → The production Nginx config already falls back to `index.html`; preserve and cover this behavior in deployment checks.
- [A copied selected-feed or review URL can become stale after archival or deletion] → Resolve unavailable selections to a usable route and tell the operator why the selection could not be restored.
- [The review count reflects processing state, not whether a person has seen a document] → Label it as items needing attention and keep the count tied to unknown or pending/failed classification processing; do not infer review history from the absence of an override.
- [Overview count request may fail independently of feed loading] → Keep the dashboard usable and show the count error in the two cards without replacing their values with misleading placeholders.

## Migration Plan

No database migration is expected. Deploy the API summary endpoint and dashboard bundle together. Existing browser routes continue to use the same sign-in flow. Rollback consists of restoring the previous API and web images; no persisted application data needs conversion. The sidebar preference is a versioned local-storage key and can be ignored safely by older clients.

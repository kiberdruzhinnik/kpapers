# Design

## Context

See proposal.md for motivation. `App.tsx` manually parses dashboard paths and builds navigation URLs with React Router. Overview currently lives at `/`, feed views at `/feeds`, and PDF review at `/pdf-review`. `web/nginx.conf.template` proxies `/feeds/` to the Rust RSS handler and uses an SPA fallback elsewhere. Feed creation is embedded in `FeedManagement.tsx`; its form is also used for editing. `YearCard` in `HierarchyManagement.tsx` eagerly loads months and always renders its content. The scheduler stores offset-bearing RFC3339 UTC timestamps, currently rendered verbatim by month cards.

This design is included because the route migration crosses navigation, creation, PDF review, deployed routing, and browser test coverage.

## Goals / Non-Goals

**Goals:** Keep direct loads and client navigation consistent, reuse current API and form validation, and isolate presentation preferences by feed/year.

**Non-Goals:** No new API, schema migration, scheduler change, cross-device preference synchronization, timezone setting, or change to public RSS/document URLs. PDF review's hierarchy selection controls retain their behavior; year collapse applies to feed management.

## Decisions

### Put the complete dashboard under `/manage/`

Use explicit paths for Overview (`/manage/`), feed list (`/manage/feeds`), feed creation (`/manage/feeds/new`), selected feed (`/manage/feeds/<id>`), and PDF review (`/manage/pdf-review` plus the existing feed/year/month suffixes). Match `new` before dynamic feed IDs so it is never treated as an unavailable feed. Build all dashboard links, fallback destinations, brand navigation, sign-out destinations, and review return paths from consistent path helpers.

Redirect the application entry `/` to `/manage/` using replacement navigation; accept `/manage` by normalizing to `/manage/`. Preserve the current behavior of showing sign-in at a protected deep link and restoring that page after authentication. Unknown dashboard paths use the existing error feedback and fall back to `/manage/`.

Retain Nginx's `/feeds/` RSS proxy and SPA fallback, verifying the deployed result. Do not redirect `/feeds/<id>` to management: those URLs are subscriptions, and the previous dashboard URLs cannot be distinguished from them. Legacy dashboard bookmarks must use the new paths; no general legacy route alias is introduced. A router basename alone was considered, but explicit parsing and shared path helpers fit the existing manual route model and make public RSS boundaries clear.

### Extract a dedicated creation view while sharing the form

Extract or export the existing feed form so both editing and the new creation page share fields, validation, pending state, and error behavior. Remove only the embedded creation card, replacing it with a Create feed navigation control. Overview's Create feed control goes directly to the creation page. Keep Feeds selected in dashboard navigation while creating.

On success, incorporate the returned feed into application feed state before navigating to its detail URL, avoiding the missing-feed guard racing the transition. Follow up with a list refresh as needed without encouraging a second creation POST if that refresh fails. Cancel goes to the list; failed creation keeps inputs and the current route. Retaining the form in a modal was considered, but a dedicated route satisfies direct loading and separates creation from management.

### Store one expansion preference per feed/year

Use versioned local storage keys containing the feed ID and numeric year, following the existing sidebar preference pattern. Only an explicit saved expanded value initializes a year as open; absent or invalid data means collapsed. Catch storage access failures and keep toggling available in component state. Ensure a switch between feeds reinitializes preferences even when the same year exists in both feeds, using feed/year component identity or equivalent state handling.

Use an explicit keyboard-accessible toggle exposing `aria-expanded` and the controlled region. Keep year heading, status, configured month count, and year actions outside the collapsed region; place the month-linking form and month cards inside it. Multiple years can be open independently. A newly created year remains collapsed until expanded. Keep eager month loading for existing header counts; lazy loading would add unrelated count and loading changes. Server-side persistence was considered, but browser-local storage matches the sidebar and needs no backend changes.

### Format timestamp instants at display time

Parse the API's offset-bearing timestamp once and assemble the output using browser-local date/time fields with explicit zero padding and a 24-hour clock. Do not strip the offset before parsing or use UTC getters; the browser applies the offset appropriate to each instant, including daylight-saving transitions. Keep API values unchanged. A small shared formatter avoids inconsistent month fields without adding a date library. Avoid locale-default output because it does not guarantee the requested field order or separators.

Keep `Never` and `Pending` for null values. For an unexpected invalid non-null timestamp, display `Unavailable` instead of throwing or rendering `Invalid Date`. Test with a fixed clock instant in explicitly configured timezones rather than relying on the test host timezone.

## Risks / Trade-offs

- Old dashboard bookmarks overlap RSS URLs -> document the new management paths and update every internal link; public subscription URLs retain priority.
- Client-only route tests can miss proxy collisions -> exercise direct navigation and reload through the deployed Nginx browser environment, and fetch RSS separately.
- Local storage can be blocked or malformed -> catch errors, validate saved values, and retain usable in-memory toggles.
- Preference state can leak between feeds sharing a year -> include feed identity in storage and component state initialization, with a regression test.
- Omitting a timezone suffix can make screenshots ambiguous -> use the requested exact display format; the browser timezone defines the local value.

## Migration Plan

Deploy the frontend as one update after component and deployed browser checks pass. No data migration is needed. Update operator documentation and tests that refer to previous dashboard paths, preserving RSS examples. Verify `/` reaches `/manage/`, all management deep links survive reload, and `/feeds/<id>` still serves RSS. Rolling back restores the previous frontend and its route collision; stored year preferences can remain harmlessly unused.

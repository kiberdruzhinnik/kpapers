# Design

## Context

See proposal.md for motivation and specs/feed-management/spec.md for the dashboard behavior contract. The current web client is one static `web/index.html` with inline styles and JavaScript. Nginx serves that file and proxies `/api/`, `/feeds/`, and `/documents/` to the Rust API. The existing management API already supports the operator workflows in scope. There is a Python/Docker Miniflux end-to-end suite for RSS contracts, but no frontend package, browser test suite, or general CI test job.

## Goals / Non-Goals

**Goals:**
- Make the frontend a maintainable, typed client while preserving the existing API and deployment boundary.
- Establish fast component tests and isolated browser tests against real application services and deterministic fixtures.
- Make the frontend checks run in CI alongside the existing container build and RSS end-to-end verification.

**Non-Goals:**
- Changing Rust endpoints, database behavior, authentication policy, or RSS output.
- Building a separate public-facing RSS reader; subscribers continue to use external readers.
- Replacing the existing Miniflux test, which continues to cover RSS publication and identity.

## Decisions

### Use React, TypeScript, Vite, Tailwind, and shadcn/ui patterns

Create a Vite-based React TypeScript application under `web/`. Use Tailwind and source-owned shadcn/ui components for dashboard primitives, with a clean default visual theme. This fits the requested shadcn/ui approach and keeps component code reviewable and customizable. Keep presentation separate from API calls and feature workflows so the form, status, confirmation, and feedback patterns can be reused.

Alternatives considered: retaining a single static file would not provide the component boundaries or test setup needed for the full redesign. A server-rendered Rust frontend would move presentation into the API and add a new server-side rendering architecture despite the existing static web/API split.

### Keep the web container static at runtime

Build the Vite bundle in a Node-based Docker build stage, then copy only the generated assets and Nginx configuration into the existing Nginx runtime image. Preserve the current reverse-proxy paths and configure SPA fallback for client routes. This keeps production runtime small and avoids introducing a frontend server or changing API routing.

### Keep existing management API contracts

Implement the dashboard against the current `/api/management/*` endpoints and session cookie behavior. Centralize fetch handling, JSON/error parsing, and unauthorized-session handling in a typed frontend API layer. UI state may be reorganized, but operations continue to use the same endpoint semantics and confirmation for permanent deletion.

### Make RSS address copying work across browser clipboard support

The feed details view SHALL copy the exact stable RSS address shown to the operator. Use the browser Clipboard API when available and permitted; if it is absent or rejects the write, use a compatible document-based fallback. Report success only after the selected method succeeds, and give a clear failure message if both methods fail. Cover the Clipboard API path, fallback path, and final failure state in component tests.

### Organize PDF review by feed, year, and month

PDF review SHALL provide a feed → year → month navigator. Selecting a month loads its discovered documents and keeps the selected hierarchy visible as context; entering review from a month in feed management SHALL select that same path. The navigator should use the existing feed and hierarchy management data, and document lists should continue to use the selected month's documents.

The "View source PDF" action SHALL resolve to the individual discovered file, not a containing provider folder or generic provider page. Do not use the provider-level source URL as though it were a document URL. Provide a document-specific route or file URL that allows operators to view each document listed for review, including documents not currently eligible for the public RSS feed. Preserve existing RSS publication rules independently of review access. Verify both the resolved link and hierarchical selection behavior with component or browser tests.

### Test UI behavior at component and browser levels

Use Vitest and React Testing Library for isolated component and interaction coverage, including validation, API payload mapping, loading/success/error states, and accessible control names. Mock network responses in these tests so they remain deterministic and fast.

Use Playwright for browser journeys against the built web client and real API in a dedicated disposable Compose environment with fixture mode. Cover sign-in and rejected credentials, feed creation/editing, year and month setup, refresh state/actions, PDF review and promotion override, archive/restore, confirmed permanent deletion, logout, and narrow viewport layout. Prefer role- and label-based selectors; check for page overflow at a phone-sized viewport. Keep destructive actions isolated to disposable data.

Retain the current Miniflux end-to-end scenario for RSS output, stable document identity, and refreshed content. Run it separately from browser tests so Miniflux's RSS assertions remain focused and browser tests do not share mutable feed state with them.

Alternatives considered: component tests alone would miss reverse-proxy, cookie, and real API integration; browser tests alone would make simple validation and error-state feedback slow to diagnose. Keeping the existing RSS suite avoids duplicating specialized feed-reader assertions in Playwright.

### Add frontend CI checks without changing release semantics

Add a frontend CI job that installs from the lockfile and runs linting, TypeScript checks, component tests, and a production build. Run the isolated Playwright browser suite in CI as a separate job or stage. Continue to build the production web container in the existing publish workflow; pull requests must pass frontend checks and the existing container build before merge.

## Risks / Trade-offs

- [A new Node toolchain and browser suite add CI time and maintenance] → Cache package-manager dependencies and browser binaries, keep unit tests fast, and use the isolated browser suite for high-value workflows.
- [A frontend/backend mismatch can break operations even when components pass] → Exercise key workflows against the real API in Playwright and preserve the existing Miniflux contract suite.
- [SPA routing can break direct navigation or Nginx proxy behavior] → Keep API and RSS proxy locations ahead of the SPA fallback and test direct loading of client routes in the built container.
- [shadcn/ui primitives do not guarantee accessible feature workflows by themselves] → Use semantic labels and roles in feature components, test controls through accessible queries, and include keyboard/focus checks in browser coverage.

## Migration Plan

1. Add the frontend source, lockfile, test configuration, and multi-stage web build while retaining the current Nginx runtime and proxy routes.
2. Replace the static page with the dashboard, preserving the management API contract and root entry point.
3. Run component tests, browser tests, the existing Miniflux suite, and the production web image build in CI.
4. Deploy the new web image with the existing API and persistent data. No database migration is required. Roll back by restoring the previous web image; backend and stored data remain compatible.

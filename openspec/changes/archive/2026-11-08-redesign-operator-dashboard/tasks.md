# Tasks

## 1. Frontend foundation

- [x] 1.1 Create the Vite React TypeScript frontend with Tailwind, shadcn/ui component setup, lockfile, and lint/typecheck/test/build scripts; verify dependency installation and the production build succeed.
- [x] 1.2 Update the web Docker build to compile frontend assets and serve them from the existing Nginx runtime, preserving API/RSS proxy paths and SPA fallback; verify direct loading of the app and proxy routes in the built container.

## 2. API client, authentication, and dashboard shell

- [x] 2.1 Implement a typed management API client that preserves session-cookie behavior and handles JSON, unauthorized, and failed responses; verify payload, response, and error handling with unit tests.
- [x] 2.2 Build the accessible sign-in, session restoration, logout, navigation, and shared feedback shell using shadcn/ui patterns; verify valid/invalid session states, keyboard-accessible controls, and failure feedback in component tests.

## 3. Feed and hierarchy management

- [x] 3.1 Build feed listing, creation, selection, metadata editing, RSS URL display, archive/restore, and permanent deletion confirmation; verify form validation, API payloads, success/error states, and confirmation behavior in component tests.
- [x] 3.2 Build year and month setup, source-link forms, refresh status presentation, and scoped refresh/archive/restore/delete actions; verify form states, month status rendering, and action handling in component tests.

## 4. PDF review and publication controls

- [x] 4.1 Build the PDF review view with classification, extraction status/errors, and publication override controls; verify automatic/include/exclude choices, save feedback, and failed saves in component tests.

## 5. Browser end-to-end coverage

- [x] 5.1 Add a disposable Playwright environment using the fixture-mode API and built web container; verify the test setup starts and cleans only its isolated resources.
- [x] 5.2 Add browser journeys for sign-in, feed creation/editing, year/month linking, refresh controls, PDF review/override, archive/restore, confirmed permanent deletion, and logout; verify the suite passes against real application services and deterministic fixtures.
- [x] 5.3 Add narrow-viewport and keyboard/accessibility-oriented browser checks for navigation, management controls, error feedback, and page overflow; verify the suite passes at a phone-sized viewport.

## 6. CI and integration

- [x] 6.1 Add CI jobs for lockfile-based frontend installation, lint, typecheck, component tests, production build, and Playwright browser tests; verify pull requests report failures for each required check.
- [x] 6.2 Keep the isolated Miniflux RSS contract suite in CI and verify both it and the new UI suite pass alongside the production web image build.

## 7. RSS address copy reliability

- [x] 7.1 Make the feed RSS address copy action use the Clipboard API when available and a compatible fallback when that API is unavailable or rejects the write; report success or failure accurately.
- [x] 7.2 Add component tests for Clipboard API success, fallback success, and failure feedback when neither method can copy the address.

## 8. PDF review navigation and source links

- [ ] 8.1 Make "View source PDF" open the individual discovered file through a document-specific URL that works for documents shown in review, regardless of public RSS eligibility; verify the file link and access behavior.
- [x] 8.2 Organize PDF review navigation by feed, year, and month, preserve the selected hierarchy when entering from feed management, and add component or browser tests for selection and document loading.

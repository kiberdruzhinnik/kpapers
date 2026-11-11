# Tasks

## 1. Workspace and Overview

- [x] 1.1 Apply approved semantic tokens, responsive controls, shell dimensions, navigation and Overview composition; add component tests for clear, attention, unavailable and zero-feed states and preserve sidebar behavior.

## 2. Feeds and creation

- [x] 2.1 Separate list/detail rendering with Active/Archived tabs, back navigation, contextual actions and focused editing; update feed and route tests to verify RSS identity, navigation, archive/restore/delete and error recovery.
- [x] 2.2 Match the approved creation form and accessible linked validation summary; test focus, invalid fields, retained input, pending submission, success and Cancel.

## 3. Source folders

- [x] 3.1 Move Add year and Connect month into focused dialogs and month metadata into Manage source disclosures; update hierarchy and browser journeys to verify preferences and every scoped operation.

## 4. PDF review

- [x] 4.1 Apply approved review layout, independent classification and override presentation, row feedback and extraction eligibility messaging; test source links, failed saves and retry extraction.

## 5. Integration verification

- [x] 5.1 Run frontend type checking, lint, component tests, production build and responsive browser checks at 320, 375, 768, 1024, 1440px and landscape; document outcomes and validate the OpenSpec change.

## Verification results

- `npm run typecheck`: passed; final `npm run check` also passed its TypeScript build, lint, all 51 component tests, and production bundle build.
- `./scripts/ui-e2e.sh`: all 13 deployed browser tests passed against the final rebuilt frontend and disposable API/Nginx stack. Covers all five screens at 320×740, 375×812, 768×1024, 1024×768, 1440×900 and 812×375; long content, overflow, reduced motion, dialog focus, validation, protected routing, RSS URLs, local timestamps, and feed/year/month operations.
- Follow-up local Actions verification exposed a reduced-motion dialog cleanup failure and an operator test relying on toast timing. Reduced motion now disables animations entirely; the operator test waits for save responses and verifies persisted decisions after reopening review.
- Final `act pull_request --workflows .github/workflows/ci.yml` run against an isolated copy of the uncommitted changes passed both jobs: image build/artifact transfer, Rust formatting/Clippy and 49 tests, frontend lint and 51 tests, all 13 browser tests, and RSS/Miniflux integration. `CI=true ./scripts/integration-e2e.sh` also passed independently after the fixes.
- Additional local checks: pinned nightly fuzz workflow passed; Ruff passed; standalone ShellCheck passed; Semgrep scan completed with zero findings. Local `act` wrappers for code quality require environment support: the runner image lacks ShellCheck, the Semgrep image lacks Node for artifact upload, and Trivy setup requires a checkout token. The direct pinned Trivy scan completed with one dependency finding and three configuration findings, reported without failing by the existing policy; these are scanner results, not validated application vulnerabilities.
- Desktop Overview/feed and phone feed/creation screenshots were rendered and inspected using illustrative mocked records.
- `openspec validate implement-approved-dashboard-design --strict` and `git diff --check`: passed.
- Non-blocking diagnostics: three pre-existing React lint warnings remain in App/PDF review; Vite reports a 528.75 kB minified application chunk (162.85 kB gzip).

## Workflow follow-up

- Review the implementation before archiving this change.

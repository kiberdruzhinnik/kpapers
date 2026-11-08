# Tasks

## 1. Overview summary API

- [x] 1.1 Add an authenticated overview-summary response and SQL counts for configured month folders under active feeds and years, plus PDFs with unknown classification or pending/failed processing; verify with handler/database tests covering active scopes, archived scopes, and document states.
- [x] 1.2 Add the summary route and frontend API type/client method; verify route authentication, response shape, and frontend request path with API tests and the existing web API test conventions.

## 2. URL-driven dashboard navigation

- [x] 2.1 Add browser routes for Overview, feed list, selected feed, PDF review chooser, and selected PDF review month, and derive section/selection state from route parameters; verify direct-route rendering, browser history, and invalid or stale selection handling in UI tests.
- [x] 2.2 Connect feed selection, create/archive/delete outcomes, and PDF review feed/year/month selectors to route navigation; verify each action updates the URL and direct reload restores the selected content.
- [x] 2.3 Verify production deep links reach the SPA through the Nginx fallback and load the correct dashboard view using a web deployment or browser smoke check.

## 3. Dashboard shell and sidebar

- [x] 3.1 Add an accessible desktop sidebar collapse/expand control with a versioned local-storage preference; verify keyboard operation and persistence after remount/reload in UI tests.
- [x] 3.2 Link the sidebar and mobile drawer Kpapers brand to Overview, remove the scheduler banner and specified footer copy, and verify navigation and the removed text with shell tests.

## 4. Overview counts

- [x] 4.1 Replace the placeholder Source folders and Needs review values with the summary response, showing loading, numeric zero and positive counts, and an explicit unavailable state; verify each state and summary request failure in UI tests.
- [x] 4.2 Run the relevant backend and web test suites plus type checking, and resolve any integration failures across routes, summary counts, and authenticated loading.

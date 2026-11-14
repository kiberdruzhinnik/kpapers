# Tasks

## 1. Global review queue API

- [x] 1.1 Add an authenticated global review-queue endpoint returning document fields with feed, year, month, and archive context; verify active and archived membership and review filtering with API tests.
- [x] 1.2 Align Overview's active attention count with the queue predicate; verify active and archived fixtures, including restored scopes, produce matching active queue and summary counts.

## 2. PDF review queue modes

- [x] 2.1 Add URL-addressable active and archived queue modes and render global documents with feed/year/month context and existing publication decision controls; verify direct navigation, empty states, and decisions with component tests.
- [x] 2.2 Preserve existing feed/year/month browsing and document deep links alongside the queue modes; verify current review route tests continue to pass.
- [x] 2.3 Explain in the operator guide how to switch queues and that archived items must be restored before opening their source or retrying extraction; verify the documentation matches the implemented controls.

## 3. Overview shortcut and integration

- [x] 3.1 Link Overview's attention action directly to the active Needs review queue when active items exist, while retaining zero, loading, and unavailable states; verify navigation and count behavior with dashboard tests.
- [x] 3.2 Verify active and archived queue behavior across archive and restore transitions and confirm existing month review remains usable with the project's relevant integration checks.

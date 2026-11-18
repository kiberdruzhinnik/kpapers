# Tasks

## 1. PDF review

- [x] 1.1 Restyle the mode switch, source selectors, queue header, and document rows in `PdfReview.tsx` to match the approved PDF review v3 reference; retain all queue, browse, selection, and per-document states, and verify with the PDF review tests.
- [x] 1.2 Check loading, error, empty, archived, extraction failure, save-pending, and save-error presentations in the PDF review tests; verify source links and decision controls remain accessible and correctly labelled.

## 2. Global proxy settings

- [x] 2.1 Reshape `ProxySettings.tsx` to match the approved Settings v4 reference, separating global proxy configuration/status from routing precedence; preserve write-only credentials and save/replace/clear behavior, and verify configured, unconfigured, validation-error, and request-error states in app-level coverage.

## 3. Feed list and detail

- [x] 3.1 Align the feed list rows, tabs, primary action, and empty states with approved Feeds v3 while preserving archived restore/delete actions; verify active and archived list workflows with feed management tests.
- [x] 3.2 Align feed detail, RSS address, per-feed proxy, and source-folder sections with approved Feed Detail v2; preserve routes, proxy mode behavior, dialog/menu actions, and per-year expansion preferences, and verify with feed management and hierarchy tests.

## 4. Cross-screen verification

- [x] 4.1 Review PDF review, Settings, feed list, and feed detail at 320, 375, 768, 1024, and 1440px plus landscape; verify no horizontal overflow, clipped actions, or lost source context.
- [x] 4.2 Verify keyboard navigation, visible focus, dialog/menu focus restoration, semantic status text, and reduced-motion behavior across the redesigned screens; run the relevant web checks and resolve regressions.

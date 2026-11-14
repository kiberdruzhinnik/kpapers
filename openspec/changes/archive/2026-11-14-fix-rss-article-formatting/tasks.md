# Tasks

## 1. Trace and correct extracted article structure

- [x] 1.1 Add a PDF extraction regression fixture with a visually distinct title, introductory lead, and multiple body paragraphs; verify extracted HTML contains separate title and paragraph elements in reading order.
- [x] 1.2 Trace the regression HTML through cached content and both RSS item content fields, then correct the earliest stage that drops or flattens block boundaries; verify the existing PDF extraction and RSS tests pass.
- [x] 1.3 If extraction or formatter output changes, increment the processing cache revision and verify an existing cached document is reprocessed; omit this task's change when the correction is RSS-only.

## 2. Verify Miniflux presentation contract

- [x] 2.1 Extend the Miniflux end-to-end assertions for the regression article to verify the entry API content preserves a separate title, lead, and body paragraphs; verify the RSS XML remains valid and safe HTML remains within the allowlist.
- [x] 2.2 Run the Miniflux end-to-end workflow and verify the article's title, lead, and paragraphs remain distinct in the ingested content without changing item identity or links.

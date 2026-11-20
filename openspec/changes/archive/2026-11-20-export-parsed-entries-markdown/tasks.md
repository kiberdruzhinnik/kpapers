# Tasks

## 1. Export API and archive generation

- [x] 1.1 Add a ZIP generation dependency and implement a shared scoped export query that selects cached, successfully extracted entries under active feed/year/month scopes using the RSS effective publication decision; verify query coverage for each scope and promotional overrides.
- [x] 1.2 Add protected feed, year, and month export endpoints that create safe `<feed>/<year>/<month>/<entry>.md` paths, resolve duplicate names, and return a downloadable ZIP; verify response type, archive paths, and valid empty-scope archives.

## 2. Dashboard export actions

- [x] 2.1 Add a binary download helper and API calls for feed, year, and month exports; verify filenames, download completion, and user-visible errors.
- [x] 2.2 Add export actions to feed, year, and month controls with their agreed scope semantics; verify each action calls the matching endpoint and works with the existing responsive management layout.

## 3. Integration verification

- [x] 3.1 Verify exports exclude pending, failed, archived, and effectively promotional entries while including successful operator-included entries, and confirm feed/year/month exports preserve the expected hierarchy in the ZIP.

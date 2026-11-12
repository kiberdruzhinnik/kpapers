# Tasks

## 1. Move dashboard routes under /manage/

- [x] 1.1 Update route parsing and shared URL construction in `web/src/App.tsx` for `/manage/`, `/manage/feeds`, selected feeds, and `/manage/pdf-review` with partial and full selection suffixes. Update brand, navigation, missing-selection fallbacks, review return links, and sign-out destinations; verify component tests cover direct loading, reload restoration, browser history, and unavailable selections at the new paths.
- [x] 1.2 Normalize `/` and `/manage` to `/manage/` with replacement navigation while preserving sign-in at protected deep links; verify route/session tests cover these entry points and post-login restoration.
- [x] 1.3 Update dashboard URL references in operator/deployment documentation where present, preserving `/feeds/<id>` RSS examples; verify the documented management URLs match the route map and no internal dashboard link still uses the old prefixes.
- [x] 1.4 Add deployed browser assertions for direct navigation and reload of Overview, selected feed, and PDF review paths, plus an RSS request at `/feeds/<id>`; verify the existing Nginx SPA fallback serves dashboard HTML and the public proxy still serves RSS without a management redirect.

## 2. Give feed creation a dedicated page

- [x] 2.1 Share the existing feed form between editing and a protected `/manage/feeds/new` view; match the static creation path before feed IDs, keep Feeds navigation active, remove the embedded creation form, and connect Overview and Feeds Create feed controls. Verify tests cover both entry points, direct load after sign-in, field validation, and absence of the form on management pages.
- [x] 2.2 On successful creation, update application feed state before navigating to the new feed; retain entered values on submission failure and send Cancel to the list. Verify tests cover success, failure/retry, Cancel without a POST, and a list-refresh failure after successful creation without duplicate creation.
- [x] 2.3 Update `FeedManagement.test.tsx` and browser operator/mobile workflows for the dedicated page, preserving edit-form behavior; verify the creation-to-feed-details flow and mobile validation with no horizontal overflow.

## 3. Remember collapsible feed years

- [x] 3.1 Add independent accessible year toggles in `HierarchyManagement.tsx`; keep headings, counts, status, and actions visible while hiding month details and linking forms. Verify tests cover initially collapsed active, archived, and newly created years, multiple expanded years, keyboard activation, and expanded-state accessibility attributes.
- [x] 3.2 Persist versioned per-feed/year preferences with safe local-storage handling and correct initialization when switching feeds. Verify tests cover reload/remount, return from PDF review, explicit collapse after expansion, same-year isolation across feeds, malformed values, and storage exceptions.
- [x] 3.3 Update hierarchy component tests and the deployed browser journey to expand years before using month controls; verify linking, refresh, archive/restore, deletion, and PDF review remain usable and the selected expansion state survives browser reload.

## 4. Display local refresh timestamps

- [x] 4.1 Add a shared local timestamp formatter and use it for month last-attempt and next-refresh values, preserving null placeholders and providing the invalid-value fallback described in design.md. Verify formatter tests cover zero padding, 24-hour time, date rollover, offset-bearing inputs, nulls, and invalid input.
- [x] 4.2 Verify exact rendered month values in explicitly configured browser timezones, including Europe/Moscow (`2026-11-08T18:09:15Z` -> `2026-11-08 21:09:15`) and a timezone with daylight-saving transitions; confirm both fields use the offset at their respective instant without changing API values.

## 5. Integration validation

- [x] 5.1 Run `npm run check` in `web` and the existing disposable UI browser suite using the repository's documented runner; verify the combined route migration, creation, year persistence, local timestamps, mobile layout, and unchanged RSS URL behavior pass through the deployed web server.

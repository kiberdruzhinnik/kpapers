# Test coverage map

This map ties production behavior and trust boundaries to their current tests and next coverage layer. Update it when production modules, user workflows, or test suites change. Tests should establish behavior or an invariant, not merely call every function.

## Rust backend

| Production area | Existing coverage | Coverage to add |
| --- | --- | --- |
| `src/main.rs`, route assembly and health endpoints | Axum router tests cover health, auth gates, malformed login/feed/year JSON, invalid numeric path/query values, valid login, and feed creation; UI/browser and Miniflux E2E exercise deployed routes | Extend HTTP route cases as new handlers or input boundaries are added |
| `src/auth.rs` | Session cookie signature, expiry, malformed encoding, and missing-cookie unit tests; input fuzz target exercises cookie parsing | Login handler outcomes and route-level auth/error behavior |
| `src/feed_api.rs` | Feed-name/interval and year/month range validation unit tests; SQLite lifecycle test covers hierarchy, overrides, archive/restore, errors, and persisted cascades; router tests cover malformed payload and invalid path/query behavior; indirect Playwright and Miniflux E2E | Expand route assertions for any newly introduced request boundary |
| `src/db.rs`, migrations, and `src/repositories/*` | SQLite management lifecycle test verifies migration-backed persistence and deletion cascades; indirect E2E coverage | Broader repository-specific invariants as repository behavior grows |
| `src/providers/mod.rs` | Source/download URL allowlist examples and rejection cases; coverage-guided input target exercises URL parsing | Additional minimized failures as they are discovered |
| `src/providers/yandex_disk.rs`, `mail_cloud.rs` | Offline JSON fixtures cover valid payloads, optional and unknown fields, malformed JSON, and missing required fields; E2E uses local fixtures | Controlled HTTP behavior tests where needed |
| `src/providers/local_fixtures.rs` | Used by E2E | Path traversal, missing files, non-PDF filtering, and download size-boundary tests |
| `src/pdf_processing.rs` | Focused unit tests for transformation/layout behavior; deterministic generated-input checks cover normalization idempotence, safe markup, and classification; PDF extraction exercised by Miniflux E2E and the PDF byte fuzz target | Additional minimized failures as they are discovered |
| `src/rss.rs` | Publication date format/invalid date and XML escaping unit tests; the input fuzz target calls production date parsing; Miniflux E2E checks emitted RSS, safe markup, GUIDs, and redirects | Additional minimized failures as they are discovered |
| `src/scheduler.rs` | Positive refresh interval parser tests; fixed-clock SQLite tests for current/older intervals, feed overrides, duplicate suppression, forced scopes, and cache preservation after provider failure | Additional edge conditions as scheduling behavior evolves |

## Web application

| Production area | Existing coverage | Coverage to add |
| --- | --- | --- |
| `web/src/App.tsx` | `App.test.tsx`, browser environment/mobile/operator journeys | New session and navigation states as workflows change |
| `web/src/components/feeds/*` | `FeedManagement.test.tsx`; browser operator journey covers trimmed user text, invalid refresh values, maximum numeric boundary, URL rejection feedback, and feed/year/month workflows | New user-input boundaries as forms change |
| `web/src/components/documents/PdfReview.tsx` | `PdfReview.test.tsx`; browser operator journey | Missing/error/empty states only where current tests do not cover expected behavior |
| `web/src/lib/api.ts` | `api.test.ts` covers same-origin JSON requests, empty responses, common HTTP/network failures, and malformed successful JSON bodies | Additional API contract errors as they are introduced |
| `web/src/components/ui/*`, `main.tsx`, `lib/utils.ts`, styles/assets | Rendered through application tests/build; no dedicated component tests identified | Add focused tests only for behavior-bearing primitives/utilities; visual primitives and static assets remain covered by integration/build checks |

## Cross-layer checks

- `web/tests/browser/*`: responsive layout, accessibility, sign-in, navigation, and operator workflows.
- `tests/e2e/test_miniflux.py`: feed publication, PDF extraction, safe RSS content, exact document links, cache identity, and reader ingestion.
- `.github/workflows/ui-tests.yml`: frontend, browser, Miniflux, and production web-image checks.
- `.github/workflows/rust-tests.yml`: Rust formatting and deterministic tests on pull requests and pushes to `main`.
- `.github/workflows/fuzz-smoke.yml` and `fuzz-long.yml`: bounded PR fuzzing plus scheduled/manual longer campaigns with retained run data.
- Fuzz coverage: request JSON and URL validation, provider JSON parsing, filename/date parsing, extracted-text transformation, and PDF bytes. Smoke runs are time-bounded; retain minimized failures as regression tests or corpus inputs.

## Expected verification commands

- Rust: `cargo fmt --all -- --check`, `cargo fmt --manifest-path fuzz/Cargo.toml -- --check`, `cargo test --all-targets`.
- Web: `npm run lint`, `npm run typecheck`, `npm test`, and `npm run build` from `web/`.
- Browser and contract: `npm run test:e2e` from `web/`, `bash scripts/ui-e2e.sh`, and `bash scripts/e2e.sh`.
- Fuzz: run `PDFIUM_LIB_DIR=/path/to/pdfium/lib FUZZ_SMOKE_SECONDS=30 ./scripts/fuzz-smoke.sh`; longer campaigns run separately from pull-request checks.

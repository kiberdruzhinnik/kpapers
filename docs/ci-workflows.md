# CI workflows, checks, and caching

This document records the pull request check map, branch-protection migration, and build-cache behavior for the GitHub Actions workflows. `docs/testing.md` remains the canonical test coverage map.

## Check map and ownership

| Trigger | Workflow | Jobs / suites | Behavior |
| --- | --- | --- | --- |
| Pull request to `main` | `ci.yml` | `Rust checks`, `Frontend lint and unit checks`, `Browser and RSS integration` | Jobs start independently. The integration job builds both production images and runs the browser and RSS suites. |
| Pull request to `main` | `code-quality.yml` | `ShellCheck scripts`, `Ruff Python checks`, `Semgrep Community Edition`, `Trivy dependency and configuration scan` | Coverage is unchanged; report artifacts are still uploaded. |
| Pull request to `main` | `fuzz-smoke.yml` | `Parser and PDF fuzzing` | Smoke duration is 15 seconds per target; crash artifacts are uploaded on failure. |
| Weekly schedule and manual dispatch | `fuzz-smoke.yml` | `Parser and PDF fuzzing` | Extended duration is 1800 seconds by default; manual runs may set another duration. |
| Push to `main` or `v*.*.*` tags | `publish-containers.yml` | `Build and publish api`, `Build and publish web` | Build, scan, and publish behavior is unchanged. |

Every existing PR, scheduled, manual, and publishing suite remains assigned to a workflow job. There is no separate owner metadata in the repository; ownership follows the workflow above.

### Branch-protection migration

The pull request jobs were previously named `Build application and test images` and `Rust, frontend, browser, and RSS checks`. They are now split into:

- `Rust checks`
- `Frontend lint and unit checks`
- `Browser and RSS integration`

Required status checks could not be read from this environment because GitHub CLI authentication is unavailable and the GitHub API was rate-limited. Before merging, verify the required contexts in repository settings. If the old names are required, replace them with all three new names in the same change. Roll back the workflow and branch-protection change together if the new checks do not report.

## Pull request orchestration

Rust, frontend, and integration validation start at the same time. The integration job builds the API and web images with the production Dockerfiles, then runs `E2E_SKIP_APP_BUILD=true ./scripts/integration-e2e.sh`. This removes the image artifact upload, download, and `docker save`/`docker load` sequence; the local transfer cost was about 11 seconds plus artifact transfer for a 64 MB archive. The Playwright runner is still built inside the integration job.

The previous PR commands map to the new jobs as follows. Each command still runs once, and any non-zero exit still fails the workflow.

| Previous command or step | New owner |
| --- | --- |
| `E2E_BUILD_ONLY=true E2E_KEEP_IMAGES=true ./scripts/ui-e2e.sh` | `Browser and RSS integration`, built with `docker/build-push-action` |
| `docker save`, artifact upload and download, `docker load` | Removed; the integration job builds and uses the images in place |
| `./scripts/check-rust.sh` | `Rust checks` |
| `cd web && npm ci && npm run lint && npm test` | `Frontend lint and unit checks` |
| `E2E_SKIP_APP_BUILD=true ./scripts/integration-e2e.sh` | `Browser and RSS integration` |

Superseded pull request runs are canceled for the same pull request in `ci.yml`, `code-quality.yml`, and PR-triggered `fuzz-smoke.yml` runs. Scheduled fuzzing and manual fuzzing use unique concurrency groups and are never canceled by another run. Publishing is not given cancellation concurrency.

## Container build caching

### API image

`Dockerfile.api` first copies `Cargo.toml` and `Cargo.lock`, builds a stub binary to compile dependencies, and then copies `src` and `migrations` for the real release build. A source-only change reuses the dependency layer. The build uses Cargo registry and Git cache mounts, and `--locked` keeps the committed lockfile authoritative. The final build touches the copied sources so Cargo cannot treat the stub binary as up-to-date when Docker preserves older source timestamps. The dependency layer is invalidated by `Cargo.toml`, `Cargo.lock`, the `rust:1-bookworm` image, and Dockerfile changes. Source and migration changes run the final release build against the cached dependency artifacts. Changing the pinned Rust image digest invalidates the builder stage, and a different `platforms` value uses a separate BuildKit platform and cache key.

Local measurement on 2026-11-12 with Docker 29.9.3 and BuildKit v0.34.2:

| API build | Elapsed |
| --- | --- |
| Cold build, including dependency compilation | 119.2 s |
| Dependency layer | 86.0 s |
| Full release compile after dependencies | 11.8 s |
| Source-only rebuild | 19.9 s |
| Source-only release compile | 14.9 s |

### Web image

`web/Dockerfile` installs dependencies in a layer keyed by `package.json` and `package-lock.json`, then copies the source and runs `npm run build`. The npm cache mount speeds up local rebuilds when the dependency layer is invalidated. A lockfile or package-manifest change rebuilds the dependency layer; a source-only change reuses it.

Local measurement: cold web image build 38.9 s, including 23.8 s for `npm ci` and 6.7 s for the production build.

### Playwright runner

`web/Dockerfile.e2e` starts from the pinned `mcr.microsoft.com/playwright:v1.65.0-noble` image, installs the matching `@playwright/test` version, then copies the browser tests. The base image already contains the browsers, so browser download is skipped and the npm cache mount is used. The base image pull dominates runner setup: locally it measured 49.9 s for a 3.65 GB image, while the runner build took 38.0 s including export. No alternative runner image was adopted without evidence, and the pinned browser version is retained.

### GitHub Actions cache

The integration job uses BuildKit's GitHub Actions cache with the same `api` and `web` scopes as `publish-containers.yml`. Pull requests can restore cache produced on `main`; cache writes from pull requests remain scoped to that pull request, including for forks. Cache export uses `ignore-error=true`, so an unavailable or rejected cache cannot fail validation. A cold cache still produces a complete, production-equivalent image. The cache is an optimization, not a correctness dependency.

### Rust cache

`Swatinem/rust-cache` caches the stable `target` directory. Stable application and fuzz Clippy checks share that directory so compatible dependency artifacts are reused; nightly fuzz testing keeps the separate `fuzz/target` directory used by `fuzz-smoke.yml`. Warm local measurements were:

| Rust check | Separate targets | Shared target |
| --- | --- | --- |
| Application Clippy | 3.4 s | 0.2 s after the first shared build |
| Fuzz Clippy | 0.4 s | 31.0 s to add fuzz-only artifacts, then 0.2 s |
| Rust tests | 2.9 s | 1.9 s |

All locked checks and 63 Rust tests passed with the shared directory. These are warm local caches; cold hosted-runner timings and cache hit rates were not available.

### Image equivalence and publishing

The integration job builds the same `Dockerfile.api` with the repository root context and the same `web/Dockerfile` with the `web` context that `publish-containers.yml` uses, both for `linux/amd64`. The browser and Miniflux suites consume those tagged `kpapers-api:e2e` and `kpapers-web:e2e` images through their isolated Compose projects. `publish-containers.yml` was not changed: it still builds each image, scans it with Trivy, and only publishes when the scan job succeeds.

## Verification and evidence

Local checks run on 2026-11-12:

- `./scripts/check-rust.sh`: formatting, application and fuzz Clippy, and 63 Rust tests passed with the shared stable target directory.
- `cd web && npm ci && npm run lint && npm test`: lint completed with the existing warnings only; 51 Vitest tests passed across 6 files.
- `E2E_SKIP_APP_BUILD=true ./scripts/integration-e2e.sh`: 13 Playwright tests and the Miniflux RSS contract suite passed against the rebuilt production images in 152.0 s, including the cached runner build and cleanup. The scripts removed their `.env` files, fixtures, Compose networks and volumes, and the `kpapers-*:e2e` images on exit. A failed integration run during development also removed the same resources through the cleanup trap.
- `shellcheck ./scripts/*.sh`, Ruff 0.15.10 against `tests/e2e`, and actionlint 1.7.9 across all workflows passed.
- A no-cache API build produced a 15.5 MB binary that returned a valid Argon2 hash; the Playwright runner resolved `/ms-playwright/chromium-1254/chrome-linux64/chrome`.

Hosted failure propagation, security-report uploads, fuzz artifact retention, cache hit rates, and runner minutes remain to be observed after the workflow runs. Their workflow definitions are unchanged except for the PR concurrency groups, so the existing `if: always()` upload and cleanup behavior is preserved.

## Local reproduction

Build and inspect the production-equivalent images directly:

```bash
docker build --progress=plain -t kpapers-api:local -f Dockerfile.api .
docker build --progress=plain -t kpapers-web:local -f web/Dockerfile web
docker build --progress=plain -t kpapers-ui-e2e:local -f web/Dockerfile.e2e web
```

Then run the canonical integration suite with prebuilt application images:

```bash
docker tag kpapers-api:local kpapers-api:e2e
docker tag kpapers-web:local kpapers-web:e2e
E2E_SKIP_APP_BUILD=true ./scripts/integration-e2e.sh
```

The scripts create fixtures under `.tmp/` and bind-mount them into Compose containers. `act` must run where the Docker daemon can see that path. If `act` itself runs in a container, mount the repository at the same absolute path inside the `act` container; otherwise the browser and RSS fixture mounts fail even though image builds succeed. The local `act` baseline observed a 2 m 11 s cold API build and 1 m 35 s Rust checks, with the browser fixture mount timing out before the suites ran.

## Baseline and evidence limits

The authoritative timings are GitHub-hosted pull request runs; they are not available in this workspace. The repository has no hosted run history, `gh` authentication, or readable branch-protection configuration. Local `act` results and Docker measurements on this machine are useful for validating orchestration, layer reuse, and coverage, but they are not hosted-runner timings and must not be presented as a measured PR speedup.

The modelled critical path changes from build images → artifact transfer → serial Rust, frontend, browser, and RSS checks to `max(Rust checks, Frontend checks, image builds + browser + RSS)`. Local measurements show that removing the artifact transfer saves roughly 11 seconds plus upload/download time, while the cold image build remains on the integration branch. Warm source-only image rebuilds rely on the dependency layer and are much shorter.

Before/after comparison to complete after the first hosted runs:

- PR critical path and per-job durations for cold and warm caches
- BuildKit and Rust cache hit rates
- Runner minutes consumed by the parallel jobs versus the previous two-job layout
- Required check names in branch protection
- Fuzz smoke duration and artifact upload behavior on failure

# Design

## Context

See proposal.md for the motivation. `ci.yml` builds images before starting a single job that runs Rust checks, frontend checks, and integration checks sequentially. The API Dockerfile copies all source before its release build; the native Rust check uses separate application and fuzz target directories; the browser runner has its own Playwright image and npm installation. Fuzz smoke and security jobs run separately. A local `act` run observed a cold API image build of 2m 11s and Rust checks of 1m 35s, but its browser fixture mount timed out; local timings and caches do not represent GitHub-hosted runners.

## Goals / Non-Goals

**Goals:**

- Shorten the PR critical path while preserving the check map, exact production-image integration coverage, and failure visibility.
- Favor measurable improvements that work on clean hosted runners and are safe when caches are absent or invalidated.
- Keep job names and branch protection migration explicit, and make results reproducible locally as far as the runner allows.

**Non-Goals:**

- Remove checks, change assertions, shorten fuzz duration, or change release publishing behavior.
- Promise a fixed completion time before representative GitHub timings are available.
- Assume release, test, Clippy, and nightly instrumented fuzz builds are interchangeable.

## Decisions

1. **Parallelize by dependency.** Start Rust checks, frontend lint/unit checks, and production image preparation independently. Run browser and RSS integration after the images are ready, preserving their isolated Compose environments. Preserve the existing checks' result visibility and ensure required branch protection contexts remain valid or provide an explicit migration plan. A single serialized test job saves some setup but unnecessarily extends the critical path.

2. **Cache builds by compatible inputs.** Use persistent BuildKit caching for PR image builds, including dependency layers that survive application source edits; evaluate a dependency-planning stage or equivalent while ensuring lockfile, toolchain, features, target, and source changes invalidate correctly. Keep image outputs equivalent to production runtime images. Scope and permissions must permit safe cache reads, including fork PRs, and cold-cache builds must work. Compare reuse versus transfer overhead for image artifacts; retain transfer or combine build and integration based on measured critical-path cost. Cache is an optimization, never a correctness dependency.

3. **Measure native Rust reuse before adopting it.** Trial a shared target directory for stable app and fuzz Clippy only if Cargo's fingerprints, feature sets, lockfiles, and toolchain allow safe reuse; retain separate directories if reuse is poor or causes invalidation or correctness issues. Keep nightly sanitizer-instrumented fuzzing separate. Preserve the canonical local check commands and check every existing target.

4. **Reduce browser setup based on evidence.** Measure the Playwright base pull, npm install, build/export, and integration runtime. Evaluate a compatible cached runner or leaner setup, preserving browser version, all tests, and production-image coverage. Do not assume caching a multi-gigabyte image is faster than pulling it; compare total restore and save time.

5. **Measure complete feedback, not just compilation.** Record job and step durations, cache hits, and wall-clock critical path for comparable cold and warm runs, including source-only and dependency changes when available. Use `act` for workflow validation with a clean checkout and a fixture path accessible to the host Docker daemon; explicitly record any remaining `act` limitation. GitHub-hosted results are authoritative for PR speed. Cancel superseded runs per PR without canceling unrelated branches or release/scheduled workflows.

## Risks / Trade-offs

- **[Parallel jobs consume more runner minutes]** → Compare elapsed time and total runner usage, and avoid duplicating builds merely to parallelize.
- **[Cache misses, stale artifacts, or fork restrictions]** → Key by compatible inputs, allow clean rebuilds, and validate dependency/source invalidation and untrusted PR behavior.
- **[Renamed checks break branch protection]** → Inspect existing required contexts when accessible, preserve them or document a coordinated migration and rollback.
- **[Local Docker-in-Docker fixtures differ from hosted CI]** → Verify bind mount visibility explicitly; report local failures and confirm hosted behavior rather than weakening tests.
- **[Cached or altered images do not match release behavior]** → Verify image target, platform, build inputs, and unchanged security scanning and publication sequence.

## Migration Plan

Record a baseline, change orchestration and caches incrementally, validate cold and warm runs and coverage, then update documentation and required check configuration if necessary. Compare hosted timings after the change; roll back workflow and build-cache changes together if coverage, image equivalence, or required checks regress.

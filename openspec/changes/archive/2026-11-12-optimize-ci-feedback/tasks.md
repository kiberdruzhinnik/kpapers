# Tasks

## 1. Baseline and check ownership

- [x] 1.1 Record available PR job and step timings, cache hits, runner usage, and the current validation/check-name map; distinguish local `act` results from hosted runs and document unavailable evidence.
- [x] 1.2 Inspect required check contexts when accessible and document how their names will be preserved or migrated; verify every existing PR, scheduled, manual, and publishing suite has an owner.

## 2. Parallel PR validation

- [x] 2.1 Separate Rust and frontend checks from image preparation and make integration depend only on required images; validate workflow syntax and verify each existing command and failure result remains represented once.
- [x] 2.2 Add scoped cancellation of superseded PR runs and verify different PRs, scheduled fuzzing, and release publishing are unaffected.
- [x] 2.3 Update the validation map and local commands in `docs/testing.md`; verify documented triggers, job names, and dependencies match the workflows.

## 3. Container build caching

- [x] 3.1 Add persistent PR BuildKit caching and dependency-aware API build layers; verify cold builds, source-only rebuilds, dependency changes, and compatible fork PR behavior without relying on a cache hit.
- [x] 3.2 Compare image artifact transfer with build-and-test in one job and adopt the lower critical-path option when practical; verify both integration suites use the intended production-equivalent API and web images and release scanning/publishing remains intact.
- [x] 3.3 Document cache keys, invalidation, and local build instructions; verify source, lockfile, toolchain, and target changes do not reuse incompatible outputs.

## 4. Rust and browser setup

- [x] 4.1 Measure stable application and fuzz Clippy with shared versus separate target directories; adopt sharing only if Cargo safely reuses compatible artifacts and both locked checks and Rust tests pass, otherwise record the result.
- [x] 4.2 Measure browser image pull, build, install, restore, and export costs; adopt a faster compatible runner setup if justified, and verify all browser tests and the RSS suite still run against production-equivalent images.
- [x] 4.3 Document the chosen Rust and browser cache behavior and `act` fixture-mount prerequisites; run the documented local checks or record any environment-specific limitation and its evidence.

## 5. End-to-end verification

- [x] 5.1 Run the revised applicable checks and compare the full validation map with the baseline, including failure propagation, security reports, fuzz artifacts, and isolated integration cleanup.
- [x] 5.2 Compare available cold and warm before/after job timings, cache hit rates, PR critical path, and runner usage; report missing hosted evidence and any branch-protection migration needed without claiming an unmeasured speedup.

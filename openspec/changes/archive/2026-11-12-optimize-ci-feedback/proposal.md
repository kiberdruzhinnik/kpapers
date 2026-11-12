# Proposal

## Why

Pull request feedback can take around 20 minutes, with image builds, Rust checks, frontend checks, and integration tests unnecessarily serialized. Cold runs also repeat dependency compilation and installation across isolated build contexts, increasing iteration time.

## What Changes

- Run independent pull request validation suites concurrently and make integration checks wait only for their required images.
- Persist compatible container build caches and reduce repeated Rust dependency work where measurement confirms reuse is correct and worthwhile.
- Evaluate the browser runner's image and dependency setup, and reduce its cold-start cost where practical.
- Preserve existing test, lint, fuzz, security, integration, and publishing coverage and failure behavior; cancel superseded runs where safe.
- Measure before and after step durations and cache behavior, document local reproduction and remaining limitations, and verify required checks remain usable.

## Capabilities

### New Capabilities

None. This change affects development tooling and CI orchestration.

### Modified Capabilities

None. Product requirements remain unchanged.

## Impact

GitHub Actions workflows, Docker build configuration, Rust and browser test setup, and CI/testing documentation may change. GitHub cache storage, runner usage, and branch protection check names require review; application behavior and release image validation remain intact.

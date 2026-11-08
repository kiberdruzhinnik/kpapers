# Proposal

## Why

The operator experience currently lives in a single static HTML file with inline CSS and JavaScript, which makes a consistent, polished interface and reliable interaction coverage difficult to maintain. A clean dashboard built from shadcn/ui patterns will make feed operations easier to scan and use while establishing a frontend test suite for its critical workflows.

## What Changes

- Replace the single-file operator UI with a cohesive dashboard covering sign-in, feeds, year and month setup, refresh status and actions, PDF review, and archive, restore, and permanent deletion.
- Adopt a clean default shadcn/ui visual style with responsive layouts and accessible labels, focus states, feedback, and destructive-action confirmation.
- Make PDF review open the selected individual PDF and organize review navigation by feed, year, and month.
- Add frontend component and interaction tests, browser end-to-end tests for the primary operator journeys and narrow layouts, and CI checks for frontend quality and production builds.
- Preserve existing management API behavior and the isolated Miniflux end-to-end test for public RSS behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `feed-management`: Define the unified responsive operator dashboard and consistent management interactions across the existing feed and hierarchy workflows.

## Impact

The web client, its container build, frontend dependencies and scripts, automated test configuration, and CI workflow are affected. The Rust API and management contracts remain unchanged. Public RSS presentation remains the responsibility of external readers.

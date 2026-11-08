# Design

## Context

See `proposal.md` for motivation and scope. The current quality, Rust, UI/integration, and fuzz smoke workflows each listen to both `pull_request` targeting `main` and `push` to `main`. The container workflow builds and scans both production images for pull requests, and publishes on `main` pushes and version tags. `ui-tests.yml` also contains a separate non-publishing production web-image build. The scheduled/dispatchable extended fuzz workflow is independent of these events.

## Goals / Non-Goals

**Goals:**

- Keep existing PR check jobs and their result names available as merge checks.
- Make a successful PR the point where validation runs; a merge push should trigger publication without repeating those checks.
- Keep container vulnerability scanning ahead of publication, and never publish images from pull requests.
- Keep version tags as a supported release path.
- Avoid building the production web image twice for one pull request.

**Non-Goals:**

- Change what code, dependencies, or image layers the existing scanners cover or whether findings are merge-blocking.
- Change the existing test commands, fuzz duration, runner selection, action versions, image naming, or supported platforms.
- Remove the scheduled or manually dispatched extended fuzz workflow.

## Decisions

### Restrict validation workflows to pull requests

Keep `pull_request` targeting `main` in `code-quality.yml`, `rust-tests.yml`, `ui-tests.yml`, and `fuzz-smoke.yml`; remove their `push` to `main` triggers. Preserve job names and steps so branch protection can continue requiring the same checks. Do not add path filters, which could cause a required check to be absent on some pull requests.

Alternative considered: retain main-push checks for defense in depth. This repeats the same work immediately after the pull request has passed and delays publication without adding a distinct validation signal.

### Keep image build and scan on PRs, and publish only after main or release tags

Retain pull-request image builds and Trivy image scans in `publish-containers.yml`, with publication steps guarded from PR events. Keep pushes to `main` and version tags as publishing events; those builds are scanned before being pushed. Preserve existing metadata behavior: default-branch builds receive `latest` and short-SHA tags, and version-tag builds receive the release tag and short-SHA tag. A pull request must not receive registry credentials or package write access beyond the workflow's current permission setup.

Alternative considered: build images only after merge. That would reduce PR build work but would no longer catch Docker build or image-scan issues before merge, contrary to the existing security design.

### Remove the UI workflow's duplicate production web-image job

Remove `production-web-image` from `ui-tests.yml`; `publish-containers.yml` already builds and scans the production web image on PRs. Keep `npm run build` in the frontend checks because it validates the frontend build independently of the Docker image.

Alternative considered: remove PR image builds from `publish-containers.yml` and keep the UI workflow image job. That would leave the API image without equivalent PR validation and split production image checks across workflows.

### Leave extended fuzzing independent

Keep the scheduled and manually dispatched `fuzz-long.yml` workflow unchanged. It is a separate, longer-running campaign and is not a duplicate of the bounded PR smoke run.

## Risks / Trade-offs

- [A main-branch push that bypasses pull requests will not run validation workflows] → Treat pull-request checks and branch protection as the intended merge path; publication still runs on every `main` push.
- [Required checks could be misconfigured if job names or event filters change together] → Preserve job names and verify each required check appears for a PR targeting `main`.
- [Images are built once for PR scanning and again for publication after merge] → Keep this separation so the merge build is produced from the exact main-branch commit and scanned before publish; shared artifact handoff is outside this change.
- [Version tag publication remains independent of PR validation] → Keep tag triggers for release compatibility and ensure tag builds still scan before pushing.

## Migration Plan

1. Remove `push: branches: [main]` from the four validation workflows and remove the duplicate production web-image job.
2. Keep the container workflow's PR build-and-scan path and its `main` and version-tag publication paths.
3. Open a PR and confirm all expected checks appear once, both production images build and scan without publishing, and the UI workflow still runs the frontend production build.
4. Merge to `main` and confirm only container build, scan, and publish work runs; confirm `latest` and short-SHA tags are produced.
5. Publish a version tag and confirm release and short-SHA tags are produced after scans succeed.

## Open Questions

None.

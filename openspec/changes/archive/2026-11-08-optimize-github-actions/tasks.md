# Tasks

## 1. Restrict validation workflows to pull requests

- [x] 1.1 Update `code-quality.yml`, `rust-tests.yml`, `fuzz-smoke.yml`, and `ui-tests.yml` to run on pull requests targeting `main` only; verify each retains its current jobs and names and no longer subscribes to `push` on `main`.
- [x] 1.2 Keep `fuzz-long.yml` on its existing schedule and manual trigger; verify its event configuration remains unchanged.

## 2. Consolidate production image work

- [x] 2.1 Remove the duplicate `production-web-image` job from `ui-tests.yml` while keeping the frontend `npm run build`; verify the UI checks still include the frontend production build.
- [x] 2.2 Keep API and web image builds and Trivy image scans on PRs without publishing, and retain publishing on `main` pushes and version tags; verify PR publish/login steps remain disabled and the `latest`, short-SHA, and version-tag metadata behavior remains intact.

## 3. Verify workflow configuration

- [x] 3.1 Validate the edited workflow YAML and confirm the event matrix: PRs run validation and image build/scan jobs; pushes to `main` and version tags run image build/scan/publish jobs without rerunning validation workflows.
- [x] 3.2 Confirm that the PR checks retain their existing job names for branch protection and that no path filters cause required checks to be omitted.

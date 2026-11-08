# Proposal

## Why

Test and quality checks are spread across several GitHub Actions workflows and local commands, with overlapping work and documentation that does not always match CI. This makes failures harder to diagnose and increases CI time without adding coverage; a consistent, non-duplicative check plan will make local verification and pull request results easier to trust.

## What Changes

- Define canonical local commands for each validation suite and make GitHub Actions use those same entry points.
- Remove redundant execution of checks while preserving unique unit, integration, browser, security, and fuzz coverage.
- Reduce repeated dependency setup and container builds across workflows where outputs can be safely reused.
- Align test documentation with the checks CI actually runs and the local commands contributors should use.
- Keep flaky tests deterministic by fixing the underlying test setup or synchronization rather than weakening assertions.

## Capabilities

### New Capabilities

None. This change concerns repository development tooling and CI behavior, not product behavior.

### Modified Capabilities

None.

## Impact

Affected areas include `.github/workflows/`, root and web package scripts, `scripts/`, test configuration, and `docs/testing.md`. The change should preserve supported application behavior and existing distinct test coverage while simplifying how checks are invoked and organized.

# Tasks

## 1. Restrict scheduled refreshes to the current month

- [x] 1.1 Update scheduler eligibility so only the current UTC year and month can queue automatic jobs, while preserving current-month defaults and feed overrides; verify with scheduler tests that older months and overridden older months are not queued and that a month becomes eligible when the calendar advances.
- [x] 1.2 Update scheduler tests for current-month-only automatic refresh, including pending-job deduplication and current-month interval overrides; verify with `cargo test scheduler::tests::scheduler_`.

## 2. Document the scheduling behavior

- [x] 2.1 Update `docs/refresh-and-cache.md` to describe current-month-only automatic refresh, remove the obsolete older-month interval setting, and retain manual refresh behavior; verify the documented defaults and environment variables match the scheduler implementation.

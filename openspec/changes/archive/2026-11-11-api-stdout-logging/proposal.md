# Proposal

## Why

Operators can inspect Nginx access logs, but the API currently emits almost no operational information. This makes API startup, request handling, and background refresh failures difficult to diagnose through the container platform's standard log stream.

## What Changes

- Add an API logging capability that writes useful operational events to stdout for collection by Docker and `docker compose logs api`.
- Cover API startup, HTTP requests, and background refresh scheduling or job failures, while keeping credentials, cookies, and sensitive provider details out of log output.

## Capabilities

### New Capabilities
- `api-logging`: Operational API events are emitted to stdout in a container-friendly format.

### Modified Capabilities

None.

## Impact

- API startup and router setup in `src/main.rs`, and scheduler reporting in `src/scheduler.rs`.
- Rust logging dependencies and tests.
- TrueNAS deployment documentation describing where API logs are available.

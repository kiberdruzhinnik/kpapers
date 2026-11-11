# Design

## Context

The API runs as the `kpapers-api` process in its own container. It currently has no tracing subscriber or HTTP request layer; scheduler failures use `eprintln!`. Docker already collects process stdout and stderr, and operators use `docker compose logs api`.

## Goals / Non-Goals

**Goals:**
- Emit structured operational events on stdout for container log collection.
- Cover startup, HTTP request completion, and scheduler failures.
- Keep request and error fields free of credentials and secret-bearing URL components.

**Non-Goals:**
- Sending logs directly to a syslog daemon or remote logging service.
- Changing Nginx access logging or deployment log-driver configuration.
- Logging request or response bodies.

## Decisions

- Use Rust tracing events with a stdout subscriber and JSON line formatting. This gives container log collectors consistently structured records without adding a syslog network dependency or requiring container changes.
- Add request tracing at the Axum service boundary, recording method, URI path without query parameters, status, and elapsed time. Do not record headers, cookies, bodies, or query strings.
- Replace scheduler `eprintln!` calls with structured error events and include operation context. Keep normal recurring scheduler ticks quiet to avoid high-volume logs.
- Initialize logging before database migration and service startup so initialization failures can be reported. Report an initialization error once as a structured stdout event, then exit with a nonzero status without duplicating the error as plain text on stderr. Emit a startup event once the listener is serving.
- Keep log fields intentionally bounded to operational identifiers and safe provider error summaries. Exclude URLs, credentials, and response bodies, including credential values separated from their labels by whitespace; fixed safe summaries are an acceptable approach.
- Preserve useful diagnostics and nonzero exit status for `--healthcheck` and `--hash-password` failures, including when the normal server logging subscriber has not been initialized.

## Risks / Trade-offs

- [Structured request logs can increase log volume] → Emit one completion event per request and avoid per-poll scheduler events or payload logging.
- [Error strings may contain sensitive provider data] → Use safe error summaries and exclude raw provider responses, credentials, and full URLs.
- [JSON logs may be less readable in a raw terminal] → Preserve one-event-per-line output compatible with `docker compose logs` and structured collectors.

## Migration Plan

No data migration is required. Deploy the updated API image; Docker will collect its stdout automatically. Roll back by restoring the previous image tag.

# Design

## Context

The web image is built from the official Nginx Alpine image. It copies `web/nginx.conf` directly to `/etc/nginx/conf.d/default.conf`; that config currently uses the literal upstream `api:8080` for the `/api/`, `/feeds/`, and `/documents/` locations. Nginx's image entrypoint supports rendering templates from `/etc/nginx/templates/` with `envsubst` before starting Nginx.

## Goals / Non-Goals

**Goals:**

- Let each deployment configure the internal API origin without rebuilding the web image.
- Fail early with an actionable error when the required value is missing or malformed.
- Preserve the existing route and URI forwarding behavior.

**Non-Goals:**

- Changing API routes, browser-facing URLs, or application behavior.
- Supporting an API base path, credentials, query string, or fragment in `API_URL`.

## Decisions

- Treat `API_URL` as an origin with `http` or `https` scheme, hostname, and optional port, with no path other than `/`, credentials, query, or fragment. This matches service-to-service Compose URLs such as `http://kpapers-api:8080` and avoids ambiguity in Nginx `proxy_pass` URI replacement.
- Convert the Nginx config into an entrypoint template under `/etc/nginx/templates/`. Use `API_URL` in each API proxy location and rely on the official image's template rendering before Nginx starts.
- Add explicit startup validation before Nginx launches. Reject a missing or malformed value with a message naming `API_URL`; only a validated origin is substituted into the config.
- Pass `API_URL` explicitly through the web service's Compose `environment` and add it to the TrueNAS `.env.example`, set to `http://api:8080` to match the provided service name. Operators who rename that service set the matching Compose hostname instead.
- Keep `.env` loading scoped intentionally: Compose reads `.env` for interpolation, while container variables must be declared under the `web` service. The API's `env_file` does not configure the web container.

Alternatives considered: keeping the hardcoded service name requires renaming every API service to `api`; rebuilding the web image per deployment is operationally cumbersome; allowing an implicit default can hide a deployment naming mismatch, contrary to the required-setting behavior.

## Risks / Trade-offs

- [Nginx template substitution can produce invalid configuration if arbitrary text is accepted] → Validate the URL format and restrict it to an origin before rendering.
- [A syntactically valid host may still be unreachable or not resolve on the shared network] → Document that the value must resolve from the web container and ensure Compose services share a network.
- [Operators upgrading without adding `API_URL` will see the web container fail to start] → Include the setting in the TrueNAS example and provide a clear startup error.

## Migration Plan

1. Add the required variable to deployment Compose configuration and environment examples.
2. Existing deployments add `API_URL=http://api:8080`, or use the actual API service name and port, then pull and recreate the web container.
3. Rollback by restoring the prior web image and Compose configuration if needed; API and database state are unaffected.

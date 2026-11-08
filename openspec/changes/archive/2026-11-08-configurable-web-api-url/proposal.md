# Proposal

## Why

The web container's Nginx configuration hardcodes the API hostname as `api`, so deployments that use another Compose service name fail during Nginx startup with an unresolved upstream. Operators need to configure the internal API endpoint without rebuilding the web image, and a missing setting should be reported clearly rather than silently using the wrong host.

## What Changes

- Make the web container's API upstream configurable through a required `API_URL` environment variable.
- Validate `API_URL` at web container startup and render the Nginx proxy configuration from it.
- Update the TrueNAS Compose example and environment template with the API service URL.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `container-deployment`: Require and document web API endpoint configuration for container deployments.

## Impact

The web Nginx configuration and Docker image startup behavior, Compose deployment examples, and container deployment documentation are affected. The API service and its HTTP interface remain unchanged.

# container-deployment Specification

## Purpose

Configures how the web container locates the API service so Nginx can proxy API routes to it, and reports a clear startup error when the configured endpoint is missing or invalid.

## Requirements

### Requirement: Require a configurable API URL for the web container
The web container SHALL require a valid `API_URL` environment variable identifying the API service endpoint and use it for all Nginx routes proxied to the API. If `API_URL` is missing or invalid, the web container SHALL report a clear configuration error and SHALL NOT start Nginx.

#### Scenario: Configure a differently named API service
- **WHEN** an operator sets `API_URL` to the reachable internal API URL such as `http://kpapers-api:8080`
- **THEN** the web container starts and Nginx proxies API routes to that endpoint

#### Scenario: Missing API_URL
- **WHEN** the web container starts without `API_URL`
- **THEN** startup reports that `API_URL` is required and Nginx does not start

#### Scenario: Invalid API_URL
- **WHEN** the web container starts with an invalid `API_URL`
- **THEN** startup reports a clear configuration error and Nginx does not start

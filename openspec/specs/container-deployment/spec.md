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

### Requirement: Publish application container images
The project SHALL publish publicly pullable API and web container images to GitHub Container Registry from the repository's GitHub Actions workflow.

#### Scenario: Build from the default branch
- **WHEN** a commit is pushed to the default branch
- **THEN** the workflow builds and publishes both API and web images with a stable default-branch tag and a commit-specific tag

#### Scenario: Build from a version tag
- **WHEN** a version tag is pushed
- **THEN** the workflow publishes both images with that version tag

#### Scenario: Pull published images without authentication
- **WHEN** a deployment host pulls a published image from GHCR
- **THEN** the image is publicly accessible without registry credentials

### Requirement: Provide a TrueNAS Compose deployment example
The project SHALL provide a Compose example that deploys the API and web images from GHCR and loads operator configuration from a colocated `.env` file.

#### Scenario: Start the deployment
- **WHEN** an operator supplies the required values in `.env` and starts the Compose project
- **THEN** Compose pulls the published API and web images, starts the web service after the API is healthy, and makes the configured web port available

#### Scenario: Preserve application data
- **WHEN** the Compose project is stopped, recreated, or its images are updated
- **THEN** the API database remains stored in a persistent named volume mounted at `/data`

#### Scenario: Write data using the TrueNAS app identity
- **WHEN** the API image runs with a TrueNAS dataset or volume configured for the conventional `app` UID:GID `568:568`
- **THEN** the API process can read and write its database under `/data` as UID:GID `568:568`

#### Scenario: Configure operator credentials
- **WHEN** an operator configures the deployment
- **THEN** the example identifies the required operator password hash and session signing key settings and keeps their values outside the Compose file

#### Scenario: Update the deployment
- **WHEN** an operator requests the latest published images and recreates the services
- **THEN** Compose updates the running deployment while retaining its named data volume and `.env` configuration

### Requirement: Preserve and migrate production database state across updates
Every application update SHALL preserve existing database data and schema compatibility, or provide and apply a versioned database migration before the updated API serves requests.

#### Scenario: Update without a schema change
- **WHEN** an operator updates to a release that does not change the database schema
- **THEN** the new API remains compatible with the existing database and preserves its data

#### Scenario: Update with a schema change
- **WHEN** an operator updates to a release that changes the database schema
- **THEN** the release includes an ordered database migration that runs before the API serves requests and preserves existing user data

#### Scenario: Migration fails
- **WHEN** a database migration cannot be applied
- **THEN** the API does not start serving against the incompatible schema and reports the migration failure

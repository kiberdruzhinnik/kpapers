# Proposal

## Why

The repository currently builds its API and web containers locally, which makes deploying on TrueNAS SCALE dependent on having the source and build toolchain available there. Publishing ready-to-pull images and providing a Compose example will make updates and first-time deployments simpler.

## What Changes

- Add a GitHub Actions workflow that builds and publishes the API and web images to public GitHub Container Registry packages.
- Provide a TrueNAS SCALE Docker Compose example that pulls those images, reads operator configuration from an `.env` file, and preserves the API database in a named volume.
- Ensure the API image's data directory is writable by TrueNAS SCALE's conventional `app` identity (`568:568`).
- Define a database upgrade contract: releases preserve compatibility with existing data, and schema changes ship as ordered database migrations that run before the updated API serves requests.
- Document the required secrets, image references, and update procedure for the example deployment.

## Capabilities

### New Capabilities
- `container-deployment`: Publish application container images and provide a Compose deployment example for TrueNAS SCALE.

### Modified Capabilities

## Impact

The change affects GitHub Actions configuration, container image build and tag conventions, Compose deployment configuration, and deployment documentation. It does not change the API behavior or database format.

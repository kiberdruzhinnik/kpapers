# Design

## Context

The existing `compose.yaml` builds two services: `api` from `Dockerfile.api` and `web` from `web/Dockerfile`. The web image proxies to the API service by Compose service name. The API stores SQLite data under `/data` and currently runs as UID 10001. The API Dockerfile selects Pdfium binaries for amd64 and arm64. TrueNAS SCALE commonly maps app data to UID:GID `568:568`, so the published API image and its `/data` ownership must use that identity for dataset access. See `proposal.md` and `specs/container-deployment/spec.md` for motivation and required behavior.

## Goals / Non-Goals

**Goals:**
- Publish versioned, multi-platform images that match both existing build contexts.
- Make the deployment example usable from a TrueNAS SCALE app directory with only Compose, an `.env` file, and public image pulls.
- Preserve current API health checks, web-to-API routing, and database persistence.
- Run the published API process and own `/data` as UID:GID `568:568` so TrueNAS dataset permissions align with the host's `app` identity.
- Preserve existing production data across application releases and apply any required schema migration before the updated API serves traffic.

**Non-Goals:**
- Change the app's authentication model, database engine, or HTTP architecture.
- Publish a combined single-container image or require a private registry login.
- Configure TLS termination or TrueNAS-specific storage pools; operators may place a reverse proxy in front and choose host storage separately.

## Decisions

### Publish separate API and web packages to GHCR

Use `ghcr.io/${owner}/kpapers-api` and `ghcr.io/${owner}/kpapers-web`, built from the existing Dockerfiles. Change the API image's application UID and `/data` ownership to `568:568`, so named volumes and TrueNAS datasets can be written using the conventional `app` identity. Build and publish `linux/amd64` and `linux/arm64` manifests, matching the API's supported Pdfium architectures and typical NAS hosts. Use GitHub Actions Buildx and the workflow's `GITHUB_TOKEN` with `packages: write`; set package visibility to public as a one-time package setting because workflow permissions alone do not make packages public.

Tag default-branch builds as `latest` and with the short commit SHA. Tag version releases with the version tag as well as the short SHA. Avoid publishing mutable `latest` from arbitrary branches or pull requests. Pull requests may build without publishing to catch Dockerfile failures.

Alternative considered: publish one combined image. This would require changing the current two-service architecture and complicate independent health checks and proxy routing.

### Keep the TrueNAS example separate from the developer Compose file

Add a dedicated TrueNAS Compose file that uses the public GHCR image references, the same service names and health-check ordering as `compose.yaml`, and a named volume mounted at `/data`. Use a colocated `.env` for Compose interpolation and reference it as the API service's `env_file`, so API configuration is passed from the file while web does not receive operator secrets. Document that Compose's project directory must contain both files and that `PUBLIC_BASE_URL` must match the externally reachable URL; secure cookies should stay enabled when served over HTTPS.

Alternative considered: replace the existing `compose.yaml`. Keeping it avoids disrupting local development, which depends on building directly from source.

### Use explicit image tags in the deployment example

The example defaults to `latest` for straightforward updates and allows operators to pin a release by changing the image tag. Updating requires pulling the images and recreating the services; the named volume and `.env` remain in place. Rollback restores the prior image tag and recreates services; if a migration made the database incompatible with that image, restore the pre-upgrade database backup as well.

### Keep database changes behind versioned SQLx migrations

The API already embeds SQLx migrations from `migrations/` and runs them through `connect_and_migrate` during startup. Require every schema change to add a new ordered migration and preserve existing data. Startup must fail before binding the HTTP listener if migrations fail. Deployment instructions should tell operators to back up the SQLite database before updates; if a migration is not backward-compatible, rollback requires restoring that backup together with the previous image, not merely changing the image tag.

Alternative considered: make every migration automatically reversible. SQL migrations are not always safely reversible, especially after new writes, so backup-based recovery is the reliable general fallback.

## Risks / Trade-offs

- Public packages may not become public automatically → include a maintainer setup step and verify anonymous pulls after the first publish.
- Building two architectures increases CI time → use Buildx layer caching and keep pull-request builds non-publishing.
- `latest` can change between updates → publish immutable SHA and version tags and document version pinning for operators who prefer controlled upgrades.
- A schema migration may make the previous API image incompatible with the upgraded database → preserve data in forward migrations, stop startup on migration failure, and document database backup and restore alongside image rollback.
- TrueNAS Compose behavior can vary by SCALE release → keep the example standard Compose syntax and validate it with `docker compose config` and image pulls on a compatible host when available.

## Migration Plan

1. Add the publishing workflow and deployment files without changing the existing local `compose.yaml`.
2. Merge to the default branch and confirm both GHCR packages are public and can be pulled anonymously.
3. Copy the TrueNAS Compose example and `.env.example` into the TrueNAS app directory, fill in secrets and the public URL, then start the stack.
4. Before updates that can apply schema migrations, back up the SQLite database. To roll back, restore the prior image tag and recreate the services; if that image cannot use the migrated schema, restore the pre-upgrade database backup as well.

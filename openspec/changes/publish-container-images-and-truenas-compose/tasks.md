# Tasks

## 1. Publish API and web images

- [x] 1.1 Update the API image to run as UID:GID `568:568` and own `/data` accordingly, then add a GitHub Actions workflow that builds amd64-only API and web images, publishes default-branch and version tags to GHCR with `GITHUB_TOKEN`, and builds pull requests without publishing; verify image filesystem ownership and workflow syntax/trigger behavior with static validation.
- [ ] 1.2 Configure the GHCR API and web packages as public after their first workflow publish, then verify each can be pulled anonymously by image reference.

## 2. Add TrueNAS deployment files and instructions

- [x] 2.1 Add a TrueNAS Compose example that pulls both GHCR images, uses `.env` for API configuration, retains `/data` in a named volume, and keeps health checks and service ordering; verify it with `docker compose -f <truenas-compose-file> --env-file .env config` using a populated test env file.
- [x] 2.2 Add a matching `.env.example` and deployment guide covering secret generation, public URL and cookie settings, TrueNAS app directory setup, image updates, version pinning, and rollback; verify every documented path and Compose command against the example files.
- [x] 2.3 Document that schema changes require ordered SQLx migrations, explain that migrations run before the API begins serving, and include database backup/restore instructions for upgrades and rollback; verify the instructions match the startup migration behavior in `src/db.rs` and `src/main.rs`.

## 3. Verify deployment integration

- [x] 3.1 Build both images locally for `linux/amd64` and run the TrueNAS Compose stack against them; verify the web health check succeeds, web requests reach the API, and the database survives container recreation.
- [x] 3.2 Exercise a production-style upgrade from the previous database schema; verify migrations run before serving, existing data is retained, and a forced migration failure prevents the API health check from passing.

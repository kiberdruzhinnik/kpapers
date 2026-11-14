# TrueNAS SCALE deployment

This stack pulls the API, web, and Marker worker images from GitHub Container Registry. Keep `compose.yaml` and `.env` together in the TrueNAS app directory. The API database is stored in `./kpapers_data`; the Marker model cache is stored in a named volume. The API image runs as UID:GID `568:568` (`app`).

## Prepare the deployment

1. Copy this directory to the TrueNAS SCALE host and enter it:

   ```sh
   cd /path/to/truenas
   cp .env.example .env
   mkdir -p kpapers_data
   chown 568:568 kpapers_data
   ```

2. Create the external network used by your Nginx Proxy Manager deployment if it does not already exist:

   ```sh
   docker network create npm
   ```

   The web service joins both this `npm` network and the stack's default network. Nginx Proxy Manager can reach it by the Compose service name `kpapers-web` on port 80. The API and Marker worker remain on the private default network.

3. Generate a session signing key and an Argon2 password hash. The API image's password command reads one password line from standard input and prints the hash:

   ```sh
   openssl rand -base64 48
   docker run --rm -i --entrypoint /usr/local/bin/kpapers-api \
     ghcr.io/kiberdruzhinnik/kpapers-api:latest --hash-password
   ```

   Put the generated values in `SESSION_SIGNING_KEY` and `OPERATOR_PASSWORD_HASH` in `.env`. Wrap the password hash in single quotes because PHC hashes contain `$`, for example `OPERATOR_PASSWORD_HASH='$argon2id$v=19$…'`. Do not commit `.env` or share its contents. Use a strong, unique operator password.

4. Set `PUBLIC_BASE_URL` to the URL operators use to reach the service. Keep `COOKIE_SECURE=true` when HTTPS is configured at the reverse proxy. The web service uses `http://kpapers-api:8080` internally; this must match the API service name in `compose.yaml`.

## Extraction engine

The stack includes the Marker worker and defaults `PDF_EXTRACTION_ENGINE` to `marker`, so Marker is the global default while feed-level engine choices can still select either Marker or Pdfium. Set `PDF_EXTRACTION_ENGINE=pdfium` in `.env` to make Pdfium the global default. The worker downloads its pinned models on first startup and caches them in the `marker_models` volume; its health check can remain starting for up to 15 minutes while it initializes. Allow enough memory and CPU for the Marker model runtime. Worker logs are available with `docker compose logs -f kpapers-marker`.

## Start and update

Validate the configuration and start the stack:

```sh
docker compose --env-file .env -f compose.yaml config
docker compose --env-file .env -f compose.yaml up -d
```

The API, web, and Marker images are published to GHCR. The web service waits for the API health check; the Marker worker initializes independently. On the first workflow publish, a repository maintainer must set all three GHCR packages to public in their GitHub package settings.

For routine updates using `latest`:

```sh
docker compose --env-file .env -f compose.yaml pull
docker compose --env-file .env -f compose.yaml up -d
```

For a pinned release, set `IMAGE_TAG` in `.env` to its version tag, such as `v1.2.3`, then run the same pull and `up -d` commands. Record the prior image tag before upgrading.

## View service logs

The API writes structured JSON events, one per line, to stdout. Follow API or worker logs from the deployment directory:

```sh
docker compose --env-file .env -f compose.yaml logs -f kpapers-api
docker compose --env-file .env -f compose.yaml logs -f kpapers-marker
```

Omit `-f` to print collected logs and exit, or add `--tail 200` to limit the initial output.

## Database upgrades and recovery

The API applies embedded, ordered SQLx migrations during startup before it binds the HTTP port. Every schema change must add a migration under `migrations/` and preserve existing data. If a migration fails, the API exits instead of serving against a schema it cannot use.

Back up the SQLite database before each application update. Stop the API to ensure the file is consistent, copy the database, then start the stack again:

```sh
docker compose --env-file .env -f compose.yaml stop kpapers-api
cp kpapers_data/kpapers.sqlite "kpapers-$(date +%Y%m%d-%H%M%S).sqlite"
docker compose --env-file .env -f compose.yaml up -d
```

If an upgrade fails before a migration completes, inspect the API logs with `docker compose logs kpapers-api`. For a rollback after a schema migration, restore the pre-upgrade database together with the prior image tag; changing only the image tag may leave it incompatible with the migrated schema. To restore a backup, stop the stack, copy the selected backup into `kpapers_data`, and restore ownership to UID:GID `568:568`:

```sh
docker compose --env-file .env -f compose.yaml down
cp kpapers-YYYYMMDD-HHMMSS.sqlite kpapers_data/kpapers.sqlite
chown 568:568 kpapers_data/kpapers.sqlite
# Restore the prior IMAGE_TAG in .env before bringing the services back up.
docker compose --env-file .env -f compose.yaml up -d
```

Replace the backup filename with the file you saved. Keep the backup outside `kpapers_data`.

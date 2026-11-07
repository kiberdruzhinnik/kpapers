# TrueNAS SCALE deployment

This stack pulls the public API and web images from GitHub Container Registry. Keep `compose.yaml` and `.env` together in the TrueNAS app directory. The API image runs as UID:GID `568:568` (`app`) and stores the SQLite database in the `kpapers_data` named volume mounted at `/data`.

## Prepare the deployment

1. Copy this directory to the TrueNAS SCALE host and enter it:

   ```sh
   cd /path/to/truenas
   cp .env.example .env
   ```

2. Generate a session signing key and an Argon2 password hash. The API image's password command reads one password line from standard input and prints the hash:

   ```sh
   openssl rand -base64 48
   docker run --rm -i --entrypoint /usr/local/bin/kpapers-api \
     ghcr.io/kiberdruzhinnik/kpapers-api:latest --hash-password
   ```

   Put the generated values in `SESSION_SIGNING_KEY` and `OPERATOR_PASSWORD_HASH` in `.env`. Wrap the password hash in single quotes because PHC hashes contain `$`, for example `OPERATOR_PASSWORD_HASH='$argon2id$v=19$…'`. Do not commit `.env` or share its contents. Use a strong, unique operator password.

3. Set `PUBLIC_BASE_URL` to the URL operators use to reach the service. Keep `COOKIE_SECURE=true` when HTTPS is configured at a reverse proxy. Set `WEB_PORT` to the desired TrueNAS host port.

## Start and update

Validate the configuration and start the stack:

```sh
docker compose --env-file .env -f compose.yaml config
docker compose --env-file .env -f compose.yaml up -d
```

The API and web images are publicly pullable; registry login is not required. On the first workflow publish, a repository maintainer must set both GHCR packages to public in their GitHub package settings. The web service waits for the API health check, and the API database persists in the named volume when containers are replaced.

For routine updates using `latest`:

```sh
docker compose --env-file .env -f compose.yaml pull
docker compose --env-file .env -f compose.yaml up -d
```

For a pinned release, set `IMAGE_TAG` in `.env` to its version tag, such as `v1.2.3`, then run the same pull and `up -d` commands. Record the prior image tag before upgrading.

## Database upgrades and recovery

The API applies embedded, ordered SQLx migrations during startup before it binds the HTTP port. Every schema change must add a migration under `migrations/` and preserve existing data. If a migration fails, the API exits instead of serving against a schema it cannot use.

Back up the SQLite database before each application update. Stop the API to ensure the file is consistent, copy the database from its named volume, then start the stack again:

```sh
docker compose --env-file .env -f compose.yaml stop api
docker run --rm -v kpapers_truenas_data:/data:ro -v "$PWD:/backup" alpine:3.20 \
  sh -c 'cp /data/kpapers.sqlite "/backup/kpapers-$(date +%Y%m%d-%H%M%S).sqlite"'
docker compose --env-file .env -f compose.yaml up -d
```

If an upgrade fails before a migration completes, inspect the API logs with `docker compose logs api`. For a rollback after a schema migration, restore the pre-upgrade database together with the prior image tag; changing only the image tag may leave it incompatible with the migrated schema. To restore a backup, stop the stack, copy the selected backup into the named volume, and restore ownership to UID:GID `568:568`:

```sh
docker compose --env-file .env -f compose.yaml down
docker run --rm -v kpapers_truenas_data:/data -v "$PWD:/backup:ro" alpine:3.20 \
  sh -c 'cp /backup/kpapers-YYYYMMDD-HHMMSS.sqlite /data/kpapers.sqlite && chown 568:568 /data/kpapers.sqlite'
# Restore the prior IMAGE_TAG in .env before bringing the services back up.
docker compose --env-file .env -f compose.yaml up -d
```

Replace the backup filename with the file you saved. Keep the backup outside the Compose data volume.

# Deployment with Docker Compose

The deployment runs one Rust API and scheduler, plus an Nginx web container. Only the web container publishes a host port. The API is available to Nginx on a private Compose network and stores SQLite and extracted content in the named `kpapers_data` volume.

## Configure and start

Install Docker Engine and the Docker Compose plugin, then from the repository directory:

```sh
cp .env.example .env
```

Before generating the hash, edit `.env` and replace the blank values with temporary nonempty values so Compose can build and run the helper. These values are only for the build/hash command; do not start the stack until you replace them:

```dotenv
OPERATOR_PASSWORD_HASH=helper
SESSION_SIGNING_KEY=01234567890123456789012345678901
```

Then build the API image:

```sh
docker compose build api
```

Generate the Argon2 hash without echoing the password in the terminal:

```sh
read -r -s -p 'Operator password: ' operator_password; printf '\n'
printf '%s\n' "$operator_password" | docker compose run --rm --no-deps -T -i api --hash-password
unset operator_password
```

Replace the temporary `OPERATOR_PASSWORD_HASH` in `.env` with the resulting hash as a single-quoted value, for example `OPERATOR_PASSWORD_HASH='$argon2id$...'`; the quotes keep Compose from treating the hash's dollar signs as variable references. Generate a signing key with `openssl rand -hex 32` and replace the temporary `SESSION_SIGNING_KEY` in `.env`. The generated key is 32 bytes; shorter values make the API reject startup. Compose checks that both secrets are nonempty before creating containers. Keep `.env` private and out of source control. Set `PUBLIC_BASE_URL` to the externally visible base URL; for HTTPS use `COOKIE_SECURE=true`. For local HTTP development only, set `COOKIE_SECURE=false`.

Start or update the stack:

```sh
docker compose up --build -d
docker compose ps
```

Both services have container health checks. Compose waits for the API health check before starting the web service. The app is available on `http://localhost:${WEB_PORT:-8080}/manage/`. Sign in with the username and password used to create the hash. Configure feeds and month links using the [operator guide](operator-guide.md).

## Local human testing

For a disposable local environment, run:

```sh
./scripts/quickstart.sh
```

The script generates and prints a random operator password, binds the web service to all host interfaces on port `18081`, and uses its own `.env.quickstart`, Compose project, and named data volume. Open `http://localhost:18081/manage/` on the host, or use the host's IP address from another device on the network. This environment is non-production and does not share the deployment's `.env` or database. To remove the test stack and permanently delete its database, cached files, local images, and generated credentials, run:

```sh
./scripts/quickstart-down.sh
```

## Backup

Stop the single API instance first so SQLite has no active writes and its WAL is checkpointed. This example archives the entire named volume, including database and cached content:

```sh
docker compose stop
mkdir -p backups
docker run --rm \
  -v "${COMPOSE_PROJECT_NAME:-kpapers}_kpapers_data:/data:ro" \
  -v "$PWD/backups:/backup" \
  alpine:3.20 \
  tar -czf /backup/kpapers-data-$(date -u +%Y%m%dT%H%M%SZ).tar.gz -C /data .
docker compose start
```

Compose names the volume `<project>_kpapers_data`; by default the project name is the repository directory (`kpapers`). If `COMPOSE_PROJECT_NAME` is set, use its value in the volume name. Store backups securely because they contain operator-managed feed data and extracted article text.

## Restore

Stop the deployment. Preserve the current volume first if it contains data you may need. Remove and recreate the named volume, then extract the archive into it:

```sh
docker compose down
docker volume rm "${COMPOSE_PROJECT_NAME:-kpapers}_kpapers_data"
docker volume create "${COMPOSE_PROJECT_NAME:-kpapers}_kpapers_data"
docker run --rm \
  -v "${COMPOSE_PROJECT_NAME:-kpapers}_kpapers_data:/data" \
  -v "$PWD/backups:/backup:ro" \
  alpine:3.20 \
  sh -c 'tar -xzf /backup/CHOOSE-BACKUP.tar.gz -C /data && chown -R 10001:10001 /data'
docker compose up -d
docker compose ps
```

Replace `CHOOSE-BACKUP.tar.gz` with the selected archive filename. The API runs migrations at startup; verify that the operator can sign in and that configured public feeds return their cached entries.

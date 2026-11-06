#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT=kpapers-quickstart
ENV_FILE="$ROOT/.env.quickstart"
COMPOSE_FILE="$ROOT/compose.yaml"

if [[ $# -ne 0 ]]; then
  printf 'Usage: %s\nThis cleanup always targets only the fixed kpapers-quickstart project.\n' "$0" >&2
  exit 2
fi
command -v docker >/dev/null 2>&1 || { printf 'quickstart cleanup: Docker is required.\n' >&2; exit 1; }
if docker info >/dev/null 2>&1; then
  DOCKER=(docker)
elif sudo -n docker info >/dev/null 2>&1; then
  DOCKER=(sudo -n docker)
else
  printf 'quickstart cleanup: Docker is unavailable.\n' >&2
  exit 1
fi
"${DOCKER[@]}" compose version >/dev/null 2>&1 || { printf 'quickstart cleanup: Docker Compose plugin is required.\n' >&2; exit 1; }

printf 'This permanently deletes the kpapers-quickstart database, cached files, local images, and generated credentials.\n'
# An empty, explicit env file prevents Compose from reading the deployment's .env.
OPERATOR_PASSWORD_HASH=cleanup SESSION_SIGNING_KEY=01234567890123456789012345678901 \
  "${DOCKER[@]}" compose -f "$COMPOSE_FILE" --env-file /dev/null -p "$PROJECT" \
  down --volumes --remove-orphans --rmi local
rm -f -- "$ENV_FILE"
printf 'Quickstart containers, network, volume, local images, and environment file removed. The normal deployment was not targeted.\n'

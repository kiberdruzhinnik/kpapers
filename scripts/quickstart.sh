#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT=kpapers-quickstart
ENV_FILE="$ROOT/.env.quickstart"
COMPOSE_FILE="$ROOT/compose.yaml"

die() { printf 'quickstart: %s\n' "$*" >&2; exit 1; }
command -v openssl >/dev/null 2>&1 || die 'OpenSSL is required.'
command -v docker >/dev/null 2>&1 || die 'Docker is required.'
if docker info >/dev/null 2>&1; then
  DOCKER=(docker)
elif sudo -n docker info >/dev/null 2>&1; then
  DOCKER=(sudo -n docker)
else
  die 'Docker is unavailable. Grant access to the Docker socket or run this script where Docker is accessible.'
fi
"${DOCKER[@]}" compose version >/dev/null 2>&1 || die 'Docker Compose plugin is required.'
compose() { "${DOCKER[@]}" compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" -p "$PROJECT" "$@"; }

if [[ -L "$ENV_FILE" ]]; then die "$ENV_FILE must not be a symlink."; fi
umask 077
if [[ -f "$ENV_FILE" ]]; then
  operator_password="$(sed -n 's/^QUICKSTART_OPERATOR_PASSWORD=//p' "$ENV_FILE")"
  password_hash="$(sed -n "s/^OPERATOR_PASSWORD_HASH='\(.*\)'$/\1/p" "$ENV_FILE")"
  signing_key="$(sed -n 's/^SESSION_SIGNING_KEY=//p' "$ENV_FILE")"
  [[ -n "$operator_password" && -n "$password_hash" && -n "$signing_key" ]] || die "$ENV_FILE is incomplete; remove it and rerun quickstart."
else
  setup_in_progress=1
  trap 'if [[ "${setup_in_progress:-0}" == 1 ]]; then rm -f -- "$ENV_FILE" "$ENV_FILE.tmp.$$"; fi' EXIT
  operator_password="$(openssl rand -hex 12)"
  signing_key="$(openssl rand -hex 32)"
  cat > "$ENV_FILE" <<EOF
OPERATOR_USERNAME=operator
OPERATOR_PASSWORD_HASH=helper
SESSION_SIGNING_KEY=$signing_key
API_URL=http://api:8080
COOKIE_SECURE=false
WEB_PORT=18081
WEB_BIND_ADDRESS=0.0.0.0
PUBLIC_BASE_URL=http://localhost:18081
CURRENT_MONTH_REFRESH_SECONDS=3600
OLDER_MONTH_REFRESH_SECONDS=86400
QUICKSTART_OPERATOR_PASSWORD=$operator_password
EOF
  chmod 600 "$ENV_FILE"
  compose build api
  password_hash="$(printf '%s\n' "$operator_password" | compose run --rm --no-deps -T -i api /usr/local/bin/kpapers-api --hash-password)"
  [[ "$password_hash" == \$argon2* ]] || die 'API did not return a valid password hash.'
  tmp_file="$ENV_FILE.tmp.$$"
  {
    printf 'OPERATOR_USERNAME=operator\n'
    printf "OPERATOR_PASSWORD_HASH='%s'\n" "$password_hash"
    printf 'SESSION_SIGNING_KEY=%s\n' "$signing_key"
    printf 'API_URL=http://api:8080\n'
    printf 'COOKIE_SECURE=false\nWEB_PORT=18081\nWEB_BIND_ADDRESS=0.0.0.0\n'
    printf 'PUBLIC_BASE_URL=http://localhost:18081\n'
    printf 'CURRENT_MONTH_REFRESH_SECONDS=3600\nOLDER_MONTH_REFRESH_SECONDS=86400\n'
    printf 'QUICKSTART_OPERATOR_PASSWORD=%s\n' "$operator_password"
  } > "$tmp_file"
  chmod 600 "$tmp_file"
  mv -- "$tmp_file" "$ENV_FILE"
  setup_in_progress=0
  trap - EXIT
fi

compose up --build -d
printf '\nQuickstart is ready at http://localhost:18081\n'
printf 'Operator username: operator\nOperator password: %s\n' "$operator_password"
printf 'This is a disposable, non-production environment. Run scripts/quickstart-down.sh to remove it and its data.\n'

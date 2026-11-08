#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT=kpapers-e2e
ENV_FILE="$ROOT/.env.e2e"
compose_files=(-f "$ROOT/compose.yaml" -f "$ROOT/compose.e2e.yaml")

die() { printf 'e2e: %s\n' "$*" >&2; exit 1; }
command -v python3 >/dev/null 2>&1 || die 'Python 3 is required to generate deterministic PDF fixtures.'
command -v docker >/dev/null 2>&1 || die 'Docker is required.'
if docker info >/dev/null 2>&1; then
  DOCKER=(docker)
elif sudo -n docker info >/dev/null 2>&1; then
  DOCKER=(sudo -n docker)
else
  die 'Docker is unavailable. Grant access to the Docker socket or run this script where Docker is accessible.'
fi
"${DOCKER[@]}" compose version >/dev/null 2>&1 || die 'Docker Compose plugin is required.'
[[ ! -L "$ENV_FILE" ]] || die "$ENV_FILE must not be a symlink."
[[ ! -e "$ENV_FILE" ]] || die "$ENV_FILE already exists; remove it after checking no E2E run is using it."

umask 077
operator_password="$(python3 -c 'import secrets; print(secrets.token_urlsafe(24))')"
fixture_dir="$(mktemp -d /tmp/kpapers-e2e-fixtures.XXXXXX)"
printf 'OPERATOR_PASSWORD_HASH=helper\nSESSION_SIGNING_KEY=%s\nAPI_URL=http://api:8080\nCOOKIE_SECURE=false\nWEB_PORT=18082\nWEB_BIND_ADDRESS=127.0.0.1\nPUBLIC_BASE_URL=http://web\nE2E_OPERATOR_PASSWORD=%s\nE2E_FIXTURE_HOST_DIR=%s\n' \
  "$(python3 -c 'import secrets; print(secrets.token_hex(32))')" "$operator_password" "$fixture_dir" > "$ENV_FILE"
chmod 600 "$ENV_FILE"

compose() { "${DOCKER[@]}" compose "${compose_files[@]}" --env-file "$ENV_FILE" -p "$PROJECT" "$@"; }
cleanup() {
  compose down --volumes --remove-orphans --rmi local >/dev/null 2>&1 || true
  rm -f -- "$ENV_FILE"
  rm -rf -- "$fixture_dir"
}
trap cleanup EXIT INT TERM
FIXTURE_ROOT="$fixture_dir" python3 "$ROOT/tests/e2e/generate_fixtures.py"
if [[ -n "${E2E_REFERENCE_PDF:-}" ]]; then
  [[ -f "$E2E_REFERENCE_PDF" ]] || die "E2E_REFERENCE_PDF is not a regular file: $E2E_REFERENCE_PDF"
  reference_filename="${E2E_REFERENCE_FILENAME:-$(basename -- "$E2E_REFERENCE_PDF")}"
  [[ "$reference_filename" == *.pdf || "$reference_filename" == *.PDF ]] || die 'E2E_REFERENCE_PDF must have a .pdf filename.'
  rm -- "$fixture_dir/current-month/[Философское] «Старые деньги» — FAQ_ как правильно считать дивиденды, «бессмертные» портфели и стоицизм.pdf"
  cp -- "$E2E_REFERENCE_PDF" "$fixture_dir/current-month/$reference_filename"
  printf 'E2E_REFERENCE_PDF_TEST=true\n' >> "$ENV_FILE"
  printf 'REFERENCE_DOCUMENT_FILENAME=%s\n' "$reference_filename" >> "$ENV_FILE"
fi
chmod -R a+rX "$fixture_dir"
compose build api web
password_hash="$(printf '%s\n' "$operator_password" | compose run --rm --no-deps -T -i api /usr/local/bin/kpapers-api --hash-password)"
[[ "$password_hash" == \$argon2* ]] || die 'API did not return a valid password hash.'
HASH_VALUE="$password_hash" python3 - "$ENV_FILE" <<'PY'
from pathlib import Path
import os
import sys

env_file = Path(sys.argv[1])
lines = env_file.read_text().splitlines()
env_file.write_text("\n".join(
    f"OPERATOR_PASSWORD_HASH='{os.environ['HASH_VALUE']}'" if line.startswith("OPERATOR_PASSWORD_HASH=") else line
    for line in lines
) + "\n")
PY
compose up --no-build --abort-on-container-exit --exit-code-from e2e e2e

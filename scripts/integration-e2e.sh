#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
KEEP_IMAGES="${E2E_KEEP_IMAGES:-false}"

cleanup() {
  result=$?
  if [[ "$KEEP_IMAGES" != true ]]; then
    if docker info >/dev/null 2>&1; then
      docker image rm kpapers-api:e2e kpapers-web:e2e >/dev/null 2>&1 || true
    elif sudo -n docker info >/dev/null 2>&1; then
      sudo -n docker image rm kpapers-api:e2e kpapers-web:e2e >/dev/null 2>&1 || true
    fi
  fi
  return "$result"
}
trap cleanup EXIT

E2E_KEEP_IMAGES=true "$ROOT/scripts/ui-e2e.sh"
E2E_KEEP_IMAGES=true E2E_SKIP_APP_BUILD=true bash "$ROOT/scripts/e2e.sh"

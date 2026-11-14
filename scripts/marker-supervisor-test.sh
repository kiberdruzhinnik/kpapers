#!/usr/bin/env bash
# Verify that the configured container supervisor recreates the Marker worker
# after a conversion timeout and that the restarted worker accepts work.
#
# This uses the worker's production HTTP server with a fake converter, so it
# needs neither Marker models nor a real PDF.  It covers both a caller that
# reads the 504 and a caller that disconnects before the response is written.
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT="${MARKER_SUPERVISOR_TEST_PROJECT:-kpapers-marker-supervisor-test}"
WORKDIR="$(mktemp -d "${TMPDIR:-/tmp}/kpapers-marker-supervisor.XXXXXX")"
PORT="${MARKER_SUPERVISOR_TEST_PORT:-18099}"

die() { printf 'marker-supervisor-test: %s\n' "$*" >&2; exit 1; }
command -v python3 >/dev/null 2>&1 || die 'Python 3 is required.'

if docker info >/dev/null 2>&1; then
  DOCKER=(docker)
elif sudo -n docker info >/dev/null 2>&1; then
  DOCKER=(sudo -n docker)
else
  die 'Docker is unavailable.'
fi
"${DOCKER[@]}" compose version >/dev/null 2>&1 || die 'Docker Compose plugin is required.'

cat > "$WORKDIR/worker.py" <<'PY'
import os
import time

from marker_worker import worker

FLAG = "/state/slow-request-served"


class Converter:
    ready = True

    def __init__(self, delay, text):
        self.delay = delay
        self.text = text

    def convert(self, data):
        time.sleep(self.delay)
        return worker.blocks_to_document([
            worker.Block(
                "Page",
                children=[worker.Block(
                    "Text",
                    html=f"<p>{self.text}</p>",
                    bbox=[10, 10, 100, 20],
                )],
            )
        ])


if os.path.exists(FLAG):
    converter = Converter(0.0, "recovered")
    timeout = 5.0
else:
    with open(FLAG, "w", encoding="utf-8") as handle:
        handle.write("slow")
    converter = Converter(5.0, "never")
    timeout = 0.5

worker.serve(
    converter,
    worker.WorkerConfig(max_concurrency=1, execution_timeout=timeout),
    "0.0.0.0",
    8000,
)
PY

cat > "$WORKDIR/compose.yaml" <<YAML
services:
  worker:
    image: python:3.12-slim
    restart: unless-stopped
    working_dir: /repo
    environment:
      PYTHONPATH: /repo
      PYTHONDONTWRITEBYTECODE: "1"
    volumes:
      - "$ROOT:/repo:ro"
      - "$WORKDIR:/state"
    ports:
      - "$PORT:8000"
    command: ["python", "/state/worker.py"]
YAML

compose() {
  "${DOCKER[@]}" compose -f "$WORKDIR/compose.yaml" -p "$PROJECT" "$@"
}
cleanup() {
  compose down --remove-orphans >/dev/null 2>&1 || true
  rm -rf -- "$WORKDIR"
}
trap cleanup EXIT INT TERM

container_id() {
  compose ps -q worker
}

wait_healthy() {
  local deadline=$((SECONDS + 30))
  while (( SECONDS < deadline )); do
    if curl -fsS -m 2 "http://127.0.0.1:$PORT/healthz" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.2
  done
  die 'worker did not become healthy.'
}

wait_restarted() {
  local container="$1" before="$2"
  local deadline=$((SECONDS + 30))
  while (( SECONDS < deadline )); do
    if [[ "$("${DOCKER[@]}" inspect -f '{{.RestartCount}}' "$container")" -gt "$before" ]]; then
      wait_healthy
      return 0
    fi
    sleep 0.2
  done
  die 'supervisor did not restart the worker.'
}

reset_worker() {
  rm -f "$WORKDIR/slow-request-served"
  compose up -d --force-recreate --quiet-pull >/dev/null
  wait_healthy
  local container
  container="$(container_id)"
  [[ -n "$container" ]] || die 'worker container was not created.'
  [[ "$("${DOCKER[@]}" inspect -f '{{.HostConfig.RestartPolicy.Name}}' "$container")" == "unless-stopped" ]] \
    || die 'worker does not use the configured restart policy.'
  printf '%s\n' "$container"
}

assert_recovered() {
  local status
  status="$(curl -s -o "$WORKDIR/response.json" -w '%{http_code}' -m 10 \
    -H 'Content-Type: application/pdf' --data-binary '%PDF-1.4' \
    "http://127.0.0.1:$PORT/extract")"
  [[ "$status" == 200 ]] || die "restarted worker returned $status."
  grep -q '"text":"recovered"' "$WORKDIR/response.json" \
    || die 'restarted worker did not convert the subsequent request.'
}

run_visible_timeout() {
  local container before status
  container="$(reset_worker)"
  before="$("${DOCKER[@]}" inspect -f '{{.RestartCount}}' "$container")"
  status="$(curl -s -o "$WORKDIR/timeout.json" -w '%{http_code}' -m 10 \
    -H 'Content-Type: application/pdf' --data-binary '%PDF-1.4' \
    "http://127.0.0.1:$PORT/extract")"
  [[ "$status" == 504 ]] || die "expected 504, got $status."
  wait_restarted "$container" "$before"
  assert_recovered
  printf 'visible timeout: 504, supervisor restarted worker, next conversion succeeded\n'
}

run_disconnect_timeout() {
  local container before
  container="$(reset_worker)"
  before="$("${DOCKER[@]}" inspect -f '{{.RestartCount}}' "$container")"
  python3 - "$PORT" <<'PY'
import socket
import sys

port = int(sys.argv[1])
body = b"%PDF-1.4"
request = (
    b"POST /extract HTTP/1.1\r\n"
    b"Host: 127.0.0.1\r\n"
    b"Content-Type: application/pdf\r\n"
    b"Content-Length: " + str(len(body)).encode() + b"\r\n"
    b"Connection: close\r\n\r\n" + body
)
with socket.create_connection(("127.0.0.1", port), timeout=5) as client:
    client.sendall(request)
    # Close without reading the timeout response.
PY
  wait_restarted "$container" "$before"
  assert_recovered
  printf 'disconnected client: supervisor restarted worker, next conversion succeeded\n'
}

run_visible_timeout
run_disconnect_timeout
printf 'marker supervisor restart test passed\n'

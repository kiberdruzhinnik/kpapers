#!/bin/sh
set -eu

if [ -z "${API_URL:-}" ]; then
    echo "ERROR: API_URL is required (for example, http://api:8080)" >&2
    exit 1
fi

if ! awk -v url="$API_URL" '
    BEGIN {
        if (url !~ /^https?:\/\/[A-Za-z0-9][A-Za-z0-9.-]*(:[0-9]+)?\/?$/) {
            exit 1
        }
        sub(/^https?:\/\//, "", url)
        sub(/\/$/, "", url)
        count = split(url, parts, ":")
        host = parts[1]
        if (host ~ /\.\./ || host ~ /\.-/ || host ~ /-\./ || host ~ /\.$/ || host ~ /-$/) {
            exit 1
        }
        if (count == 2 && (parts[2] < 1 || parts[2] > 65535)) {
            exit 1
        }
        if (count > 2) {
            exit 1
        }
    }
'; then
    echo "ERROR: API_URL must be an HTTP(S) origin with a hostname and optional port (for example, http://api:8080)" >&2
    exit 1
fi

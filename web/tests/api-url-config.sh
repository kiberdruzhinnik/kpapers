#!/bin/sh
set -eu

web_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

API_URL='http://kpapers-api:8080' sh "$web_dir/validate-api-url.sh"

if env -u API_URL sh "$web_dir/validate-api-url.sh" >"$tmp_dir/missing.log" 2>&1; then
    echo "Expected missing API_URL to fail" >&2
    exit 1
fi
grep -q 'API_URL is required' "$tmp_dir/missing.log"

if API_URL='ftp://api:8080' sh "$web_dir/validate-api-url.sh" >"$tmp_dir/invalid.log" 2>&1; then
    echo "Expected invalid API_URL to fail" >&2
    exit 1
fi
grep -q 'API_URL must be an HTTP(S) origin' "$tmp_dir/invalid.log"

API_URL='http://kpapers-api:8080' envsubst '${API_URL}' \
    < "$web_dir/nginx.conf.template" > "$tmp_dir/default.conf"
grep -Fq 'proxy_pass http://kpapers-api:8080;' "$tmp_dir/default.conf"
grep -Fq 'proxy_pass http://kpapers-api:8080/feeds/;' "$tmp_dir/default.conf"
grep -Fq 'proxy_pass http://kpapers-api:8080/documents/;' "$tmp_dir/default.conf"
grep -Fq 'proxy_set_header Host $host;' "$tmp_dir/default.conf"

echo "API_URL configuration checks passed"

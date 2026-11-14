#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
SECONDS_PER_TARGET="${FUZZ_SMOKE_SECONDS:-10}"
FUZZ_RUST_TOOLCHAIN="${FUZZ_RUST_TOOLCHAIN:-nightly-2026-11-08}"
WORK="$(mktemp -d)"
OUTPUT_DIR="${FUZZ_OUTPUT_DIR:-}"
preserve="${FUZZ_PRESERVE_OUTPUTS:-false}"
finish() {
  result=$?
  if [[ -n "$OUTPUT_DIR" && ( "$preserve" == true || $result -ne 0 ) ]]; then
    mkdir -p "$OUTPUT_DIR"
    cp -a "$WORK/." "$OUTPUT_DIR/"
  elif [[ $result -ne 0 ]]; then
    mkdir -p "$ROOT/fuzz/artifacts"
    cp -a "$WORK/." "$ROOT/fuzz/artifacts/"
  fi
  rm -rf -- "$WORK"
  return "$result"
}
trap finish EXIT

command -v cargo-fuzz >/dev/null 2>&1 || { echo 'cargo-fuzz is required' >&2; exit 1; }
for target in pdf_bytes untrusted_inputs; do
  corpus="$WORK/corpus-$target"
  artifacts="$WORK/artifacts-$target"
  mkdir -p "$corpus" "$artifacts"
  cp -a "$ROOT/fuzz/corpus/$target/." "$corpus/"
  max_len=4096
  [[ "$target" != pdf_bytes ]] || max_len=2097153
  echo "Fuzzing $target for ${SECONDS_PER_TARGET}s (max input ${max_len} bytes)"
  cargo "+$FUZZ_RUST_TOOLCHAIN" fuzz run "$target" "$corpus" -- \
    -max_len="$max_len" -timeout=5 -rss_limit_mb=1024 \
    -max_total_time="$SECONDS_PER_TARGET" -artifact_prefix="$artifacts/"
done

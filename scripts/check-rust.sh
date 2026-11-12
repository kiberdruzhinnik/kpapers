#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo fmt --all -- --check
cargo fmt --manifest-path fuzz/Cargo.toml -- --check
cargo clippy --all-targets --locked -- -D warnings
# Share the stable target directory so the fuzz crate reuses compatible
# dependency artifacts. Nightly fuzz testing keeps its own target directory.
CARGO_TARGET_DIR="$ROOT/target" cargo clippy --manifest-path fuzz/Cargo.toml --all-targets --locked -- -D warnings
cargo test --all-targets --locked

# Fuzz testing

Fuzz targets use cargo-fuzz and call production parsing functions. Install nightly Rust with `llvm-tools-preview`, `cargo-fuzz`, . `scripts/fuzz-smoke.sh` checks these prerequisites, copies the hand-authored seeds into a temporary writable corpus, and runs each target for a bounded duration. It defaults to 10 seconds per target.

```sh
FUZZ_SMOKE_SECONDS=30 ./scripts/fuzz-smoke.sh
```

The PDF target rejects inputs larger than 2 MiB before calling `pdf_oxide`. Both targets apply a five-second per-input timeout and a 1 GiB RSS bound. The script preserves mutated corpora and crash artifacts on failure under `fuzz/artifacts/`; CI uploads its run data on failure. The scheduled workflow retains its generated corpus and artifacts for 30 days. To reproduce a reported crash, copy its input into `fuzz/corpus/<target>/` and run `cargo +nightly fuzz run <target> fuzz/corpus/<target> -- -runs=1 -timeout=5`.

Only curated starting seeds belong in `fuzz/corpus/`. When a failure is found, minimize it with `cargo +nightly fuzz cmin <target> fuzz/corpus/<target>` and retain the minimized input there, or encode it as a deterministic regression test.

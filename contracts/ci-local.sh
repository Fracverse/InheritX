#!/usr/bin/env bash
# Run the same steps as .github/workflows/contracts.yml
# Usage: from repo root: ./contracts/ci-local.sh
#    or: cd contracts && ./ci-local.sh

set -e
cd "$(dirname "$0")"

echo "==> Check formatting"
cargo fmt --all -- --check

echo "==> Clippy"
cargo clippy --all-targets --all-features -- -D warnings

echo "==> Build (release)"
cargo build --release

echo "==> Tests"
cargo test

echo "==> CI checks passed."

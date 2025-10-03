#!/usr/bin/env bash
set -euo pipefail

# Run the full CI validation pipeline locally.
# The script requires cargo-audit and cargo-deny to be installed.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_DIR="${ROOT_DIR}/imir"

run() {
  echo "[ci-check] $1"
  shift
  "$@"
}

cd "${CRATE_DIR}"

run "formatting" cargo +nightly fmt --
run "clippy" cargo clippy --all-targets --all-features -- -D warnings
run "build" cargo build --all-targets --locked
run "tests" cargo test --all
run "documentation" cargo doc --no-deps

if ! command -v cargo-audit >/dev/null 2>&1; then
  echo "cargo-audit is required. Install it via 'cargo install cargo-audit'." >&2
  exit 1
fi
run "audit" cargo audit -f Cargo.lock

if ! command -v cargo-deny >/dev/null 2>&1; then
  echo "cargo-deny is required. Install it via 'cargo install cargo-deny'." >&2
  exit 1
fi
run "deny" cargo deny check --config "${ROOT_DIR}/deny.toml"

#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
#
# SPDX-License-Identifier: MIT

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

run_stable() {
  local label="$1"
  shift
  run "${label}" cargo +stable "$@"
}

cd "${CRATE_DIR}"

run "formatting" cargo +nightly fmt --
run_stable "clippy" clippy --all-targets --all-features -- -D warnings
run_stable "build" build --all-targets --locked
run_stable "tests" test --all
run_stable "documentation" doc --no-deps

BADGE_TMP="$(mktemp -d)"
cleanup() {
  rm -rf "${BADGE_TMP}"
}
trap cleanup EXIT

run_stable "badge-smoke" run --locked --manifest-path "${CRATE_DIR}/Cargo.toml" -- \
  badge generate --config "${ROOT_DIR}/targets/targets.yaml" --target profile --output "${BADGE_TMP}"

if [ ! -f "${BADGE_TMP}/profile.svg" ] || [ ! -f "${BADGE_TMP}/profile.json" ]; then
  echo "badge smoke test did not produce expected artifacts" >&2
  exit 1
fi

if ! command -v cargo-audit >/dev/null 2>&1; then
  echo "cargo-audit is required. Install it via 'cargo install cargo-audit'." >&2
  exit 1
fi
run_stable "audit" audit -f Cargo.lock

if ! command -v cargo-deny >/dev/null 2>&1; then
  echo "cargo-deny is required. Install it via 'cargo install cargo-deny'." >&2
  exit 1
fi
run_stable "deny" deny check --config "${ROOT_DIR}/deny.toml"

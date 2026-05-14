#!/usr/bin/env bash
# Bat_OS — strict lint gate.
# Runs `cargo build` then `cargo clippy -D warnings` against the
# aarch64-unknown-none target with `gicv3`. Both must produce
# zero warnings for this script to exit 0.
#
# Run this before every commit, before every release tag, and
# before every public artifact (README claim, grant app, etc.)
# that says "0 warnings · 0 clippy lints" — that claim is only
# true if this script exits 0.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "[lint] cargo build (release, aarch64-unknown-none, gicv3)..."
cargo build --release --target aarch64-unknown-none --features gicv3

echo "[lint] cargo clippy (-D warnings)..."
cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings

echo "[lint] PASS — zero compiler warnings, zero clippy lints"

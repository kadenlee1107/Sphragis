#!/usr/bin/env bash
# Bat_OS — Chromium build orchestrator
# Runs inside the Linux ARM64 build container. Drives gclient sync and ninja.

set -euo pipefail

# Pin to a known-good Chromium release for reproducibility.
# M132 stable as of plan creation. Bump this when we want to update.
CHROMIUM_REF="${CHROMIUM_REF:-132.0.6834.83}"

CHROMIUM_DIR="/home/build/chromium"
SRC_DIR="$CHROMIUM_DIR/src"
OUT_DIR="$SRC_DIR/out/BatOs"
GN_ARGS_FILE="/home/build/.gn-args"

cd "$CHROMIUM_DIR"

phase() { printf "\n\033[1;36m=== %s ===\033[0m\n" "$*"; }
note()  { printf "  \033[0;90m%s\033[0m\n" "$*"; }
ok()    { printf "  \033[1;32m✓\033[0m %s\n" "$*"; }
fail()  { printf "  \033[1;31m✗\033[0m %s\n" "$*"; exit 1; }

# ─────────────────────────────────────────────
# Phase 0: gclient + sync source
# ─────────────────────────────────────────────
phase "Phase 0: gclient config + sync"

if [ ! -f .gclient ]; then
    note "Configuring gclient for chromium/src..."
    fetch --nohooks --no-history chromium || fail "fetch failed"
    ok ".gclient created"
else
    note ".gclient already exists, skipping fetch"
fi

if [ ! -d src ]; then
    fail "src/ missing — fetch did not produce a source tree"
fi

cd src

note "Pinning to ref: $CHROMIUM_REF"
git fetch --depth=1 origin "refs/tags/$CHROMIUM_REF" 2>/dev/null || \
    git fetch origin "refs/tags/$CHROMIUM_REF"
git checkout "$CHROMIUM_REF"

cd ..
note "Running gclient sync (this is the long part — minutes to hours)..."
gclient sync --with_branch_heads --with_tags -D --no-history --shallow

ok "Source tree synced"
df -h "$CHROMIUM_DIR" | tail -1

# ─────────────────────────────────────────────
# Phase 1: GN gen + ninja
# ─────────────────────────────────────────────
phase "Phase 1: GN gen for content_shell"

cd "$SRC_DIR"

mkdir -p "$OUT_DIR"
cp "$GN_ARGS_FILE" "$OUT_DIR/args.gn"

note "Running gn gen..."
gn gen "$OUT_DIR" || fail "gn gen failed"
ok "GN args accepted"

note "Building content_shell (this takes hours on first run)..."
autoninja -C "$OUT_DIR" content_shell || fail "ninja build failed"

ok "Build complete"
ls -lh "$OUT_DIR/content_shell"
file "$OUT_DIR/content_shell"

phase "Done"
note "Binary: $OUT_DIR/content_shell"
note "Try: $OUT_DIR/content_shell --headless --no-sandbox --disable-gpu \\"
note "       --screenshot=/tmp/test.png https://example.com"

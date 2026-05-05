#!/usr/bin/env bash
# Bat_OS — Ladybird build orchestrator
# Runs inside the Linux ARM64 build container. Drives `Meta/ladybird.py`
# to produce WebContent + helpers + DT_NEEDED libs.

set -euo pipefail

# Pin to a known-good Ladybird commit for reproducibility. Bump
# deliberately when upstream lands a relevant change. Empty string
# means "track main HEAD" — fine for early porting, lock once the
# build is reproducible.
LADYBIRD_REF="${LADYBIRD_REF:-}"

LADYBIRD_DIR="/home/build/ladybird-src"
SRC_DIR="$LADYBIRD_DIR"
OUT_DIR="$LADYBIRD_DIR/Build/release"
ARTIFACT_DIR="${ARTIFACT_DIR:-/home/build/out}"

phase() { printf "\n\033[1;36m=== %s ===\033[0m\n" "$*"; }
note()  { printf "  \033[0;90m%s\033[0m\n" "$*"; }
ok()    { printf "  \033[1;32m✓\033[0m %s\n" "$*"; }

# ─── Phase 1: clone ───────────────────────────────────────────────
if [[ ! -d "$SRC_DIR/.git" ]]; then
    phase "Cloning Ladybird"
    git clone https://github.com/LadybirdBrowser/ladybird.git "$SRC_DIR"
    if [[ -n "$LADYBIRD_REF" ]]; then
        cd "$SRC_DIR"
        git checkout "$LADYBIRD_REF"
        note "checked out $LADYBIRD_REF"
    fi
    ok "clone done"
else
    note "source tree present, skipping clone"
fi

cd "$SRC_DIR"

# ─── Phase 2: build via Meta/ladybird.py ──────────────────────────
# Ladybird's canonical build entry point is Meta/ladybird.py. It
# manages CMake configure + ninja build with sensible defaults.
# `rebuild` wipes the build dir and rebuilds from scratch — use
# `build` for incremental.
phase "Building Ladybird (release, native arm64)"
note "first run: ~30 min on M-series (cmake configure + ninja Web*)"
export CC=clang-21
export CXX=clang++-21
# Use ld.lld for faster linking of the WebContent shared graph.
export LDFLAGS="-fuse-ld=lld"

# Skip the Qt UI build — we only need the Services. The WebContent
# binary doesn't link to Qt at all; UI only matters for the Ladybird
# desktop wrapper which we don't run inside Bat_OS.
#
# Meta/ladybird.py exposes a --target flag that takes a CMake target
# name. The Lagom build defines `WebContent`, `RequestServer`,
# `ImageDecoder`, `WebWorker` as separate targets.
python3 Meta/ladybird.py build --preset Release \
    --target WebContent RequestServer ImageDecoder WebWorker

ok "build complete"

# ─── Phase 3: collect artifacts ───────────────────────────────────
phase "Collecting artifacts"
mkdir -p "$ARTIFACT_DIR/bin" "$ARTIFACT_DIR/lib" "$ARTIFACT_DIR/share"

# Service binaries
for svc in WebContent RequestServer ImageDecoder WebWorker; do
    if [[ -f "$OUT_DIR/bin/$svc" ]]; then
        cp -v "$OUT_DIR/bin/$svc" "$ARTIFACT_DIR/bin/"
    elif [[ -f "$OUT_DIR/$svc" ]]; then
        cp -v "$OUT_DIR/$svc" "$ARTIFACT_DIR/bin/"
    else
        echo "WARN: $svc not found in $OUT_DIR — build may have skipped it" >&2
    fi
done

# DT_NEEDED libs — copy the dynamic library closure of WebContent.
# Bat_OS's loader doesn't look at the system /lib path, so we have
# to ship every DT_NEEDED entry alongside the binary.
if [[ -f "$ARTIFACT_DIR/bin/WebContent" ]]; then
    note "computing DT_NEEDED closure for WebContent"
    deps=$(ldd "$ARTIFACT_DIR/bin/WebContent" 2>/dev/null | \
           awk '/=>/ && !/=> not/ { print $3 }' | \
           grep -E "^/" || true)
    for d in $deps; do
        if [[ -f "$d" ]]; then
            cp -v "$d" "$ARTIFACT_DIR/lib/" 2>/dev/null || true
        fi
    done
    # The dynamic linker itself
    interp=$(readelf -p .interp "$ARTIFACT_DIR/bin/WebContent" 2>/dev/null | \
             awk '/\/lib/ { print $NF }')
    [[ -n "$interp" && -f "$interp" ]] && cp -v "$interp" "$ARTIFACT_DIR/lib/"
    ok "DT_NEEDED closure copied"
fi

# Liberation fonts (Ladybird requires them at runtime)
mkdir -p "$ARTIFACT_DIR/share/fonts"
for f in /usr/share/fonts/truetype/liberation2/*.ttf; do
    [[ -f "$f" ]] && cp -v "$f" "$ARTIFACT_DIR/share/fonts/"
done

ok "artifacts in $ARTIFACT_DIR"
ls -lh "$ARTIFACT_DIR/bin/" "$ARTIFACT_DIR/lib/" 2>/dev/null | head -20

phase "Build complete"
note "next step on host: tools/bake_ladybird_initrd.sh"

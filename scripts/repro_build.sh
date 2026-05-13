#!/usr/bin/env bash
# Reproducible build driver — gov-grade §3.11 (supply chain integrity).
#
# Builds the kernel TWICE with deterministic flags and asserts the
# two output binaries are bit-identical (sha256). A third party can
# fetch the source at the same git rev, run this script, and
# arrive at the same hash — provenance without trusting the
# original build host.
#
# Determinism levers:
#   * `--frozen --offline` — cargo refuses to update Cargo.lock and
#     won't hit the network, so no drift from registry resolution.
#   * `--locked` — cargo aborts if Cargo.lock would change.
#   * `SOURCE_DATE_EPOCH` — fixed mtime baked into rustc's debug
#     info and any FS-touching build script.
#   * `CARGO_BUILD_RUSTFLAGS="--remap-path-prefix"` — strip the
#     absolute /Users/... prefix from debug info so two checkouts
#     in different directories produce the same binary.
#   * `CARGO_TARGET_DIR=target-repro1/2` — separate caches per build
#     so we know neither one is pulling cached artifacts from the
#     other.
#
# Usage:
#   bash scripts/repro_build.sh
set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$(pwd -P)"

GIT_SHA=$(git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)
echo "[repro] git sha: $GIT_SHA"

# Pin SOURCE_DATE_EPOCH to the HEAD commit's timestamp so two
# checkouts of the same rev produce the same binary.
SOURCE_DATE_EPOCH="$(git log -1 --pretty=%ct HEAD 2>/dev/null || echo 1700000000)"
export SOURCE_DATE_EPOCH
echo "[repro] SOURCE_DATE_EPOCH: $SOURCE_DATE_EPOCH"

# Cargo's RUSTFLAGS env REPLACES `.cargo/config.toml`'s rustflags
# entirely instead of merging — so we have to repeat every flag the
# config sets (linker script, stack canaries, PAC/BTI, fixed-x18)
# and append our path-remap on top. Keep this list synced with
# `.cargo/config.toml`'s `[target.aarch64-unknown-none]` rustflags
# block.
RUSTFLAGS_LIST=(
    "-C" "link-arg=-Tlinker.ld"
    "-Zfixed-x18"
    "-Z" "stack-protector=all"
    "-C" "target-feature=+paca,+pacg,+bti"
    "-Z" "branch-protection=bti,pac-ret"
    "--remap-path-prefix=${REPO_ROOT}=/build"
)
export RUSTFLAGS="${RUSTFLAGS_LIST[*]}"

declare -a TARGET_DIRS=("target-repro1" "target-repro2")
declare -a HASHES=()

for d in "${TARGET_DIRS[@]}"; do
    echo "[repro] building into $d ..."
    rm -rf "$d"
    CARGO_TARGET_DIR="$d" \
        cargo build --release \
            --target aarch64-unknown-none \
            --features gicv3 \
            --locked --frozen --offline \
        > "$d.log" 2>&1 || {
            echo "[repro] BUILD FAILED — see $d.log"
            tail -20 "$d.log"
            exit 1
        }
    BIN="$d/aarch64-unknown-none/release/bat_os"
    if [[ ! -f "$BIN" ]]; then
        echo "[repro] $BIN missing after build"
        exit 1
    fi
    H=$(shasum -a 256 "$BIN" | awk '{print $1}')
    HASHES+=("$H")
    SIZE=$(stat -f%z "$BIN" 2>/dev/null || stat -c%s "$BIN")
    echo "[repro]   $BIN: $SIZE bytes sha256=$H"
done

echo "[repro] comparing..."
if [[ "${HASHES[0]}" == "${HASHES[1]}" ]]; then
    echo "[repro] PASS — both builds produced sha256=${HASHES[0]}"
    # Stash the canonical hash next to the SBOM so a release tag
    # carries (sha256, sbom) together.
    REPRO_FILE="${REPO_ROOT}/repro.sha256"
    {
        echo "git_sha:           $GIT_SHA"
        echo "source_date_epoch: $SOURCE_DATE_EPOCH"
        echo "rustflags:         $RUSTFLAGS"
        echo "kernel_sha256:     ${HASHES[0]}"
    } > "$REPRO_FILE"
    echo "[repro] wrote $REPRO_FILE"
    rm -rf "${TARGET_DIRS[@]}" "${TARGET_DIRS[@]/%/.log}"
    exit 0
else
    echo "[repro] FAIL — builds differ"
    echo "[repro]   build1: ${HASHES[0]}"
    echo "[repro]   build2: ${HASHES[1]}"
    exit 1
fi

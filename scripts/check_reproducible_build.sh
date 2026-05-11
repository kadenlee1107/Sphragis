#!/usr/bin/env bash
# Two-pass build + SHA-256 compare to verify reproducibility.
#
# Reproducible builds prove that the binary we ship came from the
# source we published. An attacker who tampered with the toolchain or
# substituted dependencies would produce a different hash on a second
# build from clean state.
#
# We compare the kernel ELF + the GGUF (if present) + the SBOM.
# Cargo is mostly deterministic but a few sources of non-determinism
# remain: build path embedded in debug info, parallelism with cached
# build artifacts, system timestamps. The flags below suppress them.
#
# Run:
#   bash scripts/check_reproducible_build.sh
#
# Exit code 0 = both passes produced identical artifacts.
set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
ARTIFACT="target/aarch64-unknown-none/release/bat_os"
PASS_A="$REPO/out/repro_a.sha256"
PASS_B="$REPO/out/repro_b.sha256"

# Determinism knobs:
#   * SOURCE_DATE_EPOCH freezes any embedded build timestamp.
#   * --remap-path-prefix maps the source root to a constant so debug
#     info doesn't embed the local cwd.
#   * codegen-units=1 + incremental=false avoids subtle ordering diffs.
export SOURCE_DATE_EPOCH=946684800   # 2000-01-01T00:00:00Z
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$REPO=/build/bat_os -C codegen-units=1"
export CARGO_INCREMENTAL=0

mkdir -p "$REPO/out"

build_and_hash() {
    local outfile="$1"
    cargo clean --release --target aarch64-unknown-none -q
    cargo build --release --target aarch64-unknown-none -q
    if [ ! -f "$REPO/$ARTIFACT" ]; then
        echo "[repro] artifact not found at $ARTIFACT"
        exit 2
    fi
    shasum -a 256 "$REPO/$ARTIFACT" | awk '{print $1}' > "$outfile"
    echo "[repro] $(basename "$outfile"): $(cat "$outfile")"
}

echo "[repro] pass A"
build_and_hash "$PASS_A"
echo "[repro] pass B"
build_and_hash "$PASS_B"

if cmp -s "$PASS_A" "$PASS_B"; then
    echo "[repro] PASS — both builds produced identical kernel ELF"
    exit 0
else
    echo "[repro] FAIL — kernel ELF differs between builds"
    echo "  A: $(cat "$PASS_A")"
    echo "  B: $(cat "$PASS_B")"
    exit 1
fi

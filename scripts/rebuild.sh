#!/usr/bin/env bash
# Bat_OS — build the Apple Silicon binary.
#
# Works on both Mac and Ubuntu. Runs `cargo build --release` with
# the Apple linker script, then objcopy to raw binary.
#
# Usage:
#   ./scripts/rebuild.sh            # full build + objcopy
#   ./scripts/rebuild.sh --check    # cargo check only (faster)
#   ./scripts/rebuild.sh --qemu     # build for QEMU virt instead
set -euo pipefail

log() { echo -e "\033[1;34m[rebuild]\033[0m $*"; }
err() { echo -e "\033[1;31m[rebuild] ERROR:\033[0m $*" >&2; exit 1; }

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# --check: fastest possible verification (type-check only, no codegen)
if [ "${1:-}" = "--check" ]; then
    log "Running cargo check"
    cargo check --release
    log "Done (check only; no binary produced)"
    exit 0
fi

# --qemu: build for QEMU virt (for Layer A/B/C HVF testing)
if [ "${1:-}" = "--qemu" ]; then
    log "Building QEMU-virt bat_os (default linker.ld)"
    cargo build --release
    SIZE=$(du -h target/aarch64-unknown-none/release/bat_os | cut -f1)
    log "Built: target/aarch64-unknown-none/release/bat_os ($SIZE)"
    exit 0
fi

# Default: Apple Silicon build.
log "Building bat_os_apple.bin (linker_apple.ld)"
RUSTFLAGS="-C link-arg=-Tlinker_apple.ld" cargo build --release

log "Producing raw binary"
# Use whichever objcopy is available.
if command -v rust-objcopy >/dev/null; then
    OBJCOPY=rust-objcopy
elif command -v llvm-objcopy >/dev/null; then
    OBJCOPY=llvm-objcopy
elif command -v aarch64-linux-gnu-objcopy >/dev/null; then
    OBJCOPY=aarch64-linux-gnu-objcopy
else
    err "no objcopy found. Install rust-objcopy (cargo install cargo-binutils) or llvm-objcopy (apt install llvm)"
fi

$OBJCOPY --strip-all -O binary \
    target/aarch64-unknown-none/release/bat_os \
    target/bat_os_apple.bin

SIZE=$(du -h target/bat_os_apple.bin | cut -f1)
log "Built: target/bat_os_apple.bin ($SIZE)"

# Sanity check: first 4 bytes should be 0xaa0003f4 (mov x20, x0).
# If they're not, the boot-stub collision is back.
FIRST=$(xxd -l 4 -p target/bat_os_apple.bin)
if [ "$FIRST" = "f40300aa" ]; then
    log "Entry check: OK (starts with mov x20, x0 — _apple_start at offset 0)"
else
    err "Entry bytes are $FIRST, expected f40300aa. Linux Image header may be at offset 0 again. See docs/DEBUGGING_RUNBOOK.md §4."
fi

log "Ready to chainload:  ./scripts/chainload.sh"

#!/usr/bin/env bash
# Bat_OS — chainload a payload via m1n1 proxy + tee serial to log.
#
# Usage:
#   ./scripts/chainload.sh [path/to/payload.bin]
#
# If no path given, defaults to target/bat_os_apple.bin.
# Requires: m1n1 stage 1 running on the M4 (Mac booted into m1n1,
# reached "Running proxy..."), and a USB-C cable from the Mac to
# this host.

set -euo pipefail

log() { echo -e "\033[1;34m[chainload]\033[0m $*"; }
err() { echo -e "\033[1;31m[chainload] ERROR:\033[0m $*" >&2; exit 1; }

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PAYLOAD="${1:-$ROOT/target/bat_os_apple.bin}"
M1N1_PY="$ROOT/external/m1n1/proxyclient/tools/chainload.py"
LOG_DIR="$ROOT/logs"
LOG="$LOG_DIR/chainload-$(date +%Y%m%d-%H%M%S).log"

[ -f "$PAYLOAD" ] || err "payload not found: $PAYLOAD"
[ -f "$M1N1_PY" ] || err "m1n1 chainload.py not found (git submodules OK?)"

mkdir -p "$LOG_DIR"

# Auto-detect the serial device. Prefer /dev/m1n1 (our udev symlink
# from setup.sh), fall back to /dev/ttyACM0.
DEV="${M1N1DEVICE:-}"
if [ -z "$DEV" ]; then
    if [ -e /dev/m1n1 ]; then DEV=/dev/m1n1
    elif [ -e /dev/ttyACM0 ]; then DEV=/dev/ttyACM0
    elif [ -e /dev/ttyACM1 ]; then DEV=/dev/ttyACM1
    else err "no serial device found. Plug USB-C, check 'ls /dev/ttyACM*'"
    fi
fi

log "payload: $PAYLOAD ($(du -h "$PAYLOAD" | cut -f1))"
log "device:  $DEV"
log "log:     $LOG"
log "starting chainload (may take 10-30s to upload)..."

# -S = skip secondary CPU RVBAR writes (required on M4, SErrors otherwise)
# --raw = treat binary as raw, not Mach-O
# --entry-point 0 = jump to offset 0 (our _apple_start lives there
#                   per linker_apple.ld + .text.apple_boot section)
sudo -E M1N1DEVICE="$DEV" python3 "$M1N1_PY" \
    --raw --entry-point 0 -S \
    "$PAYLOAD" 2>&1 | tee "$LOG"

log "chainload finished. Last 10 lines of log:"
tail -10 "$LOG"

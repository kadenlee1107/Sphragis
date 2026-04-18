#!/usr/bin/env bash
# Bat_OS — passively capture m1n1 / Bat_OS serial output to a log.
#
# Use this when you want to just WATCH what's happening over serial
# (e.g. after Bat_OS has already been chainloaded and you want to see
# its subsequent output), without running the proxy protocol.
#
# Note: chainload.sh already tees its own output to logs/. This script
# is for the case where you want to listen passively AFTER chainload.
#
# Usage:
#   ./scripts/capture_serial.sh            # prints to stdout + logs/
#   ./scripts/capture_serial.sh -q         # quiet (log only)

set -euo pipefail

log() { echo -e "\033[1;34m[serial]\033[0m $*"; }

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOG_DIR="$ROOT/logs"
LOG="$LOG_DIR/serial-$(date +%Y%m%d-%H%M%S).log"
mkdir -p "$LOG_DIR"

DEV="${M1N1DEVICE:-}"
if [ -z "$DEV" ]; then
    if [ -e /dev/m1n1 ]; then DEV=/dev/m1n1
    elif [ -e /dev/ttyACM0 ]; then DEV=/dev/ttyACM0
    elif [ -e /dev/ttyACM1 ]; then DEV=/dev/ttyACM1
    else echo "No serial device found" >&2; exit 1
    fi
fi

QUIET=0
[ "${1:-}" = "-q" ] && QUIET=1

log "device: $DEV"
log "log:    $LOG"
log "Ctrl+C to stop"

# Use stty to set the port to raw 115200 8N1 first.
sudo stty -F "$DEV" 115200 cs8 -cstopb -parenb raw -echo 2>/dev/null || true

if [ "$QUIET" = 1 ]; then
    sudo cat "$DEV" > "$LOG"
else
    sudo cat "$DEV" | tee "$LOG"
fi

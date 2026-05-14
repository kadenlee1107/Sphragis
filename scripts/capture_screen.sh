#!/usr/bin/env bash
# Sphragis — grab a frame from the Elgato capture card.
#
# Assumes HDMI from the M4 (via USB-C → HDMI adapter) is connected
# into an Elgato USB capture card, which enumerates as a V4L2 device.
#
# Usage:
#   ./scripts/capture_screen.sh                 # single PNG
#   ./scripts/capture_screen.sh --video 10s     # record 10s of video
#   ./scripts/capture_screen.sh --device /dev/video1

set -euo pipefail

log() { echo -e "\033[1;34m[capture]\033[0m $*"; }
err() { echo -e "\033[1;31m[capture] ERROR:\033[0m $*" >&2; exit 1; }

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$ROOT/captures"
mkdir -p "$OUT_DIR"

DEVICE="/dev/video0"
MODE="photo"
DURATION=""

while [ "$#" -gt 0 ]; do
    case "$1" in
        --device) DEVICE="$2"; shift 2;;
        --video) MODE="video"; DURATION="$2"; shift 2;;
        -h|--help) echo "See comment at top of $0"; exit 0;;
        *) err "unknown arg: $1";;
    esac
done

[ -c "$DEVICE" ] || err "$DEVICE is not a V4L2 device. Available:\n$(ls /dev/video* 2>/dev/null || echo 'none — plug Elgato?')"

command -v ffmpeg >/dev/null || err "ffmpeg not installed. Run ./scripts/setup.sh first."

TS=$(date +%Y%m%d-%H%M%S)

if [ "$MODE" = "photo" ]; then
    OUT="$OUT_DIR/screen-$TS.png"
    log "capturing single frame from $DEVICE → $OUT"
    ffmpeg -hide_banner -loglevel error \
        -f v4l2 -i "$DEVICE" \
        -frames:v 1 -y "$OUT"
    log "done: $OUT ($(du -h "$OUT" | cut -f1))"
else
    OUT="$OUT_DIR/screen-$TS.mp4"
    log "recording $DURATION from $DEVICE → $OUT (Ctrl+C to stop early)"
    ffmpeg -hide_banner -loglevel error \
        -f v4l2 -i "$DEVICE" \
        -t "$DURATION" -c:v libx264 -pix_fmt yuv420p \
        -y "$OUT"
    log "done: $OUT ($(du -h "$OUT" | cut -f1))"
fi

log "tip: to commit this into the repo for sharing:"
log "  git add captures/ && git commit -m 'capture: $(basename "$OUT")' && git push"

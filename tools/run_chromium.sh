#!/usr/bin/env bash
# Bat_OS — Boot QEMU with the Chromium-baked kernel image.
#
# Assumes `tools/bake_chromium.sh` has already produced
# `target/aarch64-unknown-none/release/bat_os_with_chromium`.
# Boots with 4 GB guest RAM (vs. 2 GB for plain run.sh) to cover
# content_shell's peak working set plus the source blob.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IMAGE="$ROOT/target/aarch64-unknown-none/release/bat_os_with_chromium"

if [ ! -f "$IMAGE" ]; then
    echo "[run_chromium] $IMAGE not found — run tools/bake_chromium.sh first" >&2
    exit 1
fi

SIZE=$(wc -c <"$IMAGE" | tr -d ' ')
echo "[run_chromium] booting $IMAGE ($SIZE bytes)"
echo "[run_chromium] type 'chromium https://example.com' at the shell prompt"
echo ""

exec qemu-system-aarch64 \
    -accel hvf \
    -machine virt \
    -cpu max \
    -m 4G \
    -nographic \
    -serial mon:stdio \
    -netdev user,id=net0 -device virtio-net-device,netdev=net0 \
    -kernel "$IMAGE"

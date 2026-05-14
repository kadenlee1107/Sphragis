#!/bin/bash
# Sphragis — Build and boot on M4 (native speed via HVF)
set -e

cd "$(dirname "$0")"

echo "[*] Building Sphragis..."
cargo build --release 2>&1 | tail -1

echo "[*] Booting on Apple M4 (native speed)..."
echo "    Type in QEMU window (GUI keyboard)"
echo "    Passphrase: sphragis-dev"
echo "    Close window or Ctrl+A X to exit"
echo ""

qemu-system-aarch64 \
  -accel hvf \
  -machine virt \
  -cpu max \
  -m 2G \
  -display cocoa \
  -device virtio-gpu-device \
  -device virtio-keyboard-device \
  -netdev user,id=net0 -device virtio-net-device,netdev=net0 \
  -serial stdio \
  -kernel target/aarch64-unknown-none/release/sphragis

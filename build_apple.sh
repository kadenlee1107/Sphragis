#!/bin/bash
# Bat_OS — Build for Apple Silicon (M4 MacBook)
# Produces a bare-metal binary that m1n1 can chainload.
set -e

echo "[*] Building Bat_OS for Apple Silicon..."

# Build with Apple Silicon linker script
RUSTFLAGS="-C link-arg=-Tlinker_apple.ld" cargo build --release 2>&1

# Create raw binary. The linker places .text.boot first (see linker_apple.ld
# ENTRY(_apple_start) + SECTIONS); after `-O binary`, _apple_start is at
# offset 0 of the raw output. chainload.py picks this up with
# `--raw --entry-point 0`.
echo "[*] Creating binary image..."
rust-objcopy --strip-all -O binary \
    target/aarch64-unknown-none/release/bat_os \
    target/bat_os_apple.bin

SIZE=$(stat -f %z target/bat_os_apple.bin 2>/dev/null || stat -c %s target/bat_os_apple.bin 2>/dev/null)
echo "[*] Apple Silicon binary: target/bat_os_apple.bin ($SIZE bytes)"

echo ""
echo "══════════════════════════════════════════════"
echo "  Bat_OS Apple Silicon Build Complete"
echo "══════════════════════════════════════════════"
echo ""
echo "  Deploy via m1n1 chainload (after m1n1 is installed):"
echo ""
echo "  1. Reboot M4 into m1n1 (will auto-expose USB CDC serial)"
echo "  2. Connect USB-C cable to a Linux host (Ubuntu live USB works;"
echo "     Windows WILL NOT work — m1n1's composite USB descriptor"
echo "     requires a vendor INF that Windows does not ship)"
echo "  3. On the host:"
echo "       python3 proxyclient/tools/chainload.py \\"
echo "           --raw --entry-point 0 \\"
echo "           /path/to/bat_os_apple.bin"
echo ""
echo "  What chainload does:"
echo "    * Preserves Apple's SEPFW + preoslog"
echo "    * Rewrites the ADT chosen/memory-map entries"
echo "    * Jumps to Bat_OS with x0 = BootArgs ptr"
echo "  Our _apple_start then runs on real M4 silicon."
echo ""

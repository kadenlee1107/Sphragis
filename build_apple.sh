#!/bin/bash
# Bat_OS — Build for Apple Silicon (M4 MacBook)
# Produces a bare-metal binary that m1n1 can load.
set -e

echo "[*] Building Bat_OS for Apple Silicon..."

# Build with Apple Silicon linker script
RUSTFLAGS="-C link-arg=-Tlinker_apple.ld" cargo build --release 2>&1

# Create raw binary
echo "[*] Creating binary image..."
rust-objcopy --strip-all -O binary \
    target/aarch64-unknown-none/release/bat_os \
    target/bat_os_apple.bin

SIZE=$(stat -f %z target/bat_os_apple.bin 2>/dev/null || stat -c %s target/bat_os_apple.bin 2>/dev/null)
echo "[*] Apple Silicon binary: target/bat_os_apple.bin ($SIZE bytes)"

# Create Mach-O for m1n1 (optional — m1n1 can load raw binaries)
echo "[*] Creating m1n1-compatible image..."
rust-objcopy -O binary \
    --set-start 0x810000000 \
    target/aarch64-unknown-none/release/bat_os \
    target/bat_os_m1n1.macho 2>/dev/null || true

echo ""
echo "══════════════════════════════════════════════"
echo "  Bat_OS Apple Silicon Build Complete"
echo "══════════════════════════════════════════════"
echo ""
echo "  To deploy on M4 MacBook:"
echo "  1. Install Asahi Linux m1n1 on the MacBook"
echo "  2. Connect USB-C cable to another Mac/PC"
echo "  3. Run: python3 m1n1/proxyclient/tools/run_guest.py \\"
echo "       target/bat_os_apple.bin"
echo ""
echo "  Or chainload from m1n1 hypervisor mode:"
echo "  m1n1> load target/bat_os_apple.bin"
echo "  m1n1> jump"
echo ""

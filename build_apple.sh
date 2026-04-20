#!/bin/bash
# Bat_OS — Build for Apple Silicon (M4 MacBook)
# Produces a bare-metal binary that m1n1 can chainload.
#
# CRITICAL: do not shortcut this with a plain `cargo build --release`.
# The default `.cargo/config.toml` links with `-Tlinker.ld` (QEMU-virt
# layout: Linux kernel Image header at offset 0, `b +0x40`). m1n1's
# `chainload.py --raw --entry-point 0` jumps to offset 0, so a
# default-linked binary lands inside the Linux header — faults and
# resets the Mac within ~2 s. Only `-Tlinker_apple.ld` places
# `_apple_start` at offset 0. See the offset-0-opcode check below.
set -e

echo "[*] Building Bat_OS for Apple Silicon..."

# Build with Apple Silicon linker script
RUSTFLAGS="-C link-arg=-Tlinker_apple.ld" cargo build --release 2>&1

# Create raw binary. The linker places .text.boot first (see linker_apple.ld
# ENTRY(_apple_start) + SECTIONS); after `-O binary`, _apple_start is at
# offset 0 of the raw output. chainload.py picks this up with
# `--raw --entry-point 0`.
echo "[*] Creating binary image..."
# Prefer rust-objcopy if on PATH, else fall back to the toolchain's
# llvm-objcopy (shipped with the llvm-tools-preview rustup component).
if command -v rust-objcopy >/dev/null 2>&1; then
    OBJCOPY=rust-objcopy
else
    OBJCOPY=$(ls ~/.rustup/toolchains/*/lib/rustlib/*/bin/rust-objcopy 2>/dev/null | head -1)
fi
if [ -z "$OBJCOPY" ]; then
    echo "ERROR: rust-objcopy not found. Run 'rustup component add llvm-tools-preview'." >&2
    exit 1
fi
"$OBJCOPY" --strip-all -O binary \
    target/aarch64-unknown-none/release/bat_os \
    target/bat_os_apple.bin

# stat(1) is macOS-vs-Linux incompatible: Linux needs `-c %s` (file size),
# macOS needs `-f %z` (same). Try Linux first; on macOS the Linux invocation
# fails with an unknown-format error so we fall through.
SIZE=$(stat -c %s target/bat_os_apple.bin 2>/dev/null || stat -f %z target/bat_os_apple.bin 2>/dev/null)

# Sanity check: the first 4 bytes MUST be `_apple_start`'s first
# instruction, not the Linux kernel Image header. We learned this the
# expensive way: wrong-linker builds produce a valid-looking binary
# that chainloads fine but faults instantly on M4 because offset 0 is
# `b +0x40` (Linux header) instead of `mov x20, x0` (_apple_start).
FIRST4=$(od -An -tx1 -N4 target/bat_os_apple.bin | tr -d ' \n')
EXPECTED="f40300aa"   # AArch64 LE: mov x20, x0
LINUX_HDR="10000014"  # AArch64 LE: b +0x40
if [ "$FIRST4" = "$LINUX_HDR" ]; then
    echo "ERROR: binary has Linux-kernel-Image header at offset 0 — wrong linker." >&2
    echo "       This means cargo was invoked without the linker_apple.ld override." >&2
    echo "       chainload.py --entry-point 0 would fault immediately on M4." >&2
    exit 1
elif [ "$FIRST4" != "$EXPECTED" ]; then
    echo "WARNING: offset 0 opcode is 0x$FIRST4 (expected 0x$EXPECTED = mov x20, x0)." >&2
    echo "         Proceeding, but this chainload may not behave as expected." >&2
else
    echo "[*] offset-0 opcode: 0x$FIRST4 (mov x20, x0 — _apple_start OK)"
fi

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

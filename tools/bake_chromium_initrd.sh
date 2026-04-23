#!/usr/bin/env bash
# Bat_OS — Produce a standalone BATCHROM-framed blob for QEMU's `-initrd`.
#
# The companion to tools/bake_chromium.sh. That one appends framing
# to the kernel image (works for flat-binary boots like m1n1). This
# one emits the framing as its OWN file — pass it to QEMU as `-initrd`
# and the DTB-supplied `/chosen/linux,initrd-*` range tells the kernel
# where to find it.
#
# Output: target/aarch64-unknown-none/release/chromium_initrd.bin
#
# Usage:
#   tools/bake_chromium_initrd.sh <path/to/content_shell>
#   tools/bake_chromium_initrd.sh tests/hello      # tiny stand-in

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$ROOT/target/aarch64-unknown-none/release"
OUT="$TARGET_DIR/chromium_initrd.bin"

SRC="${1:-}"
if [ -z "$SRC" ]; then
    echo "[bake-initrd] usage: $0 <path/to/content_shell>" >&2
    exit 1
fi
if [ ! -f "$SRC" ]; then
    echo "[bake-initrd] ERROR: $SRC not found" >&2
    exit 1
fi

mkdir -p "$TARGET_DIR"
BSIZE=$(wc -c <"$SRC" | tr -d ' ')
CRC_HEX=$(python3 -c 'import sys, zlib
d = open(sys.argv[1], "rb").read()
print(f"{zlib.crc32(d) & 0xFFFFFFFF:08x}")' "$SRC")

WORK="$(mktemp -d -t batos-bake-initrd-XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

python3 - "$WORK" "$BSIZE" "$CRC_HEX" <<'PY'
import struct, sys
work, size_s, crc_s = sys.argv[1], sys.argv[2], sys.argv[3]
size = int(size_s); crc = int(crc_s, 16)
with open(f"{work}/head.bin", "wb") as f:
    f.write(b"BATCHROM")
    f.write(struct.pack("<Q", size))
with open(f"{work}/tail.bin", "wb") as f:
    f.write(struct.pack("<I", crc))
    f.write(b"CHROMEND")
PY

cat "$WORK/head.bin" "$SRC" "$WORK/tail.bin" >"$OUT"
TOTAL=$(wc -c <"$OUT" | tr -d ' ')

echo "[bake-initrd] wrote $OUT"
echo "[bake-initrd]   content_shell: $BSIZE bytes ($(python3 -c "print(f'{$BSIZE/1024/1024:.1f}')")MB)"
echo "[bake-initrd]   framed total:  $TOTAL bytes"
echo "[bake-initrd]   crc32:         0x$CRC_HEX"
echo "[bake-initrd] boot with: qemu-system-aarch64 ... -kernel <bat_os> -initrd $OUT"

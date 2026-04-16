#!/usr/bin/env bash
# Bat_OS — Bake the Chromium content_shell blob onto the kernel image.
#
# Produces `target/aarch64-unknown-none/release/bat_os_with_chromium`,
# which is the plain kernel image followed by:
#   [BATCHROM (8 bytes)][size u64 LE][content_shell bytes][crc32 LE][CHROMEND]
#
# The kernel's `src/kernel/mm/initrd.rs` walks this framing at boot
# and refuses to advertise the blob if the magic or CRC doesn't line
# up. Plain cargo builds remain unaffected — this script produces a
# SECOND output file.
#
# Usage:
#   tools/bake_chromium.sh [path/to/content_shell]
#
# If no path is supplied, the script pulls content_shell out of the
# Docker volume `batos-chromium-src` (see ports/chromium_port).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$ROOT/target/aarch64-unknown-none/release"
KERNEL="$TARGET_DIR/bat_os"
OUT="$TARGET_DIR/bat_os_with_chromium"

SRC="${1:-}"
WORK="$(mktemp -d -t batos-bake-XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

# -- 1. Ensure the kernel image exists. ---------------------------------
echo "[bake] cargo build --release"
( cd "$ROOT" && cargo build --release )

if [ ! -f "$KERNEL" ]; then
    echo "[bake] ERROR: $KERNEL not found after cargo build" >&2
    exit 1
fi

KSIZE=$(wc -c <"$KERNEL" | tr -d ' ')
echo "[bake] kernel image: $KERNEL ($KSIZE bytes)"

# -- 2. Obtain the content_shell blob. ----------------------------------
if [ -z "$SRC" ]; then
    # Try the Docker volume mount first.
    echo "[bake] extracting content_shell from docker volume batos-chromium-src"
    if ! command -v docker >/dev/null 2>&1; then
        echo "[bake] ERROR: docker not available; pass content_shell path explicitly" >&2
        exit 1
    fi
    docker run --rm \
        -v batos-chromium-src:/src:ro \
        -v "$WORK:/out" \
        alpine:3.19 \
        sh -c 'cp /src/out/BatOs/content_shell /out/content_shell'
    SRC="$WORK/content_shell"
fi

if [ ! -f "$SRC" ]; then
    echo "[bake] ERROR: content_shell not found at $SRC" >&2
    exit 1
fi

BSIZE=$(wc -c <"$SRC" | tr -d ' ')
if [ "$BSIZE" -eq 0 ]; then
    echo "[bake] ERROR: content_shell is empty" >&2
    exit 1
fi
echo "[bake] content_shell: $SRC ($BSIZE bytes)"

# -- 3. Validate ARM64 ELF. ---------------------------------------------
# Check the ELF header: 0x7F 'E' 'L' 'F' + class=2 (64) + machine=0xB7 (AArch64).
MAGIC=$(head -c4 "$SRC" | od -An -tx1 | tr -d ' \n')
if [ "$MAGIC" != "7f454c46" ]; then
    echo "[bake] ERROR: $SRC is not an ELF file (magic=$MAGIC)" >&2
    exit 1
fi
# e_machine is at offset 0x12, little-endian u16. 0xB7 = AArch64.
MACHINE=$(dd if="$SRC" bs=1 skip=18 count=2 status=none | od -An -tx1 | tr -d ' \n')
if [ "$MACHINE" != "b700" ]; then
    echo "[bake] ERROR: $SRC is not AArch64 (e_machine=$MACHINE, expected b700)" >&2
    exit 1
fi
echo "[bake] content_shell is a valid AArch64 ELF"

# -- 4. Compute CRC32 over the blob. ------------------------------------
CRC_HEX=$(python3 - "$SRC" <<'PY'
import binascii, sys
with open(sys.argv[1], "rb") as f:
    crc = binascii.crc32(f.read()) & 0xFFFFFFFF
print(f"{crc:08x}")
PY
)
echo "[bake] crc32 = 0x$CRC_HEX"

# -- 5. Build header / footer binary blobs (little-endian). -------------
python3 - "$WORK" "$BSIZE" "$CRC_HEX" <<'PY'
import struct, sys
work, size_s, crc_s = sys.argv[1], sys.argv[2], sys.argv[3]
size = int(size_s)
crc  = int(crc_s, 16)
with open(f"{work}/head.bin", "wb") as f:
    f.write(b"BATCHROM")
    f.write(struct.pack("<Q", size))
with open(f"{work}/tail.bin", "wb") as f:
    f.write(struct.pack("<I", crc))
    f.write(b"CHROMEND")
PY

# -- 6. Concatenate into the final image. -------------------------------
cat "$KERNEL" "$WORK/head.bin" "$SRC" "$WORK/tail.bin" >"$OUT"

TOTAL=$(wc -c <"$OUT" | tr -d ' ')

# -- 7. Summary. --------------------------------------------------------
fmt_mb() {
    python3 -c "print(f'{$1/1024/1024:.1f} MB')"
}

echo ""
echo "[bake] ========================================================"
echo "[bake] baked: $OUT"
echo "[bake]   kernel:        $KSIZE bytes ($(fmt_mb $KSIZE))"
echo "[bake]   content_shell: $BSIZE bytes ($(fmt_mb $BSIZE))"
echo "[bake]   total:         $TOTAL bytes ($(fmt_mb $TOTAL))"
echo "[bake]   crc32:         0x$CRC_HEX"
echo "[bake] ========================================================"
echo "[bake] boot with: tools/run_chromium.sh"

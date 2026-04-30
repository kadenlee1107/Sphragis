#!/usr/bin/env bash
# Bat_OS — Build a multi-file initrd archive with content_shell + its
# glibc runtime dependencies.
#
# Format (everything little-endian):
#   BATCHROM                 <- existing 8-byte magic
#   size: u64                <- inner archive size
#   <archive bytes>
#   crc32: u32               <- CRC over the archive bytes
#   CHROMEND                 <- existing 8-byte magic
#
# The archive bytes are:
#   BATARCH\0                <- 8-byte archive magic
#   n_files: u32
#   reserved: u32
#   (repeated n_files times, 128 bytes each):
#     name: [u8; 64]         <- null-padded POSIX path (e.g. "bin/content_shell")
#     file_size: u64
#     file_offset: u64       <- relative to the START of the archive (BATARCH)
#     reserved: [u8; 48]
#   then files follow, each aligned to 16 bytes.
#
# See src/kernel/mm/initrd.rs::probe_archive for the matching parser.
#
# Usage:
#   tools/bake_chromium_archive.sh \
#       ports/chromium_port/out/content_shell \
#       ports/chromium_port/out/lib_runtime

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$ROOT/target/aarch64-unknown-none/release"
OUT="$TARGET_DIR/chromium_initrd.bin"

if [ $# -lt 2 ]; then
    echo "[bake-archive] usage: $0 <path/to/content_shell> <path/to/lib_dir>" >&2
    exit 1
fi
SHELL_BIN="$1"
LIB_DIR="$2"

if [ ! -f "$SHELL_BIN" ]; then echo "[bake-archive] ERROR: $SHELL_BIN not found" >&2; exit 1; fi
if [ ! -d "$LIB_DIR" ]; then echo "[bake-archive] ERROR: $LIB_DIR not a directory" >&2; exit 1; fi

mkdir -p "$TARGET_DIR"
WORK="$(mktemp -d -t batos-bake-archive-XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

# Materialize a default hello.html next to content_shell if one isn't
# already there — it's the target of `chromium file:///bin/hello.html`
# from the shell and gives a local test page that doesn't need
# network / TLS infrastructure.
SHELL_DIR="$(dirname "$SHELL_BIN")"
if [ ! -f "$SHELL_DIR/hello.html" ]; then
    cat >"$SHELL_DIR/hello.html" <<'HTML'
<!DOCTYPE html>
<html>
<head><title>Bat_OS First Render</title></head>
<body>
  <h1>Hello from Bat_OS</h1>
  <p>If you see this, Blink parsed HTML on a bare-metal Rust kernel for Apple M4.</p>
  <p id="mark">April 2026, first DOM render.</p>
</body>
</html>
HTML
    echo "[bake-archive] materialised default hello.html at $SHELL_DIR"
fi

python3 - "$WORK" "$SHELL_BIN" "$LIB_DIR" "$OUT" <<'PY'
import os, struct, sys, zlib
work, shell_bin, lib_dir, out = sys.argv[1:]

# Build the list of files we're packing. Each entry is (name_in_archive, host_path).
files = [("bin/content_shell", shell_bin)]

# icudtl.dat ships alongside content_shell in Chromium's output dir;
# Chromium's PathService looks for it at `DIR_ASSETS/icudtl.dat`,
# which resolves to the executable directory in our setup. Pack it
# under bin/ so `/bin/icudtl.dat` exists in the VFS.
shell_dir = os.path.dirname(os.path.abspath(shell_bin))
icu_candidate = os.path.join(shell_dir, "icudtl.dat")
if os.path.isfile(icu_candidate):
    files.append(("bin/icudtl.dat", icu_candidate))
    print(f"[bake-archive] including icudtl.dat from {icu_candidate}")

# Also pack any *.html, *.bat_os_*, and *.bin (minus icudtl.dat
# already packed above) files next to the shell — these are test
# pages, inherited configs, and V8 snapshots for headless runs.
for entry in sorted(os.listdir(shell_dir)):
    if entry == "icudtl.dat":
        continue
    if (entry.endswith(".html") or entry.startswith("bat_os_")
            or entry.endswith(".bin") or entry.endswith(".pak")
            or entry.endswith(".png") or entry.endswith(".jpg")
            or entry.endswith(".css")):
        full = os.path.join(shell_dir, entry)
        if os.path.isfile(full):
            files.append((f"bin/{entry}", full))
            print(f"[bake-archive]   + {entry}")

for entry in sorted(os.listdir(lib_dir)):
    full = os.path.join(lib_dir, entry)
    if os.path.isfile(full):
        files.append((f"lib/{entry}", full))

HEADER_SIZE = 128
NAME_BYTES = 64
PREAMBLE = 8 + 4 + 4  # BATARCH\0 + n_files + reserved
table_size = HEADER_SIZE * len(files)

# Compute offsets (each file 16-byte aligned).
offsets = []
cur = PREAMBLE + table_size
for _, path in files:
    sz = os.path.getsize(path)
    cur = (cur + 15) & ~15
    offsets.append((cur, sz))
    cur += sz
archive_size = cur

archive = bytearray(archive_size)
archive[0:8] = b"BATARCH\0"
struct.pack_into("<II", archive, 8, len(files), 0)

# Headers.
for i, ((name, path), (off, sz)) in enumerate(zip(files, offsets)):
    hdr = PREAMBLE + i * HEADER_SIZE
    nm = name.encode("utf-8")
    if len(nm) >= NAME_BYTES:
        raise SystemExit(f"name too long: {name}")
    archive[hdr:hdr + len(nm)] = nm
    # bytes after the name are already 0 (null-padding)
    struct.pack_into("<QQ", archive, hdr + NAME_BYTES, sz, off)

# Payload.
for (name, path), (off, sz) in zip(files, offsets):
    with open(path, "rb") as f:
        archive[off:off + sz] = f.read()

# Wrap with BATCHROM framing + CRC.
crc = zlib.crc32(bytes(archive)) & 0xFFFFFFFF
with open(out, "wb") as f:
    f.write(b"BATCHROM")
    f.write(struct.pack("<Q", archive_size))
    f.write(archive)
    f.write(struct.pack("<I", crc))
    f.write(b"CHROMEND")

print(f"[bake-archive] wrote {out}")
print(f"[bake-archive]   files:        {len(files)}")
print(f"[bake-archive]   archive size: {archive_size} bytes ({archive_size / (1024 * 1024):.1f} MB)")
print(f"[bake-archive]   crc32:        0x{crc:08x}")
for (name, _), (off, sz) in zip(files, offsets):
    print(f"   {off:>10}  {sz:>10}  {name}")
PY

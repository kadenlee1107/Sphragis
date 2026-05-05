#!/usr/bin/env bash
# Bat_OS — Pack Ladybird binaries + DT_NEEDED libs into a BATARCH
# multi-file archive, framed under the same BATCHROM…CHROMEND header
# the kernel's initrd module already understands.
#
# This is the Ladybird counterpart to tools/bake_chromium_archive.sh.
# Same archive format, same kernel-side `probe_archive` parser; only
# the contents change.
#
# The first iteration packs Ladybird's `js` CLI (the LibJS REPL) as a
# minimal baseline. js exercises: ELF load → glibc init → AK strings
# → LibCrypto → LibJS → printf → exit. If THAT runs and emits output,
# we know our pipeline can host Ladybird's libs end-to-end. Once that
# passes, swap in WebContent or headless-browser for the real DOM
# render path.
#
# Usage:
#   tools/bake_ladybird_initrd.sh ports/ladybird_port/out
#
# Layout consumed:
#   $OUT_DIR/bin/{js, WebContent, RequestServer, ImageDecoder}
#   $OUT_DIR/lib/<all DT_NEEDED libs>
#   $OUT_DIR/share/fonts/*.ttf
#
# Output:
#   target/aarch64-unknown-none/release/ladybird_initrd.bin

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$ROOT/target/aarch64-unknown-none/release"
OUT="$TARGET_DIR/ladybird_initrd.bin"

if [ $# -lt 1 ]; then
    echo "[bake-ladybird] usage: $0 <ports/ladybird_port/out>" >&2
    exit 1
fi
ARTIFACT_DIR="$1"
if [ ! -d "$ARTIFACT_DIR" ]; then
    echo "[bake-ladybird] ERROR: $ARTIFACT_DIR not a directory" >&2
    echo "  hint: run ports/ladybird_port/build.sh first to populate it." >&2
    exit 1
fi

# Pick the primary binary. js is the smallest baseline — preferred for
# the first port pass. Fall back to WebContent if js wasn't built.
PRIMARY=""
for cand in js WebContent; do
    if [ -f "$ARTIFACT_DIR/bin/$cand" ]; then
        PRIMARY="$cand"
        break
    fi
done
if [ -z "$PRIMARY" ]; then
    echo "[bake-ladybird] ERROR: neither bin/js nor bin/WebContent found in $ARTIFACT_DIR" >&2
    exit 1
fi
echo "[bake-ladybird] primary: $PRIMARY"

mkdir -p "$TARGET_DIR"
WORK="$(mktemp -d -t batos-bake-ladybird-XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

# Materialise the same hello.html the Chromium port uses, so the
# `ladybird` shell command can point at file:///bin/hello.html and
# we can compare DOM dumps between the two browser ports.
HELLO_HTML="$WORK/hello.html"
cat >"$HELLO_HTML" <<'HTML'
<!DOCTYPE html>
<html>
<head><title>Bat_OS First Render (Ladybird)</title></head>
<body>
  <h1>Hello from Bat_OS</h1>
  <p>Rendered by Ladybird's LibWeb on a bare-metal Rust kernel for Apple M4.</p>
  <p id="mark">May 2026, Ladybird first render.</p>
</body>
</html>
HTML

python3 - "$WORK" "$ARTIFACT_DIR" "$PRIMARY" "$OUT" <<'PY'
import os, struct, sys, zlib

work, artifact_dir, primary, out = sys.argv[1:]

# Build the (archive-name, host-path) list. Same convention as the
# Chromium port: binaries under bin/, runtime libs under lib/.
files = []
files.append((f"bin/{primary}", os.path.join(artifact_dir, "bin", primary)))

# All bin/* binaries (so multi-process variants can reach each other
# via /bin/<svc> at exec time).
bin_dir = os.path.join(artifact_dir, "bin")
for entry in sorted(os.listdir(bin_dir)) if os.path.isdir(bin_dir) else []:
    full = os.path.join(bin_dir, entry)
    if os.path.isfile(full) and entry != primary:
        files.append((f"bin/{entry}", full))

# DT_NEEDED libs (Ladybird typically ships ~10 libs vs Chromium's ~13)
lib_dir = os.path.join(artifact_dir, "lib")
if os.path.isdir(lib_dir):
    for entry in sorted(os.listdir(lib_dir)):
        full = os.path.join(lib_dir, entry)
        if os.path.isfile(full):
            files.append((f"lib/{entry}", full))

# Default hello.html — drop it under bin/ so file:///bin/hello.html
# resolves the same way it does for the Chromium port.
files.append(("bin/hello.html", os.path.join(work, "hello.html")))

# Optional fonts directory
share_fonts = os.path.join(artifact_dir, "share", "fonts")
if os.path.isdir(share_fonts):
    for entry in sorted(os.listdir(share_fonts)):
        full = os.path.join(share_fonts, entry)
        if os.path.isfile(full) and entry.endswith((".ttf", ".otf")):
            files.append((f"share/fonts/{entry}", full))

if len(files) > 16:
    print(f"[bake-ladybird] WARN: {len(files)} files exceeds runner.rs's "
          f"16-file cap; trimming to 16 (drop fonts first)", file=sys.stderr)
    bins   = [f for f in files if f[0].startswith("bin/")]
    libs   = [f for f in files if f[0].startswith("lib/")]
    others = [f for f in files if not (f[0].startswith("bin/") or f[0].startswith("lib/"))]
    files = (bins + libs + others)[:16]

# Header: BATARCH\0  +  n_files (u32)  +  reserved (u32)
ARCHIVE_MAGIC = b"BATARCH\0"
HEADER_SIZE = 8 + 4 + 4
ENTRY_SIZE = 64 + 8 + 8 + 48
ENTRIES_SIZE = ENTRY_SIZE * len(files)
data_start = (HEADER_SIZE + ENTRIES_SIZE + 15) & ~15

archive = bytearray()
archive += ARCHIVE_MAGIC
archive += struct.pack("<I", len(files))
archive += struct.pack("<I", 0)

# Two-pass: first compute offsets, then emit entries + files.
file_records = []
cursor = data_start
for arc_name, host_path in files:
    sz = os.path.getsize(host_path)
    file_records.append((arc_name, host_path, sz, cursor))
    cursor = (cursor + sz + 15) & ~15

for arc_name, host_path, sz, off in file_records:
    name_bytes = arc_name.encode("utf-8")
    if len(name_bytes) > 64:
        print(f"[bake-ladybird] ERROR: name too long: {arc_name}", file=sys.stderr)
        sys.exit(1)
    archive += name_bytes + b"\0" * (64 - len(name_bytes))
    archive += struct.pack("<Q", sz)
    archive += struct.pack("<Q", off)
    archive += b"\0" * 48

# Pad to data_start
archive += b"\0" * (data_start - len(archive))

for arc_name, host_path, sz, off in file_records:
    if len(archive) != off:
        print(f"[bake-ladybird] WARN: offset drift at {arc_name}: "
              f"{len(archive)} != {off}", file=sys.stderr)
    with open(host_path, "rb") as f:
        archive += f.read()
    pad = ((len(archive) + 15) & ~15) - len(archive)
    archive += b"\0" * pad

# BATCHROM frame
crc = zlib.crc32(archive) & 0xFFFFFFFF
framed = bytearray()
framed += b"BATCHROM"
framed += struct.pack("<Q", len(archive))
framed += archive
framed += struct.pack("<I", crc)
framed += b"CHROMEND"

with open(out, "wb") as f:
    f.write(framed)

mb = len(framed) / (1024 * 1024)
print(f"[bake-ladybird] wrote {out}")
print(f"[bake-ladybird]   archive: {len(archive)} bytes, {len(files)} files")
print(f"[bake-ladybird]   framed:  {len(framed)} bytes ({mb:.1f} MB)")
print(f"[bake-ladybird]   crc32:   0x{crc:08x}")
for arc_name, _, sz, off in file_records:
    print(f"     {sz:>10}  @{off:#10x}  {arc_name}")
PY

echo "[bake-ladybird] boot with: qemu-system-aarch64 ... -kernel <bat_os> -initrd $OUT"

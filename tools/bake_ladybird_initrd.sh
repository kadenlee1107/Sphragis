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
#
# STUMP #161 v2: prefer a precomputed `js_closure.txt` if it exists.
# The closure is the recursive DT_NEEDED set for the primary binary,
# which is much smaller (~33 files for js) than the full ldd-of-
# everything dump build.sh produces (~110 files including video
# codec deps Skia pulls in but `js` never touches). The closure file
# is one path-per-line, archive-relative (`bin/js`, `lib/libc.so.6`).
files = []
USE_CLOSURE = False
closure_path = os.path.join(artifact_dir, f"{primary}_closure.txt")
if os.path.isfile(closure_path):
    with open(closure_path) as f:
        for line in f:
            line = line.strip()
            if not line: continue
            host = os.path.join(artifact_dir, line)
            if os.path.exists(host):
                files.append((line, host))
    print(f"[bake-ladybird] using {primary}_closure.txt: {len(files)} files",
          file=sys.stderr)
    USE_CLOSURE = True
else:
    files.append((f"bin/{primary}", os.path.join(artifact_dir, "bin", primary)))

# All bin/* binaries (so multi-process variants can reach each other
# via /bin/<svc> at exec time). Skipped when USE_CLOSURE: closure
# mode means "ship only what the primary binary actually loads."
if not USE_CLOSURE:
    bin_dir = os.path.join(artifact_dir, "bin")
    for entry in sorted(os.listdir(bin_dir)) if os.path.isdir(bin_dir) else []:
        full = os.path.join(bin_dir, entry)
        if os.path.isfile(full) and entry != primary:
            files.append((f"bin/{entry}", full))

# DT_NEEDED libs. Lagom ships each lib as a symlink chain
# (liblagom-X.so → liblagom-X.so.0 → liblagom-X.so.0.1.0). Pack
# only the SONAME entry (.so.0) — that's what the dynamic linker
# looks up via DT_NEEDED. The .so.0.1.0 version comes through too
# only when it IS the file we picked.
#
# We resolve symlinks via os.path.realpath so each lib's content
# only lands once even though we use the SONAME name in the
# archive. This cuts ~50% of the lib payload.
lib_dir = os.path.join(artifact_dir, "lib")
seen_realpaths = {}  # realpath → archive_name we already used
if os.path.isdir(lib_dir) and not USE_CLOSURE:
    for entry in sorted(os.listdir(lib_dir)):
        full = os.path.join(lib_dir, entry)
        if not (os.path.isfile(full) or os.path.islink(full)):
            continue
        # Pack only the canonical SONAME — drop the bare `liblagom-X.so`
        # (developer convenience link with no version) AND the
        # `liblagom-X.so.0.1.0` (the real file). The dynamic linker
        # asks for `liblagom-X.so.0` per DT_NEEDED, which is what we
        # ship.
        is_dev_link = entry.endswith(".so") and ".so." not in entry
        is_full_version = entry.count(".so.") >= 1 and entry.count(".") >= 4
        if is_dev_link or is_full_version:
            continue
        real = os.path.realpath(full)
        if real in seen_realpaths:
            continue
        seen_realpaths[real] = entry
        files.append((f"lib/{entry}", real))

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

# STUMP #161: runner.rs cap was bumped 16 → 64 to fit Ladybird's
# ~30 lagom-* libs + ~12 system libs js needs at runtime.
if len(files) > 64:
    print(f"[bake-ladybird] WARN: {len(files)} files exceeds runner.rs's "
          f"64-file cap; trimming (drop fonts first, then non-essential libs)",
          file=sys.stderr)
    bins   = [f for f in files if f[0].startswith("bin/")]
    libs   = [f for f in files if f[0].startswith("lib/")]
    # Critical-first sort. Without ld-linux + libc + libstdc++ a glibc
    # binary won't even start, so those MUST be in the 40 even if it
    # means dropping a lagom-something. lagom comes before libavcodec
    # / libavformat / etc which are video deps Skia pulls in but `js`
    # never touches.
    def lib_prio(name):
        if "ld-linux" in name: return 0
        if name.endswith("/libc.so.6"): return 1
        if "libstdc++" in name: return 2
        if "libgcc_s" in name: return 3
        if name.endswith("/libm.so.6"): return 4
        if "libcrypto" in name: return 5
        if "libsimdjson" in name: return 6
        # libcpptrace + its deps (libdwarf, libzstd, libelf) — ladybird
        # links against cpptrace for stack traces. Without these the
        # dynamic linker errors out before main() runs.
        if "libcpptrace" in name: return 7
        if "libdwarf" in name: return 8
        if "libzstd" in name: return 9
        if "libelf" in name: return 10
        if "libpthread" in name: return 11
        if "libdl" in name: return 12
        if "libvulkan" in name: return 13
        if "lagom-" in name: return 14
        # Video/audio codecs come last — they're only needed by
        # WebContent's media path, never by `js`.
        if any(s in name for s in ("libav", "libsdl", "libpulse",
                                    "libasound", "libFLAC", "libogg",
                                    "libvorbis", "libtheora", "libvpx",
                                    "libdav1d", "libaom", "libwebp",
                                    "libavif", "libjxl", "libtiff",
                                    "libjpeg", "libpng", "libgif",
                                    "libtheora", "libdrm", "libGL")):
            return 90
        return 99
    libs.sort(key=lambda f: lib_prio(f[0]))
    others = [f for f in files if not (f[0].startswith("bin/") or f[0].startswith("lib/"))]
    files = (bins + libs + others)[:64]

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

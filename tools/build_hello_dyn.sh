#!/usr/bin/env bash
# Bat_OS — Build a tiny dynamic-linked reproducer.
#
# Purpose: `ports/chromium_port/out/content_shell` is 280 MB with
# 540k relocations and a crash somewhere in glibc's libc_start_main
# path. When the bug is in OUR loader / TLS / dynamic-linker setup
# rather than Chromium-specific, a 67 KB test binary that links
# against the same libc hits the same failure mode at 1/100,000th
# the surface area.
#
# Produces: ports/chromium_port/out/hello_test/hello_dyn
#
# Usage:
#   tools/build_hello_dyn.sh
#   ./tools/bake_chromium_archive.sh \
#       ports/chromium_port/out/hello_test/hello_dyn \
#       ports/chromium_port/out/lib_runtime
#   python3 scripts/qemu_chromium_pipeline_smoke.py
#
# Last-observed crash with this binary:
#   ELR 0x10427824 (inside libc.so.6 text, offset 0x27824)
#   FAR 0x000000a0 — glibc NULL-pointer dereference at offset 0xa0
#   insn 0xf94052a0 = `ldr x0, [x21, #0xa0]` with x21 uninitialized
# So glibc's init path reaches a point where it expects TLS/GOT
# state the dynamic linker should have set up. That's what's
# missing in our multi-ELF loader (init_array + proper tcbhead_t).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$ROOT/ports/chromium_port/out/hello_test"
mkdir -p "$OUT_DIR"

SRC="$(mktemp -t bat-hello-dyn-XXXX.c)"
trap "rm -f '$SRC'" EXIT

cat >"$SRC" <<'C'
// Tiny dynamic-linked reproducer for the glibc-init pathway.
// Exercises: PIE + DT_NEEDED libc + PLT/GOT + TLS + write() wrapper.
#include <unistd.h>
int main(int argc, char **argv) {
    const char *msg = "hello from dyn-linked elf!\n";
    long len = 0;
    while (msg[len]) len++;
    write(1, msg, len);
    return 42;
}
C

echo "[hello-dyn] compiling in debian:bookworm-slim arm64..."
docker run --rm --platform linux/arm64 \
    -v "$SRC:/work/hello_dyn.c:ro" \
    -v "$OUT_DIR:/out" \
    debian:bookworm-slim sh -c "
        apt-get update -qq >/dev/null 2>&1
        apt-get install -y -qq gcc >/dev/null 2>&1
        gcc -pie -o /out/hello_dyn /work/hello_dyn.c
        strip /out/hello_dyn
        ls -la /out/hello_dyn
    " | tail -5

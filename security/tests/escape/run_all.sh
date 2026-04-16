#!/usr/bin/env bash
# run_all.sh — build the four escape-test ELFs and point the kernel at
# them. This assumes you have `aarch64-linux-musl-gcc` (the same toolchain
# used to build tests/hello) on PATH, and that `src/batcave/linux/runner.rs`
# has been patched to embed these ELFs and offer a shell command like
# `batcave run <name>` that boots a cave with NO capabilities and executes
# the chosen ELF.
#
# If you don't want to patch runner.rs, you can drop the built ELFs into
# test_binaries/ and invoke them via the existing run_small_elf path.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
CC=${AARCH64_CC:-aarch64-linux-musl-gcc}
CFLAGS="-static -nostdlib -ffreestanding -Os -Wall -Wextra"

echo "[escape] building escape test ELFs in $HERE"
"$CC" $CFLAGS -o "$HERE/test_memory_peek" "$HERE/test_memory_peek.c"
"$CC" $CFLAGS -o "$HERE/test_mmio_probe"  "$HERE/test_mmio_probe.c"
"$CC" $CFLAGS -o "$HERE/test_pt_write"    "$HERE/test_pt_write.c"
"$CC" $CFLAGS -o "$HERE/test_blit_nocap"  "$HERE/test_blit_nocap.c"

echo "[escape] built:"
ls -la "$HERE"/test_memory_peek "$HERE"/test_mmio_probe \
       "$HERE"/test_pt_write  "$HERE"/test_blit_nocap

cat <<EOF

Next steps:
  1. Wire these four ELFs into src/batcave/linux/runner.rs via
     include_bytes!() and a new dispatch table (e.g. 'escape_run NAME').
  2. Boot Bat_OS under QEMU with:
       cargo run --release -p bat_os
  3. From the shell:
       batcave create test-escape
       # grant no caps, just enter
       batcave enter test-escape
       escape run memory_peek
       escape run mmio_probe
       escape run pt_write
       escape run blit_nocap
  4. Each test prints PASS/FAIL. Any FAIL is a confirmed isolation
     violation — cross-reference with security/PENTEST_SANDBOX_ESCAPE.md.
EOF

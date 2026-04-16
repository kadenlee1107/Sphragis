#!/usr/bin/env bash
# Driver for the attacks that MUST run on the live kernel under QEMU.
# Today this is a stub: the kernel's QEMU harness isn't wired for a
# pentest ELF loader yet, so the concrete exploit ELFs (KM-007, KM-009,
# KM-011, KM-015, KM-016, KM-029, KM-030) need to be built as BatCave
# guests and handed to the runner.
#
# Usage:
#   ./run_qemu_attacks.sh                     # run the host-unit tests
#   ./run_qemu_attacks.sh --qemu <path>       # (future) drive QEMU harness
#
# Each runtime attack below carries a self-contained description of what
# the guest ELF would do, so a C programmer can flesh it out without
# re-reading the audit.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$HERE"

echo "=== Running host-side KM attack tests ==="
cargo test --no-fail-fast 2>&1 | tail -40

cat <<'EOF'

=== Runtime attacks (still need a QEMU harness) ===

ATTACK-KM-009  Alignment-emulator kernel write
  Guest: an ARM64 binary that does:
      mov x1, #0x4020_0000            // kernel RAM
      movk x1, #0x0001                // pick a byte offset
      str x0, [x1]                    // unaligned if offset is odd
  Expected symptom on vulnerable kernel: the byte appears in the
  running kernel's image (observable via a later UART dump of the same
  VA).

ATTACK-KM-011  Atomic-emulator kernel write
  Guest:
      mov x5, #0x4020_0000
      mov x1, #0xdead
      stxr w0, x1, [x5]               // HVF reports EC=0, kernel emulates
  Expected: 0xDEAD written at PA 0x4020_0000.

ATTACK-KM-007  Cave page-table aliasing
  Guest scans its own low VA window for a known PTE bit pattern
  (0x0741 ORed with a 2 MB-aligned PA) and dumps hits to UART. Hits
  prove the cave can see its own page tables.

ATTACK-KM-029/030  execve path/argv leak
  Guest calls `execve(kernel_VA, {kernel_VA, NULL}, NULL)` and then
  `echo $0` — if the argv-copy path treats kernel memory as user memory,
  those bytes show up on stdout.

Wire each of the above to the harness as:
  - build:  clang --target=aarch64-none-elf -nostdlib -static attack.c -o attack
  - embed:  ports/chromium_port/bake-blob  attack  > attack.bin
  - run:    qemu-system-aarch64 … -device loader,file=attack.bin,addr=0x…
  - verify: grep -q 'UART pattern' serial.log
EOF

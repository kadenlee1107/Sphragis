#!/usr/bin/env bash
# run_all.sh — drive every DoS bomb in sequence against a Sphragis image.
#
# This script assumes:
#   * you have already built the DoS guests with
#     `aarch64-linux-musl-gcc -static -O1 foo.c -o foo` (or similar),
#     dropped them into initrd, and booted the image in HVF/QEMU.
#   * there is a way to inject `<name> <arg>` into a BatCave via the
#     existing test harness (see tests/ at repo root for patterns).
#
# Each run pairs the bomb with `canary` in a second cave and records
# the first failure mode. NOT intended to auto-execute inside CI — DoS
# tests intentionally take the system down.

set -u
HERE="$(cd "$(dirname "$0")" && pwd)"

BOMBS=(
    "mem_bomb small"
    "mem_bomb leak"
    "mem_bomb huge"
    "fd_bomb open"
    "fd_bomb socket"
    "fd_bomb eventfd"
    "fd_bomb epoll"
    "thread_bomb fill"
    "thread_bomb leak"
    "cpu_loop tight"
    "cpu_loop yield"
    "futex_park one"
    "futex_park bucket"
    "timerfd_spin"
)

echo "=== Sphragis DoS stress matrix ==="
echo "NOTE: each bomb requires a FRESH boot. The harness does not try to"
echo "      recover between runs — many DoS paths are unrecoverable by"
echo "      design (e.g. mmap leak consumes the whole frame pool)."
echo ""
for b in "${BOMBS[@]}"; do
    echo "-- bomb: $b"
    echo "   1. boot fresh Sphragis image"
    echo "   2. launch canary in cave A"
    echo "   3. launch $b in cave B"
    echo "   4. observe canary UART output; capture to logs/${b// /_}.log"
    echo ""
done

echo "When implemented against the actual boot harness, pipe each log"
echo "through security/tests/dos/grade.sh to assign A / B / C."

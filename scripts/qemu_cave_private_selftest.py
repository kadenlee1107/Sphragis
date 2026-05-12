#!/usr/bin/env python3
"""Headless smoke for the per-cave L1 restriction selftest (slice 1).

Boots Bat_OS in QEMU virt, clears the empty-passphrase auth gate,
runs `cave-private-selftest` at the shell, and asserts:

  - cave_private::ensure_page allocated the page + installed a PTE
    in sys-wg's L1 only (walked + verified via mmu::pte_lookup)
  - PRIMARY_L1 does NOT map the cave-private VA (cross-cave
    isolation property)
  - in-cave write/read round trip preserves a magic value
  - ensure_page is idempotent on re-call

Pass: exit 0. Fail: exit non-zero with serial log path.
"""
from __future__ import annotations

import sys
import time
from datetime import datetime
from pathlib import Path

import pexpect

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
LOG = (
    ROOT
    / f"logs/qemu-tests/cave-private-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
)
LOG.parent.mkdir(parents=True, exist_ok=True)

QEMU_ARGS = [
    "qemu-system-aarch64",
    "-machine", "virt",
    "-cpu", "max",
    "-m", "2G",
    "-display", "none",
    "-netdev", "user,id=net0",
    "-device", "virtio-net-device,netdev=net0",
    "-serial", "mon:stdio",
    "-kernel", str(KERNEL),
]


def main() -> int:
    if not KERNEL.exists():
        print(f"[cave-private] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[cave-private] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"bat_os > ", timeout=90)
        time.sleep(0.5)

        c.sendline("cave-private-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 per-cave L1 restriction slice-1 verified",
            rb"\xe2\x9c\x97 FAIL:",
        ], timeout=30)
        if idx == 1:
            print("[cave-private] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[cave-private] log: {LOG}", file=sys.stderr)
            return 1

        c.expect(rb"bat_os > ", timeout=10)
        print("[cave-private] PASS — per-cave L1 isolation property verified")
        print(f"[cave-private] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[cave-private] FAIL — timeout waiting for selftest output", file=sys.stderr)
        print(f"[cave-private] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

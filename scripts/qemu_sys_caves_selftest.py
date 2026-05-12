#!/usr/bin/env python3
"""Headless smoke for the sys-caves Arc-2 round-trip selftest.

Boots Bat_OS in QEMU virt, clears the empty-passphrase auth gate,
runs `sys-caves-selftest` at the shell, and asserts both legs of
the cross-cave MMU swap pass:

  - forward: kernel-ns → sys-wg loaded the cave's L1
  - return:  sys-wg → kernel-ns restored PRIMARY_L1

Pass: exit 0. Fail: exit non-zero with serial log path.
"""
from __future__ import annotations

import re
import sys
import time
from datetime import datetime
from pathlib import Path

import pexpect

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
LOG = (
    ROOT
    / f"logs/qemu-tests/sys-caves-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[sys-caves] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[sys-caves] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"bat_os > ", timeout=90)
        time.sleep(0.5)

        c.sendline("sys-caves-selftest")
        # Test prints a "✓ Arc-2 full round trip verified" line on
        # full success. Any FAIL line aborts the round trip.
        idx = c.expect([
            rb"\xe2\x9c\x93 Arc-2 full round trip verified",
            rb"\xe2\x9c\x97 FAIL:",
        ], timeout=30)
        if idx == 1:
            print("[sys-caves] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[sys-caves] log: {LOG}", file=sys.stderr)
            return 1

        # Drain back to prompt so we know the test fully ran.
        c.expect(rb"bat_os > ", timeout=10)
        print("[sys-caves] PASS — Arc-2 cross-cave MMU round trip verified")
        print(f"[sys-caves] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[sys-caves] FAIL — timeout waiting for selftest output", file=sys.stderr)
        print(f"[sys-caves] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

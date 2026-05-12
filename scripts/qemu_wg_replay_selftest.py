#!/usr/bin/env python3
"""Headless smoke for the WireGuard Phase-2.6 replay-window selftest.

Drives `wg-replay-selftest` through the shell; the kernel exercises
six spec scenarios for the 64-packet sliding window per WireGuard
whitepaper §5.4.6.

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
    / f"logs/qemu-tests/wg-replay-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[wg-replay] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[wg-replay] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"bat_os > ", timeout=90)
        time.sleep(0.5)

        c.sendline("wg-replay-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 Phase-2\.6 replay window verified",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try:
                c.expect(rb"bat_os > ", timeout=5)
            except Exception:
                pass
            print("[wg-replay] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[wg-replay] log: {LOG}", file=sys.stderr)
            return 1

        c.expect(rb"bat_os > ", timeout=10)
        print("[wg-replay] PASS — replay-window scenarios all behaved as expected")
        print(f"[wg-replay] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[wg-replay] FAIL — timeout waiting for selftest output", file=sys.stderr)
        print(f"[wg-replay] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Headless smoke for `taint-selftest` — information-flow taint
propagation on BatFS reads/writes."""
from __future__ import annotations

import sys, time
from datetime import datetime
from pathlib import Path
import pexpect

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
LOG = (ROOT / f"logs/qemu-tests/taint-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log")
LOG.parent.mkdir(parents=True, exist_ok=True)
QEMU_ARGS = [
    "qemu-system-aarch64", "-machine", "virt", "-cpu", "max",
    "-m", "2G", "-display", "none",
    "-netdev", "user,id=net0",
    "-device", "virtio-net-device,netdev=net0",
    "-serial", "mon:stdio", "-kernel", str(KERNEL),
]


def main() -> int:
    if not KERNEL.exists():
        print(f"[taint] kernel not found: {KERNEL}", file=sys.stderr); return 2
    print(f"[taint] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60); c.sendline("")
        c.expect(rb"sphragis > ", timeout=90); time.sleep(0.5)
        c.sendline("taint-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 taint-selftest PASS",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=60)
        if idx == 1:
            print("[taint] FAIL", file=sys.stderr); return 1
        c.expect(rb"sphragis > ", timeout=10)
        print("[taint] PASS — taint propagates on read+write, monotonic"); return 0
    except pexpect.TIMEOUT:
        print("[taint] FAIL — timeout", file=sys.stderr); return 1
    finally:
        try: c.close(force=True)
        except Exception: pass


if __name__ == "__main__":
    sys.exit(main())

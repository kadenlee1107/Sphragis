#!/usr/bin/env python3
"""Headless smoke for `secmark-selftest` — gov-grade §3.2 SECMARK slice."""
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
    / f"logs/qemu-tests/secmark-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
)
LOG.parent.mkdir(parents=True, exist_ok=True)

QEMU_ARGS = [
    "qemu-system-aarch64", "-machine", "virt", "-cpu", "max",
    "-m", "2G", "-display", "none",
    "-netdev", "user,id=net0",
    "-device", "virtio-net-device,netdev=net0",
    "-serial", "mon:stdio",
    "-kernel", str(KERNEL),
]


def main() -> int:
    if not KERNEL.exists():
        print(f"[secmark] kernel not found: {KERNEL}", file=sys.stderr); return 2
    print(f"[secmark] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60); c.sendline("")
        c.expect(rb"bat_os > ", timeout=90); time.sleep(0.5)
        c.sendline("secmark-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 SECMARK: CIPSO emit \+ parse \+ DOI-filter verified",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try: c.expect(rb"bat_os > ", timeout=5)
            except Exception: pass
            print("[secmark] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[secmark] log: {LOG}", file=sys.stderr); return 1
        c.expect(rb"bat_os > ", timeout=10)
        print("[secmark] PASS — CIPSO option emission + parsing verified")
        print(f"[secmark] log: {LOG}"); return 0
    except pexpect.TIMEOUT:
        print("[secmark] FAIL — timeout", file=sys.stderr)
        print(f"[secmark] log: {LOG}", file=sys.stderr); return 1
    finally:
        try: c.close(force=True)
        except Exception: pass


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Headless smoke for `audit-seal-selftest` — gov-grade §3.7 audit
off-platform seal: a frozen `(count, chain_head)` checkpoint
detects full-ring-rewrite attacks that the in-ring chain alone
can't (because the attacker would rebuild CHAIN in the same memory
write)."""
from __future__ import annotations

import sys
import time
from datetime import datetime
from pathlib import Path

import pexpect

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
LOG = (
    ROOT
    / f"logs/qemu-tests/audit-seal-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[audit-seal] kernel not found: {KERNEL}", file=sys.stderr); return 2
    print(f"[audit-seal] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60); c.sendline("")
        c.expect(rb"sphragis > ", timeout=90); time.sleep(0.5)
        c.sendline("audit-seal-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 audit-seal: full-ring-rewrite attack detected via frozen checkpoint hash",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try: c.expect(rb"sphragis > ", timeout=5)
            except Exception: pass
            print("[audit-seal] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[audit-seal] log: {LOG}", file=sys.stderr); return 1
        c.expect(rb"sphragis > ", timeout=10)
        print("[audit-seal] PASS — off-platform seal catches attacker-rebuilt chain")
        print(f"[audit-seal] log: {LOG}"); return 0
    except pexpect.TIMEOUT:
        print("[audit-seal] FAIL — timeout", file=sys.stderr)
        print(f"[audit-seal] log: {LOG}", file=sys.stderr); return 1
    finally:
        try: c.close(force=True)
        except Exception: pass


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Headless smoke for `audit-chain-selftest` — gov-grade §3.7
(audit & forensics): tamper-evident hash chain over the audit ring.

Boots Sphragis in QEMU virt, clears the empty-passphrase auth gate,
runs `audit-chain-selftest` at the shell, and asserts:

  - audit::record now updates audit_chain::CHAIN on every entry.
  - verify_chain returns Ok on a clean ring.
  - A single-byte tamper at a known index produces
    FirstMismatchAt(that index).
  - Restoring the byte recovers verify_chain back to Ok (the
    detection isn't sticky — it tracks live data).

Pass: exit 0. Fail: exit non-zero with serial log path.
"""
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
    / f"logs/qemu-tests/audit-chain-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[audit-chain] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[audit-chain] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"sphragis > ", timeout=90)
        time.sleep(0.5)

        c.sendline("audit-chain-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 audit-chain tamper-detection: verify finds the edit, recovers on restore",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try:
                c.expect(rb"sphragis > ", timeout=5)
            except Exception:
                pass
            print("[audit-chain] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[audit-chain] log: {LOG}", file=sys.stderr)
            return 1

        c.expect(rb"sphragis > ", timeout=10)
        print("[audit-chain] PASS — tamper detection on every audit entry verified")
        print(f"[audit-chain] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[audit-chain] FAIL — timeout waiting for selftest output", file=sys.stderr)
        print(f"[audit-chain] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Headless smoke for the sys-wg service Arc-3-slice-1 selftest.

Boots Sphragis in QEMU virt, clears the empty-passphrase auth gate,
runs `sys-wg-selftest` at the shell, and asserts:

  - sys-wg static pubkey is reachable (module-private keypair owns it)
  - the with_sys_wg_cave trampoline restores cave_id on return
  - handshake keys are mirror-consistent across initiator/responder
  - wrap/unwrap round trips through the service boundary

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
    / f"logs/qemu-tests/sys-wg-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[sys-wg] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[sys-wg] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"sphragis > ", timeout=90)
        time.sleep(0.5)

        c.sendline("sys-wg-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 Arc-3 slice-3 cave-private state relocation verified",
            rb"\xe2\x9c\x97 FAIL:",
        ], timeout=30)
        if idx == 1:
            print("[sys-wg] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[sys-wg] log: {LOG}", file=sys.stderr)
            return 1

        c.expect(rb"sphragis > ", timeout=10)
        print("[sys-wg] PASS — sys-wg service boundary verified end-to-end")
        print(f"[sys-wg] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[sys-wg] FAIL — timeout waiting for selftest output", file=sys.stderr)
        print(f"[sys-wg] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

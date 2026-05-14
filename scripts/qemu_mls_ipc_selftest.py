#!/usr/bin/env python3
"""Headless smoke for `mls-ipc-selftest` — gov-grade §3.2 labeled IPC."""
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
    / f"logs/qemu-tests/mls-ipc-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[mls-ipc] kernel not found: {KERNEL}", file=sys.stderr); return 2
    print(f"[mls-ipc] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60); c.sendline("")
        c.expect(rb"sphragis > ", timeout=90); time.sleep(0.5)
        c.sendline("mls-ipc-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 MLS labeled-IPC: BLP write-down \+ read-up enforcement verified",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try: c.expect(rb"sphragis > ", timeout=5)
            except Exception: pass
            print("[mls-ipc] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[mls-ipc] log: {LOG}", file=sys.stderr); return 1
        c.expect(rb"sphragis > ", timeout=10)
        print("[mls-ipc] PASS — labeled-IPC write-down + read-up enforcement verified")
        print(f"[mls-ipc] log: {LOG}"); return 0
    except pexpect.TIMEOUT:
        print("[mls-ipc] FAIL — timeout", file=sys.stderr)
        print(f"[mls-ipc] log: {LOG}", file=sys.stderr); return 1
    finally:
        try: c.close(force=True)
        except Exception: pass


if __name__ == "__main__":
    sys.exit(main())

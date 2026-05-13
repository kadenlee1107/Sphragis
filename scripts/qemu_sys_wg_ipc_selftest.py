#!/usr/bin/env python3
"""Headless smoke for the sys-wg IPC mailbox selftest (Arc 3 slice 3).

Drives `sys-wg-ipc-selftest` through the shell; the kernel posts
an OP_PUBKEY request into the global mailbox, spawns a service
task tagged with sys-wg's cave_id, the service task reads the
request and calls `sys_wg_service::service_pubkey`, writes the
32-byte response, and terminates. Client picks up the response
and compares to the direct API.

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
    / f"logs/qemu-tests/sys-wg-ipc-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[sys-wg-ipc] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[sys-wg-ipc] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"bat_os > ", timeout=90)
        time.sleep(0.5)

        c.sendline("sys-wg-ipc-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 Arc-3 slice-3 IPC mailbox path verified",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try:
                c.expect(rb"bat_os > ", timeout=5)
            except Exception:
                pass
            print("[sys-wg-ipc] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[sys-wg-ipc] log: {LOG}", file=sys.stderr)
            return 1

        c.expect(rb"bat_os > ", timeout=10)
        print("[sys-wg-ipc] PASS — IPC mailbox round trip verified")
        print(f"[sys-wg-ipc] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[sys-wg-ipc] FAIL — timeout waiting for selftest output", file=sys.stderr)
        print(f"[sys-wg-ipc] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

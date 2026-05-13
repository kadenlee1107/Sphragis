#!/usr/bin/env python3
"""Headless smoke for the WG initiator-role end-to-end selftest.

Drives `wg-initiator-e2e-selftest`. sys-wg initiates via the IPC
mailbox (OP_START_HANDSHAKE); the test plays responder; the
resulting Response wire bytes go back into dispatch_wire which
internally calls OP_FINISH_HANDSHAKE; a transport round trip
confirms the session keys work.

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
    / f"logs/qemu-tests/wg-initiator-e2e-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[wg-init-e2e] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[wg-init-e2e] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"bat_os > ", timeout=90)
        time.sleep(0.5)

        c.sendline("wg-initiator-e2e-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 WG initiator-role end-to-end \(IPC \+ dispatch\) verified",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try:
                c.expect(rb"bat_os > ", timeout=5)
            except Exception:
                pass
            print("[wg-init-e2e] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[wg-init-e2e] log: {LOG}", file=sys.stderr)
            return 1

        c.expect(rb"bat_os > ", timeout=10)
        print("[wg-init-e2e] PASS — initiator-role through IPC + dispatch verified")
        print(f"[wg-init-e2e] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[wg-init-e2e] FAIL — timeout waiting for selftest output", file=sys.stderr)
        print(f"[wg-init-e2e] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

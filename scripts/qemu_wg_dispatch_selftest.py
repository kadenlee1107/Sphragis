#!/usr/bin/env python3
"""Headless smoke for the WireGuard Phase-2.5 dispatch-layer selftest.

Drives `wg-dispatch-selftest` through the shell; the kernel
exercises the end-to-end inbound path: synthetic Init wire bytes
through `wg_dispatch::dispatch_wire`, parse the returned Response,
finish handshake initiator-side, encrypt + send via dispatch,
verify decrypted plaintext.

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
    / f"logs/qemu-tests/wg-dispatch-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[wg-dispatch] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[wg-dispatch] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"bat_os > ", timeout=90)
        time.sleep(0.5)

        c.sendline("wg-dispatch-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 Phase-2\.5 dispatch path verified",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try:
                c.expect(rb"bat_os > ", timeout=5)
            except Exception:
                pass
            print("[wg-dispatch] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[wg-dispatch] log: {LOG}", file=sys.stderr)
            return 1

        c.expect(rb"bat_os > ", timeout=10)
        print("[wg-dispatch] PASS — Phase-2.5 dispatch path end-to-end verified")
        print(f"[wg-dispatch] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[wg-dispatch] FAIL — timeout waiting for selftest output", file=sys.stderr)
        print(f"[wg-dispatch] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Headless smoke for the WG endpoint-config + outbound-connect selftest.

Drives `wg-endpoint-selftest`. Validates the IPC opcodes
OP_SET_ENDPOINT / OP_GET_ENDPOINT round-trip a (127.0.0.1, 51820)
endpoint through sys-wg's cave-private storage, and that
`wg_dispatch::initiate_connect` builds the InitMsg and invokes
udp::send to the configured peer.

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
    / f"logs/qemu-tests/wg-endpoint-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[wg-endpoint] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[wg-endpoint] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"sphragis > ", timeout=90)
        time.sleep(0.5)

        c.sendline("wg-endpoint-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 WG endpoint config \+ outbound-connect plumbing verified",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try:
                c.expect(rb"sphragis > ", timeout=5)
            except Exception:
                pass
            print("[wg-endpoint] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[wg-endpoint] log: {LOG}", file=sys.stderr)
            return 1

        c.expect(rb"sphragis > ", timeout=10)
        print("[wg-endpoint] PASS — endpoint config + outbound connect plumbing verified")
        print(f"[wg-endpoint] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[wg-endpoint] FAIL — timeout waiting for selftest output", file=sys.stderr)
        print(f"[wg-endpoint] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

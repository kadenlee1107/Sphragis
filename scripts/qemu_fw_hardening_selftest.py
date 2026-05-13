#!/usr/bin/env python3
"""Headless smoke for `fw-hardening-selftest` — gap-audit item 045
hardening pass.

Boots Bat_OS in QEMU virt, clears the empty-passphrase auth gate,
runs `fw-hardening-selftest` at the shell, and asserts the new
stateful inbound-TCP policy:

  - Unsolicited SYN to a random ephemeral port (no conntrack flow,
    no listener) -> DROPPED.
  - Inbound segment matching a registered conntrack flow ->
    ALLOWED (reply traffic for Bat_OS-initiated connection).
  - The same flow does NOT leak to a different remote.
  - Inbound SYN to a registered listener port -> ALLOWED.
  - listen_close revokes the per-port allow.
  - release_local_port revokes the conntrack-derived allow.

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
    / f"logs/qemu-tests/fw-hardening-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[fw-hardening] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[fw-hardening] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"bat_os > ", timeout=90)
        time.sleep(0.5)

        c.sendline("fw-hardening-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 stateful firewall hardening: unsolicited SYN drop \+ flow/listener gating verified",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try:
                c.expect(rb"bat_os > ", timeout=5)
            except Exception:
                pass
            print("[fw-hardening] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[fw-hardening] log: {LOG}", file=sys.stderr)
            return 1

        c.expect(rb"bat_os > ", timeout=10)
        print("[fw-hardening] PASS — unsolicited SYN dropped, flow + listener gating verified")
        print(f"[fw-hardening] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[fw-hardening] FAIL — timeout waiting for selftest output", file=sys.stderr)
        print(f"[fw-hardening] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

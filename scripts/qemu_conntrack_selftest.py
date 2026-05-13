#!/usr/bin/env python3
"""Headless smoke for `conntrack-selftest` — gap-audit item 045.

Boots Bat_OS in QEMU virt, clears the empty-passphrase auth gate,
runs `conntrack-selftest` at the shell, and asserts the stateful
flow-table lifecycle:

  register_outbound -> lookup_inbound -> mark_established ->
  idempotent re-register -> release_local_port (scoped).

This module is the foundation for stateful inbound filtering
(removing the wildcard inbound TCP firewall rule); today's slice
ships the table + lifecycle + outbound-side hook in
`tcp::connect_start` / `tcp::close_pcb`.

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
    / f"logs/qemu-tests/conntrack-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[conntrack] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[conntrack] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"bat_os > ", timeout=90)
        time.sleep(0.5)

        c.sendline("conntrack-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 conntrack lifecycle \(register \xe2\x86\x92 lookup \xe2\x86\x92 upgrade \xe2\x86\x92 release\) verified",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try:
                c.expect(rb"bat_os > ", timeout=5)
            except Exception:
                pass
            print("[conntrack] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[conntrack] log: {LOG}", file=sys.stderr)
            return 1

        c.expect(rb"bat_os > ", timeout=10)
        print("[conntrack] PASS — conntrack flow-table lifecycle verified")
        print(f"[conntrack] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[conntrack] FAIL — timeout waiting for selftest output", file=sys.stderr)
        print(f"[conntrack] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

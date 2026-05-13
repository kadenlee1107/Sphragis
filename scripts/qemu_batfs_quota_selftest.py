#!/usr/bin/env python3
"""Headless smoke for `batfs-quota-selftest` — gap-audit item 030
second slice: cave memory quota enforced on the BatFS write path.

The selftest drives sys-wg via `cave::with_cave_active` and asserts
that `batfs::ns_create` charges the cave's quota, `ns_delete`
releases it, and quota-exceeded creates fail fast with the
expected error string.

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
    / f"logs/qemu-tests/batfs-quota-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[batfs-quota] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[batfs-quota] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"bat_os > ", timeout=90)
        time.sleep(0.5)

        c.sendline("batfs-quota-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 batfs quota-enforcement: charge \+ release verified",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try:
                c.expect(rb"bat_os > ", timeout=5)
            except Exception:
                pass
            print("[batfs-quota] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[batfs-quota] log: {LOG}", file=sys.stderr)
            return 1

        c.expect(rb"bat_os > ", timeout=10)
        print("[batfs-quota] PASS — quota charged on ns_create, released on ns_delete")
        print(f"[batfs-quota] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[batfs-quota] FAIL — timeout waiting for selftest output", file=sys.stderr)
        print(f"[batfs-quota] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

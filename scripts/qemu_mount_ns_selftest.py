#!/usr/bin/env python3
"""Headless smoke for `mount-ns-selftest` — gap-audit item 032
mount-namespace auto-application.

Boots Sphragis in QEMU virt, clears the empty-passphrase auth gate,
runs `mount-ns-selftest` at the shell, and asserts the cross-cave
file isolation property:

  - Two caves can create the same logical filename without
    collision (`sealfs::ns_create` prepends the active cave's
    mount prefix).
  - Each cave's `sealfs::ns_read` returns its own content; the
    other cave's view is invisible.
  - `sealfs::ns_list` from inside a cave never leaks the on-disk
    prefix (no `<cave>:` in any visible name).
  - The un-prefixed `sealfs::list` (kernel/admin context) sees
    BOTH on-disk entries.

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
    / f"logs/qemu-tests/mount-ns-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[mount-ns] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[mount-ns] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"sphragis > ", timeout=90)
        time.sleep(0.5)

        c.sendline("mount-ns-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 mount-namespace auto-application: per-cave file isolation verified",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try:
                c.expect(rb"sphragis > ", timeout=5)
            except Exception:
                pass
            print("[mount-ns] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[mount-ns] log: {LOG}", file=sys.stderr)
            return 1

        c.expect(rb"sphragis > ", timeout=10)
        print("[mount-ns] PASS — per-cave file-namespace isolation verified")
        print(f"[mount-ns] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[mount-ns] FAIL — timeout waiting for selftest output", file=sys.stderr)
        print(f"[mount-ns] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Headless smoke for `redirect-selftest` — gap-audit item 039
(shell pipes / job control — output-capture slice).

Boots Sphragis in QEMU virt, clears the empty-passphrase auth gate,
runs `redirect-selftest` at the shell, and asserts:

  - `console::begin_capture` / `end_capture` round-trip a string.
  - `parse_redirect` correctly extracts a filename from
    "<inner> > <file>" while NOT splitting on a ` > ` inside a
    quoted argument.
  - `execute_with_redirect("whoami", "redirect-probe.txt")` writes
    captured bytes to BatFS via `ns_create`, and a follow-up
    `ns_read` returns the same content.

This is the load-bearing primitive behind real `|` pipes; pipes
themselves are a follow-up that needs a handful of commands to
accept a buffer-input shape.

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
    / f"logs/qemu-tests/redirect-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
        print(f"[redirect] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[redirect] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"sphragis > ", timeout=90)
        time.sleep(0.5)

        c.sendline("redirect-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 shell `>` redirect end-to-end \(capture -> ns_create -> ns_read\) verified",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try:
                c.expect(rb"sphragis > ", timeout=5)
            except Exception:
                pass
            print("[redirect] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[redirect] log: {LOG}", file=sys.stderr)
            return 1

        c.expect(rb"sphragis > ", timeout=10)
        print("[redirect] PASS — output capture + shell `>` redirect verified")
        print(f"[redirect] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[redirect] FAIL — timeout waiting for selftest output", file=sys.stderr)
        print(f"[redirect] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Headless tab-autofill smoke for the Bat_OS shell.

Builds the kernel, boots in QEMU virt, drives the auth gate with the
default empty passphrase, then types partial commands followed by Tab
and asserts the shell completes correctly:

  Test 1 — unique-prefix completion:
    Type `pq-c` then Tab → expect `pq-comms-selftest` to appear in serial.

  Test 2 — multi-match listing:
    Type `pq-` then Tab → expect at least four `pq-…` candidates listed
    on a new line (pq-comms-selftest, pq-selftest, pq-sig-selftest,
    pq-tls-selftest).

Pass: both assertions hold.
Fail: missing completion, wrong match, timeout, or kernel panic.

The smoke builds with `gicv3,selftest-on-boot` so the boot reaches the
auth gate (where stdin keystrokes get handed to the real shell after
auth). After the empty passphrase fall-through, the kernel runs
`ui::desktop::run` if a virtio-gpu is present — to keep this smoke
headless we omit virtio-gpu so the kernel falls through to
`main::serial_shell()`, which is the path autofill needs to validate.
"""
from __future__ import annotations

import re
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path

import pexpect

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
LOG = (
    ROOT
    / f"logs/qemu-tests/shell-autofill-smoke-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
)
LOG.parent.mkdir(parents=True, exist_ok=True)

# No virtio-gpu/keyboard so boot lands in main::serial_shell, not desktop.
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


def build() -> int:
    print("[autofill-smoke] building --release --features gicv3...")
    r = subprocess.run(
        [
            "cargo", "build",
            "--release",
            "--target", "aarch64-unknown-none",
            "--features", "gicv3",
        ],
        cwd=ROOT, capture_output=True, text=True,
    )
    if r.returncode != 0:
        print("[autofill-smoke] cargo build FAILED:", file=sys.stderr)
        print(r.stderr[-2000:], file=sys.stderr)
        return r.returncode
    print(f"[autofill-smoke] build ok ({KERNEL.stat().st_size:,} bytes)")
    return 0


def run() -> int:
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        # Wait for the auth-gate prompt (serial path).
        c.expect(rb"Enter passphrase", timeout=60)
        # Empty passphrase falls through to dev default.
        c.sendline("")
        # Auth eventually times out / falls through to the serial shell.
        c.expect(rb"bat_os > ", timeout=90)
        time.sleep(0.5)

        # ── Test 1: unique-prefix completion ──
        c.send(b"pq-c")
        time.sleep(0.2)
        c.send(b"\t")
        # After Tab, the shell should have written `omms-selftest` (the
        # rest of `pq-comms-selftest`). Wait for the full token to
        # appear in serial.
        c.expect(rb"pq-comms-selftest", timeout=10)
        # Cancel this line — Ctrl+C resets the prompt.
        c.send(b"\x03")
        c.expect(rb"bat_os > ", timeout=10)
        print("[autofill-smoke]   PASS unique: 'pq-c' + Tab → 'pq-comms-selftest'")

        # ── Test 2: multi-match listing ──
        c.send(b"pq-")
        time.sleep(0.2)
        c.send(b"\t")
        # Should see all the pq-* candidates on the listing line.
        c.expect(rb"pq-comms-selftest", timeout=10)
        c.expect(rb"pq-selftest", timeout=5)
        c.expect(rb"pq-sig-selftest", timeout=5)
        c.expect(rb"pq-tls-selftest", timeout=5)
        c.send(b"\x03")
        print("[autofill-smoke]   PASS multi: 'pq-' + Tab listed pq-comms/selftest/sig/tls")

        print("[autofill-smoke] PASS — tab autofill works for unique + multi-match")
        print(f"[autofill-smoke] log: {LOG}")
        return 0
    except pexpect.TIMEOUT:
        print("[autofill-smoke] FAIL — timeout", file=sys.stderr)
        print(f"[autofill-smoke] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


def main() -> int:
    rc = build()
    if rc != 0:
        return rc
    if not KERNEL.exists():
        print(f"[autofill-smoke] kernel missing: {KERNEL}", file=sys.stderr)
        return 2
    return run()


if __name__ == "__main__":
    sys.exit(main())

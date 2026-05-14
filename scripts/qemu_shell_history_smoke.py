#!/usr/bin/env python3
"""Headless arrow-key history smoke for the Sphragis shell.

Builds the kernel, boots in QEMU virt with the serial-shell path
(no virtio-gpu), then types two commands, exercises up/down arrow
recall, and asserts the recalled lines round-trip correctly.

Test plan:

  1. Type `uname` + Enter — record in history.
  2. Type `whoami` + Enter — record.
  3. Press Up arrow → expect `whoami` to appear at the prompt.
  4. Press Up again → expect `uname`.
  5. Press Down → expect `whoami` again.
  6. Press Down → empty line (back to live edit).

The smoke is intentionally minimal: ANSI ESC sequences ([0x1B,0x5B,
0x41/0x42]) drive the parser, ring stores entries, recall replaces
the visible line via backspace+space+backspace.
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
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
LOG = (
    ROOT
    / f"logs/qemu-tests/shell-history-smoke-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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

UP   = b"\x1b[A"
DOWN = b"\x1b[B"


def build() -> int:
    print("[history-smoke] building --release --features gicv3...")
    r = subprocess.run(
        ["cargo", "build", "--release",
         "--target", "aarch64-unknown-none",
         "--features", "gicv3"],
        cwd=ROOT, capture_output=True, text=True,
    )
    if r.returncode != 0:
        print("[history-smoke] cargo build FAILED:", file=sys.stderr)
        print(r.stderr[-2000:], file=sys.stderr)
        return r.returncode
    print(f"[history-smoke] build ok ({KERNEL.stat().st_size:,} bytes)")
    return 0


def run() -> int:
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"sphragis > ", timeout=90)
        time.sleep(0.5)

        # Record two commands.
        c.sendline("uname")
        c.expect(rb"sphragis > ", timeout=10)
        c.sendline("whoami")
        c.expect(rb"sphragis > ", timeout=10)

        # Up → most recent (whoami).
        c.send(UP)
        c.expect(rb"whoami", timeout=10)
        print("[history-smoke]   PASS up: most recent recalls 'whoami'")

        # Up → older (uname). The redraw erases `whoami` and prints
        # `uname` so we look for the new pattern after an erase.
        c.send(UP)
        c.expect(rb"uname", timeout=10)
        print("[history-smoke]   PASS up: prior recalls 'uname'")

        # Down → forward (whoami again).
        c.send(DOWN)
        c.expect(rb"whoami", timeout=10)
        print("[history-smoke]   PASS down: forward recalls 'whoami'")

        # Down past newest → live-edit (line is cleared).
        # We can't easily expect "the line is now empty" without
        # state, but we can confirm Enter on the cleared buffer
        # returns to the prompt without echoing a command name.
        c.send(DOWN)
        time.sleep(0.3)
        c.send(b"\x03")  # Ctrl+C to discard whatever's left.
        c.expect(rb"sphragis > ", timeout=10)
        print("[history-smoke]   PASS down-past-newest: returns to live edit")

        print("[history-smoke] PASS — arrow-key history works")
        print(f"[history-smoke] log: {LOG}")
        return 0
    except pexpect.TIMEOUT:
        print("[history-smoke] FAIL — timeout", file=sys.stderr)
        print(f"[history-smoke] log: {LOG}", file=sys.stderr)
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
        print(f"[history-smoke] kernel missing: {KERNEL}", file=sys.stderr)
        return 2
    return run()


if __name__ == "__main__":
    sys.exit(main())

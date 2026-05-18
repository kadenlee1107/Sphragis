#!/usr/bin/env python3
"""Headless smoke for `cap-mls-selftest` — Eng-3 §3 gov-grade
capability tokens + MLS labels. Runs the six TDD scenarios from
`docs/superpowers/plans/2026-05-17-multi-team-push.md` §3 in QEMU
and asserts the kernel reports each scenario as PASS.

Pattern mirrors `qemu_biba_selftest.py`.
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
    / f"logs/qemu-tests/cap-mls-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
)
LOG.parent.mkdir(parents=True, exist_ok=True)

QEMU_ARGS = [
    "qemu-system-aarch64", "-machine", "virt", "-cpu", "max",
    "-m", "2G", "-display", "none",
    "-netdev", "user,id=net0",
    "-device", "virtio-net-device,netdev=net0",
    "-serial", "mon:stdio",
    "-kernel", str(KERNEL),
]

# Per-scenario success markers. The kernel prints each line as
# `  ✓ <name>` (UTF-8 check mark + name); the bytes below are the
# UTF-8 encoding.
SCENARIOS = [
    "label_dominance_self",
    "label_dominance_strict",
    "bell_lapadula_read_up_denied",
    "biba_write_up_denied",
    "cap_token_forge_attempt",
    "cap_token_valid_call_passes",
]
FINAL = b"\xe2\x9c\x93 Cap-token \\+ MLS-label: all 6 scenarios verified"


def main() -> int:
    if not KERNEL.exists():
        print(f"[cap-mls] kernel not found: {KERNEL}", file=sys.stderr); return 2
    print(f"[cap-mls] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60); c.sendline("")
        c.expect(rb"sphragis > ", timeout=90); time.sleep(0.5)
        c.sendline("cap-mls-selftest")
        for name in SCENARIOS:
            pat_ok   = rb"\xe2\x9c\x93 " + name.encode()
            pat_fail = rb"\xe2\x9c\x97 FAIL: " + name.encode()
            idx = c.expect([pat_ok, pat_fail], timeout=30)
            if idx == 1:
                print(f"[cap-mls] FAIL — selftest reported failure on {name}",
                      file=sys.stderr)
                print(f"[cap-mls] log: {LOG}", file=sys.stderr)
                return 1
        # Wait for the final summary line so we know all 6 ran
        # (vs. the kernel emitting partial output and hanging).
        c.expect(FINAL, timeout=30)
        c.expect(rb"sphragis > ", timeout=10)
        print("[cap-mls] PASS — 6/6 scenarios verified")
        print(f"[cap-mls] log: {LOG}")
        return 0
    except pexpect.TIMEOUT:
        print("[cap-mls] FAIL — timeout", file=sys.stderr)
        print(f"[cap-mls] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try: c.close(force=True)
        except Exception: pass


if __name__ == "__main__":
    sys.exit(main())

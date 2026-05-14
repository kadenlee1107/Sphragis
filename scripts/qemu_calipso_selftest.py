#!/usr/bin/env python3
"""Headless smoke for `calipso-selftest` — RFC 5570 IPv6 SECMARK
option encode/parse. IPv6 stack itself isn't yet in tree, so this
covers the format work only; the wire integration plugs in when
v6 lands."""
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
    / f"logs/qemu-tests/calipso-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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


def main() -> int:
    if not KERNEL.exists():
        print(f"[calipso] kernel not found: {KERNEL}", file=sys.stderr); return 2
    print(f"[calipso] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60); c.sendline("")
        c.expect(rb"sphragis > ", timeout=90); time.sleep(0.5)
        c.sendline("calipso-selftest")
        idx = c.expect([
            rb"\xe2\x9c\x93 CALIPSO: RFC 5570 encode/parse \+ checksum \+ DOI gate verified",
            rb"\xe2\x9c\x97 FAIL: \S+",
        ], timeout=30)
        if idx == 1:
            try: c.expect(rb"sphragis > ", timeout=5)
            except Exception: pass
            print("[calipso] FAIL — selftest reported a failure", file=sys.stderr)
            print(f"[calipso] log: {LOG}", file=sys.stderr); return 1
        c.expect(rb"sphragis > ", timeout=10)
        print("[calipso] PASS — CALIPSO encode/parse + checksum + DOI gate verified")
        print(f"[calipso] log: {LOG}"); return 0
    except pexpect.TIMEOUT:
        print("[calipso] FAIL — timeout", file=sys.stderr)
        print(f"[calipso] log: {LOG}", file=sys.stderr); return 1
    finally:
        try: c.close(force=True)
        except Exception: pass


if __name__ == "__main__":
    sys.exit(main())

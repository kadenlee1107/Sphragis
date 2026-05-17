#!/usr/bin/env python3
"""Headless argument-completion smoke for the Sphragis shell.

Creates two SealFS files via `write`, then tests that:
  1. `read fo<Tab>` completes uniquely to `read foobar` (only file
     starting with `fo`).
  2. `read <Tab>` lists both files (multi-match).

This validates the past-space Tab path: `arg_kind_for("read", 0)`
returns `ArgKind::File`, and `complete_argument` enumerates SealFS
to find candidates.
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
    / f"logs/qemu-tests/shell-argcomplete-smoke-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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


def build() -> int:
    print("[argcomplete-smoke] building --release --features gicv3...")
    r = subprocess.run(
        ["cargo", "build", "--release",
         "--target", "aarch64-unknown-none",
         "--features", "gicv3"],
        cwd=ROOT, capture_output=True, text=True,
    )
    if r.returncode != 0:
        print("[argcomplete-smoke] cargo build FAILED:", file=sys.stderr)
        print(r.stderr[-2000:], file=sys.stderr)
        return r.returncode
    print(f"[argcomplete-smoke] build ok ({KERNEL.stat().st_size:,} bytes)")
    return 0


def run() -> int:
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None)
    try:
        c.expect(rb"Enter passphrase", timeout=60)
        c.sendline("")
        c.expect(rb"sphragis > ", timeout=90)
        time.sleep(0.5)

        # Seed two files.
        c.sendline('write foobar "hello"')
        c.expect(rb"sphragis > ", timeout=10)
        c.sendline('write foozap "world"')
        c.expect(rb"sphragis > ", timeout=10)
        c.sendline('write barbaz "third"')
        c.expect(rb"sphragis > ", timeout=10)

        # Test 1: unique-prefix → `read foob<Tab>` completes to `read foobar`.
        c.send(b"read foob")
        time.sleep(0.2)
        c.send(b"\t")
        c.expect(rb"read foobar", timeout=10)
        print("[argcomplete-smoke]   PASS unique: 'read foob' + Tab → 'read foobar'")
        c.send(b"\x03")
        c.expect(rb"sphragis > ", timeout=10)

        # Test 2: multi-match → `read fo<Tab>` lists both `foo*` files
        # and extends to `foo` (the common prefix).
        c.send(b"read fo")
        time.sleep(0.2)
        c.send(b"\t")
        c.expect(rb"foobar", timeout=10)
        c.expect(rb"foozap", timeout=10)
        print("[argcomplete-smoke]   PASS multi: 'read fo' + Tab listed foobar + foozap")
        c.send(b"\x03")
        c.expect(rb"sphragis > ", timeout=10)

        # Test 3: empty-prefix → `read <Tab>` lists every file.
        c.send(b"read ")
        time.sleep(0.2)
        c.send(b"\t")
        c.expect(rb"foobar", timeout=10)
        c.expect(rb"barbaz", timeout=10)
        print("[argcomplete-smoke]   PASS empty: 'read ' + Tab listed all files")
        c.send(b"\x03")
        c.expect(rb"sphragis > ", timeout=10)

        # Test 4: subcommand completion — `pkg <Tab>` lists the four
        # pkg subcommands (`install / list / remove / stage`).
        c.send(b"pkg ")
        time.sleep(0.2)
        c.send(b"\t")
        c.expect(rb"install", timeout=10)
        c.expect(rb"stage", timeout=10)
        print("[argcomplete-smoke]   PASS subcmd: 'pkg ' + Tab listed pkg subcommands")
        c.send(b"\x03")
        c.expect(rb"sphragis > ", timeout=10)

        # Test 5: unique-prefix subcommand — `pkg in<Tab>` extends to
        # `pkg install`.
        c.send(b"pkg in")
        time.sleep(0.2)
        c.send(b"\t")
        c.expect(rb"pkg install", timeout=10)
        print("[argcomplete-smoke]   PASS subcmd-unique: 'pkg in' + Tab -> 'pkg install'")
        c.send(b"\x03")
        c.expect(rb"sphragis > ", timeout=10)

        # Test 6: subcommand-aware arg — `pkg install fo<Tab>` should
        # complete from SealFS files (the (cmd, subcommand) lookup
        # picks ArgKind::File at arg_index=1).
        c.send(b"pkg install fo")
        time.sleep(0.2)
        c.send(b"\t")
        c.expect(rb"foobar", timeout=10)
        c.expect(rb"foozap", timeout=10)
        print("[argcomplete-smoke]   PASS subcmd-arg: 'pkg install fo' + Tab listed sealfs files")
        c.send(b"\x03")
        c.expect(rb"sphragis > ", timeout=10)

        # Test 7: common-path enum — `tz <Tab>` lists the curated
        # offset suggestions (+8, -5, etc.).
        c.send(b"tz ")
        time.sleep(0.2)
        c.send(b"\t")
        c.expect(rb"-5", timeout=10)
        c.expect(rb"\+8", timeout=10)
        print("[argcomplete-smoke]   PASS common-path: 'tz ' + Tab listed offset suggestions")
        c.send(b"\x03")
        c.expect(rb"sphragis > ", timeout=10)

        # Test 8: kbd-trace on/off — `kbd-trace o<Tab>` is ambiguous
        # between `on` and `off`. The candidate list comes back in
        # alphabetical order so we expect `off` first, then `on`.
        c.send(b"kbd-trace o")
        time.sleep(0.2)
        c.send(b"\t")
        c.expect(rb"off", timeout=10)
        c.expect(rb"on", timeout=10)
        print("[argcomplete-smoke]   PASS common-path: 'kbd-trace o' + Tab listed off/on")

        print("[argcomplete-smoke] PASS — argument + subcommand + common-path completion verified")
        print(f"[argcomplete-smoke] log: {LOG}")
        return 0
    except pexpect.TIMEOUT:
        print("[argcomplete-smoke] FAIL — timeout", file=sys.stderr)
        print(f"[argcomplete-smoke] log: {LOG}", file=sys.stderr)
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
        print(f"[argcomplete-smoke] kernel missing: {KERNEL}", file=sys.stderr)
        return 2
    return run()


if __name__ == "__main__":
    sys.exit(main())

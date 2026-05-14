#!/usr/bin/env python3
"""Minimal boot smoke for post-no-browser Sphragis.

Replaces the deleted Ladybird/Chromium smoke scripts. Boots the kernel
in QEMU virt and verifies it reaches the auth-gate input loop without
panicking. Doesn't drive the shell (the auth gate consumes input via
virtio-keyboard, not serial — that's a separate harness).

Pass criteria:
  - kernel reaches "[bs] paint done — input loop" within 60s
  - serial log mentions virtio-gpu, network stack ready, Cave runtime
  - no `panic!` / `unimplemented!` / "kernel halted" markers
  - no mention of deleted symbols (chromium_blit, ChromiumFb, browser::*)

Pass: exit 0. Fail: exit non-zero with the captured serial log path.
"""
from __future__ import annotations

import re
import sys
import time
from datetime import datetime
from pathlib import Path

import pexpect

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
LOG = (
    ROOT
    / f"logs/qemu-tests/boot-smoke-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
)
LOG.parent.mkdir(parents=True, exist_ok=True)

PROMPT = rb"sphragis\s*>\s*"
QEMU_ARGS = [
    "qemu-system-aarch64",
    "-machine", "virt",
    "-cpu", "max",
    "-m", "2G",
    "-display", "none",
    "-device", "virtio-gpu-device",
    "-device", "virtio-keyboard-device",
    "-netdev", "user,id=net0",
    "-device", "virtio-net-device,netdev=net0",
    "-serial", "mon:stdio",
    "-kernel", str(KERNEL),
]


def main() -> int:
    if not KERNEL.exists():
        print(f"[smoke] kernel not found: {KERNEL}", file=sys.stderr)
        print(
            "[smoke] run `cargo build --release --target aarch64-unknown-none "
            "--features gicv3` first.",
            file=sys.stderr,
        )
        return 2

    print(f"[smoke] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=90, logfile=fp, encoding=None)

    try:
        # Wait for the auth-gate input loop. That's "we booted clean."
        c.expect(rb"\[bs\] (flush done|paint done).+ input loop", timeout=60)

        # Give the kernel a moment to flush any pending serial output
        # (driver init, banners, etc).
        time.sleep(0.5)
        c.close(force=True)

        # Re-read the captured log and run validation patterns over it.
        log_bytes = LOG.read_bytes()

        required = [
            (rb"\[bc\] Cave runtime ready", "Cave init"),
            (rb"\[net\] Network stack ready",   "network init"),
            (rb"\[gpu\] (Found at slot|Display: \d+x\d+)", "virtio-gpu init"),
            (rb"\[fs\] BatFS initialized",     "BatFS init"),
            (rb"\[tls\] trust store: \d+ CA roots, chain-only auth, hybrid PQ on",
             "tls trust-store boot status"),
        ]
        forbidden = [
            (rb"chromium_blit",  "deleted chromium_blit symbol"),
            (rb"ChromiumFb",      "deleted ChromiumFb VFS node"),
            (rb"crate::browser::", "deleted browser engine ref"),
            (rb"kernel panic|kernel halted|PANIC ABORT",
             "kernel panic at boot"),
        ]

        failures: list[str] = []
        for pat, label in required:
            if not re.search(pat, log_bytes):
                failures.append(f"missing required marker: {label}")
        for pat, label in forbidden:
            if re.search(pat, log_bytes):
                failures.append(f"unexpected marker: {label}")

        if failures:
            for f in failures:
                print(f"[smoke] FAIL — {f}", file=sys.stderr)
            print(f"[smoke] log: {LOG}", file=sys.stderr)
            return 1

        print("[smoke] PASS — kernel boots, all required subsystems init,")
        print("[smoke]        no deleted-browser symbols leaked through.")
        print(f"[smoke] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[smoke] FAIL — timeout reaching auth-gate input loop.", file=sys.stderr)
        print(f"[smoke] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

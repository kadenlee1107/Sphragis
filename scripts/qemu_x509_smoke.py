#!/usr/bin/env python3
"""Headless x509-selftest smoke.

Builds the kernel with --features gicv3,selftest-on-boot, boots it in
QEMU virt, and verifies both `[x509-selftest] PASS:` lines appear in
serial output before the auth gate.

The `selftest-on-boot` Cargo feature wires `cmd_x509_selftest()` into
src/main.rs after net/vfs init and before the auth gate, so this
smoke doesn't need virtio-keyboard injection — it just watches serial.

Pass: both PASS lines present, no FAIL lines, no kernel panic. Exit 0.
Fail: any FAIL line, missing PASS, timeout, or panic. Exit 1.
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
    / f"logs/qemu-tests/x509-smoke-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
)
LOG.parent.mkdir(parents=True, exist_ok=True)

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


def build_with_feature() -> int:
    """Build the kernel with the selftest-on-boot feature."""
    print("[x509-smoke] building with --features gicv3,selftest-on-boot...")
    result = subprocess.run(
        [
            "cargo", "build",
            "--release",
            "--target", "aarch64-unknown-none",
            "--features", "gicv3,selftest-on-boot",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print("[x509-smoke] cargo build FAILED:", file=sys.stderr)
        print(result.stderr[-2000:], file=sys.stderr)
        return result.returncode
    print(f"[x509-smoke] build ok ({KERNEL.stat().st_size:,} bytes)")
    return 0


def run_smoke() -> int:
    """Boot the selftest-enabled kernel and check for PASS lines."""
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=90, logfile=fp, encoding=None)

    try:
        # Selftest fires after VFS init, before auth gate. Wait until
        # we see either both PASS lines, a FAIL, or the auth-gate
        # banner (which means the selftest hook didn't run — feature
        # build problem).
        # Give it 60s for boot + selftest. The selftest itself is
        # microseconds; the wait is dominated by boot.
        # Match either "running x509-selftest" (selftest hook fired)
        # or "Launching auth gate" (selftest hook missed).
        idx = c.expect([
            rb"\[selftest\] running x509-selftest before auth gate",
            rb"\[security\] Launching auth gate",
            pexpect.TIMEOUT,
        ], timeout=60)
        if idx == 1:
            print("[x509-smoke] FAIL — selftest hook did not fire (feature flag problem?)", file=sys.stderr)
            print(f"[x509-smoke] log: {LOG}", file=sys.stderr)
            return 1
        if idx == 2:
            print("[x509-smoke] FAIL — timeout reaching selftest hook", file=sys.stderr)
            print(f"[x509-smoke] log: {LOG}", file=sys.stderr)
            return 1

        # Selftest is running. Wait for both PASS lines (or any FAIL).
        # The selftest prints two PASS/FAIL lines, then control returns
        # to main.rs which prints "Launching auth gate". Read until
        # auth-gate banner; collect everything we saw.
        c.expect(rb"\[security\] Launching auth gate", timeout=15)
        # Re-read the captured log and scan for PASS/FAIL lines.
        fp.flush()
        log_bytes = LOG.read_bytes()

        # console::puts echoes to both framebuffer and serial, so each
        # PASS/FAIL line shows up twice in the captured log. Dedupe by
        # subtest name (the bit after "PASS: " / "FAIL: ").
        pass_raw = re.findall(rb"\[x509-selftest\] PASS: (\S+)", log_bytes)
        fail_raw = re.findall(rb"\[x509-selftest\] FAIL: (\S+)", log_bytes)
        pass_subtests = sorted(set(s.decode("utf-8", "replace") for s in pass_raw))
        fail_subtests = sorted(set(s.decode("utf-8", "replace") for s in fail_raw))

        for s in pass_subtests:
            print(f"[x509-smoke]   PASS: {s}")
        for s in fail_subtests:
            print(f"[x509-smoke]   FAIL: {s}")

        if fail_subtests:
            print("[x509-smoke] FAIL — selftest reported failures.", file=sys.stderr)
            print(f"[x509-smoke] log: {LOG}", file=sys.stderr)
            return 1

        # Expect at least 2 unique sub-tests passing.
        if len(pass_subtests) < 2:
            print(
                f"[x509-smoke] FAIL — expected 2 PASS sub-tests, got {len(pass_subtests)}",
                file=sys.stderr,
            )
            print(f"[x509-smoke] log: {LOG}", file=sys.stderr)
            return 1

        print("[x509-smoke] PASS — both sub-tests reported PASS, no FAIL lines.")
        print(f"[x509-smoke] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[x509-smoke] FAIL — timeout during selftest run.", file=sys.stderr)
        print(f"[x509-smoke] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


def main() -> int:
    rc = build_with_feature()
    if rc != 0:
        return rc
    if not KERNEL.exists():
        print(f"[x509-smoke] kernel not found after build: {KERNEL}", file=sys.stderr)
        return 2
    return run_smoke()


if __name__ == "__main__":
    sys.exit(main())

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
from datetime import datetime
from pathlib import Path

import pexpect

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
LOG = (
    ROOT
    / f"logs/qemu-tests/selftests-smoke-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
    print("[selftests-smoke] building with --features gicv3,selftest-on-boot...")
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
        print("[selftests-smoke] cargo build FAILED:", file=sys.stderr)
        print(result.stderr[-2000:], file=sys.stderr)
        return result.returncode
    print(f"[selftests-smoke] build ok ({KERNEL.stat().st_size:,} bytes)")
    return 0


def run_smoke() -> int:
    """Boot the selftest-enabled kernel and check for PASS lines."""
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=90, logfile=fp, encoding=None)

    try:
        # Selftests fire after VFS init, before auth gate. Wait until we
        # see "running x509-selftest" (first selftest hook fired) or
        # "Launching auth gate" (hook missed = feature flag problem).
        idx = c.expect([
            rb"\[selftest\] running x509-selftest before auth gate",
            rb"\[security\] Launching auth gate",
            pexpect.TIMEOUT,
        ], timeout=60)
        if idx == 1:
            print("[selftests-smoke] FAIL — selftest hook did not fire (feature flag problem?)", file=sys.stderr)
            print(f"[selftests-smoke] log: {LOG}", file=sys.stderr)
            return 1
        if idx == 2:
            print("[selftests-smoke] FAIL — timeout reaching selftest hook", file=sys.stderr)
            print(f"[selftests-smoke] log: {LOG}", file=sys.stderr)
            return 1

        # Both selftests run sequentially; auth-gate banner comes after.
        c.expect(rb"\[security\] Launching auth gate", timeout=15)
        fp.flush()
        log_bytes = LOG.read_bytes()

        # console::puts echoes to both framebuffer and serial, so each
        # PASS/FAIL line shows up twice. Dedupe by subtest name.
        x509_pass_raw = re.findall(rb"\[x509-selftest\] PASS: (\S+)", log_bytes)
        x509_fail_raw = re.findall(rb"\[x509-selftest\] FAIL: (\S+)", log_bytes)
        sched_pass_raw = re.findall(rb"\[scheduler-selftest\] PASS: (\S+)", log_bytes)
        sched_fail_raw = re.findall(rb"\[scheduler-selftest\] FAIL: (\S+)", log_bytes)

        x509_pass = sorted(set(s.decode("utf-8", "replace") for s in x509_pass_raw))
        x509_fail = sorted(set(s.decode("utf-8", "replace") for s in x509_fail_raw))
        sched_pass = sorted(set(s.decode("utf-8", "replace") for s in sched_pass_raw))
        sched_fail = sorted(set(s.decode("utf-8", "replace") for s in sched_fail_raw))

        for s in x509_pass:
            print(f"[selftests-smoke]   x509 PASS: {s}")
        for s in x509_fail:
            print(f"[selftests-smoke]   x509 FAIL: {s}")
        for s in sched_pass:
            print(f"[selftests-smoke]   scheduler PASS: {s}")
        for s in sched_fail:
            print(f"[selftests-smoke]   scheduler FAIL: {s}")

        if x509_fail or sched_fail:
            print("[selftests-smoke] FAIL — one or more selftests reported failures.", file=sys.stderr)
            print(f"[selftests-smoke] log: {LOG}", file=sys.stderr)
            return 1

        if len(x509_pass) < 2:
            print(
                f"[selftests-smoke] FAIL — expected 2 x509 PASS sub-tests, got {len(x509_pass)}",
                file=sys.stderr,
            )
            print(f"[selftests-smoke] log: {LOG}", file=sys.stderr)
            return 1
        if len(sched_pass) < 4:
            print(
                f"[selftests-smoke] FAIL — expected 4 scheduler PASS sub-tests, got {len(sched_pass)}",
                file=sys.stderr,
            )
            print(f"[selftests-smoke] log: {LOG}", file=sys.stderr)
            return 1

        print("[selftests-smoke] PASS — all sub-tests reported PASS, no FAIL lines.")
        print(f"[selftests-smoke] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[selftests-smoke] FAIL — timeout during selftest run.", file=sys.stderr)
        print(f"[selftests-smoke] log: {LOG}", file=sys.stderr)
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
        print(f"[selftests-smoke] kernel not found after build: {KERNEL}", file=sys.stderr)
        return 2
    return run_smoke()


if __name__ == "__main__":
    sys.exit(main())

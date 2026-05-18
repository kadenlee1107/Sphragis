#!/usr/bin/env python3
"""Headless smoke for SealFS rotation + journal + audit (2026-05-17 Eng-2 push).

Builds the kernel with `--features selftest-on-boot`, boots it in
QEMU virt, and asserts that all 6 TDD scenarios from the push plan
§3 (Eng-2) report PASS via serial. The selftest function lives in
`src/ui/shell.rs::cmd_sealfs_rotation_selftest` and is invoked from
`src/main.rs` before the auth gate when `selftest-on-boot` is
enabled.

Pass criteria
-------------
All 6 expected `[sealfs-rotation] <label> PASS` lines appear on the
serial console before the auth gate banner, and no
`[sealfs-rotation] <label> FAIL <reason>` line appears.

The 6 expected labels (in scenario order, per §3 (Eng-2)):
  1. rotation_old_data_still_decryptable
  2. rotation_new_data_uses_new_key
  3. journal_recovery_after_partial_write
  4. audit_log_records_mount
  5. audit_log_records_rotation
  6. audit_log_append_only

Pass: PASS line for every expected label, no FAIL lines. Exit 0.
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
    / f"logs/qemu-tests/sealfs-rotation-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
)
LOG.parent.mkdir(parents=True, exist_ok=True)

EXPECTED_LABELS = [
    b"rotation_old_data_still_decryptable",
    b"rotation_new_data_uses_new_key",
    b"journal_recovery_after_partial_write",
    b"audit_log_records_mount",
    b"audit_log_records_rotation",
    b"audit_log_append_only",
]

QEMU_ARGS = [
    "qemu-system-aarch64",
    "-machine", "virt",
    "-cpu", "max",
    "-m", "2G",
    "-display", "none",
    # The boot-time `selftest-on-boot` block in `src/main.rs` lives
    # inside the `Some(())` branch of `gpu::init()`. Without a
    # virtio-gpu device the kernel falls through to `serial_shell()`
    # and skips the SealFS rotation selftest entirely. Same trick
    # `qemu_x509_chain_selftest.py` uses.
    "-device", "virtio-gpu-device",
    "-device", "virtio-keyboard-device",
    "-netdev", "user,id=net0",
    "-device", "virtio-net-device,netdev=net0",
    "-serial", "mon:stdio",
    "-kernel", str(KERNEL),
]


def build_with_feature() -> int:
    """Build the kernel with the selftest-on-boot feature.

    Honours `--skip-build` so the caller can run the smoke against
    an existing kernel binary (useful when a cross-team checkout is
    already built).
    """
    if "--skip-build" in sys.argv:
        print("[sealfs-rotation-smoke] --skip-build set; using existing kernel binary")
        return 0
    print("[sealfs-rotation-smoke] cargo build --features selftest-on-boot ...")
    rc = subprocess.run(
        [
            "cargo", "build", "--release",
            "--target", "aarch64-unknown-none",
            "--features", "selftest-on-boot",
        ],
        cwd=str(ROOT),
    ).returncode
    if rc != 0:
        print(f"[sealfs-rotation-smoke] cargo build failed rc={rc}", file=sys.stderr)
    return rc


def main() -> int:
    rc = build_with_feature()
    if rc != 0:
        return rc

    if not KERNEL.exists():
        print(f"[sealfs-rotation-smoke] kernel not found: {KERNEL}", file=sys.stderr)
        return 2

    print(f"[sealfs-rotation-smoke] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(
        QEMU_ARGS[0], QEMU_ARGS[1:], timeout=120, logfile=fp, encoding=None,
    )

    saw_pass = {lab: False for lab in EXPECTED_LABELS}
    saw_fail: list[tuple[bytes, bytes]] = []

    try:
        # Wait for the suite-start marker.
        c.expect(rb"\[sealfs-rotation\] suite start", timeout=90)
        # Then walk PASS/FAIL lines until "suite end".
        while True:
            idx = c.expect([
                rb"\[sealfs-rotation\] (\S+) PASS",
                rb"\[sealfs-rotation\] (\S+) FAIL ([^\r\n]*)",
                rb"\[sealfs-rotation\] suite end",
                rb"Enter passphrase",
            ], timeout=60)
            if idx == 0:
                lab = c.match.group(1)
                if lab in saw_pass:
                    saw_pass[lab] = True
                    print(f"[sealfs-rotation-smoke] PASS {lab.decode()}")
                else:
                    print(f"[sealfs-rotation-smoke] WARN unexpected PASS label {lab!r}")
            elif idx == 1:
                lab = c.match.group(1)
                reason = c.match.group(2)
                saw_fail.append((lab, reason))
                print(f"[sealfs-rotation-smoke] FAIL {lab.decode()}: {reason.decode(errors='replace')}",
                      file=sys.stderr)
            else:
                # suite end OR auth gate prompt — done.
                break

        missing = [lab for lab, ok in saw_pass.items() if not ok]
        if saw_fail or missing:
            for lab in missing:
                print(f"[sealfs-rotation-smoke] MISSING {lab.decode()}", file=sys.stderr)
            print(f"[sealfs-rotation-smoke] log: {LOG}", file=sys.stderr)
            return 1

        print(f"[sealfs-rotation-smoke] PASS — all 6 §3 (Eng-2) scenarios green")
        print(f"[sealfs-rotation-smoke] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[sealfs-rotation-smoke] FAIL — timeout waiting for selftest output",
              file=sys.stderr)
        print(f"[sealfs-rotation-smoke] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

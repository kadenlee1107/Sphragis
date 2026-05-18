#!/usr/bin/env python3
"""Headless smoke for the X.509 chain-validator (2026-05-17 Eng-1 push).

Builds the kernel with `--features selftest-on-boot`, boots it in
QEMU virt, and asserts that all 6 TDD scenarios from the push plan
§3 (Eng-1) report PASS via serial. The selftest function lives in
`src/net/x509.rs::run_chain_selftest` and is driven from
`src/ui/shell.rs::cmd_x509_selftest`, which `src/main.rs` invokes
before the auth gate when `selftest-on-boot` is enabled.

Pass criteria
-------------
All 6 expected `[x509-chain-selftest] <label> PASS` lines appear on
the serial console before the auth gate banner, and no
`[x509-chain-selftest] <label> FAIL <reason>` line appears.

The 6 expected labels (in scenario order):
  1. valid_chain_3_levels
  2. chain_signature_mismatch
  3. chain_expired_intermediate
  4. chain_unknown_root
  5. chain_basic_constraints_violated
  6. revocation_stub_returns_ok

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
    / f"logs/qemu-tests/x509-chain-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
)
LOG.parent.mkdir(parents=True, exist_ok=True)

EXPECTED_LABELS = [
    b"valid_chain_3_levels",
    b"chain_signature_mismatch",
    b"chain_expired_intermediate",
    b"chain_unknown_root",
    b"chain_basic_constraints_violated",
    b"revocation_stub_returns_ok",
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
    # and skips the chain selftest entirely. The same trick is used
    # by `qemu_pq_interop_smoke.py`; `-display none` keeps the host
    # quiet while still letting the guest see the gpu device.
    "-device", "virtio-gpu-device",
    "-device", "virtio-keyboard-device",
    "-netdev", "user,id=net0",
    "-device", "virtio-net-device,netdev=net0",
    "-serial", "mon:stdio",
    "-kernel", str(KERNEL),
]


def build_with_feature() -> int:
    """Build the kernel with the selftest-on-boot feature.

    Honours `--skip-build` so the caller can run the smoke against an
    existing kernel binary (useful when a cross-team checkout is
    temporarily un-buildable for unrelated reasons — the kernel on
    disk still works for boot tests).
    """
    if "--skip-build" in sys.argv:
        if KERNEL.exists():
            print(
                f"[x509-chain-selftest] --skip-build set, using existing "
                f"kernel ({KERNEL.stat().st_size:,} bytes)"
            )
            return 0
        print(
            "[x509-chain-selftest] --skip-build set but no kernel on disk",
            file=sys.stderr,
        )
        return 2
    print("[x509-chain-selftest] building with --features selftest-on-boot...")
    result = subprocess.run(
        [
            "cargo", "build",
            "--release",
            "--target", "aarch64-unknown-none",
            "--features", "selftest-on-boot",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print("[x509-chain-selftest] cargo build FAILED:", file=sys.stderr)
        print(result.stderr[-2000:], file=sys.stderr)
        return result.returncode
    print(f"[x509-chain-selftest] build ok ({KERNEL.stat().st_size:,} bytes)")
    return 0


def run_smoke() -> int:
    """Boot the selftest-enabled kernel and check for PASS/FAIL lines."""
    fp = open(LOG, "wb")
    # 180 s timeout: net init + chain selftest is cheap (~ms), but cold
    # QEMU boot of the kernel can take a while on a busy host.
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=180, logfile=fp, encoding=None)

    try:
        # First wait for the selftest banner so we know the hook fired.
        idx = c.expect([
            rb"running x509-selftest before auth gate",
            rb"\[security\] Launching auth gate",
            pexpect.TIMEOUT,
        ], timeout=120)
        if idx == 1:
            print(
                "[x509-chain-selftest] FAIL — selftest hook did not fire "
                "(feature flag problem?)",
                file=sys.stderr,
            )
            print(f"[x509-chain-selftest] log: {LOG}", file=sys.stderr)
            return 1
        if idx == 2:
            print(
                "[x509-chain-selftest] FAIL — timeout reaching selftest hook",
                file=sys.stderr,
            )
            print(f"[x509-chain-selftest] log: {LOG}", file=sys.stderr)
            return 1

        # Wait for the auth-gate banner — that's our end marker; by the
        # time it prints, every scenario has emitted its PASS or FAIL.
        c.expect([
            rb"\[security\] Launching auth gate",
            pexpect.TIMEOUT,
        ], timeout=120)
        fp.flush()
        log_bytes = LOG.read_bytes()

        # `console::puts` mirrors to framebuffer + serial, AND inside
        # the framebuffer path each char is forwarded byte-by-byte via
        # `uart::putc` while the whole-string mirror also runs — so
        # every printed string appears DOUBLED on the serial line (see
        # the comment block in `src/ui/console.rs` around line 569).
        # We therefore allow the label to repeat zero-or-one times in
        # the regex and match the PASS/FAIL suffix on the whole line.
        # We also tolerate leading whitespace that doubling inserts
        # between the tag and the body.
        pass_re = re.compile(
            rb"\[x509-chain-selftest\]\s+(?P<label>\w+?)(?:(?P=label))?\s+PASS"
        )
        fail_re = re.compile(
            rb"\[x509-chain-selftest\]\s+(?P<label>\w+?)(?:(?P=label))?\s+FAIL\s+(?P<reason>[^\r\n]*)"
        )
        pass_set = {m.group("label") for m in pass_re.finditer(log_bytes)}
        fail_set = {
            (m.group("label"), m.group("reason"))
            for m in fail_re.finditer(log_bytes)
        }

        ok = True
        for label in EXPECTED_LABELS:
            if label in pass_set:
                print(f"[x509-chain-selftest]   PASS: {label.decode()}")
            else:
                print(
                    f"[x509-chain-selftest]   MISSING: {label.decode()}",
                    file=sys.stderr,
                )
                ok = False
        for lbl, reason in fail_set:
            print(
                f"[x509-chain-selftest]   FAIL: {lbl.decode()} — "
                f"{reason.decode('utf-8', 'replace')}",
                file=sys.stderr,
            )
            ok = False

        if not ok:
            print("[x509-chain-selftest] FAIL — see PASS/FAIL/MISSING list above",
                  file=sys.stderr)
            print(f"[x509-chain-selftest] log: {LOG}", file=sys.stderr)
            return 1

        print("[x509-chain-selftest] PASS — all 6 chain-validator scenarios green.")
        print(f"[x509-chain-selftest] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[x509-chain-selftest] FAIL — timeout during selftest run.",
              file=sys.stderr)
        print(f"[x509-chain-selftest] log: {LOG}", file=sys.stderr)
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
        print(
            f"[x509-chain-selftest] kernel not found after build: {KERNEL}",
            file=sys.stderr,
        )
        return 2
    return run_smoke()


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Headless smoke for the X.509 chain-validator (2026-05-17 Eng-1 push).

Boots Sphragis in QEMU virt, clears the empty-passphrase auth gate,
runs the `x509-selftest` shell command, and asserts that all 6 TDD
scenarios from the push plan §3 (Eng-1) report PASS via serial.

The selftest function lives in
`src/net/x509.rs::run_chain_selftest` and is driven from
`src/ui/shell.rs::cmd_x509_selftest`, which appends the 6 chain
scenarios to the existing 2 legacy selftests (hostname-mismatch +
truncated-DER). This smoke greps for the new `[x509-chain-selftest]
<label> PASS|FAIL <reason>` lines specifically.

Pass criteria
-------------
All 6 expected labels appear on PASS lines and no FAIL line
appears for any label.

The 6 expected labels (in scenario order):
  1. valid_chain_3_levels
  2. chain_signature_mismatch
  3. chain_expired_intermediate
  4. chain_unknown_root
  5. chain_basic_constraints_violated
  6. revocation_stub_returns_ok

Build modes
-----------
This script does NOT rebuild the kernel — it boots whatever binary
is currently at `target/aarch64-unknown-none/release/sphragis`. The
shell command path means no `selftest-on-boot` Cargo feature is
required. Build the kernel normally with:
  cargo build --release --target aarch64-unknown-none

Pass: PASS line for every expected label, no FAIL lines. Exit 0.
Fail: any FAIL line, missing PASS, timeout, or panic. Exit 1.
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
    "-netdev", "user,id=net0",
    "-device", "virtio-net-device,netdev=net0",
    "-serial", "mon:stdio",
    "-kernel", str(KERNEL),
]


def main() -> int:
    if not KERNEL.exists():
        print(f"[x509-chain-selftest] kernel not found: {KERNEL}", file=sys.stderr)
        print(
            "[x509-chain-selftest] run "
            "`cargo build --release --target aarch64-unknown-none` first",
            file=sys.stderr,
        )
        return 2

    print(f"[x509-chain-selftest] booting kernel ({KERNEL.stat().st_size:,} bytes)...")
    fp = open(LOG, "wb")
    c = pexpect.spawn(
        QEMU_ARGS[0], QEMU_ARGS[1:], timeout=180, logfile=fp, encoding=None,
    )

    try:
        # Wait for the auth-gate passphrase prompt.
        c.expect(rb"Enter passphrase", timeout=120)
        # Empty passphrase → dev default (per `cmd_x509_selftest`
        # design — same path the existing audit-chain smoke uses).
        c.sendline("")
        c.expect(rb"sphragis > ", timeout=120)
        time.sleep(0.5)

        # Run the shell command. `cmd_x509_selftest` first emits the
        # legacy 2-case `[x509-selftest]` lines, then the 6 new
        # `[x509-chain-selftest]` lines (the Eng-1 chunk).
        c.sendline("x509-selftest")
        # Wait for the prompt to come back (selftest completes).
        c.expect(rb"sphragis > ", timeout=60)
        fp.flush()
        log_bytes = LOG.read_bytes()

        # `console::puts` mirrors to framebuffer + serial, AND inside
        # the framebuffer path each char is forwarded byte-by-byte
        # via `uart::putc` while the whole-string mirror also runs —
        # so every printed string appears DOUBLED on the serial line
        # (see the comment block in `src/ui/console.rs` around line
        # 569). We tolerate the label repeating zero-or-one times.
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
            print(
                "[x509-chain-selftest] FAIL — see PASS/FAIL/MISSING list above",
                file=sys.stderr,
            )
            print(f"[x509-chain-selftest] log: {LOG}", file=sys.stderr)
            return 1

        print("[x509-chain-selftest] PASS — all 6 chain-validator scenarios green.")
        print(f"[x509-chain-selftest] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print(
            "[x509-chain-selftest] FAIL — timeout during selftest run.",
            file=sys.stderr,
        )
        print(f"[x509-chain-selftest] log: {LOG}", file=sys.stderr)
        return 1
    finally:
        try:
            c.close(force=True)
        except Exception:
            pass


if __name__ == "__main__":
    sys.exit(main())

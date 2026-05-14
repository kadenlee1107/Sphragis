#!/usr/bin/env python3
"""Headless PQ-interop smoke.

Builds the kernel with --features gicv3,pq-interop-test, boots it in
QEMU virt with virtio-net (user networking), and verifies that a real
TLS 1.3 + X25519MLKEM768 hybrid PQ handshake to pq.cloudflareresearch.com
succeeds — i.e. our IETF draft-ietf-tls-ecdhe-mlkem-04 wire format
interops with a real third-party PQ-capable TLS server.

Why this exists
---------------
Our closed-loop tls_hybrid::selftest round-trips client+server through
the same bytes; a wire-format bug there would round-trip silently
(both sides read/write the same broken layout). Only an external
peer running the actual spec catches that. pq.cloudflareresearch.com
is Cloudflare's published PQ-TLS demo endpoint and selects
X25519MLKEM768 by preference.

Pass criteria
-------------
A single `[pq-interop] PASS hybrid-pq-handshake-ok` line on the serial
console, with no `[pq-interop] FAIL` lines and no kernel panic. The
boot-hook (cmd_pq_interop in src/ui/shell.rs) explicitly fails the
smoke if the server fell back to plain X25519 — that prevents the
test from passing on a wire-format bug that only triggers in the PQ
path.

Pass: PASS line present, no FAIL lines, no panic. Exit 0.
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
    / f"logs/qemu-tests/pq-interop-smoke-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
    # User networking: lets the guest reach the real internet via the
    # host's resolver / NAT. Required to actually contact
    # pq.cloudflareresearch.com.
    "-netdev", "user,id=net0",
    "-device", "virtio-net-device,netdev=net0",
    "-serial", "mon:stdio",
    "-kernel", str(KERNEL),
]


def build_with_feature() -> int:
    """Build the kernel with the pq-interop-test feature."""
    print("[pq-interop-smoke] building with --features gicv3,pq-interop-test...")
    result = subprocess.run(
        [
            "cargo", "build",
            "--release",
            "--target", "aarch64-unknown-none",
            "--features", "gicv3,pq-interop-test",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print("[pq-interop-smoke] cargo build FAILED:", file=sys.stderr)
        print(result.stderr[-2000:], file=sys.stderr)
        return result.returncode
    print(f"[pq-interop-smoke] build ok ({KERNEL.stat().st_size:,} bytes)")
    return 0


def run_smoke() -> int:
    """Boot the pq-interop-enabled kernel and check for the PASS line."""
    fp = open(LOG, "wb")
    # Wider timeout: the handshake includes DNS + TCP + TLS over user
    # networking from a cold-booted QEMU guest. 180 s is generous but
    # the hook is gated and runs once.
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=180, logfile=fp, encoding=None)

    try:
        idx = c.expect([
            rb"\[pq-interop\] running hybrid PQ handshake",
            rb"\[security\] Launching auth gate",
            pexpect.TIMEOUT,
        ], timeout=120)
        if idx == 1:
            print(
                "[pq-interop-smoke] FAIL — interop hook did not fire (feature flag problem?)",
                file=sys.stderr,
            )
            print(f"[pq-interop-smoke] log: {LOG}", file=sys.stderr)
            return 1
        if idx == 2:
            print("[pq-interop-smoke] FAIL — timeout reaching interop hook", file=sys.stderr)
            print(f"[pq-interop-smoke] log: {LOG}", file=sys.stderr)
            return 1

        # Wait for either the PASS line, a FAIL line, or auth-gate banner.
        c.expect([
            rb"\[pq-interop\] PASS",
            rb"\[pq-interop\] FAIL",
            rb"\[security\] Launching auth gate",
        ], timeout=120)
        fp.flush()
        log_bytes = LOG.read_bytes()

        # console::puts mirrors to framebuffer + serial, so each line
        # may show up twice. Dedupe by reason.
        pass_lines = sorted(set(re.findall(rb"\[pq-interop\] PASS\s+(\S+)", log_bytes)))
        fail_lines = sorted(set(re.findall(rb"\[pq-interop\] FAIL\s+(.+)", log_bytes)))

        for s in pass_lines:
            print(f"[pq-interop-smoke]   PASS: {s.decode('utf-8', 'replace')}")
        for s in fail_lines:
            print(f"[pq-interop-smoke]   FAIL: {s.decode('utf-8', 'replace')}")

        if fail_lines:
            print("[pq-interop-smoke] FAIL — interop reported failures.", file=sys.stderr)
            print(f"[pq-interop-smoke] log: {LOG}", file=sys.stderr)
            return 1
        if not pass_lines:
            print(
                "[pq-interop-smoke] FAIL — no PASS line observed (timeout or silent abort).",
                file=sys.stderr,
            )
            print(f"[pq-interop-smoke] log: {LOG}", file=sys.stderr)
            return 1

        print(
            "[pq-interop-smoke] PASS — real-world hybrid PQ TLS handshake succeeded."
        )
        print(f"[pq-interop-smoke] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[pq-interop-smoke] FAIL — timeout during interop run.", file=sys.stderr)
        print(f"[pq-interop-smoke] log: {LOG}", file=sys.stderr)
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
        print(f"[pq-interop-smoke] kernel not found after build: {KERNEL}", file=sys.stderr)
        return 2
    return run_smoke()


if __name__ == "__main__":
    sys.exit(main())

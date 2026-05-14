#!/usr/bin/env python3
"""Headless end-to-end HTTPS smoke.

Builds the kernel with --features gicv3,https-smoke-test, boots it in
QEMU virt with virtio-net (user networking), and verifies that a real
HTTPS request — TLS handshake + GET / HTTP/1.1 + response drain —
against pq.cloudflareresearch.com succeeds.

Why this exists
---------------
The pq-interop-test feature already proves the TLS handshake works
end-to-end against a real PQ-capable server. This smoke goes one step
further: it proves the kernel can actually send an HTTP request and
read back an HTTP response over that TLS session — i.e. HTTPS works
as a feature, not just as a protocol implementation.

The boot-hook (run_https_smoke in src/main.rs) calls
net::https::open_kernel directly so this smoke covers the kernel-side
machinery. A separate cave-side ABI smoke (driving the actual
sys_bat_https_open syscall from a test cave) lands in a follow-up.

Pass criteria
-------------
A single `[https-smoke] PASS http-status=2XX body-bytes=…` line on
the serial console with no `[https-smoke] FAIL` lines and no kernel
panic. Expect the host to return `200`, `301`, `302`, or `308` —
any 2xx/3xx is fine; we're proving the round-trip, not the content.

Pass: PASS line present (status starts with 2 or 3), no FAIL lines,
      no panic. Exit 0.
Fail: any FAIL line, missing PASS, status >= 4xx, timeout, or panic.
      Exit 1.
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
    / f"logs/qemu-tests/https-smoke-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
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
    """Build the kernel with the https-smoke-test feature."""
    print("[https-smoke] building with --features gicv3,https-smoke-test...")
    result = subprocess.run(
        [
            "cargo", "build",
            "--release",
            "--target", "aarch64-unknown-none",
            "--features", "gicv3,https-smoke-test",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print("[https-smoke] cargo build FAILED:", file=sys.stderr)
        print(result.stderr[-2000:], file=sys.stderr)
        return result.returncode
    print(f"[https-smoke] build ok ({KERNEL.stat().st_size:,} bytes)")
    return 0


def run_smoke() -> int:
    """Boot the smoke-enabled kernel and check the result."""
    fp = open(LOG, "wb")
    # Wider timeout: the request includes DNS + TCP + TLS handshake +
    # HTTP request/response over user networking from a cold-booted
    # QEMU guest. 180 s is generous; the hook is gated and runs once.
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=180, logfile=fp, encoding=None)

    try:
        idx = c.expect([
            rb"\[https-smoke\] starting end-to-end HTTPS request",
            rb"\[security\] Launching auth gate",
            pexpect.TIMEOUT,
        ], timeout=120)
        if idx == 1:
            print(
                "[https-smoke] FAIL — smoke hook did not fire (feature flag problem?)",
                file=sys.stderr,
            )
            print(f"[https-smoke] log: {LOG}", file=sys.stderr)
            return 1
        if idx == 2:
            print("[https-smoke] FAIL — timeout reaching smoke hook", file=sys.stderr)
            print(f"[https-smoke] log: {LOG}", file=sys.stderr)
            return 1

        # Wait for either PASS, FAIL, or auth-gate banner.
        c.expect([
            rb"\[https-smoke\] PASS",
            rb"\[https-smoke\] FAIL",
            rb"\[security\] Launching auth gate",
        ], timeout=120)
        fp.flush()
        log_bytes = LOG.read_bytes()

        # console::puts mirrors framebuffer + serial — dedupe by reason.
        pass_lines = sorted(set(re.findall(rb"\[https-smoke\] PASS\s+(.+)", log_bytes)))
        fail_lines = sorted(set(re.findall(rb"\[https-smoke\] FAIL\s+(.+)", log_bytes)))

        for s in pass_lines:
            print(f"[https-smoke]   PASS: {s.decode('utf-8', 'replace')}")
        for s in fail_lines:
            print(f"[https-smoke]   FAIL: {s.decode('utf-8', 'replace')}")

        if fail_lines:
            print("[https-smoke] FAIL — smoke reported failures.", file=sys.stderr)
            print(f"[https-smoke] log: {LOG}", file=sys.stderr)
            return 1
        if not pass_lines:
            print(
                "[https-smoke] FAIL — no PASS line observed (timeout or silent abort).",
                file=sys.stderr,
            )
            print(f"[https-smoke] log: {LOG}", file=sys.stderr)
            return 1

        # Pull status off the PASS line for explicit reporting.
        first = pass_lines[0].decode("utf-8", "replace")
        m = re.search(r"http-status=(\d{3})", first)
        if m:
            status = int(m.group(1))
            if not (200 <= status < 400):
                print(
                    f"[https-smoke] FAIL — server returned {status} (expected 2xx/3xx).",
                    file=sys.stderr,
                )
                print(f"[https-smoke] log: {LOG}", file=sys.stderr)
                return 1

        print("[https-smoke] PASS — real HTTPS request/response succeeded.")
        print(f"[https-smoke] log: {LOG}")
        return 0

    except pexpect.TIMEOUT:
        print("[https-smoke] FAIL — timeout during smoke run.", file=sys.stderr)
        print(f"[https-smoke] log: {LOG}", file=sys.stderr)
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
        print(f"[https-smoke] kernel not found after build: {KERNEL}", file=sys.stderr)
        return 2
    return run_smoke()


if __name__ == "__main__":
    sys.exit(main())

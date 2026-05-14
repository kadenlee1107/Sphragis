#!/usr/bin/env python3
"""Followup 3c-nat: drive `nat-selftest` in QEMU.

Boots, authenticates, runs the synthetic-frame NAT classifier test.
Expected: 2 allow / 2 drop-policy / 1 drop-unknown / 1 drop-parse,
2 IP bindings installed.
"""
import pexpect
import re
import socket
import subprocess
import sys
import time
from pathlib import Path
from datetime import datetime

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
LOG = ROOT / f"logs/qemu-tests/nat-selftest-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"sphragis\s*>\s*"

QEMU = [
    "qemu-system-aarch64",
    "-machine", "virt", "-cpu", "max", "-m", "2G",
    "-display", "none",
    "-device", "virtio-gpu-device",
    "-device", "virtio-keyboard-device",
    "-netdev", "user,id=hostnet",
    "-device", "virtio-net-device,netdev=hostnet",
    "-serial", "mon:stdio",
    "-kernel", str(KERNEL),
]

def main():
    # A throwaway daemon keeps boot-time deadman arm from hanging.
    daemon = subprocess.Popen(
        ["python3", str(ROOT / "scripts" / "batcaved.py")],
        stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT,
    )
    for _ in range(40):
        try:
            socket.create_connection(("127.0.0.1", 9999), timeout=0.3).close()
            break
        except OSError: time.sleep(0.2)

    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU[0], QEMU[1:], timeout=90, logfile=fp, encoding=None)
    verdict = "FAIL"
    try:
        c.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
        time.sleep(0.3); c.sendline(b"sphragis-dev")
        c.expect(PROMPT, timeout=30)

        c.sendline(b"nat-selftest")
        c.expect([b"PASS", b"FAIL"], timeout=15)
        verdict = c.match.group(0).decode()
        try:
            c.expect(PROMPT, timeout=5)
        except pexpect.TIMEOUT: pass

        with open(LOG, "rb") as f:
            raw = f.read().decode("utf-8", "replace")
        if "NAT SELF-TEST" in raw:
            chunk = raw[raw.index("NAT SELF-TEST"):]
            # Find the next prompt, then trim to there
            end = chunk.find("sphragis >", 40)
            print("--- nat-selftest output ---")
            print(chunk[: end if end > 0 else 1000])
    except pexpect.TIMEOUT:
        print("[nat] TIMEOUT")
    finally:
        c.terminate(force=True); fp.close()
        daemon.terminate()
        try: daemon.wait(timeout=3)
        except subprocess.TimeoutExpired: daemon.kill()

    print(f"Log: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict == "PASS" else 1

if __name__ == "__main__":
    sys.exit(main())

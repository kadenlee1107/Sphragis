#!/usr/bin/env python3
"""Followup #3a demo: drive cave-policy-selftest in QEMU.

Boots Sphragis, authenticates, runs the new `cave-policy-selftest`
shell command and prints its output. Pass-fail reported based on the
"PASS" or "FAIL" marker in the selftest output.
"""
import pexpect
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
LOG = ROOT / "logs/qemu-tests/cave-policy.log"
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

def main():
    print(f"[cave-policy] kernel: {KERNEL}")
    log_fp = open(LOG, "wb")
    child = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=60,
                          logfile=log_fp, encoding=None)
    try:
        print("[cave-policy] wait for auth gate input loop...")
        child.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
        time.sleep(0.3)
        child.sendline(b"sphragis-dev")

        print("[cave-policy] wait for shell prompt...")
        child.expect(rb"sphragis\s*>\s*", timeout=30)

        # Drive the selftest
        child.sendline(b"cave-policy-selftest")
        child.expect([b"PASS", b"FAIL"], timeout=15)
        verdict = child.match.group(0).decode()
        # Grab a bit more after to print the bookkeeping numbers
        try:
            child.expect(rb"sphragis\s*>\s*", timeout=5)
            child.before.decode("utf-8", "replace")
        except pexpect.TIMEOUT:
            pass

        print(f"[cave-policy] verdict: {verdict}")
        print("--- self-test output ---")
        # Reassemble: the "PASS"/"FAIL" was matched and consumed, so
        # stitch it back together for display.
        banner = "CAVE-POLICY SELF-TEST"
        with open(LOG, "rb") as f:
            blob = f.read().decode("utf-8", "replace")
        if banner in blob:
            chunk = blob[blob.index(banner):]
            print(chunk[:1200])
        return 0 if verdict == "PASS" else 1
    except pexpect.TIMEOUT:
        print("[cave-policy] TIMEOUT")
        log_fp.flush()
        with open(LOG, "rb") as f:
            f.seek(0, 2)
            size = f.tell()
            f.seek(max(0, size - 2000))
            print(f.read().decode("utf-8", "replace"))
        return 2
    finally:
        child.terminate(force=True)
        log_fp.close()
        print(f"[cave-policy] log: {LOG}")

if __name__ == "__main__":
    sys.exit(main())

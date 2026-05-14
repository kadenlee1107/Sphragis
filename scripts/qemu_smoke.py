#!/usr/bin/env python3
"""Sphragis QEMU smoke test — boot + auth + couple of shell commands.

Quick sanity check that the binary boots, auth accepts 'sphragis-dev',
and we reach the shell. Full test suite is qemu_test_suite.py."""
import pexpect
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
LOG = ROOT / "logs/qemu-tests/smoke.log"
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
    print(f"[smoke] launching {KERNEL}")
    log_fp = open(LOG, "wb")
    child = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=30,
                          logfile=log_fp, encoding=None)
    try:
        # Wait for boot to complete + auth prompt
        # Look for the auth banner or passphrase prompt
        print("[smoke] waiting for auth prompt...")
        child.expect([b"Passphrase:", b"PASSPHRASE", b"AUTH PASSED", b"\\$"], timeout=60)
        hit = child.match.group(0) if child.match else b"?"
        print(f"[smoke] got: {hit}")

        # Send passphrase
        time.sleep(1)
        print("[smoke] sending 'sphragis-dev'")
        child.sendline(b"sphragis-dev")

        # Wait for desktop or shell prompt
        print("[smoke] waiting for shell prompt...")
        child.expect([b"sphragis>", b"bat:/>", b"\\$ "], timeout=30)
        print(f"[smoke] shell up, got: {child.before[-100:]}")

        # Try a couple of commands
        for cmd in [b"help", b"uname", b"status", b"mem"]:
            print(f"[smoke] -> {cmd.decode()}")
            child.sendline(cmd)
            try:
                child.expect([b"sphragis>", b"bat:/>", b"\\$ "], timeout=10)
                out = child.before.decode("utf-8", "replace")
                # Trim to last 500 chars for display
                print("    " + out[-500:].replace("\n", "\n    "))
            except pexpect.TIMEOUT:
                print("    [timeout]")
                break

        print("[smoke] done")
    except pexpect.TIMEOUT:
        print("[smoke] TIMEOUT")
        # Dump the end of the log
        log_fp.flush()
        print("--- last 2000 bytes of log ---")
        with open(LOG, "rb") as f:
            f.seek(0, 2)
            size = f.tell()
            f.seek(max(0, size - 2000))
            print(f.read().decode("utf-8", "replace"))
    finally:
        child.terminate(force=True)
        log_fp.close()
        print(f"[smoke] full log: {LOG}")

if __name__ == "__main__":
    main()

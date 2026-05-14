#!/usr/bin/env python3
"""STUMP #159: AES-128-GCM + AES-256-GCM NIST vector check on QEMU.

Boots Sphragis, auths via the splash gate, runs `gcm-selftest`. PASS
if both NIST vectors (Test Case 2 + Test Case 14) reproduce their
published tags AND the tamper-detection rejects flipped ciphertext.
"""
import pexpect
import sys
import time
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
LOG = ROOT / f"logs/qemu-tests/gcm-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)
PROMPT = rb"sphragis\s*>\s*"

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
    print(f"[gcm] launching {KERNEL}")
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:], timeout=90,
                      logfile=fp, encoding=None)
    verdict = "FAIL"
    try:
        c.expect(rb"\[bs\] paint done .+ input loop", timeout=60)
        time.sleep(0.3); c.sendline(b"sphragis-dev")
        c.expect(PROMPT, timeout=30)
        c.sendline(b"gcm-selftest")
        idx = c.expect([
            b"PASS  both ciphers reproduce",
            b"FAIL:",
        ], timeout=20)
        if idx == 0:
            verdict = "PASS"
        try: c.expect(PROMPT, timeout=5)
        except pexpect.TIMEOUT: pass
        with open(LOG, "rb") as f:
            raw = f.read().decode("utf-8", "replace")
        marker = "AES-GCM KNOWN-ANSWER"
        i = raw.find(marker)
        if i >= 0:
            tail = raw[i:]
            # Stop at the next prompt to keep output focused.
            j = tail.find("sphragis >", len(marker))
            print(tail[: j if j > 0 else 1500])
    except pexpect.TIMEOUT:
        print("[gcm] TIMEOUT")
        fp.flush()
        with open(LOG, "rb") as f:
            f.seek(0, 2); size = f.tell()
            f.seek(max(0, size - 1500))
            print("--- last 1500 bytes ---")
            print(f.read().decode("utf-8", "replace"))
    finally:
        c.terminate(force=True)
        fp.close()
    print(f"Log: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict == "PASS" else 1

if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""virtio-blk round-trip test.

Creates a tiny raw disk image, attaches it to QEMU as virtio-blk,
boots Bat_OS, drives `blk-selftest` which writes a pattern to
sector 42 then reads it back.
"""
import os, pexpect, re, socket, subprocess, sys, tempfile, time
from pathlib import Path
from datetime import datetime

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
LOG = ROOT / f"logs/qemu-tests/blk-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)
PROMPT = rb"bat_os\s*>\s*"

def main():
    # Create a 4 MiB raw disk image (enough for sector 42 + headroom).
    tf = tempfile.NamedTemporaryFile(prefix="batos-blk-", suffix=".img",
                                     delete=False)
    tf.close()
    img = tf.name
    with open(img, "wb") as f:
        f.write(b"\x00" * (4 * 1024 * 1024))

    daemon = subprocess.Popen(["python3", str(ROOT / "scripts" / "batcaved.py")],
        stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT)
    for _ in range(40):
        try: socket.create_connection(("127.0.0.1", 9999), timeout=0.3).close(); break
        except OSError: time.sleep(0.2)

    args = ["qemu-system-aarch64", "-machine", "virt", "-cpu", "max", "-m", "2G",
            "-display", "none",
            "-device", "virtio-gpu-device", "-device", "virtio-keyboard-device",
            "-netdev", "user,id=net0", "-device", "virtio-net-device,netdev=net0",
            # virtio-blk backed by our temp image.
            "-drive", f"file={img},if=none,format=raw,id=batosdisk",
            "-device", "virtio-blk-device,drive=batosdisk",
            "-serial", "mon:stdio", "-kernel", str(KERNEL)]
    fp = open(LOG, "wb")
    c = pexpect.spawn(args[0], args[1:], timeout=90, logfile=fp, encoding=None)
    verdict = "FAIL"
    try:
        c.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
        time.sleep(0.3); c.sendline(b"batman")
        c.expect(PROMPT, timeout=30)
        c.sendline(b"blk-selftest")
        c.expect([b"PASS", b"FAIL", b"skipping"], timeout=15)
        hit = c.match.group(0).decode()
        if hit == "PASS":
            verdict = "PASS"
        elif hit == "skipping":
            verdict = "NO-BLK"
        try: c.expect(PROMPT, timeout=5)
        except pexpect.TIMEOUT: pass
        with open(LOG, "rb") as f:
            raw = f.read().decode("utf-8", "replace")
        idx = raw.find("BLK SELF-TEST")
        if idx >= 0:
            chunk = raw[idx:]
            end = chunk.find("bat_os >", 40)
            print(chunk[: end if end > 0 else 1200])
    except pexpect.TIMEOUT:
        print("[blk] TIMEOUT")
    finally:
        c.terminate(force=True); fp.close()
        daemon.terminate()
        try: daemon.wait(timeout=3)
        except subprocess.TimeoutExpired: daemon.kill()
        try: os.unlink(img)
        except Exception: pass
    print(f"Log: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict == "PASS" else 1

if __name__ == "__main__":
    sys.exit(main())

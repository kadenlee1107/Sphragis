#!/usr/bin/env python3
"""Followup 3c-daemon-bind: kernel pulls IP bindings from daemon.

1. Start batcaved, push two fake container bindings:
     CPOL_BIND_SET 192.168.77.10 kali
     CPOL_BIND_SET 192.168.77.11 alpine
2. Boot Sphragis.
3. In shell: `nat-sync` (daemon → kernel).
4. `nat-bindings` must list both with correct cave names.
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
LOG = ROOT / f"logs/qemu-tests/nat-sync-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"sphragis\s*>\s*"

def run_cmd(c, cmd, timeout=10):
    c.sendline(cmd.encode())
    c.expect(PROMPT, timeout=timeout)
    return ANSI.sub(b"", c.before or b"").decode("utf-8", "replace")

def push_bindings(port=9999):
    """AUTH + push two bindings, then disconnect."""
    s = socket.create_connection(("127.0.0.1", port), timeout=3)
    s.sendall(b"AUTH SPHRAGIS-DEV-2026\n")
    s.settimeout(3)
    # Read 1 line
    buf = b""
    while b"\n" not in buf:
        c = s.recv(256); buf += c
    for cmd in [b"CPOL_BIND_SET 192.168.77.10 kali\n",
                b"CPOL_BIND_SET 192.168.77.11 alpine\n"]:
        s.sendall(cmd)
        buf = b""
        while b"\n" not in buf: buf += s.recv(256)
    s.sendall(b"QUIT\n"); s.close()

def main():
    daemon = subprocess.Popen(
        ["python3", str(ROOT / "scripts" / "batcaved.py")],
        stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT,
    )
    for _ in range(40):
        try: socket.create_connection(("127.0.0.1", 9999), timeout=0.3).close(); break
        except OSError: time.sleep(0.2)
    push_bindings()

    args = [
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
    fp = open(LOG, "wb")
    c = pexpect.spawn(args[0], args[1:], timeout=90, logfile=fp, encoding=None)
    verdict = "FAIL"; details = []
    try:
        c.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
        time.sleep(0.3); c.sendline(b"sphragis-dev")
        c.expect(PROMPT, timeout=30)

        out = run_cmd(c, "nat-sync")
        details.append(out.strip())
        bindings = run_cmd(c, "nat-bindings")
        details.append(bindings.strip())

        ok1 = "192.168.77.10" in bindings and "kali" in bindings
        ok2 = "192.168.77.11" in bindings and "alpine" in bindings
        if ok1 and ok2: verdict = "PASS"
    except pexpect.TIMEOUT:
        details.append("TIMEOUT")
    finally:
        c.terminate(force=True); fp.close()
        daemon.terminate()
        try: daemon.wait(timeout=3)
        except subprocess.TimeoutExpired: daemon.kill()

    print("--- output ---")
    for d in details:
        for line in d.splitlines():
            print("  " + line[:160])
    print(f"\nLog: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict == "PASS" else 1

if __name__ == "__main__":
    sys.exit(main())

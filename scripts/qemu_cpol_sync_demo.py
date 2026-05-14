#!/usr/bin/env python3
"""Followup 3b-sync: end-to-end kernel ↔ daemon cave_policy sync.

Flow:
  1. Start batcaved in subprocess (port 9999, proxy 9998).
  2. Boot Sphragis in QEMU.
  3. Authenticate and reach shell.
  4. cpol-add      kali github.com 443 tcp
  5. cpol-add      kali api.anthropic.com 443 tcp
  6. cpol-sync     kali            — pushes to daemon mirror
  7. cpol-daemon-list              — daemon reports ['kali']
  8. cpol-daemon-show kali         — daemon reports both rules
  9. cpol-clear    kali
 10. cpol-sync     kali            — empty push (clear on daemon)
 11. cpol-daemon-list              — daemon reports empty

Exit 0 iff every step's expected substring appears in its output.
"""
import pexpect
import re
import subprocess
import sys
import time
from pathlib import Path
from datetime import datetime

ROOT     = Path(__file__).resolve().parent.parent
KERNEL   = ROOT / "target/aarch64-unknown-none/release/sphragis"
BATCAVED = ROOT / "scripts" / "batcaved.py"
LOG_DIR  = ROOT / "logs/qemu-tests"
LOG_DIR.mkdir(parents=True, exist_ok=True)
STAMP    = datetime.now().strftime('%Y%m%d-%H%M%S')
QEMU_LOG = LOG_DIR / f"cpol-sync-qemu-{STAMP}.log"
DAEMON_LOG = LOG_DIR / f"cpol-sync-daemon-{STAMP}.log"

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"sphragis\s*>\s*"

QEMU = [
    "qemu-system-aarch64",
    "-machine", "virt", "-cpu", "max", "-m", "2G",
    "-display", "none",
    "-device", "virtio-gpu-device",
    "-device", "virtio-keyboard-device",
    "-netdev", "user,id=net0",
    "-device", "virtio-net-device,netdev=net0",
    "-serial", "mon:stdio",
    "-kernel", str(KERNEL),
]

def run_cmd(c, cmd, timeout=30):
    c.sendline(cmd.encode())
    c.expect(PROMPT, timeout=timeout)
    raw = ANSI.sub(b"", c.before or b"").decode("utf-8", "replace")
    return "\n".join(l.rstrip() for l in raw.splitlines())

def main():
    # 1. Start the daemon.
    print(f"[sync] starting batcaved; log={DAEMON_LOG}")
    daemon_log = open(DAEMON_LOG, "wb")
    daemon = subprocess.Popen(
        ["python3", str(BATCAVED)],
        stdout=daemon_log, stderr=subprocess.STDOUT,
    )
    # Wait until it binds port 9999.
    import socket
    for _ in range(50):
        try:
            s = socket.create_connection(("127.0.0.1", 9999), timeout=0.3)
            s.close(); break
        except OSError:
            time.sleep(0.2)
    else:
        print("[sync] batcaved failed to start"); daemon.terminate(); return 2

    fp = open(QEMU_LOG, "wb")
    c = pexpect.spawn(QEMU[0], QEMU[1:], timeout=120, logfile=fp, encoding=None)
    failures = 0
    try:
        c.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
        time.sleep(0.3); c.sendline(b"batman")
        c.expect(PROMPT, timeout=30)
        print("[sync] shell ready\n")

        steps = [
            ("cpol-add    kali github.com 443 tcp",         "OK"),
            ("cpol-add    kali api.anthropic.com 443 tcp",  "OK"),
            ("cpol-show   kali",                            "github.com"),
            ("cpol-sync   kali",                            "2 rules pushed"),
            ("cpol-daemon-list",                            "kali"),
            ("cpol-daemon-show kali",                       "api.anthropic.com"),
            ("cpol-clear  kali",                            "OK"),
            ("cpol-sync   kali",                            "0 rules pushed"),
            ("cpol-daemon-list",                            "empty"),
        ]
        for cmd, expect in steps:
            print(f"sphragis > {cmd}")
            out = run_cmd(c, cmd, timeout=30)
            for line in out.splitlines():
                print(f"   {line[:120]}")
            if expect not in out:
                print(f"   ✗ expected '{expect}' in output — FAIL")
                failures += 1
            print()

    except pexpect.TIMEOUT:
        print("[sync] QEMU TIMEOUT")
        failures += 1
    finally:
        try: c.terminate(force=True)
        except Exception: pass
        fp.close()
        daemon.terminate()
        try: daemon.wait(timeout=3)
        except subprocess.TimeoutExpired: daemon.kill()
        daemon_log.close()

    print(f"QEMU log: {QEMU_LOG}")
    print(f"Daemon log: {DAEMON_LOG}")
    total = 9
    print(f"Result: {total-failures}/{total} steps OK"
          + (" — PASS" if failures == 0 else f" — FAIL ({failures} issues)"))
    return 0 if failures == 0 else 1

if __name__ == "__main__":
    sys.exit(main())

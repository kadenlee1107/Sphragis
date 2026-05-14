#!/usr/bin/env python3
"""Followup #3b demo: drive the cpol- shell commands end-to-end.

Boots Sphragis, authenticates, then runs:
  cpol-list           (empty)
  cpol-add    kali github.com        443 tcp
  cpol-add    kali api.anthropic.com 443 tcp
  cpol-show   kali                              (expect 2 rules)
  cpol-check  kali github.com        443 tcp   (expect ALLOW)
  cpol-check  kali httpbin.org       443 tcp   (expect DROP)
  cpol-clear  kali
  cpol-check  kali github.com        443 tcp   (expect DROP  — after clear)
  cpol-list                                     (empty again)

Each step's raw output is echoed. Exit code 0 iff every expected
ALLOW/DROP matches, nonzero otherwise.
"""
import pexpect
import re
import sys
import time
from pathlib import Path
from datetime import datetime

ROOT   = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
LOG    = ROOT / f"logs/qemu-tests/cpol-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)

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

def run_cmd(c, cmd: str, timeout=10) -> str:
    """Send cmd, return the raw post-command output up to the next prompt.

    We keep the raw text (no dedup) so that substring matches for
    `ALLOW` / `DROP` / `OK` hit real kernel output and aren't thrown
    off by the doubled-letter terminal echo. Readability suffers
    slightly but correctness wins.
    """
    c.sendline(cmd.encode())
    c.expect(PROMPT, timeout=timeout)
    raw = ANSI.sub(b"", c.before or b"").decode("utf-8", "replace")
    lines = [l.rstrip() for l in raw.splitlines()]
    out = [l for l in lines if l and
           not l.strip().startswith(("[docker]", "[tcp]", "sphragis >"))]
    return "\n".join(out)

def main():
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU[0], QEMU[1:], timeout=120, logfile=fp, encoding=None)

    c.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
    time.sleep(0.3); c.sendline(b"sphragis-dev")
    c.expect(PROMPT, timeout=30)
    print("[cpol] shell ready\n")

    steps = [
        ("cpol-list",                                    None,     "empty before we add anything"),
        ("cpol-add    kali github.com 443 tcp",          "OK",     "add rule 1"),
        ("cpol-add    kali api.anthropic.com 443 tcp",   "OK",     "add rule 2"),
        ("cpol-show   kali",                             "443",    "show lists our 443 entries"),
        ("cpol-check  kali github.com 443 tcp",          "ALLOW",  "github should allow"),
        ("cpol-check  kali httpbin.org 443 tcp",         "DROP",   "httpbin should drop"),
        ("cpol-clear  kali",                             "OK",     "clear cave"),
        ("cpol-check  kali github.com 443 tcp",          "DROP",   "after clear, github should drop"),
        ("cpol-list",                                    None,     "empty again after clear"),
    ]

    failures = 0
    for cmd, expect, note in steps:
        print(f"sphragis > {cmd}    # {note}")
        out = run_cmd(c, cmd, timeout=10)
        for line in out.splitlines():
            print(f"   {line[:120]}")
        if expect is not None and expect not in out:
            print(f"   ✗ expected '{expect}' in output — FAIL")
            failures += 1
        print()

    c.terminate(force=True); fp.close()
    print(f"Log: {LOG}")
    print(f"Result: {len(steps)-failures}/{len(steps)} steps OK"
          + (" — PASS" if failures == 0 else f" — FAIL ({failures} issues)"))
    return 0 if failures == 0 else 1

if __name__ == "__main__":
    sys.exit(main())

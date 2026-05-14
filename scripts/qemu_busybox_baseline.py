#!/usr/bin/env python3
"""Baseline: does `batcave run <applet>` actually work today?"""
import pexpect
import re
import time
from pathlib import Path
from datetime import datetime

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
LOG = ROOT / f"logs/qemu-tests/busybox-baseline-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"sphragis\s*>\s*"
QEMU = ["qemu-system-aarch64","-machine","virt","-cpu","max","-m","2G","-display","none",
    "-device","virtio-gpu-device","-device","virtio-keyboard-device",
    "-netdev","user,id=net0","-device","virtio-net-device,netdev=net0",
    "-serial","mon:stdio","-kernel",str(KERNEL)]

def main():
    fp = open(LOG, "wb")
    c = pexpect.spawn(QEMU[0], QEMU[1:], timeout=90, logfile=fp, encoding=None)
    c.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
    time.sleep(0.3); c.sendline(b"batman")
    c.expect(PROMPT, timeout=30)
    print("[baseline] shell up. Trying busybox applets...\n")

    for cmd in ["batcave run uname", "batcave run echo hello-from-cave",
                "batcave run nslookup", "batcave run nc"]:
        print(f"$ {cmd}")
        c.sendline(cmd.encode())
        try:
            c.expect(PROMPT, timeout=30)
            out = ANSI.sub(b"", c.before).decode("utf-8", "replace").strip()
            for line in out.splitlines()[-10:]:
                line = line.strip()
                if line: print(f"  {line[:100]}")
        except pexpect.TIMEOUT:
            print("  [TIMEOUT]")
            break
        print()
    c.terminate(force=True); fp.close()
    print(f"Log: {LOG}")

if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Phase 5/6 — PQ hybrid self-test live from Bat_OS shell."""
import pexpect, re, time
from pathlib import Path
from datetime import datetime

ROOT   = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
LOG    = ROOT / f"logs/qemu-tests/pq-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"bat_os\s*>\s*"

def main():
    fp = open(LOG, "wb")
    c = pexpect.spawn("qemu-system-aarch64", [
        "-machine", "virt", "-cpu", "max", "-m", "2G",
        "-display", "none",
        "-device", "virtio-gpu-device", "-device", "virtio-keyboard-device",
        "-netdev", "user,id=net0", "-device", "virtio-net-device,netdev=net0",
        "-serial", "mon:stdio", "-kernel", str(KERNEL),
    ], timeout=120, logfile=fp, encoding=None)

    c.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
    time.sleep(0.3); c.sendline(b"batman")
    c.expect(PROMPT, timeout=30)
    print("[qemu] shell ready\n")

    print("bat_os > pq-selftest")
    c.sendline(b"pq-selftest")
    c.expect(PROMPT, timeout=30)
    out = ANSI.sub(b"", c.before or b"").decode("utf-8", "replace")
    for line in out.splitlines():
        s = line.rstrip()
        if not s.strip(): continue
        if s.strip().startswith(("[docker]", "[tcp]", "bat_os >", "pq-selftest", "[shell]")): continue
        # Dedup doubled-letter echo cosmetic
        d = sum(1 for i in range(0, len(s)-1, 2) if s[i].isalpha() and s[i] == s[i+1])
        if d*2 > len(s)*0.5:
            out_buf = []
            i = 0
            while i < len(s):
                if i+1 < len(s) and s[i] == s[i+1] and s[i].isalpha():
                    out_buf.append(s[i]); i += 2
                else:
                    out_buf.append(s[i]); i += 1
            s = "".join(out_buf)
        print(f"   {s[:120]}")
    c.terminate(force=True); fp.close()
    print(f"\nLog: {LOG}")

if __name__ == "__main__":
    main()

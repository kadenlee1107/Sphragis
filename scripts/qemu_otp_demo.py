#!/usr/bin/env python3
"""DESIGN_CRYPTO.md #11/#12 — live OTP duress + deadman demo.

Exercises:
  1. otp-dump          — show the 32-token pad (this is the provisioning
                          step the operator runs offline)
  2. otp-stats         — 8 duress + 24 deadman = 32 tokens
  3. otp-consume <tok> — consume a deadman token, verify stats decrement
  4. otp-consume <dur> — consume a duress token, verify WIPE fires
"""
import pexpect
import re
import time
from pathlib import Path
from datetime import datetime

ROOT   = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
LOG    = ROOT / f"logs/qemu-tests/otp-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"bat_os\s*>\s*"


def dedup(s):
    if not s: return s
    d = sum(1 for i in range(0, len(s)-1, 2) if s[i].isalpha() and s[i] == s[i+1])
    if d*2 < len(s)*0.5: return s
    out, i = [], 0
    while i < len(s):
        if i+1 < len(s) and s[i] == s[i+1] and s[i].isalpha():
            out.append(s[i]); i += 2
        else:
            out.append(s[i]); i += 1
    return "".join(out)


def main():
    fp = open(LOG, "wb")
    child = pexpect.spawn("qemu-system-aarch64", [
        "-machine", "virt", "-cpu", "max", "-m", "2G",
        "-display", "none",
        "-device", "virtio-gpu-device",
        "-device", "virtio-keyboard-device",
        "-netdev", "user,id=net0",
        "-device", "virtio-net-device,netdev=net0",
        "-serial", "mon:stdio",
        "-kernel", str(KERNEL),
    ], timeout=120, logfile=fp, encoding=None)

    child.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
    time.sleep(0.3); child.sendline(b"batman")
    child.expect(PROMPT, timeout=30)
    print("[qemu] shell ready\n")

    def run(cmd, wait=10):
        print(f"bat_os > {cmd}")
        child.sendline(cmd.encode())
        try:
            child.expect(PROMPT, timeout=wait)
        except pexpect.TIMEOUT:
            print("   [TIMEOUT]")
            return ""
        out = ANSI.sub(b"", child.before or b"").decode("utf-8", "replace")
        cleaned = []
        for line in out.splitlines():
            s = dedup(line.rstrip())
            if not s or not s.strip(): continue
            if s.strip().startswith(("[docker]", "[tcp]", "bat_os >", cmd)):
                continue
            cleaned.append(s)
        for l in cleaned[-25:]:
            print(f"   {l[:140]}")
        print()
        return "\n".join(cleaned)

    print("=" * 74)
    print(" OTP DUMP — full pad")
    print("=" * 74)
    dump_out = run("otp-dump", wait=5)

    # Extract a deadman token (any slot >= 8) + a duress token (slot < 8)
    deadman_tok = None
    duress_tok = None
    for line in dump_out.splitlines():
        m = re.search(r"\[\s*(\d+)\].*?(deadman|duress)\s+([0-9a-f]{64})", line)
        if m:
            slot, region, tok = int(m.group(1)), m.group(2), m.group(3)
            if region == "deadman" and deadman_tok is None:
                deadman_tok = tok
                print(f"  [harvested deadman token from slot {slot}]")
            elif region == "duress" and duress_tok is None:
                duress_tok = tok
                print(f"  [harvested duress token from slot {slot}]")
        if deadman_tok and duress_tok: break

    if not (deadman_tok and duress_tok):
        print("  FAIL: could not extract tokens from dump")
        child.terminate(force=True); fp.close(); return 1

    print()
    print("=" * 74)
    print(" STATS — baseline")
    print("=" * 74)
    run("otp-stats", wait=5)

    print()
    print("=" * 74)
    print(" CONSUME deadman token — expect accept + refresh")
    print("=" * 74)
    run(f"otp-consume {deadman_tok}", wait=10)

    print()
    print("=" * 74)
    print(" STATS — deadman count should drop by 1")
    print("=" * 74)
    run("otp-stats", wait=5)

    print()
    print("=" * 74)
    print(" REPLAY SAME TOKEN — expect rejection (single-use)")
    print("=" * 74)
    run(f"otp-consume {deadman_tok}", wait=10)

    print()
    print("=" * 74)
    print(" CONSUME duress token — expect wipe")
    print("=" * 74)
    child.sendline(f"otp-consume {duress_tok}".encode())
    try:
        child.expect(rb"WIPE COMPLETE|DURESS|wiping now", timeout=30)
        print("   [WIPE FIRED]")
        # Slurp a bit more of the post-wipe output
        try:
            child.expect(PROMPT, timeout=5)
        except pexpect.TIMEOUT:
            pass
        out = ANSI.sub(b"", child.before or b"").decode("utf-8", "replace")
        for line in out.splitlines()[-20:]:
            s = dedup(line.rstrip())
            if s.strip(): print(f"   {s[:140]}")
    except pexpect.TIMEOUT:
        print("   [TIMEOUT — duress didn't fire]")

    child.terminate(force=True); fp.close()
    print()
    print(f"Log: {LOG}")


if __name__ == "__main__":
    main()

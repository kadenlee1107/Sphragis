#!/usr/bin/env python3
"""Extra QEMU checks: `screen` capture, `clear`, `browse`, interactive quirks."""
import pexpect, re, time
from pathlib import Path
from datetime import datetime

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
LOG_DIR = ROOT / "logs/qemu-tests"; LOG_DIR.mkdir(parents=True, exist_ok=True)
STAMP = datetime.now().strftime("%Y%m%d-%H%M%S")
LOG = LOG_DIR / f"extras-{STAMP}.log"

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
QEMU_ARGS = ["qemu-system-aarch64","-machine","virt","-cpu","max","-m","2G",
    "-display","none","-device","virtio-gpu-device","-device","virtio-keyboard-device",
    "-netdev","user,id=net0","-device","virtio-net-device,netdev=net0",
    "-serial","mon:stdio","-kernel",str(KERNEL)]
PROMPT = rb"bat_os\s*>\s*"

def clean(b): return ANSI.sub(b"", b or b"").decode("utf-8","replace").strip()

def run_cmd(child, cmd, timeout=10):
    child.sendline(cmd.encode())
    try:
        child.expect(PROMPT, timeout=timeout)
        return clean(child.before), False
    except pexpect.TIMEOUT:
        return clean(child.before), True

def main():
    print(f"[extras] log = {LOG}")
    log_fp = open(LOG, "wb")
    child = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:],
                          timeout=90, logfile=log_fp, encoding=None)

    child.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
    time.sleep(0.3); child.sendline(b"batman")
    child.expect(PROMPT, timeout=30)
    print("[extras] shell ready\n")

    extras = [
        ("clear",        "clear screen"),
        ("browse http://example.com", "open in BatBrowser"),
        ("fw",           "firewall"),
        ("panic",        "test panic (should halt)"),  # last
    ]
    for cmd, desc in extras:
        out, timed = run_cmd(child, cmd, timeout=10)
        status = "HANG" if timed else ("CRASH" if "abort" in out.lower() else "OK")
        print(f"[{status:7s}] {cmd:<32} — {out[-100:]}")
        if status == "CRASH" or cmd == "panic":
            break

    # Also test Ctrl+A (switch to shell) from desktop
    print("\n[extras] testing Ctrl+A hotkey to switch panes")
    child.send(b"\x01")
    time.sleep(0.3)
    try:
        recent = child.read_nonblocking(size=4096, timeout=0.5)
        print(f"  after Ctrl+A: {clean(recent)[:80]}")
    except: pass

    child.terminate(force=True); log_fp.close()
    print(f"\n[extras] full log: {LOG}")

if __name__ == "__main__":
    main()

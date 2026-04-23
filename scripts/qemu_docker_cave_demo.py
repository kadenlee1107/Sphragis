#!/usr/bin/env python3
"""
Design-aligned BatCave-over-Docker demo (Phases 1+2).

Bat_OS (inside QEMU) issues `batcave docker-*` commands. Those travel
as a TCP connection to 10.0.2.2:9999 (QEMU slirp host alias), where
the Mac-side `batcaved` daemon is listening. The daemon translates to
Docker operations and streams output back.

This proves the control-plane loop: Bat_OS's shell → microkernel TCP
stack → daemon → Docker → container stdout → daemon → Bat_OS → user.

Phases 3 (encrypted rootfs), 4 (net pipeline), 5 (deadman), 7 (seal)
are scaffolded but not yet implemented.
"""
import pexpect, re, time, subprocess, threading, atexit
from pathlib import Path
from datetime import datetime

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
LOG_DIR = ROOT / "logs/qemu-tests"; LOG_DIR.mkdir(parents=True, exist_ok=True)
STAMP = datetime.now().strftime("%Y%m%d-%H%M%S")
RUN_LOG = LOG_DIR / f"docker-cave-{STAMP}.log"

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"bat_os\s*>\s*"


def dedup_echo_line(s):
    if not s: return s
    doubled = sum(1 for i in range(0, len(s)-1, 2)
                  if s[i].isalpha() and s[i] == s[i+1])
    if doubled * 2 < len(s) * 0.5: return s
    out, i = [], 0
    while i < len(s):
        if i+1 < len(s) and s[i] == s[i+1] and s[i].isalpha():
            out.append(s[i]); i += 2
        else:
            out.append(s[i]); i += 1
    return "".join(out)


def main():
    # ── Start the daemon ────────────────────────────────────
    print("=" * 76)
    print(" DESIGN-ALIGNED BatCave-over-Docker — live demo (phases 1+2)")
    print("=" * 76)

    # Kill any existing daemon on port 9999
    subprocess.run(["pkill", "-f", "batcaved.py"], capture_output=True)
    time.sleep(0.5)

    daemon_log = LOG_DIR / f"batcaved-{STAMP}.log"
    daemon = subprocess.Popen(
        ["python3", str(ROOT / "scripts/batcaved.py"), "--port", "9999"],
        stdout=open(daemon_log, "w"), stderr=subprocess.STDOUT,
    )
    atexit.register(lambda: daemon.terminate())
    print(f"[mac]  batcaved started (pid={daemon.pid}, log={daemon_log.name})")
    time.sleep(0.5)

    # Verify daemon is up
    try:
        r = subprocess.run(
            ["bash", "-c",
             "printf 'AUTH BATMAN-DEV-2026\\nPING\\nQUIT\\n' | nc -w 2 127.0.0.1 9999"],
            capture_output=True, text=True, timeout=5)
        print(f"[mac]  daemon self-test: {r.stdout.strip().splitlines()}")
    except Exception as e:
        print(f"[mac]  daemon not reachable: {e}")
        return 1

    # ── Boot Bat_OS ─────────────────────────────────────────
    print()
    print("[qemu] booting Bat_OS...")
    log_fp = open(RUN_LOG, "wb")
    qemu = [
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
    child = pexpect.spawn(qemu[0], qemu[1:], timeout=120,
                          logfile=log_fp, encoding=None)
    child.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
    time.sleep(0.3)
    child.sendline(b"batman")
    child.expect(PROMPT, timeout=30)
    print("[qemu] shell ready")

    def run_cmd(cmd, wait=60):
        print()
        print(f"bat_os > {cmd}")
        child.sendline(cmd.encode())
        try:
            child.expect(PROMPT, timeout=wait)
        except pexpect.TIMEOUT:
            print("   [TIMEOUT]")
            return
        out = ANSI.sub(b"", child.before or b"").decode("utf-8", "replace")
        # Skip kernel chatter + our own echo
        skip_prefixes = (
            "[mmu]", "[loader]", "[reloc]", "[runner]", "[shell]",
            "[batcave", "[dms]", "[vfs]", "[sec", "[kbd]", "[gpu]",
            "[fs]", "[firewall]", "[net]", "[boot]", "[chromium",
            "[bs]", "[auth]", "[ipc]", "[arch]", "[rng]", "[sched]",
            "[mm]", "[security]", "[initrd]", "[dtb]", "[mmap]",
            "[docker]", "bat_os >", "Microkernel", "Ctrl+", cmd,
        )
        for line in out.splitlines():
            s = dedup_echo_line(line.rstrip())
            if not s or not s.strip(): continue
            if any(s.lstrip().startswith(p) for p in skip_prefixes):
                continue
            if set(s.strip()) <= set(" _|/\\()"): continue
            print(f"   {s[:120]}")

    # ── Scenario ────────────────────────────────────────────
    print()
    print("=" * 76)
    print(" SCENARIO: Bat_OS shell commands drive real Docker caves")
    print("=" * 76)

    # 1. Quick daemon connectivity check from inside Bat_OS
    run_cmd("batcave docker-ping", wait=15)

    # 2. Create a Kali cave from Bat_OS shell
    run_cmd("batcave docker-create kali kalilinux/kali-rolling NET_RAW,NET_ADMIN",
            wait=30)

    # 3. List — should show kali cave
    run_cmd("batcave docker-list", wait=15)

    # 4. Run uname inside the cave via Bat_OS
    run_cmd("batcave docker-run kali uname -a", wait=15)

    # 5. Run cat /etc/os-release — prove it IS Kali
    run_cmd("batcave docker-run kali cat /etc/os-release", wait=15)

    # 6. Install nmap via the daemon (apt-get inside the cave)
    print()
    print("[ direct-daemon: install nmap inside kali cave — takes ~15s ]")
    subprocess.run([
        "bash", "-c",
        "printf 'AUTH BATMAN-DEV-2026\\nRUN kali apt-get update -qq\\n"
        "RUN kali apt-get install -y --no-install-recommends nmap\\nQUIT\\n' "
        "| nc -w 60 127.0.0.1 9999 | tail -6"
    ], timeout=120)

    # 7. Now scan the Mac HTTP server via nmap from inside the Kali cave,
    #    driven from Bat_OS:
    run_cmd("batcave docker-run kali nmap -sV -Pn -p80 10.0.2.2", wait=60)

    # 8. Destroy the cave from Bat_OS
    run_cmd("batcave docker-destroy kali", wait=15)

    # 9. List after destroy — should be empty
    run_cmd("batcave docker-list", wait=15)

    # ── Teardown ────────────────────────────────────────────
    print()
    child.terminate(force=True)
    log_fp.close()
    daemon.terminate()
    daemon.wait(timeout=5)
    print("=" * 76)
    print(" DONE — Bat_OS shell drove end-to-end Docker BatCave lifecycle")
    print("=" * 76)
    print(f" QEMU log:    {RUN_LOG}")
    print(f" Daemon log:  {daemon_log}")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Sphragis QEMU full-feature test suite.

Two phases:
 1. MAIN — one long-lived QEMU: boot → auth → shell cmds (non-ELF) → desktop-app tab cycle
 2. ELFS — one QEMU per ELF test (they're noreturn so each needs a clean boot)

Both phases skip Chromium per user directive.

Usage:
  SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release   # ensure fresh kernel
  python3 scripts/qemu_test_suite.py
"""
import pexpect
import re
import time
from pathlib import Path
from datetime import datetime

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
LOG_DIR = ROOT / "logs/qemu-tests"
LOG_DIR.mkdir(parents=True, exist_ok=True)

RUN_STAMP = datetime.now().strftime("%Y%m%d-%H%M%S")
MAIN_LOG = LOG_DIR / f"main-{RUN_STAMP}.log"
ELF_LOG_DIR = LOG_DIR / f"elfs-{RUN_STAMP}"
ELF_LOG_DIR.mkdir(parents=True, exist_ok=True)
REPORT = LOG_DIR / f"report-{RUN_STAMP}.md"

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")

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

# ── Shell commands (non-ELF, run in main QEMU) ──
SHELL_TESTS = [
    ("help",       "show command help"),
    ("status",     "security status"),
    ("uname",      "system information"),
    ("whoami",     "current identity"),
    ("uptime",     "system uptime"),
    ("mem",        "memory usage"),
    ("ls",         "list SealFS files (empty)"),
    ("write f1 hello-world",  "create encrypted file"),
    ("ls",         "list SealFS after write"),
    ("cat f1",     "read+decrypt file"),
    ("verify f1",  "check integrity hash"),
    ("rm f1",      "secure delete"),
    ("ls",         "list SealFS post-delete"),
    ("net",        "network interface info"),
    ("fw",         "firewall stats"),
    ("ping 8.8.8.8",   "ping (QEMU user-net ICMP)"),
    ("dns example.com","dns resolve via QEMU user-net"),
    ("caves list",       "list Caves"),
    ("caves create test","create test Cave"),
    ("caves grant test fs","grant fs cap"),
    ("caves grant test mem","grant mem cap"),
    ("caves list",       "list Caves after create"),
    ("caves destroy test","destroy test Cave"),
    ("caves list",       "list Caves post-destroy"),
]

# ── ELF tests (each in its own QEMU, noreturn) ──
ELF_TESTS = [
    ("hello",    "hello world ELF"),
    ("libc",     "hello_libc (libc-linked)"),
    ("threads",  "hello_threads"),
    ("freetype", "freetype test"),
    ("png",      "libpng test"),
    ("posix",    "POSIX suite"),
    ("netsurf",  "NetSurf HTML render"),
    ("v8",       "V8 JavaScript"),
    ("blink",    "Blink HTML test"),
]

# ── Desktop app cycling ──
APP_TESTS = [
    (1, "Dashboard"),
    (2, "Files"),
    (3, "NetMon"),
    (4, "Editor"),
    (5, "Security"),
    (6, "Comms"),
    # (7, "Browser"),   # user says skip — it's chromium host
    (8, "Cave"),
]

PROMPT = rb"sphragis\s*>\s*"
PROMPT_WAIT = 8


def clean(b: bytes) -> str:
    return ANSI.sub(b"", b or b"").decode("utf-8", "replace").strip()


def wait_for_shell(child):
    """Boot → auth → shell prompt."""
    child.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
    time.sleep(0.4)
    child.sendline(b"sphragis-dev")
    child.expect(PROMPT, timeout=30)


def send_cmd(child, cmd, timeout=PROMPT_WAIT):
    child.sendline(cmd.encode())
    try:
        child.expect(PROMPT, timeout=timeout)
        return clean(child.before), False
    except pexpect.TIMEOUT:
        return clean(child.before), True


def run_main_phase():
    print("=" * 68)
    print("PHASE 1: MAIN SUITE (shell + desktop apps in one QEMU)")
    print("=" * 68)

    results = []
    log_fp = open(MAIN_LOG, "wb")
    child = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:],
                          timeout=90, logfile=log_fp, encoding=None)

    try:
        print("[main] booting + authenticating...")
        wait_for_shell(child)
        print("[main] shell prompt reached\n")

        # Shell commands
        for cmd, desc in SHELL_TESTS:
            out, timed = send_cmd(child, cmd)
            if timed:
                status = "HANG"
            else:
                low = out.lower()
                if "unknown command" in low:
                    status = "UNKNOWN"
                elif "panic" in low or "data abort" in low:
                    status = "CRASH"
                elif "blocked" in low and "syscall" in low:
                    status = "BLOCKED"
                elif "error" in low and "integrity" not in low:
                    status = "ERROR"
                else:
                    status = "OK"
            last = [l.strip() for l in out.splitlines() if l.strip()][-3:]
            preview = " | ".join(last)[:100]
            print(f"[{status:7s}] {cmd:<28} — {preview[:90]}")
            results.append(("shell", cmd, desc, status, preview))
            if status == "CRASH":
                break

        # Desktop apps
        print()
        print("-" * 68)
        print("Desktop app tab cycle")
        print("-" * 68)
        child.send(b"\t")  # cycle from shell → next app (Dashboard)
        time.sleep(0.5)
        for i, (app_id, name) in enumerate(APP_TESTS):
            try:
                recent = child.read_nonblocking(size=16384, timeout=0.4)
            except Exception:
                recent = b""
            rec_s = clean(recent)
            low = rec_s.lower()
            if "panic" in low or "abort" in low:
                status = "CRASH"
            elif "[tab]" in rec_s:
                status = "OK"
            else:
                status = "SILENT"  # may have rendered fine, just no serial log
            preview = rec_s.replace("\r", " ").replace("\n", " ")[:100]
            print(f"[{status:7s}] app#{app_id} {name:<12} — {preview[:80]}")
            results.append(("desktop", name, f"cycle to {name}", status, preview))

            # Tab to next app (except after last)
            if i < len(APP_TESTS) - 1:
                child.send(b"\t")
                time.sleep(0.5)

    except pexpect.TIMEOUT:
        print("[main] TIMEOUT — aborting phase 1")
        results.append(("main", "_harness", "harness-timeout", "TIMEOUT", ""))

    child.terminate(force=True)
    log_fp.close()
    return results


def run_elf_phase():
    print()
    print("=" * 68)
    print("PHASE 2: ELFS (one QEMU per ELF, they are noreturn)")
    print("=" * 68)

    results = []
    for name, desc in ELF_TESTS:
        elf_log = ELF_LOG_DIR / f"{name}.log"
        log_fp = open(elf_log, "wb")
        child = pexpect.spawn(QEMU_ARGS[0], QEMU_ARGS[1:],
                              timeout=120, logfile=log_fp, encoding=None)
        status = "?"
        summary = ""
        try:
            wait_for_shell(child)
            child.sendline(name.encode())
            # ELFs: wait for [linux] process exited, OR 45s cap
            try:
                child.expect(rb"\[linux\] process exited with code \d+",
                             timeout=60)
                out = clean(child.before)
                # Capture exit code
                after = child.after.decode("utf-8", "replace")
                low = out.lower()
                blocked_count = out.count("BLOCKED syscall")
                if "panic" in low or "data abort" in low:
                    status = "CRASH"
                    summary = "[panic/abort during run]"
                elif blocked_count > 0:
                    status = "PARTIAL"
                    summary = f"{after.strip()}, {blocked_count} blocked syscalls"
                else:
                    status = "OK"
                    summary = after.strip()
            except pexpect.TIMEOUT:
                out = clean(child.before)
                low = out.lower()
                if "panic" in low or "data abort" in low:
                    status = "CRASH"
                    # grab panic-related context
                    lines = [l for l in out.splitlines() if "panic" in l.lower() or "abort" in l.lower()]
                    summary = " | ".join(lines[:3])[:100]
                else:
                    status = "HANG"
                    summary = "no exit within 60s"
        except pexpect.TIMEOUT:
            status = "BOOT-TIMEOUT"
            summary = "never reached shell"
        except Exception as e:
            status = "ERROR"
            summary = str(e)[:100]

        print(f"[{status:7s}] {name:<12} — {summary[:80]}")
        results.append(("elf", name, desc, status, summary))
        child.terminate(force=True)
        log_fp.close()
    return results


def write_report(main_results, elf_results):
    all_results = main_results + elf_results
    counts = {}
    for row in all_results:
        counts[row[3]] = counts.get(row[3], 0) + 1

    with open(REPORT, "w") as f:
        f.write(f"# Sphragis QEMU test report — {RUN_STAMP}\n\n")
        f.write(f"Kernel: `{KERNEL.relative_to(ROOT)}`  ({KERNEL.stat().st_size:,} bytes)\n")
        f.write(f"Main log: `{MAIN_LOG.relative_to(ROOT)}`\n")
        f.write(f"ELF logs: `{ELF_LOG_DIR.relative_to(ROOT)}/`\n\n")
        f.write("## Summary\n\n")
        for k in sorted(counts.keys()):
            f.write(f"- **{k}**: {counts[k]}\n")
        f.write("\n## Shell commands\n\n")
        f.write("| Cmd | Desc | Status | Preview |\n|---|---|---|---|\n")
        for row in main_results:
            if row[0] == "shell":
                cmd, desc, status, prev = row[1], row[2], row[3], row[4]
                prev = (prev or "").replace("|", "\\|")[:90]
                f.write(f"| `{cmd}` | {desc} | **{status}** | {prev} |\n")
        f.write("\n## Desktop apps\n\n")
        f.write("| App | Status | Preview |\n|---|---|---|\n")
        for row in main_results:
            if row[0] == "desktop":
                prev = (row[4] or "").replace("|", "\\|")[:90]
                f.write(f"| {row[1]} | **{row[3]}** | {prev} |\n")
        f.write("\n## ELF programs\n\n")
        f.write("| Name | Desc | Status | Summary |\n|---|---|---|---|\n")
        for row in elf_results:
            prev = (row[4] or "").replace("|", "\\|")[:90]
            f.write(f"| `{row[1]}` | {row[2]} | **{row[3]}** | {prev} |\n")
    return counts


def main():
    print(f"[suite] kernel = {KERNEL}  ({KERNEL.stat().st_size:,} bytes)")
    print(f"[suite] report = {REPORT}")
    print()

    main_results = run_main_phase()
    elf_results = run_elf_phase()

    counts = write_report(main_results, elf_results)

    print()
    print("=" * 68)
    print("FINAL SUMMARY")
    print("=" * 68)
    for k in sorted(counts.keys()):
        print(f"  {k:10s} {counts[k]}")
    print()
    print(f"Report: {REPORT}")
    print(f"Logs:   {MAIN_LOG} + {ELF_LOG_DIR}/")


if __name__ == "__main__":
    main()

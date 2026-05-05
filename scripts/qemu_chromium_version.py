#!/usr/bin/env python3
"""STUMP #160 iter 6 — minimal Chromium init test (`--version`).

Boots Bat_OS, runs `chromium-version`. content_shell parses
`--version` extremely early in init — before SequenceManager
spawns the worker pool, before NetworkService, before anything
in the dump-dom hang path. If this PASSes, our boot →
ELF load → glibc init → ContentMain → printf → _exit
pipeline is fully wired. If it hangs, we have a deeper bug.

PASS: log contains "Content Shell" version string.
"""
import pexpect
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KERNEL_ELF = ROOT / "target/aarch64-unknown-none/release/bat_os"
KERNEL_BIN = ROOT / "target/aarch64-unknown-none/release/bat_os.bin"
INITRD     = ROOT / "target/aarch64-unknown-none/release/chromium_initrd.bin"
LOG = ROOT / f"logs/qemu-tests/chromium-version-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)
PROMPT = rb"bat_os\s*>\s*"

def main():
    if not INITRD.exists():
        print(f"[version] no initrd at {INITRD}")
        return 1
    if not KERNEL_BIN.exists() or KERNEL_BIN.stat().st_mtime < KERNEL_ELF.stat().st_mtime:
        import shutil
        rust_objcopy = Path.home() / (
            ".rustup/toolchains/nightly-aarch64-apple-darwin/"
            "lib/rustlib/aarch64-apple-darwin/bin/rust-objcopy")
        if not rust_objcopy.exists():
            alt = shutil.which("llvm-objcopy") or shutil.which("objcopy")
            rust_objcopy = Path(alt) if alt else None
        if rust_objcopy is None:
            print("[version] need rust-objcopy / llvm-objcopy")
            return 1
        r = subprocess.run(
            [str(rust_objcopy), "-O", "binary", str(KERNEL_ELF), str(KERNEL_BIN)],
            capture_output=True, text=True,
        )
        if r.returncode != 0:
            print(f"[version] objcopy failed: {r.stderr}")
            return 1

    # No -display none / GPU device — keep the serial-shell boot path.
    # We don't need the framebuffer for chromium-version output.
    args = [
        "qemu-system-aarch64",
        "-accel", "hvf",
        "-machine", "virt,gic-version=3",
        "-cpu", "host",
        "-m", "4G",
        "-display", "none",
        "-serial", "mon:stdio",
        "-kernel", str(KERNEL_BIN),
        "-initrd", str(INITRD),
    ]
    fp = open(LOG, "wb")
    # Generous timeout — content_shell ELF load + reloc takes ~30s,
    # init runs ~10s under our cooperative scheduler.
    c = pexpect.spawn(args[0], args[1:], timeout=180, logfile=fp, encoding=None)
    verdict = "FAIL"
    try:
        # Two boot paths possible:
        #   - Splash gate:  "[bs] paint done — input loop", needs passphrase
        #   - Serial shell: goes straight to "bat_os >" if no GPU available
        idx = c.expect([
            rb"\[bs\] paint done .+ input loop",
            PROMPT,
        ], timeout=60)
        if idx == 0:
            time.sleep(0.3); c.sendline(b"batman")
            c.expect(PROMPT, timeout=30)
        # else already at prompt
        c.sendline(b"chromium-version")
        idx = c.expect([
            rb"Content Shell \d+\.\d+\.\d+",
            rb"chromium-version:",
            rb"chromium-version exited OK",
            PROMPT,
        ], timeout=120)
        if idx == 0:
            print(f"[version] ✓ PASS — got: {c.match.group(0).decode()}")
            verdict = "PASS"
        elif idx == 2:
            # Exit OK without seeing version — might still be PASS,
            # check log for the version string.
            with open(LOG, "rb") as f:
                raw = f.read().decode("utf-8", "replace")
            if "Content Shell" in raw:
                print("[version] ✓ PASS (version string found post-exit)")
                verdict = "PASS"
            else:
                print("[version] exited but no version string emitted")
        else:
            print(f"[version] FAIL — got: {c.match.group(0).decode() if c.match else '?'}")
        try: c.expect(PROMPT, timeout=10)
        except pexpect.TIMEOUT: pass
    except pexpect.TIMEOUT:
        print("[version] TIMEOUT")
        fp.flush()
        with open(LOG, "rb") as f:
            f.seek(0, 2); size = f.tell()
            f.seek(max(0, size - 2000))
            print("--- last 2000 bytes ---")
            print(f.read().decode("utf-8", "replace"))
    finally:
        c.terminate(force=True); fp.close()
    print(f"Log: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict == "PASS" else 1

if __name__ == "__main__":
    sys.exit(main())

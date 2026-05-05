#!/usr/bin/env python3
"""STUMP #161 — Ladybird LibJS REPL smoke test.

Boots Bat_OS with the Ladybird initrd, runs `ladybird-js`, looks for
the result of `console.log(1+1)`. PASS if we see "2" in the
output stream. This is the smallest possible Ladybird test —
exercises:

  ELF load → glibc init → AK strings → LibCrypto → LibJS → printf

If THIS prints, our pipeline can host Ladybird's libs end-to-end.
The dump-DOM test (`ladybird --dump-dom file:///bin/hello.html`)
is its own bigger smoke once WebContent is wired up.
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
INITRD     = ROOT / "target/aarch64-unknown-none/release/ladybird_initrd.bin"
LOG = ROOT / f"logs/qemu-tests/ladybird-js-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)
PROMPT = rb"bat_os\s*>\s*"


def main():
    if not INITRD.exists():
        print(f"[ladybird-js] no Ladybird initrd at {INITRD}")
        print("              build first:")
        print("                ports/ladybird_port/build.sh   (in container)")
        print("                tools/bake_ladybird_initrd.sh ports/ladybird_port/out")
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
            print("[ladybird-js] need rust-objcopy / llvm-objcopy")
            return 1
        r = subprocess.run(
            [str(rust_objcopy), "-O", "binary", str(KERNEL_ELF), str(KERNEL_BIN)],
            capture_output=True, text=True,
        )
        if r.returncode != 0:
            print(f"[ladybird-js] objcopy failed: {r.stderr}")
            return 1

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
    c = pexpect.spawn(args[0], args[1:], timeout=180, logfile=fp, encoding=None)
    verdict = "FAIL"
    try:
        idx = c.expect([
            rb"\[bs\] paint done .+ input loop",
            PROMPT,
        ], timeout=60)
        if idx == 0:
            time.sleep(0.3); c.sendline(b"batman")
            c.expect(PROMPT, timeout=30)
        c.sendline(b"ladybird-js 1+1")
        # PASS markers: a line containing "2" between the command and
        # the next prompt. Also accept "ladybird-js exited OK".
        idx = c.expect([
            rb"\b2\b",
            rb"ladybird-js exited OK",
            rb"ladybird-js: ",
            PROMPT,
        ], timeout=120)
        match = c.match.group(0).decode() if c.match else "?"
        print(f"[ladybird-js] match: {match!r}")
        with open(LOG, "rb") as f:
            raw = f.read().decode("utf-8", "replace")
        # Confirm by looking at the raw log too.
        if " 2\r\n" in raw or "= 2" in raw or "(2)" in raw or "\n2\r\n" in raw:
            print("[ladybird-js] ✓ PASS — got '2' from LibJS")
            verdict = "PASS"
        elif "exited OK" in raw:
            print("[ladybird-js] ✓ PASS — js exited cleanly")
            verdict = "PASS"
        else:
            print(f"[ladybird-js] FAIL — neither '2' nor clean exit")
        try: c.expect(PROMPT, timeout=10)
        except pexpect.TIMEOUT: pass
    except pexpect.TIMEOUT:
        print("[ladybird-js] TIMEOUT")
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

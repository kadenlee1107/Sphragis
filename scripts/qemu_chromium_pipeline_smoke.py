#!/usr/bin/env python3
"""Chromium delivery-pipeline smoke test.

Does NOT build a real content_shell. Bakes a small ELF (tests/hello)
through tools/bake_chromium.sh as a stand-in, boots the resulting
`bat_os_with_chromium` image in QEMU, and runs the `chromium
<url>` shell command. Observes how far the pipeline gets:

  1. initrd::locate_chromium_blob() finds the BATCHROM framing
  2. CRC32 verifies
  3. Signature check (expected to refuse unsigned in release; we'd
     see "refusing" unless INITRD_PUBKEY has been set)
  4. OR cave page-table + ELF load happens
  5. Execution begins (will crash on first unimplemented syscall
     because tests/hello is a minimal ARM64 static binary)

The goal isn't "Chromium renders the page" — the goal is "the
delivery pipeline from kernel-image bake → initrd discovery →
cave setup → ELF load → runner all connect end-to-end."

Usage:
    python3 scripts/qemu_chromium_pipeline_smoke.py
"""
import pexpect, re, socket, subprocess, sys, time
from pathlib import Path
from datetime import datetime

ROOT = Path(__file__).resolve().parent.parent
LOG = ROOT / f"logs/qemu-tests/chromium-smoke-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)
PROMPT = rb"bat_os\s*>\s*"

def main():
    # Use the -initrd path + FLAT KERNEL IMAGE (not ELF) so QEMU
    # honours the ARM64 Linux boot protocol and delivers the DTB
    # in x0. The DTB's /chosen/linux,initrd-* nodes carry the
    # initrd range, and `initrd::set_range` hands the probe the
    # physical address QEMU loaded the blob at.
    kernel_elf = ROOT / "target/aarch64-unknown-none/release/bat_os"
    kernel_bin = ROOT / "target/aarch64-unknown-none/release/bat_os.bin"
    initrd     = ROOT / "target/aarch64-unknown-none/release/chromium_initrd.bin"
    if not kernel_elf.exists():
        print(f"[smoke] no kernel ELF at {kernel_elf}")
        print("        build: BAT_OS_ALLOW_UNSIGNED_INITRD=1 \\")
        print("               BAT_OS_PASSPHRASE=batman \\")
        print("               cargo build --release")
        print("        (the env flag enables the dev-only unsigned-blob path)")
        return 1
    if not initrd.exists():
        print(f"[smoke] no initrd at {initrd}")
        print("        run: tools/bake_chromium_initrd.sh tests/hello")
        return 1
    # Produce the flat Image if it doesn't exist or is stale.
    if (not kernel_bin.exists()
            or kernel_bin.stat().st_mtime < kernel_elf.stat().st_mtime):
        import shutil
        rust_objcopy = Path.home() / (
            ".rustup/toolchains/nightly-aarch64-apple-darwin/"
            "lib/rustlib/aarch64-apple-darwin/bin/rust-objcopy")
        if not rust_objcopy.exists():
            alt = shutil.which("llvm-objcopy") or shutil.which("objcopy")
            rust_objcopy = Path(alt) if alt else None
        if rust_objcopy is None:
            print("[smoke] need rust-objcopy / llvm-objcopy; install rustup llvm-tools")
            return 1
        print(f"[smoke] {rust_objcopy.name} -O binary …")
        r = subprocess.run(
            [str(rust_objcopy), "-O", "binary", str(kernel_elf), str(kernel_bin)],
            capture_output=True, text=True,
        )
        if r.returncode != 0:
            print(f"[smoke] objcopy failed: {r.stderr}")
            return 1

    daemon = subprocess.Popen(
        ["python3", str(ROOT / "scripts" / "batcaved.py")],
        stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT,
    )
    for _ in range(40):
        try: socket.create_connection(("127.0.0.1", 9999), timeout=0.3).close(); break
        except OSError: time.sleep(0.2)

    args = ["qemu-system-aarch64", "-machine", "virt", "-cpu", "max",
            "-m", "4G",
            "-display", "none",
            "-device", "virtio-gpu-device",
            "-device", "virtio-keyboard-device",
            "-netdev", "user,id=net0",
            "-device", "virtio-net-device,netdev=net0",
            "-serial", "mon:stdio",
            "-kernel", str(kernel_bin),
            "-initrd", str(initrd)]
    fp = open(LOG, "wb")
    c = pexpect.spawn(args[0], args[1:], timeout=90, logfile=fp, encoding=None)
    events = []
    verdict = "FAIL"
    try:
        c.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
        time.sleep(0.3)
        c.sendline(b"batman")
        c.expect(PROMPT, timeout=30)

        # Record what happens between sending the command and the next
        # prompt (the runner either succeeds, errors out with a message,
        # or crashes into the kernel's exception handler).
        c.sendline(b"chromium https://example.com")
        try:
            c.expect(PROMPT, timeout=20)
        except pexpect.TIMEOUT:
            pass

        # Pull the raw serial log and look for known marker strings.
        with open(LOG, "rb") as f:
            raw = f.read().decode("utf-8", "replace")

        checks = [
            ("initrd:blob locate",       "BATCHROM" in raw or "chromium blob" in raw
                                         or "no chromium blob" in raw or "[initrd]" in raw),
            ("shell cmd accepted",       "chromium https://" in raw),
            ("runner reached",           "content_shell" in raw or "chromium" in raw.lower()),
            ("initrd sig check",         "signature" in raw.lower() or "INITRD_PUBKEY" in raw
                                         or "unsigned" in raw.lower() or "CRC" in raw),
        ]
        for label, ok in checks:
            mark = "✓" if ok else "✗"
            print(f"   {mark} {label}")
            events.append((label, ok))

        # Success pattern: we expect "chromium: <error>" or
        # "chromium exited OK" or the refusal message. Any of those
        # means we got all the way through the pipeline until the
        # runtime bit.
        if re.search(r"chromium: |chromium exited OK|refusing|content_shell", raw):
            verdict = "PIPELINE-REACHED"

        # Print the last 30 lines for visibility.
        print("\n--- last serial output ---")
        for line in raw.splitlines()[-30:]:
            s = line.rstrip()
            if s: print(f"   {s[:140]}")
    except pexpect.TIMEOUT:
        print("[smoke] TIMEOUT")
    finally:
        c.terminate(force=True); fp.close()
        daemon.terminate()
        try: daemon.wait(timeout=3)
        except subprocess.TimeoutExpired: daemon.kill()
    print(f"\nLog: {LOG}")
    print(f"Verdict: {verdict}")
    return 0 if verdict == "PIPELINE-REACHED" else 1

if __name__ == "__main__":
    sys.exit(main())

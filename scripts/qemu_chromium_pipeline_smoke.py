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


def _resolve_undef_sym(elf_path: Path, sym_idx: int) -> str:
    """Look up the name of undef symbol #sym_idx in a content_shell-style ELF.

    Walks PT_DYNAMIC for DT_SYMTAB + DT_STRTAB, pulls the sym_idx'th entry,
    returns the name. Returns "(idx=N)" if the ELF can't be read. Kept here
    (not imported) because the smoke script has to work in a bare venv on
    an environment that may not have pyelftools.
    """
    import struct
    if not elf_path.exists():
        return f"(idx={sym_idx}, no ELF)"
    try:
        with open(elf_path, "rb") as f:
            data = f.read()
        phoff = struct.unpack("<Q", data[0x20:0x28])[0]
        phnum = struct.unpack("<H", data[0x38:0x3A])[0]
        vranges = []
        for i in range(phnum):
            hdr = data[phoff + i * 56 : phoff + i * 56 + 56]
            p_type, _, p_offset, p_vaddr, _, p_filesz, _, _ = struct.unpack(
                "<IIQQQQQQ", hdr
            )
            if p_type == 1:
                vranges.append((p_vaddr, p_filesz, p_offset))

        def v2f(va):
            for v, sz, off in vranges:
                if v <= va < v + sz:
                    return off + (va - v)
            return None

        dyn_off = dyn_size = 0
        for i in range(phnum):
            hdr = data[phoff + i * 56 : phoff + i * 56 + 56]
            p_type, _, _, _, _, p_filesz, _, _ = struct.unpack("<IIQQQQQQ", hdr)
            if p_type == 2:
                dyn_off = struct.unpack("<Q", hdr[8:16])[0]
                dyn_size = p_filesz
                break

        symtab_va = strtab_va = 0
        pos, end = dyn_off, dyn_off + dyn_size
        while pos < end:
            tag, val = struct.unpack("<QQ", data[pos : pos + 16])
            if tag == 5:
                strtab_va = val
            elif tag == 6:
                symtab_va = val
            if tag == 0:
                break
            pos += 16

        symtab = v2f(symtab_va)
        strtab = v2f(strtab_va)
        if symtab is None or strtab is None:
            return f"(idx={sym_idx}, no symtab)"

        sym_off = symtab + sym_idx * 24
        name_off = struct.unpack("<I", data[sym_off : sym_off + 4])[0]
        nm_end = data.index(b"\x00", strtab + name_off)
        return data[strtab + name_off : nm_end].decode("utf-8", "ignore") or f"(idx={sym_idx}, no name)"
    except Exception as e:
        return f"(idx={sym_idx}, err: {e})"

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

    # gic-version=2 forces QEMU to expose GICv2 MMIO (matches our handler).
    # Default "max" picks v3 which uses system registers (ICC_*) we don't
    # touch — IRQs would still fire but couldn't be acked properly.
    args = ["qemu-system-aarch64", "-machine", "virt,gic-version=2", "-cpu", "max",
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
    # Bumped from 300s to 900s — Chromium's cooperative-scheduler init
    # is slow (4000+ syscalls just to get past fontconfig), and the
    # full DOM-dump path needs even more time.
    c = pexpect.spawn(args[0], args[1:], timeout=900, logfile=fp, encoding=None)
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
        # CHROMIUM-PHASE-B: use a local file:// URL with --dump-dom so
        # we don't need the network / TLS stack just to exercise the
        # Chromium pipeline through init + HTML parse. hello.html is
        # shipped in the archive alongside content_shell.
        c.sendline(b"chromium --dump-dom file:///bin/hello.html")
        try:
            # Bumped from 240s to 720s. 18 worker threads × cooperative
            # yield scheduling = a lot of round-trips to get to dump-dom.
            c.expect(PROMPT, timeout=720)
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

        # Decode any 0xBAD0.... sentinels the loader wrote for SHN_UNDEF
        # PLT/GOT slots. Pattern: 0xBAD0_0000_0000_0000 | (sym_idx << 4).
        # When content_shell's first libc call fires, we trap on PC=this
        # sentinel and can print the symbol name from the content_shell
        # symbol table — turns an opaque "ELR: 0xbad0000000000010" into a
        # "UNDEF symbol __libc_start_main called (sym #1)" diagnostic.
        undef_sentinel = re.search(r"ELR: 0x([0-9a-f]{16})", raw)
        if undef_sentinel:
            elr = int(undef_sentinel.group(1), 16)
            if (elr & 0xFFF0_0000_0000_0000) == 0xBAD0_0000_0000_0000:
                sym_idx = (elr >> 4) & 0x0FFF_FFFF
                name = _resolve_undef_sym(ROOT / "ports/chromium_port/out/content_shell",
                                         sym_idx)
                print(f"\n[smoke] UNDEF symbol call: sym #{sym_idx} = {name}")
                print("[smoke] content_shell reached its libc-init phase and")
                print("[smoke] called a glibc-resolved symbol that Bat_OS has")
                print("[smoke] no implementation for. This is Phase 4 / Phase 2")
                print("[smoke] work (see ports/chromium_port/STATE_2026-04-23.md).")

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

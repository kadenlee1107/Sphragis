#!/usr/bin/env python3
"""Bat_OS + Ladybird port — windowed QEMU demo.

Boots Bat_OS with the Ladybird initrd in a visible QEMU window
(virtio-gpu scanout). At the bat_os shell prompt try:

    render file:///bin/hello.html
        → Bat_OS's built-in HTML/CSS/Layout browser paints the
          page to /batos/fb0; the chromium_blit kthread copies
          to the virtio-gpu framebuffer; you see it in the window.
          This is NOT Ladybird's LibWeb rendering — but it IS the
          same display pipeline Ladybird's WebContent will paint
          into once we wire that up. Built-in browser is from
          earlier port work; reusing the framework here.

    ladybird-js console.log(1+1)
        → Ladybird's actual LibJS engine. Output on the serial
          console (this terminal), not the window.

    ladybird-dump
        → Ladybird's HTMLTokenizer parses a hello-world doc and
          prints each token. Also serial-console only for now.

Auth: kernel was built with BAT_OS_PASSPHRASE=batman, so type
`batman` at the passphrase prompt (or just press Enter for the
dev-default empty passphrase).

QEMU exits via Ctrl-A X.
"""
from __future__ import annotations

import platform
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT / "target/aarch64-unknown-none/release"
KERNEL_ELF = TARGET / "bat_os"
KERNEL_BIN = TARGET / "bat_os.bin"
INITRD = TARGET / "ladybird_initrd.bin"

display = "cocoa" if platform.system() == "Darwin" else "gtk"

# Refresh kernel.bin if stale.
if not KERNEL_BIN.exists() or KERNEL_BIN.stat().st_mtime < KERNEL_ELF.stat().st_mtime:
    rust_objcopy = Path.home() / (
        ".rustup/toolchains/nightly-aarch64-apple-darwin/"
        "lib/rustlib/aarch64-apple-darwin/bin/rust-objcopy")
    print(f"[ladybird-window] refreshing {KERNEL_BIN.name}")
    subprocess.run(
        [str(rust_objcopy), "-O", "binary", str(KERNEL_ELF), str(KERNEL_BIN)],
        check=True,
    )

args = [
    "qemu-system-aarch64",
    "-accel", "hvf",
    "-machine", "virt,gic-version=3",
    "-cpu", "host",
    "-m", "4G",
    "-display", f"{display},show-cursor=on",
    "-serial", "mon:stdio",
    "-kernel", str(KERNEL_BIN),
    "-initrd", str(INITRD),
    "-device", "virtio-gpu-device",
    "-device", "virtio-keyboard-device",
]

print("[ladybird-window] launching QEMU with virtio-gpu")
print("[ladybird-window] window should open within a few seconds")
print("[ladybird-window] at the bat_os> prompt, try:")
print()
print("    render file:///bin/hello.html live=1")
print("        ^^^^^ live=1 is the magic flag — without it,")
print("              render lays out the page but doesn't blit")
print("              to the virtio-gpu framebuffer. With it,")
print("              you'll see the page in the QEMU window.")
print()
print("    ladybird-js console.log(1+1)")
print("    ladybird-dump")
print()
print("Auth: passphrase is 'batman' (Enter also works for dev default).")
print()

# Pass through to the user's terminal — they get a real interactive shell.
# Press Ctrl-A X (QEMU) or Ctrl-C twice to exit.
subprocess.run(args)

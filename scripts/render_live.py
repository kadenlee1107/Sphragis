#!/usr/bin/env python3
"""Sprint 1.4 (STUMP #97): boot Bat_OS in a windowed QEMU session,
attach virtio-gpu + virtio-keyboard, and run `render <URL> live=1` in
the kernel shell so the rendered page appears in the QEMU display.

Differences vs `scripts/render_to_png.py`:
  * `-display cocoa` (Mac) / `-display gtk` (Linux) instead of `none`,
    so QEMU opens a window.
  * `-device virtio-gpu-device` so the kernel's virtio-gpu driver
    initializes and accepts a flush.
  * `-device virtio-keyboard-device` so the user can type in the
    window — useful when we wire keyboard input through to the
    renderer next sprint.
  * Skips the post-prompt PNG decode — the whole point is the live
    window. The base64 dump is still emitted on serial in case you
    want to grep it.

Usage:
  python3 scripts/render_live.py [url]
"""
from __future__ import annotations

import os
import platform
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT / "target/aarch64-unknown-none/release"
KERNEL = TARGET / "bat_os"
KERNEL_BIN = TARGET / "bat_os.bin"
INITRD = TARGET / "chromium_initrd.bin"

URL = sys.argv[1] if len(sys.argv) > 1 else "file:///bin/hello.html"


def find_objcopy() -> str:
    for cand in ("rust-objcopy", "llvm-objcopy", "aarch64-linux-gnu-objcopy"):
        try:
            subprocess.run([cand, "--version"], check=True,
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            return cand
        except (FileNotFoundError, subprocess.CalledProcessError):
            continue
    raise FileNotFoundError("no objcopy in PATH")


def refresh_bin() -> None:
    if KERNEL_BIN.exists() and KERNEL_BIN.stat().st_mtime >= KERNEL.stat().st_mtime:
        return
    objcopy = find_objcopy()
    print(f"[render-live] {objcopy} -O binary {KERNEL.name} {KERNEL_BIN.name}")
    subprocess.run([objcopy, "-O", "binary", str(KERNEL), str(KERNEL_BIN)],
                   check=True)


def main() -> int:
    if not KERNEL.exists():
        print(f"[render-live] no kernel at {KERNEL}; run `make build`", file=sys.stderr)
        return 1
    if not INITRD.exists():
        print(f"[render-live] no initrd at {INITRD}; run `make initrd`", file=sys.stderr)
        return 1
    refresh_bin()

    display = "cocoa" if platform.system() == "Darwin" else "gtk"

    args = [
        "qemu-system-aarch64",
        "-accel", "hvf",
        "-machine", "virt,gic-version=3",
        "-cpu", "host",
        "-m", "4G",
        "-display", display,
        "-serial", "mon:stdio",
        "-kernel", str(KERNEL_BIN),
        "-initrd", str(INITRD),
        "-netdev", "user,id=net0",
        "-device", "virtio-net-device,netdev=net0",
        "-device", "virtio-gpu-device",
        "-device", "virtio-keyboard-device",
    ]

    print(f"[render-live] launching QEMU ({display}). URL: {URL}")
    print(f"[render-live] kernel will receive: render {URL} live=1")
    print(f"[render-live] type that command in the kernel shell once you see the prompt")
    print(f"[render-live] close the window or Ctrl-A X to quit")

    # Run interactively — user types `render <URL> live=1` themselves once
    # the kernel boots. Future iteration: pre-feed the command via a
    # serial pipe.
    try:
        subprocess.run(args, check=False)
    except KeyboardInterrupt:
        pass
    return 0


if __name__ == "__main__":
    sys.exit(main())

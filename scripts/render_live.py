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


def ensure_passphrase_baked(passphrase: str = "batman") -> None:
    """STUMP #99 follow-up: the auth gate uses the dev-fallback
    secret (a binary-derived string, untypeable) when
    BAT_OS_PASSPHRASE wasn't set at build time. We rebuild the kernel
    with BAT_OS_PASSPHRASE=<passphrase> baked in if the current
    binary doesn't already contain it. Without this, every
    render-live session strands the user at the bat-logo prompt
    with no working passphrase.
    """
    if not KERNEL.exists():
        return
    raw = KERNEL.read_bytes()
    if passphrase.encode() in raw:
        return  # already baked
    print(f"[render-live] kernel doesn't have BAT_OS_PASSPHRASE={passphrase} baked — rebuilding")
    main_rs = ROOT / "src/main.rs"
    if main_rs.exists():
        # Touch so cargo notices the env change.
        main_rs.touch()
    env = os.environ.copy()
    env["BAT_OS_PASSPHRASE"] = passphrase
    env.setdefault("BAT_OS_ALLOW_UNSIGNED_INITRD", "1")
    env.setdefault("BAT_OS_KEEP_GOING", "1")
    subprocess.run(
        ["cargo", "build", "--release", "--features", "gicv3"],
        cwd=str(ROOT), env=env, check=True,
    )


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
    ensure_passphrase_baked("batman")
    refresh_bin()

    # Cocoa is the only display backend brew's QEMU ships with on
    # Mac. cocoa+virtio-tablet doesn't deliver motion events, so the
    # interactive loop also accepts arrow-key cursor movement (Enter
    # to click) — no mouse required.
    if platform.system() == "Darwin":
        display = os.environ.get("BAT_OS_DISPLAY", "cocoa")
    else:
        display = os.environ.get("BAT_OS_DISPLAY", "gtk")

    args = [
        "qemu-system-aarch64",
        "-accel", "hvf",
        "-machine", "virt,gic-version=3",
        "-cpu", "host",
        "-m", "4G",
        # show-cursor=on keeps the host arrow visible over the
        # QEMU window AND (on most backends) ensures motion is
        # delivered to virtio-tablet. SDL is preferred on Mac
        # because cocoa silently drops motion events.
        "-display", f"{display},show-cursor=on",
        "-serial", "mon:stdio",
        "-kernel", str(KERNEL_BIN),
        "-initrd", str(INITRD),
        "-netdev", "user,id=net0",
        "-device", "virtio-net-device,netdev=net0",
        "-device", "virtio-gpu-device",
        "-device", "virtio-keyboard-device",
        # Tablet MUST come after keyboard — the kernel takes input
        # device #0 as keyboard and #1 as tablet (Sprint 1.5b).
        "-device", "virtio-tablet-device",
    ]

    print(f"[render-live] launching QEMU ({display}). URL: {URL}")
    print(f"[render-live]")
    print(f"[render-live] At the bat-logo screen: type 'batman' Enter.")
    print(f"[render-live] At the shell prompt:")
    print(f"[render-live]     render {URL} live=1")
    print(f"[render-live]")
    print(f"[render-live] Interactive controls (works in EITHER window):")
    print(f"[render-live]   Ctrl+W / A / S / D — move cursor up/left/down/right")
    print(f"[render-live]   Ctrl+E             — click at cursor")
    print(f"[render-live]   Ctrl+G             — recenter cursor")
    print(f"[render-live]   typing             — into focused <input> after Ctrl+E")
    print(f"[render-live]   ESC                — exit interactive loop")
    print(f"[render-live]")
    print(f"[render-live] (Mac cocoa drops mouse motion to virtio-tablet, hence")
    print(f"[render-live]  the keyboard cursor.) Close window or Ctrl-A X to quit.")

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

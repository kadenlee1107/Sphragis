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


def _free_port(default: int = 4444) -> int:
    """STUMP #110: pick a port for QMP. Tries `default`, falls back to
    OS-assigned if it's taken. Prevents the 'Address already in use'
    crash when an earlier QEMU/bridge run left something bound."""
    import socket as _s
    for candidate in (default,):
        try:
            with _s.socket(_s.AF_INET, _s.SOCK_STREAM) as t:
                t.setsockopt(_s.SOL_SOCKET, _s.SO_REUSEADDR, 1)
                t.bind(("127.0.0.1", candidate))
                return candidate
        except OSError:
            pass
    with _s.socket(_s.AF_INET, _s.SOCK_STREAM) as t:
        t.bind(("127.0.0.1", 0))
        return t.getsockname()[1]


def _pointer_device_arg() -> str:
    """STUMP #109: pick virtio-mouse vs virtio-tablet based on host
    OS / env override. Mac default = mouse (cocoa delivers EV_REL but
    not EV_ABS). Linux default = tablet (absolute coords, no relative
    accumulation drift)."""
    override = os.environ.get("BAT_OS_POINTER", "").strip().lower()
    if override == "mouse":
        return "virtio-mouse-device"
    if override == "tablet":
        return "virtio-tablet-device"
    if platform.system() == "Darwin":
        return "virtio-mouse-device"
    return "virtio-tablet-device"


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

    qmp_port = _free_port()

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
        # STUMP #110: QMP socket for the mouse-injection sidecar.
        # Cocoa drops both EV_ABS and EV_REL motion to virtio input
        # devices on Mac. The mouse_bridge.py sidecar reads host
        # mouse via CoreGraphics and injects rel/btn events through
        # this socket — bypasses cocoa, real mouse follows.
        "-qmp", f"tcp:127.0.0.1:{qmp_port},server,nowait",
        "-device", "virtio-gpu-device",
        "-device", "virtio-keyboard-device",
        # Pointer device. STUMP #109: QEMU's cocoa display silently
        # drops EV_ABS motion events to virtio-tablet on Mac. The
        # virtio-mouse-device (relative) takes a different cocoa code
        # path that actually delivers EV_REL deltas. Default to mouse
        # on Mac, tablet on Linux GTK/SDL (more accurate). Override
        # with `BAT_OS_POINTER=mouse` or `BAT_OS_POINTER=tablet`. The
        # kernel takes input #1 as the pointer device; tablet.rs
        # handles both EV_ABS and EV_REL events.
        "-device", _pointer_device_arg(),
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

    # STUMP #110: launch the mouse-injection sidecar so real mouse
    # motion drives the kernel cursor. It connects to the QMP socket
    # we just opened. Mac-only (CoreGraphics dependency); silently
    # skipped on Linux (where virtio-tablet works natively).
    bridge_proc = None
    if platform.system() == "Darwin":
        bridge_path = ROOT / "scripts/mouse_bridge.py"
        if bridge_path.exists():
            print(f"[render-live] launching mouse_bridge.py against QMP 127.0.0.1:{qmp_port}")
            try:
                # Inherit our stdout/stderr so the user sees [bridge] lines.
                bridge_proc = subprocess.Popen(
                    [sys.executable, str(bridge_path), f"127.0.0.1:{qmp_port}"]
                )
            except Exception as e:
                print(f"[render-live] mouse_bridge.py failed: {e}", file=sys.stderr)

    # Run interactively — user types `render <URL> live=1` themselves once
    # the kernel boots. Future iteration: pre-feed the command via a
    # serial pipe.
    try:
        subprocess.run(args, check=False)
    except KeyboardInterrupt:
        pass
    finally:
        if bridge_proc is not None and bridge_proc.poll() is None:
            bridge_proc.terminate()
            try: bridge_proc.wait(timeout=2)
            except subprocess.TimeoutExpired: bridge_proc.kill()
    return 0


if __name__ == "__main__":
    sys.exit(main())

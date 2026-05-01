#!/usr/bin/env python3
"""STUMP #110: QMP mouse-injection bridge for Bat_OS on macOS.

Why this exists:
  QEMU's cocoa display backend silently drops mouse motion events
  to virtio-tablet AND virtio-mouse devices on macOS. The host
  cursor disappears over the QEMU window but neither EV_ABS nor
  EV_REL events reach the guest. The kernel's interactive cursor
  loop has a Ctrl+WASD keyboard fallback, but real mouse pointing
  is what we'd actually want.

This sidecar bypasses cocoa entirely: it reads the host mouse
position via Apple's CoreGraphics framework (CGEventGetLocation),
calculates per-tick deltas, and injects them into QEMU as relative
mouse events via the QMP (QEMU Machine Protocol) socket. The kernel
sees them through its virtio-mouse-device driver as if they came
from the cocoa input pipeline that wasn't delivering them.

Run:
   python3 scripts/mouse_bridge.py [host:port]   (default 127.0.0.1:4444)

Pair with:
   make render-live URL=...    (which launches QEMU with -qmp on 4444)

Stop with Ctrl-C — no cleanup needed; QEMU's QMP socket reaccepts
on next bridge run.

Limitations:
 * macOS only. Linux / Windows would need different host-mouse APIs.
 * Reads mouse globally — when the host cursor is over the kernel's
   QEMU window the kernel cursor follows; when it's over your
   IDE the kernel still sees motion. That's actually fine for a
   demo, just expected.
 * Only left-click is wired; middle/right are TODO.
"""
from __future__ import annotations

import ctypes
import ctypes.util
import json
import socket
import sys
import time

# ── macOS host mouse (CoreGraphics via ctypes) ────────────────────

class CGPoint(ctypes.Structure):
    _fields_ = [("x", ctypes.c_double), ("y", ctypes.c_double)]

_cg_path = ctypes.util.find_library("ApplicationServices")
if not _cg_path:
    print("[bridge] no ApplicationServices framework — macOS only", file=sys.stderr)
    sys.exit(1)
_cg = ctypes.cdll.LoadLibrary(_cg_path)
_cg.CGEventCreate.restype = ctypes.c_void_p
_cg.CGEventCreate.argtypes = [ctypes.c_void_p]
_cg.CGEventGetLocation.restype = CGPoint
_cg.CGEventGetLocation.argtypes = [ctypes.c_void_p]
_cg.CGEventSourceButtonState.restype = ctypes.c_bool
_cg.CGEventSourceButtonState.argtypes = [ctypes.c_uint32, ctypes.c_uint32]
_cg.CFRelease.argtypes = [ctypes.c_void_p]

# CGEventSourceStateID: 0 = HID system state (current real input)
_HID_SYSTEM = 0
# CGMouseButton: 0 = left
_BTN_LEFT = 0


def host_mouse_pos() -> tuple[float, float]:
    e = _cg.CGEventCreate(None)
    p = _cg.CGEventGetLocation(e)
    _cg.CFRelease(e)
    return (p.x, p.y)


def host_left_button_down() -> bool:
    return bool(_cg.CGEventSourceButtonState(_HID_SYSTEM, _BTN_LEFT))


# ── QMP client ───────────────────────────────────────────────────

class QMP:
    def __init__(self, host: str, port: int):
        self.s = socket.create_connection((host, port), timeout=10)
        self.f = self.s.makefile("rwb", buffering=0)
        # Read banner line, ignore.
        self.f.readline()
        self._send({"execute": "qmp_capabilities"})
        self._recv()  # {"return": {}}

    def _send(self, obj: dict) -> None:
        self.f.write((json.dumps(obj) + "\n").encode())

    def _recv(self) -> dict:
        line = self.f.readline()
        return json.loads(line.decode()) if line else {}

    def send_input_events(self, events: list[dict]) -> None:
        self._send({
            "execute": "input-send-event",
            "arguments": {"events": events},
        })
        # Drain a response; ignore async events.
        try:
            self.s.settimeout(0.05)
            self._recv()
        except socket.timeout:
            pass
        finally:
            self.s.settimeout(10)


def rel_event(axis: str, value: int) -> dict:
    return {"type": "rel", "data": {"axis": axis, "value": value}}


def btn_event(button: str, down: bool) -> dict:
    return {"type": "btn", "data": {"down": down, "button": button}}


# ── Bridge loop ──────────────────────────────────────────────────

def run(host: str, port: int) -> int:
    print(f"[bridge] connecting to QMP at {host}:{port} …", flush=True)
    for _ in range(50):  # 5 s of retries — QEMU might still be booting
        try:
            qmp = QMP(host, port)
            break
        except (ConnectionRefusedError, OSError):
            time.sleep(0.1)
    else:
        print("[bridge] QMP socket never accepted — is QEMU running with -qmp?", file=sys.stderr)
        return 1
    print("[bridge] QMP up. Move host mouse to drive kernel cursor.", flush=True)

    last_x, last_y = host_mouse_pos()
    last_btn = host_left_button_down()

    try:
        while True:
            now_x, now_y = host_mouse_pos()
            dx = int(round(now_x - last_x))
            dy = int(round(now_y - last_y))
            events: list[dict] = []
            if dx != 0:
                events.append(rel_event("x", dx))
                last_x = now_x
            if dy != 0:
                events.append(rel_event("y", dy))
                last_y = now_y
            now_btn = host_left_button_down()
            if now_btn != last_btn:
                events.append(btn_event("left", now_btn))
                last_btn = now_btn
            if events:
                qmp.send_input_events(events)
            time.sleep(0.016)  # ~60 Hz
    except KeyboardInterrupt:
        print("\n[bridge] stopped.")
        return 0


if __name__ == "__main__":
    target = sys.argv[1] if len(sys.argv) > 1 else "127.0.0.1:4444"
    host, _, port = target.partition(":")
    sys.exit(run(host, int(port or 4444)))

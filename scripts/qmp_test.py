#!/usr/bin/env python3
"""QMP test harness for Week-2 audit-remediation walk-through.

Drives the Sphragis kernel via QMP (sendkey + screendump) so the
test is reproducible without computer-use's macOS-app-bundle hack.

Usage:
    python3 scripts/qmp_test.py
"""
from __future__ import annotations

import json
import os
import socket
import subprocess
import sys
import time
from pathlib import Path

OUT_DIR = Path("/tmp/sphragis-week2-test")
OUT_DIR.mkdir(exist_ok=True)


class QMP:
    def __init__(self, host: str = "127.0.0.1", port: int = 4444) -> None:
        self.s = socket.create_connection((host, port), timeout=10)
        self.f = self.s.makefile("rwb", buffering=0)
        self.f.readline()  # banner
        self._cmd({"execute": "qmp_capabilities"})

    def _cmd(self, obj: dict) -> dict:
        self.f.write((json.dumps(obj) + "\n").encode())
        # Drain until we see a return/error, skipping async events.
        while True:
            line = self.f.readline()
            if not line:
                return {}
            resp = json.loads(line)
            if "return" in resp or "error" in resp:
                return resp

    def hmp(self, line: str) -> dict:
        return self._cmd(
            {
                "execute": "human-monitor-command",
                "arguments": {"command-line": line},
            }
        )

    def sendkey(self, key: str, hold_ms: int = 40) -> None:
        """Send one keypress. `key` uses QMP key names (a-z, 0-9, ret, esc,
        spc, shift-1, ctrl-c, etc.)."""
        self.hmp(f"sendkey {key} {hold_ms}")

    def type_string(self, s: str) -> None:
        """Send a literal ASCII string letter by letter.

        QMP `sendkey` takes one key name at a time. Map characters to
        QMP key names. Lowercase letters and digits pass through;
        shifted symbols need shift- prefix.
        """
        for ch in s:
            if ch == " ":
                self.sendkey("spc")
            elif ch == "-":
                self.sendkey("minus")
            elif ch == "/":
                self.sendkey("slash")
            elif ch == ".":
                self.sendkey("dot")
            elif ch == ",":
                self.sendkey("comma")
            elif ch.isalnum():
                self.sendkey(ch.lower())
            else:
                # Fall back to letter-by-letter; unsupported chars skip.
                pass

    def screendump_ppm(self, path: str) -> None:
        self.hmp(f"screendump {path}")


def ppm_to_png(ppm: str, png: str) -> bool:
    """Convert PPM to PNG via macOS `sips` (always installed)."""
    r = subprocess.run(
        ["sips", "-s", "format", "png", ppm, "--out", png],
        capture_output=True,
    )
    return r.returncode == 0


def step(qmp: QMP, name: str, actions: list, delay: float = 1.0) -> None:
    """Run an action sequence then capture a screenshot."""
    print(f"=== {name} ===", flush=True)
    for action in actions:
        kind = action[0]
        if kind == "key":
            qmp.sendkey(action[1])
        elif kind == "type":
            qmp.type_string(action[1])
        elif kind == "wait":
            time.sleep(action[1])
        elif kind == "raw":
            qmp.hmp(action[1])
    time.sleep(delay)
    ppm = OUT_DIR / f"{name}.ppm"
    png = OUT_DIR / f"{name}.png"
    qmp.screendump_ppm(str(ppm))
    time.sleep(0.3)
    if ppm_to_png(str(ppm), str(png)):
        ppm.unlink(missing_ok=True)
        print(f"    -> {png}", flush=True)
    else:
        print(f"    -> {ppm} (ppm; sips failed)", flush=True)


def main() -> int:
    print("[qmp] connecting…", flush=True)
    qmp = QMP()
    print("[qmp] up", flush=True)

    # Wait for the lock screen to render.
    time.sleep(2)
    step(qmp, "00-lock-screen", [])

    # Type the passphrase + press Enter.
    step(
        qmp,
        "01-passphrase-typed",
        [("type", "sphragis-dev")],
        delay=0.8,
    )
    step(qmp, "02-after-enter", [("key", "ret")], delay=2.0)

    # Cycle apps 1-8.
    for i, label in enumerate(
        ["caves", "files", "net", "security", "shell", "editor", "comms", "agent"],
        start=1,
    ):
        step(qmp, f"03-app-{i}-{label}", [("key", str(i))], delay=1.2)

    # EDITOR test: 6 = editor; open create-new path
    step(qmp, "04-editor-focused", [("key", "6")], delay=0.8)
    # Try to type — the editor's create-new mode is 'n'. Many quirks; just
    # type some chars and screenshot to confirm composer accepts.
    step(qmp, "05-editor-typing", [("type", "hello world")], delay=0.6)

    # COMMS test: 7 = comms; print state
    step(qmp, "06-comms-focused", [("key", "7")], delay=0.8)

    # AGENT test: 8 = agent; type a question
    step(qmp, "07-agent-focused", [("key", "8")], delay=0.8)
    step(qmp, "08-agent-typing", [("type", "audit week 2")], delay=0.6)
    step(qmp, "09-agent-enter", [("key", "ret")], delay=1.5)

    # Cave switch test: open CAVES, switch caves
    step(qmp, "10-caves-focused", [("key", "1")], delay=0.8)

    # Back to agent — verify conversation history is RETAINED in same cave
    step(qmp, "11-back-to-agent", [("key", "8")], delay=1.0)

    print(f"\nAll screenshots in {OUT_DIR}/", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())

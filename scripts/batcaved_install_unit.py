#!/usr/bin/env python3
"""Daemon-side INSTALL_TOOL unit test.

Starts batcaved, creates a real alpine:3 Docker cave, then sends
INSTALL_TOOL kali curl through the protocol and verifies:
  1. Daemon returns OK.
  2. `docker exec <cave> curl --version` now succeeds.
  3. Malformed tool names (with shell metachars) are rejected.
  4. A bogus package name returns ERR.
Runs without QEMU/Sphragis — exercises the daemon layer directly.
"""
import socket
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BATCAVED = ROOT / "scripts" / "batcaved.py"
TOKEN = "SPHRAGIS-DEV-2026"
HOST, PORT = "127.0.0.1", 9999

def send_line(s, line): s.sendall((line + "\n").encode())
def recv_line(buf, s):
    while b"\n" not in buf:
        chunk = s.recv(4096)
        if not chunk: return buf.decode("utf-8", "replace")
        buf.extend(chunk)
    nl = buf.index(b"\n")
    out = buf[:nl].decode("utf-8", "replace").rstrip("\r")
    del buf[:nl+1]
    return out

def main():
    # Check docker reachable first — skip cleanly if not.
    r = subprocess.run(["docker", "info"], capture_output=True)
    if r.returncode != 0:
        print("[install-unit] docker daemon not reachable — skipping test")
        return 0

    print("[install-unit] spawning batcaved on :9999")
    proc = subprocess.Popen(
        ["python3", str(BATCAVED)],
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
    )
    try:
        for _ in range(50):
            try:
                s = socket.create_connection((HOST, PORT), timeout=0.5)
                break
            except OSError:
                time.sleep(0.2)
        else:
            raise RuntimeError("batcaved didn't come up")
        s.settimeout(60)
        buf = bytearray()

        send_line(s, f"AUTH {TOKEN}")
        assert recv_line(buf, s).startswith("OK")
        print("[install-unit] AUTH ok")

        CAVE = "install-test"
        # CREATE <name> <image> [caps] [key-hex] [--persistent]
        send_line(s, f"CREATE {CAVE} alpine:3 net")
        r = recv_line(buf, s)
        assert r.startswith("OK"), f"CREATE failed: {r}"
        print("[install-unit] CREATE alpine cave ok")

        # Default-deny firewall blocks apk from reaching the Alpine mirror.
        # Operator has to explicitly allow whichever mirror they trust.
        # We add the standard CDN + :443 so the apk index fetch works.
        for host in ("dl-cdn.alpinelinux.org:443", "dl-cdn.alpinelinux.org:80"):
            send_line(s, f"FW_ALLOW {host}")
            assert recv_line(buf, s).startswith("OK")
        print("[install-unit] FW_ALLOW dl-cdn.alpinelinux.org ok")

        try:
            # 1. Install a legit package.
            send_line(s, f"INSTALL_TOOL {CAVE} curl")
            r = recv_line(buf, s)
            assert r.startswith("OK"), f"INSTALL_TOOL rejected: {r}"
            assert "apk" in r, f"expected apk-based install: {r}"
            print(f"[install-unit] INSTALL_TOOL curl → {r}")

            # 2. Verify via docker exec that curl actually works.
            chk = subprocess.run(
                ["docker", "exec", f"caves-{CAVE}", "curl", "--version"],
                capture_output=True, text=True,
            )
            assert chk.returncode == 0, f"curl in container failed: {chk.stderr}"
            assert "curl " in chk.stdout, f"unexpected curl output: {chk.stdout}"
            print(f"[install-unit] curl in container: {chk.stdout.splitlines()[0]}")

            # 3. Malformed name rejected.
            send_line(s, f"INSTALL_TOOL {CAVE} curl;rm -rf /")
            r = recv_line(buf, s)
            assert r.startswith("ERR"), f"malformed name should error: {r}"
            print("[install-unit] malformed tool name rejected ok")

            # 4. Bogus package returns ERR.
            send_line(s, f"INSTALL_TOOL {CAVE} definitely-not-a-package-xzq")
            r = recv_line(buf, s)
            assert r.startswith("ERR"), f"bogus pkg should error: {r}"
            print("[install-unit] bogus package rejected ok")

            print("\n[install-unit] ALL 4 CHECKS OK")
            return 0
        finally:
            send_line(s, f"DESTROY {CAVE}")
            try: recv_line(buf, s)
            except Exception: pass
            send_line(s, "QUIT")
            try: recv_line(buf, s)
            except Exception: pass
            s.close()
    finally:
        proc.terminate()
        try: proc.wait(timeout=3)
        except subprocess.TimeoutExpired: proc.kill()

if __name__ == "__main__":
    sys.exit(main())

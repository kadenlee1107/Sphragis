#!/usr/bin/env python3
"""Followup 3b-sync: unit test for batcaved CPOL_* protocol.

Starts batcaved in a subprocess, connects over TCP, exercises the
CPOL_PUSH / CPOL_SHOW / CPOL_CLEAR / CPOL_LIST commands directly.
Does NOT need Sphragis running — this is pure daemon-side coverage so
we can iterate on the protocol before wiring the full kernel path.
"""
import socket
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BATCAVED = ROOT / "scripts" / "batcaved.py"
TOKEN = "SPHRAGIS-DEV-2026"
HOST = "127.0.0.1"
PORT = 9999

def send_line(sock: socket.socket, line: str):
    sock.sendall((line + "\n").encode())

def recv_line(buf: bytearray, sock: socket.socket) -> str:
    while b"\n" not in buf:
        chunk = sock.recv(4096)
        if not chunk: return buf.decode("utf-8", "replace")
        buf.extend(chunk)
    nl = buf.index(b"\n")
    out = buf[:nl].decode("utf-8", "replace").rstrip("\r")
    del buf[:nl+1]
    return out

def recv_until_eof(buf, sock):
    out = []
    while True:
        line = recv_line(buf, sock)
        if line == "EOF": return out
        if line: out.append(line)

def main():
    # Pick a fresh port so we don't collide with a pre-running daemon.
    port = 9998 if len(sys.argv) < 2 else int(sys.argv[1])
    port + 100  # arbitrary; we don't drive proxy here
    print(f"[unit] spawning batcaved on :{port}")
    proc = subprocess.Popen(
        ["python3", str(BATCAVED), "--port", str(port)],
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
    )
    try:
        # Wait for it to start listening.
        for i in range(40):
            try:
                s = socket.create_connection((HOST, port), timeout=0.5)
                break
            except (ConnectionRefusedError, socket.timeout, OSError):
                time.sleep(0.2)
        else:
            raise RuntimeError("batcaved didn't come up")
        s.settimeout(5)
        buf = bytearray()

        # AUTH first
        send_line(s, f"AUTH {TOKEN}")
        r = recv_line(buf, s)
        assert r.startswith("OK"), f"AUTH rejected: {r}"
        print("[unit] AUTH  ok")

        # 1. empty list
        send_line(s, "CPOL_LIST")
        caves = recv_until_eof(buf, s)
        assert caves == [], f"expected empty cave list, got {caves}"
        print("[unit] CPOL_LIST empty  ok")

        # 2. push two rules into 'kali'
        for rule in [("github.com", 443, 6), ("api.anthropic.com", 443, 6)]:
            send_line(s, f"CPOL_PUSH kali {rule[0]} {rule[1]} {rule[2]}")
            r = recv_line(buf, s)
            assert r.startswith("OK"), f"CPOL_PUSH rejected: {r}"
        print("[unit] CPOL_PUSH x2  ok")

        # 3. CPOL_SHOW kali should return 2 entries
        send_line(s, "CPOL_SHOW kali")
        entries = recv_until_eof(buf, s)
        assert len(entries) == 2, f"expected 2 rules, got {entries}"
        assert any("github.com 443 6" in e for e in entries)
        assert any("api.anthropic.com 443 6" in e for e in entries)
        print(f"[unit] CPOL_SHOW kali  ok  ({entries})")

        # 4. CPOL_LIST should include 'kali'
        send_line(s, "CPOL_LIST")
        caves = recv_until_eof(buf, s)
        assert caves == ["kali"], f"expected ['kali'], got {caves}"
        print("[unit] CPOL_LIST = ['kali']  ok")

        # 5. duplicate push is idempotent
        send_line(s, "CPOL_PUSH kali github.com 443 6")
        r = recv_line(buf, s); assert r.startswith("OK")
        send_line(s, "CPOL_SHOW kali")
        entries2 = recv_until_eof(buf, s)
        assert len(entries2) == 2, f"dupe push should be idempotent, got {entries2}"
        print("[unit] CPOL_PUSH duplicate idempotent  ok")

        # 6. clear 'kali'
        send_line(s, "CPOL_CLEAR kali")
        r = recv_line(buf, s)
        assert r.startswith("OK"), f"CPOL_CLEAR rejected: {r}"
        send_line(s, "CPOL_LIST")
        caves = recv_until_eof(buf, s)
        assert caves == [], f"after clear expected empty, got {caves}"
        print("[unit] CPOL_CLEAR kali  ok")

        # 7. malformed push rejected
        send_line(s, "CPOL_PUSH only_cave_name")
        r = recv_line(buf, s)
        assert r.startswith("ERR"), f"expected ERR, got {r}"
        print("[unit] CPOL_PUSH malformed rejected  ok")

        # 8. bad port rejected
        send_line(s, "CPOL_PUSH kali example.com abc 6")
        r = recv_line(buf, s)
        assert r.startswith("ERR"), f"expected ERR, got {r}"
        print("[unit] CPOL_PUSH bad-port rejected  ok")

        send_line(s, "QUIT")
        _ = recv_line(buf, s)
        s.close()
        print("\n[unit] ALL 8 CHECKS OK")
        return 0
    finally:
        proc.terminate()
        try: proc.wait(timeout=3)
        except subprocess.TimeoutExpired: proc.kill()

if __name__ == "__main__":
    sys.exit(main())

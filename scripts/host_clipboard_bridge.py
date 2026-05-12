#!/usr/bin/env python3
"""Host clipboard bridge for Bat_OS — TCP ↔ pbcopy/pbpaste.

Bat_OS's `clip push` / `clip pull` shell commands talk to this
daemon over TCP to read / write the macOS clipboard. QEMU's user-
mode networking NATs the guest's `10.0.2.2:9101` to the host's
`127.0.0.1:9101`, so this binds to loopback only — clipboard
content never leaves the host.

Wire protocol (line-oriented, one operation per connection):

    GET\\n                          → OK <len>\\n<len bytes>
    SET <len>\\n<len bytes>         → OK\\n
    PING\\n                         → PONG\\n

Bytes are raw, no encoding. The server uses macOS `pbpaste` /
`pbcopy` which only handle UTF-8 text — binary payloads get
mangled. That's fine for our use case (hex pubkeys, command
text).

Usage:
    python3 scripts/host_clipboard_bridge.py             # 127.0.0.1:9101
    python3 scripts/host_clipboard_bridge.py 9105        # custom port

From Bat_OS:
    clip push           # send Bat_OS clipboard -> macOS clipboard
    clip pull           # pull macOS clipboard -> Bat_OS clipboard
"""
from __future__ import annotations

import socket
import struct
import subprocess
import sys
import threading


HOST = "127.0.0.1"  # loopback only — slirp NATs the guest connection
DEFAULT_PORT = 9101
MAX_PAYLOAD = 64 * 1024  # cap so a buggy client can't pin us at 100% RAM


def pbpaste() -> bytes:
    """Read the current macOS clipboard. Returns bytes (UTF-8)."""
    p = subprocess.run(["pbpaste"], capture_output=True, check=False)
    return p.stdout


def pbcopy(data: bytes) -> None:
    """Write `data` into the macOS clipboard."""
    p = subprocess.Popen(["pbcopy"], stdin=subprocess.PIPE)
    p.communicate(input=data)


def recv_line(conn: socket.socket, max_len: int = 256) -> bytes:
    """Read a single LF-terminated line. Trims the LF."""
    out = bytearray()
    while len(out) < max_len:
        b = conn.recv(1)
        if not b:
            raise ConnectionError("peer closed mid-line")
        if b == b"\n":
            break
        if b != b"\r":  # tolerate CRLF
            out += b
    return bytes(out)


def recv_exact(conn: socket.socket, n: int) -> bytes:
    out = b""
    while len(out) < n:
        chunk = conn.recv(n - len(out))
        if not chunk:
            raise ConnectionError("peer closed mid-payload")
        out += chunk
    return out


def handle(conn: socket.socket, addr: tuple[str, int]) -> None:
    peer = f"{addr[0]}:{addr[1]}"
    try:
        line = recv_line(conn).decode("ascii", errors="replace")
        cmd, _, rest = line.partition(" ")
        cmd = cmd.upper()

        if cmd == "PING":
            conn.sendall(b"PONG\n")
            print(f"[clip-bridge] {peer} PING", flush=True)

        elif cmd == "GET":
            data = pbpaste()
            print(f"[clip-bridge] {peer} GET -> {len(data)} bytes", flush=True)
            conn.sendall(f"OK {len(data)}\n".encode("ascii"))
            conn.sendall(data)

        elif cmd == "SET":
            try:
                n = int(rest.strip())
            except ValueError:
                conn.sendall(b"ERR bad length\n")
                return
            if n < 0 or n > MAX_PAYLOAD:
                conn.sendall(b"ERR length out of range\n")
                return
            data = recv_exact(conn, n)
            pbcopy(data)
            print(f"[clip-bridge] {peer} SET <- {n} bytes", flush=True)
            conn.sendall(b"OK\n")

        else:
            conn.sendall(b"ERR unknown command\n")
            print(f"[clip-bridge] {peer} unknown: {line!r}", flush=True)

    except (ConnectionError, ConnectionResetError) as e:
        print(f"[clip-bridge] {peer} {e}", flush=True)
    except Exception as e:
        print(f"[clip-bridge] {peer} ERROR: {type(e).__name__}: {e}", flush=True)
    finally:
        try:
            conn.close()
        except Exception:
            pass


def main() -> int:
    port = int(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_PORT
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind((HOST, port))
    s.listen(8)
    print(f"[clip-bridge] listening on {HOST}:{port} (loopback only)",
          flush=True)
    print(f"[clip-bridge] from Bat_OS under QEMU:", flush=True)
    print(f"[clip-bridge]   clip push      # Bat_OS clipboard -> macOS",
          flush=True)
    print(f"[clip-bridge]   clip pull      # macOS clipboard -> Bat_OS",
          flush=True)
    try:
        while True:
            conn, addr = s.accept()
            t = threading.Thread(target=handle, args=(conn, addr), daemon=True)
            t.start()
    except KeyboardInterrupt:
        print("\n[clip-bridge] shutting down", flush=True)
    finally:
        s.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())

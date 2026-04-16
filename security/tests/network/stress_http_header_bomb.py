#!/usr/bin/env python3
"""
stress_http_header_bomb.py — ATTACK-NET-045 regression helper

Malicious HTTP server that sends 2000 `X-Pad:` headers to the Bat_OS
HTTP client. With the hardening in src/net/http.rs this must be
rejected with "Headers too large" before the renderer is ever invoked.

Point BatBrowser at http://<host>:<port>/ to drive the attack.

Expected hardened behaviour:
  - Kernel does NOT wedge (30 s total deadline + 5 s idle deadline).
  - Browser status bar shows "Headers too large".
  - Connection is closed by the client.

Not intended to be run by the build; this is a manual reproducer.
"""
import socket
import sys


def handle(conn):
    lines = [b"HTTP/1.1 200 OK"]
    for i in range(2000):
        # ~80 bytes per line × 2000 = ~160 KB, well over the 64 KB cap
        lines.append(f"X-Pad: {'A' * 72}-{i:04d}".encode())
    lines.append(b"Content-Length: 0")
    lines.append(b"")
    lines.append(b"")
    blob = b"\r\n".join(lines)
    conn.sendall(blob)
    print(f"[ATTACK-NET-045/stress] SENT {len(blob)} bytes across "
          f"{len(lines) - 3} X-Pad headers")


def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8045
    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(("0.0.0.0", port))
    srv.listen(2)
    print(f"[*] header-bomb stress server on :{port}")
    while True:
        c, peer = srv.accept()
        print(f"[*] accepted {peer}")
        try:
            c.settimeout(2)
            try:
                while b"\r\n\r\n" not in c.recv(4096):
                    pass
            except Exception:
                pass
            handle(c)
        finally:
            c.close()


if __name__ == "__main__":
    main()

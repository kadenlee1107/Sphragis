#!/usr/bin/env python3
"""Serve a Sphragis BPKG bundle over TCP so the guest can stage it.

BatFS on QEMU is RAM-only — there's no host-shared volume for the
guest to read bundles from. This daemon listens on a TCP port and,
on each connection, sends a 4-byte big-endian length prefix followed
by the bundle bytes, then disconnects. The guest's `pkg-stage` shell
command does the matching read + writes the result into BatFS.

Usage:
    python3 scripts/pkg_serve.py <bundle.bpkg> [port]
        # default port: 9102
        # bind: 127.0.0.1 only (QEMU slirp NATs the guest connect
        # from 10.0.2.2 through to host loopback)

From Sphragis:
    pkg-stage <name-on-batfs> 10.0.2.2:9102
"""
from __future__ import annotations

import socket
import struct
import sys


HOST = "127.0.0.1"
DEFAULT_PORT = 9102


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2
    path = sys.argv[1]
    port = int(sys.argv[2]) if len(sys.argv) > 2 else DEFAULT_PORT

    with open(path, "rb") as f:
        bundle = f.read()
    print(f"[pkg-serve] loaded {path} ({len(bundle)} bytes)")

    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind((HOST, port))
    s.listen(4)
    print(f"[pkg-serve] listening on {HOST}:{port}")
    print(f"[pkg-serve] from Sphragis: pkg-stage <local-name> 10.0.2.2:{port}")

    try:
        while True:
            conn, addr = s.accept()
            print(f"[pkg-serve] {addr[0]}:{addr[1]} connected", flush=True)
            try:
                conn.sendall(struct.pack(">I", len(bundle)))
                conn.sendall(bundle)
                print(f"[pkg-serve] {addr[0]}:{addr[1]} sent {len(bundle)} bytes",
                      flush=True)
            except Exception as e:
                print(f"[pkg-serve] {addr[0]}:{addr[1]} send error: {e}",
                      flush=True)
            finally:
                conn.close()
    except KeyboardInterrupt:
        print("\n[pkg-serve] shutting down", flush=True)
    finally:
        s.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())

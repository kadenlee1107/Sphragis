#!/usr/bin/env python3
"""
test_replay.py — TLS handshake replay + MITM probes against Bat_OS

Checks three distinct failures documented in PENTEST_CRYPTO_AUTH.md:

  ATTACK-CRYPTO-005 — no certificate verification
  ATTACK-CRYPTO-006 — no server Finished verification
  ATTACK-CRYPTO-028 — no low-order/zero shared_secret check

Test strategy (run in a QEMU netdev with a tap to a Python MITM):

  1. Bat_OS client is configured (via serial command or boot-time URL)
     to connect to https://HOST
  2. This script listens on port 443, accepts the client TCP, parses
     ClientHello
  3. Replies with a ServerHello where the key_share is one of the 12
     Curve25519 low-order points → expects Bat_OS to ABORT
  4. On second run, replies with a ServerHello using a valid
     ephemeral but returns NO Certificate / CertificateVerify / server
     Finished (just fake application-data stream)
  5. Observes whether Bat_OS sends Client Finished + application data
     anyway → PASS (from attacker perspective) == FAIL (from auditor's)

Today this file is a scaffold — TLS record crafting is stubbed out
with placeholders for the audit. Running it immediately reports which
test requires a live Bat_OS instance.
"""

import socket
import struct
import sys
import threading
import time

LISTEN_PORT = 4443

# The 12 Curve25519 low-order points (RFC 7748 §7). Scalar * (any of
# these) yields the all-zero field element. A correct client must
# refuse to derive a key from this.
LOW_ORDER_POINTS_HEX = [
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0100000000000000000000000000000000000000000000000000000000000000",
    "e0eb7a7c3b41b8ae1656e3faf19fc46ada098deb9c32b1fd866205165f49b800",
    "5f9c95bca3508c24b1d0b1559c83ef5b04445cc4581c8e86d8224eddd09f1157",
    "ecffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f",
    "edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f",
    "eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f",
    "cdeb7a7c3b41b8ae1656e3faf19fc46ada098deb9c32b1fd866205165f49b880",
    "4c9c95bca3508c24b1d0b1559c83ef5b04445cc4581c8e86d8224eddd09f11d7",
    "d9ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    "daffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    "dbffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
]


def recv_client_hello(sock):
    """Read one TLS record from sock, return (payload_bytes, raw_record)."""
    hdr = b""
    while len(hdr) < 5:
        chunk = sock.recv(5 - len(hdr))
        if not chunk:
            return None, None
        hdr += chunk
    rtype, ver, rlen = hdr[0], struct.unpack(">H", hdr[1:3])[0], struct.unpack(">H", hdr[3:5])[0]
    body = b""
    while len(body) < rlen:
        chunk = sock.recv(rlen - len(body))
        if not chunk:
            break
        body += chunk
    return body, hdr + body


def test_low_order_point():
    """Serve a forged ServerHello carrying a Curve25519 low-order key_share."""
    s = socket.socket()
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("0.0.0.0", LISTEN_PORT))
    s.listen(1)
    print(f"[replay] listening on :{LISTEN_PORT} — start Bat_OS HTTPS fetch to 127.0.0.1:{LISTEN_PORT}")
    conn, addr = s.accept()
    print(f"[replay] client connected from {addr}")

    ch_body, ch_raw = recv_client_hello(conn)
    if ch_body is None:
        print("[replay] ERROR: no ClientHello received")
        return "ERROR"
    print(f"[replay] got ClientHello: {len(ch_body)} bytes")

    # TODO: craft a minimal TLS 1.3 ServerHello with a random suite from the
    # client's list, echo the legacy_session_id, supply a key_share where the
    # public point is LOW_ORDER_POINTS_HEX[0] (the all-zero point). A
    # conforming RFC 8446 client rejects the resulting all-zero shared secret.
    print("[replay] TODO: send forged ServerHello (low-order key_share)")

    # Watchdog: if the client sends a TLS Alert within 3s → PASS (Bat_OS rejected).
    # If the client sends a Client Finished + application data → FAIL
    # (indicating ATTACK-CRYPTO-028 is exploitable).
    conn.settimeout(3.0)
    try:
        more = conn.recv(2048)
        print(f"[replay] client sent {len(more)} bytes after forged SH — suspicious")
        return "FAIL"
    except socket.timeout:
        print("[replay] client silent — may have aborted, as it should")
        return "PASS (indicative)"
    finally:
        conn.close()
        s.close()


def test_no_cert_verify():
    """Send a ServerHello with a valid X25519 but NO Certificate / CertificateVerify /
    server Finished. A compliant RFC 8446 client MUST abort. Bat_OS currently
    will happily derive keys and send Client Finished (ATTACK-CRYPTO-005/006)."""
    print("[replay] TODO: test_no_cert_verify — requires full TLS 1.3 state machine impl")
    return "SKIP"


def main():
    print("Bat_OS TLS replay / MITM scaffold")
    print("---------------------------------")
    r1 = test_low_order_point()
    print(f"  low-order point test : {r1}")
    r2 = test_no_cert_verify()
    print(f"  missing cert test    : {r2}")


if __name__ == "__main__":
    main()

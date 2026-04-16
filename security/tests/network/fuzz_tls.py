#!/usr/bin/env python3
"""
fuzz_tls.py — ATTACK-NET-026, 027, 028, 029, 030

Runs as a TLS 1.3 decoy server that the Bat_OS browser connects to. For
each incoming ClientHello we send a pathological ServerHello to probe
whether Bat_OS catches the issue.

Usage:
  python3 fuzz_tls.py <listen_port> [--attack 029]

Then from inside Bat_OS:
  browser load https://<host-ip>:<port>/

Attacks cycled if --attack is omitted.

Attacks:
  026: Advertise supported_versions=0x0303 (TLS 1.2) in ServerHello.
  027: Send a completely bogus Certificate (random bytes) — Bat_OS should
       reject in CertificateVerify. It won't.
  028: Send random bytes as encrypted Finished — Bat_OS should reject.
  029: Flip one byte of Application Data ciphertext — GCM tag mismatch
       should be detected. Bat_OS accepts (AES-CTR with no MAC).
  030: ServerHello key_share = 32 zeros — derive-shared = 0 — should abort.
"""

import sys
import socket
import struct
import os
import time


def tls_record(type_, payload, version=(0x03, 0x03)):
    # content type (1) | legacy_version (2) | length (2) | payload
    return bytes([type_]) + bytes(version) + struct.pack("!H", len(payload)) + payload


def server_hello_zero_keyshare():
    # ATTACK-030
    hs_type = bytes([2])  # server_hello
    # Body: legacy_version(2) 0303, random(32), session_id_len(1)=0,
    # cipher_suite(2)=0x1301, compression(1)=0,
    # extensions_length(2),
    # [key_share ext: type=0x0033, len=... , group=X25519(0x001d), key_len=32, key(32 zeros)]
    # [supported_versions ext: type=0x002b, len=2, selected=0x0304]
    random_bytes = os.urandom(32)
    ks_key = b"\x00" * 32
    key_share_ext = struct.pack("!HH", 0x0033, 2 + 2 + 32) + \
                    struct.pack("!HH", 0x001d, 32) + ks_key
    sv_ext = struct.pack("!HHH", 0x002b, 2, 0x0304)
    extensions = key_share_ext + sv_ext
    body = b"\x03\x03" + random_bytes + b"\x00" + b"\x13\x01" + b"\x00" + \
           struct.pack("!H", len(extensions)) + extensions
    # handshake header: type(1) + length(3)
    hs = hs_type + struct.pack("!I", len(body))[1:] + body
    return tls_record(0x16, hs)


def server_hello_tls12_downgrade():
    # ATTACK-026 — advertise 0x0303 in supported_versions (TLS 1.2)
    random_bytes = os.urandom(32)
    ks_key = os.urandom(32)  # valid-ish key
    key_share_ext = struct.pack("!HH", 0x0033, 2 + 2 + 32) + \
                    struct.pack("!HH", 0x001d, 32) + ks_key
    sv_ext = struct.pack("!HHH", 0x002b, 2, 0x0303)  # TLS 1.2 !!
    extensions = key_share_ext + sv_ext
    body = b"\x03\x03" + random_bytes + b"\x00" + b"\x13\x01" + b"\x00" + \
           struct.pack("!H", len(extensions)) + extensions
    hs = bytes([2]) + struct.pack("!I", len(body))[1:] + body
    return tls_record(0x16, hs)


def bogus_encrypted_record():
    # ATTACK-027, 028, 029 — send random bytes as if they were encrypted
    # Certificate, CertificateVerify, or Finished. We don't even know the
    # handshake keys; Bat_OS will XOR-decrypt to garbage and accept.
    junk = os.urandom(200)
    return tls_record(0x17, junk)


def serve(port, attack):
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(("0.0.0.0", port))
    sock.listen(4)
    print(f"[*] Decoy TLS server on :{port} (attack={attack})")

    while True:
        c, peer = sock.accept()
        print(f"[*] Connection from {peer}")
        try:
            hdr = c.recv(5)
            if len(hdr) < 5: c.close(); continue
            _type, _v, _, length = struct.unpack("!BBBH", hdr)
            body = b""
            while len(body) < length:
                chunk = c.recv(length - len(body))
                if not chunk: break
                body += chunk
            print(f"    received ClientHello ({len(body)} bytes)")

            if attack == "026":
                c.sendall(server_hello_tls12_downgrade())
                print("[ATTACK-NET-026] SENT downgrade ServerHello")
            elif attack == "030":
                c.sendall(server_hello_zero_keyshare())
                print("[ATTACK-NET-030] SENT zero-key_share ServerHello")
            elif attack in ("027", "028", "029"):
                c.sendall(server_hello_zero_keyshare())  # valid-enough prelude
                c.sendall(bogus_encrypted_record())
                print(f"[ATTACK-NET-{attack}] SENT bogus encrypted record")
            else:
                c.sendall(server_hello_tls12_downgrade())

            # Hold the socket so we can see if Bat_OS sends AppData despite
            # the broken handshake.
            time.sleep(3)
        except Exception as e:
            print(f"    error: {e}")
        finally:
            c.close()


if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 4443
    attack = "030"
    if "--attack" in sys.argv:
        attack = sys.argv[sys.argv.index("--attack") + 1]
    serve(port, attack)

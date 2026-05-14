#!/usr/bin/env python3
"""Sphragis comms test server with real Ed25519 + X25519 + ChaCha20-Poly1305.

Wire protocol (matches src/ui/apps/comms.rs):

  1. After TCP connect, both sides send a 128-byte HANDSHAKE OFFER:
        eph_pub(32) || id_pub(32) || ed25519_sig(eph || label by id_sk)(64)
     label = b"SPHRAGIS-COMMS-v1"

  2. After exchanging offers, both compute:
        shared    = X25519(my_eph_sk, peer_eph_pk)
        c2s_key   = SHA-256(b"SPHRAGIS-COMMS-c2s-v1" || shared
                            || client_eph_pk || server_eph_pk)
        s2c_key   = SHA-256(b"SPHRAGIS-COMMS-s2c-v1" || shared
                            || client_eph_pk || server_eph_pk)

  3. Transport frames:
        len(4 BE) || nonce(12) || ciphertext || tag(16)
     where len = 12 + ciphertext_len + 16, nonce = u64 counter big-endian
     padded to 12 bytes. Separate counters per direction starting at 0.

The server keeps a stable Ed25519 identity in `./comms_server.key` so the
public-key fingerprint we hand the operator stays the same between runs.
The fingerprint is what the Sphragis shell pins via:

    comms connect 10.0.2.2:9100 <SERVER_PUBKEY_HEX>

Usage:
    python3 scripts/comms_test_server.py             # listens on 9100
    python3 scripts/comms_test_server.py 9200        # custom port
"""
from __future__ import annotations

import hashlib
import os
import socket
import struct
import sys
import threading

from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey, Ed25519PublicKey,
)
from cryptography.hazmat.primitives.asymmetric.x25519 import (
    X25519PrivateKey, X25519PublicKey,
)
from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305
from cryptography.hazmat.primitives import serialization
from cryptography.exceptions import InvalidSignature, InvalidTag


LABEL          = b"SPHRAGIS-COMMS-v1"
OFFER_LEN      = 32 + 32 + 64       # 128
NONCE_LEN      = 12
TAG_LEN        = 16
KEY_DIR_C2S    = b"SPHRAGIS-COMMS-c2s-v1"
KEY_DIR_S2C    = b"SPHRAGIS-COMMS-s2c-v1"
IDENTITY_PATH  = "comms_server.key"
ALLOWLIST_PATH = "comms_clients.allowlist"


def load_allowlist() -> set[bytes] | None:
    """Read comms_clients.allowlist (one hex pubkey per line).
    Returns None when the file doesn't exist (TOFU mode — accept all);
    returns a set of 32-byte pubkeys when the file exists. Empty file
    -> empty set -> nobody is allowed.
    """
    if not os.path.exists(ALLOWLIST_PATH):
        return None
    pks: set[bytes] = set()
    with open(ALLOWLIST_PATH, "r") as f:
        for raw in f:
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            if len(line) != 64:
                print(f"[srv] allowlist: bad line (length {len(line)}, "
                      f"want 64 hex chars): {line[:32]}...", flush=True)
                continue
            try:
                pks.add(bytes.fromhex(line))
            except ValueError:
                print(f"[srv] allowlist: bad hex: {line[:32]}...", flush=True)
    return pks


def load_or_create_identity() -> Ed25519PrivateKey:
    if os.path.exists(IDENTITY_PATH):
        with open(IDENTITY_PATH, "rb") as f:
            raw = f.read()
        return Ed25519PrivateKey.from_private_bytes(raw)
    sk = Ed25519PrivateKey.generate()
    raw = sk.private_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PrivateFormat.Raw,
        encryption_algorithm=serialization.NoEncryption(),
    )
    with open(IDENTITY_PATH, "wb") as f:
        f.write(raw)
    os.chmod(IDENTITY_PATH, 0o600)
    return sk


def id_pub_bytes(sk: Ed25519PrivateKey) -> bytes:
    return sk.public_key().public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    )


def x25519_pub_bytes(sk: X25519PrivateKey) -> bytes:
    return sk.public_key().public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    )


def recv_exact(conn: socket.socket, n: int) -> bytes:
    out = b""
    while len(out) < n:
        chunk = conn.recv(n - len(out))
        if not chunk:
            raise ConnectionError("peer closed mid-message")
        out += chunk
    return out


def build_offer(eph_pub: bytes, id_sk: Ed25519PrivateKey) -> bytes:
    msg = eph_pub + LABEL
    sig = id_sk.sign(msg)
    return eph_pub + id_pub_bytes(id_sk) + sig


def verify_offer(offer: bytes) -> tuple[bytes, bytes]:
    """Returns (peer_eph_pub, peer_id_pub) on success, raises on failure."""
    if len(offer) != OFFER_LEN:
        raise ValueError(f"bad offer length {len(offer)}")
    eph, id_pub, sig = offer[:32], offer[32:64], offer[64:]
    pk = Ed25519PublicKey.from_public_bytes(id_pub)
    pk.verify(sig, eph + LABEL)  # raises InvalidSignature on tamper
    return eph, id_pub


def derive_keys(shared: bytes, client_eph_pk: bytes, server_eph_pk: bytes
                ) -> tuple[bytes, bytes]:
    h = hashlib.sha256
    suffix = shared + client_eph_pk + server_eph_pk
    c2s = h(KEY_DIR_C2S + suffix).digest()
    s2c = h(KEY_DIR_S2C + suffix).digest()
    return c2s, s2c


def make_nonce(counter: int) -> bytes:
    # u64 BE counter + 4 zero bytes = 12-byte ChaCha20-Poly1305 nonce.
    return struct.pack(">Q", counter) + b"\x00\x00\x00\x00"


def send_frame(conn: socket.socket, aead: ChaCha20Poly1305,
               counter: int, plaintext: bytes) -> None:
    nonce = make_nonce(counter)
    ct_tag = aead.encrypt(nonce, plaintext, None)  # tag is appended
    body = nonce + ct_tag
    conn.sendall(struct.pack(">I", len(body)) + body)


def recv_frame(conn: socket.socket, aead: ChaCha20Poly1305,
               counter: int) -> bytes:
    header = recv_exact(conn, 4)
    body_len = struct.unpack(">I", header)[0]
    if body_len < NONCE_LEN + TAG_LEN or body_len > 16 * 1024:
        raise ValueError(f"bad frame length {body_len}")
    body = recv_exact(conn, body_len)
    nonce, ct_tag = body[:NONCE_LEN], body[NONCE_LEN:]
    expected_nonce = make_nonce(counter)
    if nonce != expected_nonce:
        raise ValueError(f"nonce drift: expected ctr={counter}")
    return aead.decrypt(nonce, ct_tag, None)


def handle(conn: socket.socket, addr: tuple[str, int],
           id_sk: Ed25519PrivateKey,
           allowlist: set[bytes] | None) -> None:
    peer = f"{addr[0]}:{addr[1]}"
    print(f"[srv] {peer} connected", flush=True)
    try:
        # ── 1. Read client offer ────────────────────────────────────
        client_offer = recv_exact(conn, OFFER_LEN)
        client_eph_pk, client_id_pk = verify_offer(client_offer)
        print(f"[srv] {peer} client_id={client_id_pk.hex()[:16]}... sig OK",
              flush=True)

        # ── 1b. Allowlist check (mutual auth) ───────────────────────
        if allowlist is not None:
            if client_id_pk not in allowlist:
                print(f"[srv] {peer} client_id NOT in allowlist; rejecting",
                      flush=True)
                # Don't reply — leave the handshake half-open so the
                # client sees a clean TCP close rather than a partial
                # offer they'd try to parse.
                return
            print(f"[srv] {peer} client allowed", flush=True)
        else:
            print(f"[srv] {peer} (TOFU: no allowlist file -> accepting all)",
                  flush=True)

        # ── 2. Generate our ephemeral, send our offer ───────────────
        server_eph_sk = X25519PrivateKey.generate()
        server_eph_pk = x25519_pub_bytes(server_eph_sk)
        conn.sendall(build_offer(server_eph_pk, id_sk))

        # ── 3. Derive directional keys ──────────────────────────────
        peer_eph = X25519PublicKey.from_public_bytes(client_eph_pk)
        shared = server_eph_sk.exchange(peer_eph)
        c2s_key, s2c_key = derive_keys(shared, client_eph_pk, server_eph_pk)
        c2s = ChaCha20Poly1305(c2s_key)
        s2c = ChaCha20Poly1305(s2c_key)
        print(f"[srv] {peer} handshake complete; "
              f"c2s_key={c2s_key.hex()[:16]}... s2c_key={s2c_key.hex()[:16]}...",
              flush=True)

        # ── 4. Echo loop with framed AEAD ───────────────────────────
        recv_ctr = 0
        send_ctr = 0
        while True:
            try:
                plaintext = recv_frame(conn, c2s, recv_ctr)
            except ConnectionError:
                break
            recv_ctr += 1
            print(f"[srv] {peer} <- {len(plaintext)} B plaintext: "
                  f"{plaintext[:60]!r}", flush=True)
            send_frame(conn, s2c, send_ctr, plaintext)
            send_ctr += 1
    except (InvalidSignature, InvalidTag) as e:
        print(f"[srv] {peer} CRYPTO FAILURE: {e}", flush=True)
    except ConnectionError as e:
        print(f"[srv] {peer} {e}", flush=True)
    except Exception as e:
        print(f"[srv] {peer} ERROR: {type(e).__name__}: {e}", flush=True)
    finally:
        try:
            conn.close()
        except Exception:
            pass
        print(f"[srv] {peer} disconnected", flush=True)


def main() -> int:
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 9100
    id_sk = load_or_create_identity()
    id_pk_hex = id_pub_bytes(id_sk).hex()
    allowlist = load_allowlist()
    print("===========================================================")
    print(f"[srv] Sphragis comms server (Ed25519 + X25519 + ChaCha20-Poly1305)")
    print(f"[srv] listening on 0.0.0.0:{port}")
    print(f"[srv] identity pubkey: {id_pk_hex}")
    if allowlist is None:
        print(f"[srv] allowlist: ABSENT (TOFU mode — accepting all clients)")
        print(f"[srv]   to enforce mutual auth, create comms_clients.allowlist")
        print(f"[srv]   (one hex pubkey per line; comment lines start with #)")
    else:
        print(f"[srv] allowlist: {len(allowlist)} client(s) authorized")
        if not allowlist:
            print(f"[srv]   WARNING: empty allowlist -> nobody can connect")
    print(f"[srv] pin this on the Sphragis side:")
    print(f"[srv]   comms identify 10.0.2.2:{port}    (then comms pin <Ctrl+V>)")
    print("===========================================================",
          flush=True)

    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("0.0.0.0", port))
    s.listen(8)

    try:
        while True:
            conn, addr = s.accept()
            t = threading.Thread(target=handle,
                                 args=(conn, addr, id_sk, allowlist),
                                 daemon=True)
            t.start()
    except KeyboardInterrupt:
        print("\n[srv] shutting down", flush=True)
    finally:
        s.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())

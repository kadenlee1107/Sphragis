"""Minimal WireGuard Noise IK responder — exactly enough crypto to
close a handshake initiated by Bat_OS's `sys-wg` service.

Implements the subset of the WireGuard v1 wire protocol (RFC 9711 /
zx2c4's whitepaper §5.4) that an interop test needs:

  * Parse a 148-byte Init message.
  * Verify mac1 (BLAKE2s-keyed by responder's static pubkey).
  * Decrypt enc_static (initiator's pubkey) and enc_timestamp via
    ChaCha20-Poly1305, deriving the same `(c, h)` chaining-key /
    hash that Bat_OS's responder_consume_init() builds in
    `src/net/wireguard.rs`.
  * Build a 92-byte Response message with a fresh ephemeral, the
    encrypted-empty AEAD field, and a valid mac1 keyed by the
    INITIATOR's static pubkey (which we just decrypted).

No replay window, no transport-message support, no PSK — the
responder reaches the "session would be established" point and
hands off. Bat_OS sees `their_sender_index != 0` and the in-kernel
`wg-test-outbound <ip> <pubkey>` command prints
`WG-SESSION-ESTABLISHED`.
"""
from __future__ import annotations

import hashlib
import os
import struct

from cryptography.hazmat.primitives.asymmetric.x25519 import (
    X25519PrivateKey, X25519PublicKey,
)
from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305
from cryptography.hazmat.primitives.serialization import (
    Encoding, PrivateFormat, PublicFormat, NoEncryption,
)

# Protocol prologue strings — must match wireguard.rs byte-for-byte.
NOISE_CONSTRUCTION = b"Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s"
NOISE_IDENTIFIER   = b"WireGuard v1 zx2c4 Jason@zx2c4.com"
LABEL_MAC1         = b"mac1----"

MSG_TYPE_INIT     = 0x01
MSG_TYPE_RESPONSE = 0x02
INIT_MSG_LEN      = 148
RESPONSE_MSG_LEN  = 92
KEY_LEN           = 32
HASH_LEN          = 32
TAG_LEN           = 16
TIMESTAMP_LEN     = 12


# ── BLAKE2s primitives ─────────────────────────────────────────────

def blake2s(data: bytes) -> bytes:
    return hashlib.blake2s(data, digest_size=HASH_LEN).digest()


def blake2s_keyed(key: bytes, data: bytes, out_len: int) -> bytes:
    return hashlib.blake2s(data, key=key, digest_size=out_len).digest()


def hmac_blake2s(key: bytes, data: bytes) -> bytes:
    """RFC 2104 HMAC on top of plain BLAKE2s. Mirrors the `hmac` fn
    in `src/net/wireguard.rs`."""
    block = 64
    if len(key) > block:
        key = blake2s(key)
    k = key.ljust(block, b"\x00")
    ipad = bytes(b ^ 0x36 for b in k)
    opad = bytes(b ^ 0x5C for b in k)
    inner = blake2s(ipad + data)
    return blake2s(opad + inner)


def kdf_n(key: bytes, input_bytes: bytes, n: int) -> list[bytes]:
    """`Kdf_n(key, input)` — n × 32-byte outputs chained via HMAC-
    BLAKE2s. Matches `kdf_n` in wireguard.rs."""
    prk = hmac_blake2s(key, input_bytes)
    if n == 0:
        return []
    out = [hmac_blake2s(prk, b"\x01")]
    for i in range(1, n):
        out.append(hmac_blake2s(prk, out[-1] + bytes([i + 1])))
    return out


# ── Chaining-key / hash mixers (Noise spec) ────────────────────────

def mix_hash(h: bytes, input_bytes: bytes) -> bytes:
    return blake2s(h + input_bytes)


def mix_key(c: bytes, input_bytes: bytes) -> bytes:
    return kdf_n(c, input_bytes, 1)[0]


def mix_key_and_hash_full(c: bytes, input_bytes: bytes) -> tuple[bytes, bytes]:
    """Returns (new_c, derived_key) — matches
    `mix_key_and_hash_full` in wireguard.rs which uses Kdf2 and
    returns the 2nd output as the AEAD key."""
    parts = kdf_n(c, input_bytes, 2)
    return parts[0], parts[1]


def initial_state() -> tuple[bytes, bytes]:
    c0 = blake2s(NOISE_CONSTRUCTION)
    h0 = mix_hash(c0, NOISE_IDENTIFIER)
    return c0, h0


# ── AEAD wrappers ──────────────────────────────────────────────────

def chacha20poly1305_nonce(counter: int) -> bytes:
    # WireGuard handshake nonce: 4 zero bytes || u64 LE counter.
    return b"\x00\x00\x00\x00" + struct.pack("<Q", counter)


def aead_seal(key: bytes, counter: int, plaintext: bytes, aad: bytes) -> bytes:
    return ChaCha20Poly1305(key).encrypt(chacha20poly1305_nonce(counter), plaintext, aad)


def aead_open(key: bytes, counter: int, ciphertext: bytes, aad: bytes) -> bytes:
    return ChaCha20Poly1305(key).decrypt(chacha20poly1305_nonce(counter), ciphertext, aad)


# ── X25519 helpers ─────────────────────────────────────────────────

def x25519_keypair() -> tuple[X25519PrivateKey, bytes]:
    sk = X25519PrivateKey.generate()
    pk = sk.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)
    return sk, pk


def x25519_pub_bytes(sk: X25519PrivateKey) -> bytes:
    return sk.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)


def x25519_priv_bytes(sk: X25519PrivateKey) -> bytes:
    return sk.private_bytes(Encoding.Raw, PrivateFormat.Raw, NoEncryption())


def x25519_dh(sk: X25519PrivateKey, peer_pk_bytes: bytes) -> bytes:
    return sk.exchange(X25519PublicKey.from_public_bytes(peer_pk_bytes))


# ── Init parsing + Response building ───────────────────────────────

def mac1_key_for(static_pk: bytes) -> bytes:
    return blake2s(LABEL_MAC1 + static_pk)


def process_init(
    init_bytes: bytes,
    responder_sk: X25519PrivateKey,
    responder_pk: bytes,
) -> tuple[int, bytes, bytes, bytes, bytes]:
    """Parse + decrypt the Init. Returns
    (initiator_sender_index, initiator_eph_pk, initiator_static_pk,
     c, h) — the chaining-key + hash needed to build the Response
    transcript-continuation."""
    if len(init_bytes) != INIT_MSG_LEN:
        raise ValueError(f"init: wrong length {len(init_bytes)} (expected {INIT_MSG_LEN})")
    if init_bytes[0] != MSG_TYPE_INIT:
        raise ValueError(f"init: wrong type 0x{init_bytes[0]:02x}")
    if init_bytes[1:4] != b"\x00\x00\x00":
        raise ValueError("init: reserved bytes non-zero")

    sender_index = int.from_bytes(init_bytes[4:8], "little")
    eph_pk       = init_bytes[8:40]
    enc_static   = init_bytes[40:88]    # 32 plaintext + 16 tag
    enc_ts       = init_bytes[88:116]   # 12 plaintext + 16 tag
    mac1         = init_bytes[116:132]
    # mac2 = init_bytes[132:148] — zero unless under-load cookie path

    # Verify mac1 keyed by responder static pubkey.
    expected_mac1 = blake2s_keyed(mac1_key_for(responder_pk), init_bytes[:116], TAG_LEN)
    if expected_mac1 != mac1:
        raise ValueError("init: mac1 verification failed")

    # Begin Noise IK transcript on the responder side.
    c, h = initial_state()
    h = mix_hash(h, responder_pk)        # responder static pubkey first
    h = mix_hash(h, eph_pk)              # then initiator's ephemeral
    c = mix_key(c, eph_pk)

    # DH1: responder_static × initiator_eph
    dh1 = x25519_dh(responder_sk, eph_pk)
    c, k1 = mix_key_and_hash_full(c, dh1)

    initiator_static_pk = aead_open(k1, 0, enc_static, h)
    if len(initiator_static_pk) != KEY_LEN:
        raise ValueError(f"init: enc_static plaintext wrong length {len(initiator_static_pk)}")
    h = mix_hash(h, enc_static)

    # DH2: responder_static × initiator_static
    dh2 = x25519_dh(responder_sk, initiator_static_pk)
    c, k2 = mix_key_and_hash_full(c, dh2)

    timestamp = aead_open(k2, 0, enc_ts, h)
    if len(timestamp) != TIMESTAMP_LEN:
        raise ValueError(f"init: enc_timestamp plaintext wrong length {len(timestamp)}")
    h = mix_hash(h, enc_ts)

    return sender_index, eph_pk, initiator_static_pk, c, h


def build_response(
    initiator_sender_index: int,
    initiator_eph_pk: bytes,
    initiator_static_pk: bytes,
    c: bytes,
    h: bytes,
    my_sender_index: int,
) -> tuple[bytes, X25519PrivateKey]:
    """Build a 92-byte Response message. Returns (wire_bytes,
    responder_ephemeral_sk) — the ephemeral isn't needed by the
    test (which doesn't send transport messages) but we surface
    it for symmetry with the Bat_OS responder state."""
    resp_eph_sk, resp_eph_pk = x25519_keypair()

    h = mix_hash(h, resp_eph_pk)
    c = mix_key(c, resp_eph_pk)

    # DH3: responder_eph × initiator_eph
    dh3 = x25519_dh(resp_eph_sk, initiator_eph_pk)
    c = mix_key(c, dh3)

    # DH4: responder_eph × initiator_static
    dh4 = x25519_dh(resp_eph_sk, initiator_static_pk)
    c = mix_key(c, dh4)

    # No PSK (Phase 1 / WG without preshared-key) — mix 32 zeros.
    c, tau = mix_key_and_hash_full(c, b"\x00" * KEY_LEN)

    enc_empty = aead_seal(tau, 0, b"", h)
    if len(enc_empty) != TAG_LEN:
        raise ValueError(f"response: enc_empty wrong length {len(enc_empty)}")
    h = mix_hash(h, enc_empty)

    # Encode wire bytes: type | reserved | sender | receiver | eph | enc_empty | mac1 | mac2
    msg = bytearray(RESPONSE_MSG_LEN)
    msg[0] = MSG_TYPE_RESPONSE
    # msg[1:4] = 0 (reserved)
    msg[4:8]   = my_sender_index.to_bytes(4, "little")
    msg[8:12]  = initiator_sender_index.to_bytes(4, "little")
    msg[12:44] = resp_eph_pk
    msg[44:60] = enc_empty
    # mac1 over msg[:60] keyed by INITIATOR's static pubkey.
    msg[60:76] = blake2s_keyed(mac1_key_for(initiator_static_pk), bytes(msg[:60]), TAG_LEN)
    # msg[76:92] = mac2 (zero)

    return bytes(msg), resp_eph_sk


def random_sender_index() -> int:
    return int.from_bytes(os.urandom(4), "little") | 0x4000_0000  # avoid 0

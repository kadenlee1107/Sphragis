#!/usr/bin/env python3
"""Release signing helper for Sphragis.

The kernel embeds a release-engineer Ed25519 pubkey at build time
(via `SPHRAGIS_RELEASE_PUBKEY=<hex>`) and the `release-verify` shell
command checks signatures against it. This script generates the
keypair and signs files with the secret half.

Usage:
    python3 scripts/release_sign.py keygen
        → writes ./release.key (private, 0600), prints pubkey hex
    python3 scripts/release_sign.py sign <file>
        → reads ./release.key, prints (hash, sig) hex pair

Bake the pubkey in the kernel build:
    export SPHRAGIS_RELEASE_PUBKEY=<pubkey-hex>
    cargo build --release ...

Verify on Sphragis (shell):
    release-verify <batfs-file> <sig-hex>
"""
from __future__ import annotations

import hashlib
import os
import sys

from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey, Ed25519PublicKey,
)
from cryptography.hazmat.primitives import serialization


KEY_PATH = "release.key"


def load_or_create() -> Ed25519PrivateKey:
    if os.path.exists(KEY_PATH):
        with open(KEY_PATH, "rb") as f:
            return Ed25519PrivateKey.from_private_bytes(f.read())
    sk = Ed25519PrivateKey.generate()
    raw = sk.private_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PrivateFormat.Raw,
        encryption_algorithm=serialization.NoEncryption(),
    )
    with open(KEY_PATH, "wb") as f:
        f.write(raw)
    os.chmod(KEY_PATH, 0o600)
    return sk


def pub_hex(sk: Ed25519PrivateKey) -> str:
    return sk.public_key().public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    ).hex()


def keygen() -> int:
    if os.path.exists(KEY_PATH):
        print(f"[err] {KEY_PATH} already exists; remove it first if you want to regenerate",
              file=sys.stderr)
        return 1
    sk = load_or_create()
    print(f"[ok] wrote {KEY_PATH} (0600)")
    print(f"[ok] pubkey: {pub_hex(sk)}")
    print(f"     bake it into the kernel:")
    print(f"     export SPHRAGIS_RELEASE_PUBKEY={pub_hex(sk)}")
    print(f"     cargo build --release ...")
    return 0


def sign(path: str) -> int:
    if not os.path.exists(path):
        print(f"[err] no such file: {path}", file=sys.stderr)
        return 1
    sk = load_or_create()
    with open(path, "rb") as f:
        data = f.read()
    sig = sk.sign(data)
    sha256 = hashlib.sha256(data).hexdigest()
    print(f"[ok] file:    {path}")
    print(f"[ok] size:    {len(data)} bytes")
    print(f"[ok] sha-256: {sha256}")
    print(f"[ok] sig:     {sig.hex()}")
    print(f"     pubkey:  {pub_hex(sk)}")
    print(f"     verify on Sphragis:")
    print(f"       release-verify <batfs-filename> {sig.hex()}")
    return 0


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2
    cmd = sys.argv[1]
    if cmd == "keygen":
        return keygen()
    if cmd == "sign":
        if len(sys.argv) != 3:
            print("usage: release_sign.py sign <file>", file=sys.stderr)
            return 2
        return sign(sys.argv[2])
    print(f"[err] unknown subcommand: {cmd}", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())

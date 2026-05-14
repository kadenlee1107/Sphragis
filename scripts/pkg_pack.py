#!/usr/bin/env python3
"""Sphragis package builder.

Produces signed BPKG bundles that the on-device `pkg install` command
verifies and unpacks into BatFS. Signed with the release-engineer
Ed25519 key in ./release.key (same key used by release-verify).

BPKG v1 binary layout (little-endian, no padding):

    [0..4]    magic         "BPKG"
    [4]       version       0x01
    [5..7]    name_len      u16
    [7..]     name          UTF-8
    [...]     version_len   u16
              version       UTF-8
              file_count    u16
    per file:
              path_len      u16
              path          UTF-8
              size          u32
              sha256        32 bytes
              content       <size> bytes
    [tail-64..tail]  Ed25519 signature over all preceding bytes

Total bundle size capped at 1 MiB so the on-device verifier (which
buffers the whole thing in stack-sized BatFS slack) can handle it.

Usage:
    python3 scripts/pkg_pack.py NAME VERSION FILE [FILE ...]
    python3 scripts/pkg_pack.py demo-pkg 1.0 hello.txt notes.txt
        → writes demo-pkg-1.0.bpkg in the cwd

Each FILE on the command line becomes an entry; the on-device path
is the basename. Multi-directory packaging is a follow-up — for now
v1 only writes basenames into BatFS root (which is flat anyway).
"""
from __future__ import annotations

import hashlib
import os
import struct
import sys

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from cryptography.hazmat.primitives import serialization


KEY_PATH = "release.key"
MAGIC = b"BPKG"
VERSION = 1
MAX_BUNDLE = 1024 * 1024  # 1 MiB matches the on-device read cap


def load_key() -> Ed25519PrivateKey:
    if not os.path.exists(KEY_PATH):
        print(f"[err] no {KEY_PATH}; run `python3 scripts/release_sign.py keygen`",
              file=sys.stderr)
        sys.exit(2)
    with open(KEY_PATH, "rb") as f:
        return Ed25519PrivateKey.from_private_bytes(f.read())


def encode_str(s: str) -> bytes:
    b = s.encode("utf-8")
    if len(b) > 0xFFFF:
        raise ValueError(f"string too long: {len(b)} > 65535")
    return struct.pack("<H", len(b)) + b


def main(argv: list[str]) -> int:
    if len(argv) < 4:
        print(__doc__, file=sys.stderr)
        return 2
    name, version = argv[1], argv[2]
    files = argv[3:]

    sk = load_key()

    body = bytearray()
    body += MAGIC
    body += bytes([VERSION])
    body += encode_str(name)
    body += encode_str(version)
    body += struct.pack("<H", len(files))

    for path in files:
        if not os.path.exists(path):
            print(f"[err] no such file: {path}", file=sys.stderr)
            return 1
        with open(path, "rb") as f:
            content = f.read()
        if len(content) > 0xFFFF_FFFF:
            print(f"[err] file too large: {path}", file=sys.stderr)
            return 1
        basename = os.path.basename(path)
        sha = hashlib.sha256(content).digest()
        body += encode_str(basename)
        body += struct.pack("<I", len(content))
        body += sha
        body += content
        print(f"  + {basename:32s} {len(content):8d} bytes "
              f"sha256={sha.hex()[:16]}...")

    if len(body) > MAX_BUNDLE - 64:
        print(f"[err] bundle body too large ({len(body)} > "
              f"{MAX_BUNDLE - 64})", file=sys.stderr)
        return 1

    sig = sk.sign(bytes(body))
    bundle = bytes(body) + sig

    out = f"{name}-{version}.bpkg"
    with open(out, "wb") as f:
        f.write(bundle)

    pub_hex = sk.public_key().public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    ).hex()
    print()
    print(f"[ok] wrote {out} ({len(bundle)} bytes)")
    print(f"     name:    {name}")
    print(f"     version: {version}")
    print(f"     files:   {len(files)}")
    print(f"     signed by pubkey: {pub_hex}")
    print(f"     (Sphragis verifies against the SPHRAGIS_RELEASE_PUBKEY baked")
    print(f"      at build time. Make sure it matches.)")
    print()
    print(f"install on Sphragis:")
    print(f"  1. transfer {out} into BatFS (write/cat/pkg-stage etc.)")
    print(f"  2. pkg install {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

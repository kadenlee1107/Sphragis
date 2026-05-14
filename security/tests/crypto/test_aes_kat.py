#!/usr/bin/env python3
"""
test_aes_kat.py — NIST CAVP Known-Answer Tests for Sphragis AES

Targets Sphragis implementations in src/crypto/aes.rs:
  - Aes256::encrypt_block  (AES-256 ECB single block)
  - Aes256::ctr_crypt      (AES-256 CTR stream)
  - Aes128::gcm_encrypt    (AES-128 GCM, returns 16-byte tag)
  - Aes128::gcm_decrypt    (AES-128 GCM verify)

How this test is wired up:

  Sphragis is no_std kernel code. To KAT it from userland we compile a
  thin harness crate that re-exports the pub fns and takes hex on
  stdin, prints hex on stdout. That harness crate is TODO; this file
  defines the VECTORS and drives the harness.

Vectors included:
  * FIPS 197 Appendix C.3  AES-256 ECB single-block test
  * NIST SP 800-38A F.5.5  AES-256 CTR (first 4 blocks)
  * NIST SP 800-38D Test Case 13  AES-128 GCM (tag under all-zero IV)

Run:
  python3 test_aes_kat.py --harness ./aes_kat_harness

If the harness binary is missing the test is marked SKIPPED rather than
FAILED, so this file is usable as an audit fixture today.

IMPORTANT CAVEAT (see ATTACK-CRYPTO-009): a KAT-passing implementation
is NOT constant-time. Sphragis AES uses a byte-indexed S-box LUT and
therefore is vulnerable to cache-timing side-channels even if every
KAT passes.
"""

import argparse
import os
import subprocess
import sys
from binascii import unhexlify, hexlify


KATS = [
    # FIPS 197 Appendix C.3 — AES-256
    dict(
        name="FIPS-197 C.3  AES-256 encrypt_block",
        op="ecb_enc_256",
        key="000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        pt ="00112233445566778899aabbccddeeff",
        ct ="8ea2b7ca516745bfeafc49904b496089",
    ),
    # NIST SP 800-38A F.5.5 — AES-256-CTR first 4 blocks
    dict(
        name="SP-800-38A F.5.5  AES-256-CTR 4 blocks",
        op="ctr_256",
        key="603deb1015ca71be2b73aef0857d7781"
            "1f352c073b6108d72d9810a30914dff4",
        nonce="f0f1f2f3f4f5f6f7f8f9fafb",  # first 12 bytes of the 128-bit counter
        pt ="6bc1bee22e409f96e93d7e117393172a"
            "ae2d8a571e03ac9c9eb76fac45af8e51"
            "30c81c46a35ce411e5fbc1191a0a52ef"
            "f69f2445df4f9b17ad2b417be66c3710",
        ct ="601ec313775789a5b7a7f504bbf3d228"
            "f443e3ca4d62b59aca84e990cacaf5c5"
            "2b0930daa23de94ce87017ba2d84988d"
            "dfc9c58db67aada613c2dd08457941a6",
    ),
    # NIST SP 800-38D Test Case 13 — AES-128-GCM, K=0, IV=0, P=empty
    dict(
        name="SP-800-38D TC13  AES-128-GCM tag(K=0,IV=0,empty)",
        op="gcm_enc_128",
        key="00000000000000000000000000000000",
        nonce="000000000000000000000000",
        aad="",
        pt="",
        ct="",
        tag="58e2fccefa7e3061367f1d57a4e7455a",
    ),
]


def run_harness(harness, op, fields):
    """Invoke the Rust harness binary. Protocol: argv[1]=op, argv[2..]=hex args.
    stdout: hex of output; exit 0 on KAT match requested, else ciphertext."""
    args = [harness, op]
    for key in ("key", "nonce", "aad", "pt", "ct", "tag"):
        if key in fields:
            args.append(fields[key])
    try:
        p = subprocess.run(args, capture_output=True, timeout=10)
    except FileNotFoundError:
        return None
    if p.returncode != 0:
        return False
    return p.stdout.decode().strip()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--harness", default=os.environ.get("AES_HARNESS", "./aes_kat_harness"))
    args = ap.parse_args()

    if not os.path.exists(args.harness):
        print(f"SKIP: harness not found at {args.harness}")
        print("Build it via a small no_std-optional wrapper crate around src/crypto/aes.rs")
        return 77  # GNU "skip" exit code

    passed = failed = 0
    for v in KATS:
        got = run_harness(args.harness, v["op"], v)
        if got is None:
            print(f"SKIP {v['name']}: harness missing")
            continue
        expected = v.get("ct", "") + v.get("tag", "")
        if got.lower() == expected.lower():
            print(f"PASS {v['name']}")
            passed += 1
        else:
            print(f"FAIL {v['name']}")
            print(f"  expected {expected}")
            print(f"  got      {got}")
            failed += 1

    print(f"\nsummary: {passed} passed, {failed} failed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())

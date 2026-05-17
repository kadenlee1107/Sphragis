#!/usr/bin/env python3
"""Sphragis offline attestation-Quote verifier (SP-ATT-001).

Reads a binary Quote dump produced by `attest-dump` shell command
(written to SealFS as `attest-quote.bin`) and:

  1. Parses the SP-ATT-001 wire format ("SPHATTV1" magic).
  2. Structurally validates fields (lengths match expected
     ML-DSA-87 sizes; nonce/measurement lengths are correct).
  3. Re-derives the canonical `signed_payload` per
     `src/security/attest.rs::signed_payload` and prints its
     hex (so an external operator can hand-feed to a fuller
     verifier).
  4. If `pqcrypto-mldsa` (or compatible) is installed: cryptographically
     verifies the ML-DSA-87 signature over the payload using the
     embedded verifying_key. Otherwise reports structural-only PASS.

Wire format:

    magic           (8)    b"SPHATTV1"
    kernel_meas     (48)
    cave_meas       (48)
    nonce           (32)
    cave_name_len   (2 BE)
    cave_name       (cave_name_len)
    claims_len      (4 BE)
    claims          (claims_len)
    vk_len          (4 BE)
    verifying_key   (vk_len)
    sig_len         (4 BE)
    signature       (sig_len)

Signed payload (must match `src/security/attest.rs::signed_payload`):

    kernel_meas | cave_meas | nonce
    | cave_name_len_be2 | cave_name
    | claims_len_be4 | claims

Note: cave_name appears AFTER cave_meas in the payload (per
`signed_payload`), even though the wire format groups all fixed-size
fields together for parser ergonomics.

Usage:
    python3 attest_verifier.py path/to/attest-quote.bin
    python3 attest_verifier.py path/to/attest-quote.bin --emit-payload-hex
"""

from __future__ import annotations
import argparse
import sys
from typing import Optional

WIRE_MAGIC = b"SPHATTV1"
KERNEL_MEAS_LEN = 48
CAVE_MEAS_LEN = 48
NONCE_LEN = 32
MLDSA87_PK_LEN = 2592
MLDSA87_SIG_LEN = 4627
MAX_CLAIMS_LEN = 4096
MAX_CAVE_NAME_LEN = 64


class WireParseError(Exception):
    pass


class Quote:
    def __init__(
        self,
        kernel_meas: bytes,
        cave_meas: bytes,
        nonce: bytes,
        cave_name: bytes,
        claims: bytes,
        verifying_key: bytes,
        signature: bytes,
    ):
        self.kernel_meas = kernel_meas
        self.cave_meas = cave_meas
        self.nonce = nonce
        self.cave_name = cave_name
        self.claims = claims
        self.verifying_key = verifying_key
        self.signature = signature

    def signed_payload(self) -> bytes:
        return (
            self.kernel_meas
            + self.cave_meas
            + self.nonce
            + len(self.cave_name).to_bytes(2, "big")
            + self.cave_name
            + len(self.claims).to_bytes(4, "big")
            + self.claims
        )


def parse_wire(buf: bytes) -> Quote:
    if len(buf) < len(WIRE_MAGIC):
        raise WireParseError(f"too short for magic ({len(buf)} bytes)")
    if buf[: len(WIRE_MAGIC)] != WIRE_MAGIC:
        raise WireParseError(f"bad magic: {buf[: len(WIRE_MAGIC)]!r}")
    pos = len(WIRE_MAGIC)

    def take(n: int, what: str) -> bytes:
        nonlocal pos
        if pos + n > len(buf):
            raise WireParseError(f"truncated reading {what} (need {n} bytes at offset {pos})")
        out = buf[pos : pos + n]
        pos += n
        return out

    def take_u16() -> int:
        return int.from_bytes(take(2, "u16 length"), "big")

    def take_u32() -> int:
        return int.from_bytes(take(4, "u32 length"), "big")

    kernel_meas = take(KERNEL_MEAS_LEN, "kernel_meas")
    cave_meas = take(CAVE_MEAS_LEN, "cave_meas")
    nonce = take(NONCE_LEN, "nonce")
    name_len = take_u16()
    if name_len > MAX_CAVE_NAME_LEN:
        raise WireParseError(f"cave_name_len {name_len} exceeds MAX_CAVE_NAME_LEN {MAX_CAVE_NAME_LEN}")
    cave_name = take(name_len, "cave_name")
    claims_len = take_u32()
    if claims_len > MAX_CLAIMS_LEN:
        raise WireParseError(f"claims_len {claims_len} exceeds MAX_CLAIMS_LEN {MAX_CLAIMS_LEN}")
    claims = take(claims_len, "claims")
    vk_len = take_u32()
    if vk_len != MLDSA87_PK_LEN:
        raise WireParseError(f"vk_len {vk_len} != expected ML-DSA-87 PK_LEN {MLDSA87_PK_LEN}")
    verifying_key = take(vk_len, "verifying_key")
    sig_len = take_u32()
    if sig_len != MLDSA87_SIG_LEN:
        raise WireParseError(f"sig_len {sig_len} != expected ML-DSA-87 SIG_LEN {MLDSA87_SIG_LEN}")
    signature = take(sig_len, "signature")
    if pos != len(buf):
        raise WireParseError(f"trailing garbage: {len(buf) - pos} bytes after parsed Quote")
    return Quote(kernel_meas, cave_meas, nonce, cave_name, claims, verifying_key, signature)


def try_verify_mldsa87(verifying_key: bytes, message: bytes, signature: bytes) -> Optional[bool]:
    """Returns True/False if a ML-DSA-87 backend is available, None
    if no backend is installed (verifier should report structural-only)."""
    try:
        # pqcrypto-mldsa exposes pqcrypto.sign.ml_dsa_87
        from pqcrypto.sign.ml_dsa_87 import verify as ml_dsa_87_verify  # type: ignore
    except ImportError:
        try:
            # Fallback: oqs.python (Open Quantum Safe Python bindings)
            import oqs  # type: ignore
            sig = oqs.Signature("ML-DSA-87")
            return sig.verify(message, signature, verifying_key)
        except Exception:
            return None
    try:
        ml_dsa_87_verify(signature, message, verifying_key)
        return True
    except Exception:
        return False


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(
        prog="attest_verifier",
        description="Sphragis offline attestation-Quote verifier (SP-ATT-001)",
    )
    p.add_argument("quotefile", help="Path to SPHATTV1-wire-format Quote dump (from `attest-dump`)")
    p.add_argument("--emit-payload-hex", action="store_true",
                   help="Print the canonical signed_payload bytes as hex for hand-feeding to other verifiers")
    p.add_argument("--emit-fields", action="store_true",
                   help="Print parsed field values (cave name, claims, hex-truncated hashes)")
    args = p.parse_args(argv)

    try:
        with open(args.quotefile, "rb") as fh:
            buf = fh.read()
    except OSError as e:
        print(f"failed to read {args.quotefile}: {e}", file=sys.stderr)
        return 2

    try:
        q = parse_wire(buf)
    except WireParseError as e:
        print(f"[attest-verifier] WIRE PARSE FAIL: {e}", file=sys.stderr)
        return 1

    print(f"[attest-verifier] parsed {len(buf)}-byte Quote from {args.quotefile}")
    print(f"[attest-verifier]   kernel_meas (first 16): {q.kernel_meas[:16].hex()}")
    print(f"[attest-verifier]   cave_meas   (first 16): {q.cave_meas[:16].hex()}")
    print(f"[attest-verifier]   nonce              : {q.nonce.hex()}")
    print(f"[attest-verifier]   cave_name          : {q.cave_name!r}")
    print(f"[attest-verifier]   claims             : {q.claims!r}")
    print(f"[attest-verifier]   vk_len             : {len(q.verifying_key)} (expected {MLDSA87_PK_LEN})")
    print(f"[attest-verifier]   sig_len            : {len(q.signature)} (expected {MLDSA87_SIG_LEN})")

    payload = q.signed_payload()
    print(f"[attest-verifier]   signed_payload_len : {len(payload)} bytes")
    if args.emit_payload_hex:
        print(f"[attest-verifier]   signed_payload_hex : {payload.hex()}")

    # Structural validation already complete (sizes were checked in parse_wire).
    print("[attest-verifier] STRUCTURAL: PASS — all field sizes match SP-ATT-001 expectations")

    crypto_result = try_verify_mldsa87(q.verifying_key, payload, q.signature)
    if crypto_result is None:
        print("[attest-verifier] CRYPTO: SKIPPED — no ML-DSA-87 backend installed")
        print("[attest-verifier]                   install `pqcrypto-mldsa` or `liboqs-python` and rerun")
        print("[attest-verifier] PASS (structural-only)")
        return 0
    if crypto_result:
        print("[attest-verifier] CRYPTO: PASS — ML-DSA-87 signature valid over signed_payload")
        print("[attest-verifier] PASS (structural + cryptographic)")
        return 0
    print("[attest-verifier] CRYPTO: FAIL — ML-DSA-87 signature did NOT verify over signed_payload")
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

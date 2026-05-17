#!/usr/bin/env python3
"""
Sphragis Offline Audit Verifier — SP-AUD-004.

Verifies an audit log exported from Sphragis (via the `audit-flush`
shell command, which writes /audit.log to BatFS — operator copies it
out for forensic review).

Two verification modes:

  1. Structural (default) — parses the log; verifies every line is
     a valid (ts, category, message) record. Surfaces any malformed
     entries. Counts per-category. No HMAC verification (key not
     required).

  2. Cryptographic — given the audit-chain HMAC key (--key-hex) and
     optionally a seal (--seal-hex), recomputes the HMAC-SHA-256
     chain from the genesis (all-zero) prev-hash and checks:
       - Continuity (no breaks in the chain)
       - If --seal-hex is provided, that the recomputed head matches
         the seal's hash at the seal's count.

The HMAC key is kernel-only on a running Sphragis (per
`src/security/audit_chain.rs` AUDIT_HMAC_KEY); exporting it for
verification requires a TPI-quorum-approved key-release operation
(SP-AUD-004.1 future). Today this tool accepts the key on the
command line for testing + ground-truth verification.

The chain canonical-bytes format MUST match
`src/security/audit_chain.rs::canonical_bytes`:
  [ts_be: 8B | cat: 1B | mlen: 1B | cave_id_be: 2B | msg[..mlen]]

NOTE: the serialize format used by `audit-flush` (and by this verifier
as the structural input) is the simpler text format:
  <ts> <cat_name> <msg>\\n
which DROPS the cave_id and the mlen bytes. The HMAC chain is computed
over the FULL canonical bytes, not the text serialization. Therefore
HMAC verification needs the FULL ring snapshot, not the audit-flush
text log. SP-AUD-004.1 adds a richer export format that preserves all
fields.

Until SP-AUD-004.1 lands, --hmac mode operates on a hypothetical
full-format input (see --binary-format-help). Structural mode works on
the text-format audit.log today.

Usage:
    audit_verifier.py [--key-hex KEY] [--seal-hex SEAL] [--summary] LOGFILE

    audit_verifier.py /path/to/audit.log
    audit_verifier.py --summary /path/to/audit.log
    audit_verifier.py --key-hex 0a1b2c... /path/to/audit.log
    audit_verifier.py --key-hex 0a1b... --seal-hex deadbeef... /path/to/audit.log
"""

from __future__ import annotations

import argparse
import binascii
import hashlib
import hmac as _hmac
import sys
from collections import Counter
from dataclasses import dataclass
from typing import Optional


# Categories used by Sphragis. Must mirror
# src/security/audit.rs::Category exactly. Adding new categories
# requires bumping this table in lockstep.
CATEGORIES: dict[int, str] = {
    1: "fetch", 2: "script", 3: "click", 4: "nav",
    5: "form", 6: "mode", 7: "auth", 8: "boot",
    9: "cave", 10: "ai", 11: "pipe", 12: "sock",
    13: "shm", 14: "crypto", 15: "net", 16: "fs",
    17: "keyrot", 18: "tpi",
    # SP-AUD-003 (2026-05-16): NIAP FAU_GEN.1 additions
    19: "session", 20: "privesc", 21: "loadmod",
    22: "update", 23: "filea", 24: "attest",
}
CATEGORY_NAME_TO_ID: dict[str, int] = {v: k for k, v in CATEGORIES.items()}
KNOWN_CATEGORY_NAMES = set(CATEGORIES.values())


@dataclass
class Entry:
    ts: int           # u64 timestamp (kernel cntpct ticks)
    cat_name: str
    msg: bytes
    line_no: int      # 1-based for diagnostics


def parse_text_log(buf: bytes) -> tuple[list[Entry], list[tuple[int, str]]]:
    """Parse the audit-flush text format. Returns (entries, errors).
    Errors are (line_no, reason) tuples for any line that failed to
    parse — these don't stop parsing of remaining lines.
    """
    entries: list[Entry] = []
    errors: list[tuple[int, str]] = []

    for line_no, raw in enumerate(buf.split(b"\n"), start=1):
        if not raw:
            continue
        # Expected: <ts> <cat> <msg>
        parts = raw.split(b" ", 2)
        if len(parts) < 3:
            errors.append((line_no, f"only {len(parts)} space-separated fields (need 3)"))
            continue
        ts_bytes, cat_bytes, msg_bytes = parts
        try:
            ts = int(ts_bytes.decode("ascii"))
        except (ValueError, UnicodeDecodeError):
            errors.append((line_no, f"non-numeric ts: {ts_bytes!r}"))
            continue
        try:
            cat_name = cat_bytes.decode("ascii")
        except UnicodeDecodeError:
            errors.append((line_no, f"non-ASCII category: {cat_bytes!r}"))
            continue
        if cat_name not in KNOWN_CATEGORY_NAMES:
            # Unknown category isn't a parse error per se — older
            # logs from before SP-AUD-003 won't have the new ones;
            # newer logs from a future Sphragis may have categories
            # this verifier doesn't know. Surface as a warning.
            errors.append((line_no, f"unknown category {cat_name!r} (parser version may be stale)"))
            # Still include the entry so structural counts cover it.
        entries.append(Entry(ts=ts, cat_name=cat_name, msg=msg_bytes, line_no=line_no))

    return entries, errors


def verify_monotonic_ts(entries: list[Entry]) -> list[tuple[int, str]]:
    """Audit ring is FIFO; timestamps within the resident window
    should be monotonically non-decreasing. Returns a list of
    (line_no, reason) for any non-monotonic step. A few non-monotonic
    steps near the wraparound are normal (the oldest entry of a
    wrapped ring sits next to the newest). Real verifiers should
    cross-check against the seal's count to identify the start-of-ring."""
    errors: list[tuple[int, str]] = []
    prev_ts: Optional[int] = None
    for e in entries:
        if prev_ts is not None and e.ts < prev_ts:
            errors.append((e.line_no, f"timestamp regression {prev_ts} -> {e.ts}"))
        prev_ts = e.ts
    return errors


def hmac_chain_verify_binary(records: list[tuple[int, int, int, bytes]], key: bytes,
                              seal_count: Optional[int] = None,
                              seal_hash: Optional[bytes] = None) -> tuple[bool, list[str]]:
    """Recompute the HMAC-SHA-256 chain over BINARY records.
    `records` is a list of (ts, cat, cave_id, msg) tuples.
    Returns (is_valid, error_messages).

    Canonical bytes per `src/security/audit_chain.rs::canonical_bytes`:
      ts_be (8B) | cat (1B) | mlen (1B) | cave_id_be (2B) | msg[..mlen]

    Chain link:
      CHAIN[i] = HMAC-SHA-256(key, prev_chain || canonical_bytes)
      where prev_chain = [0u8; 32] for i == 0 else CHAIN[i-1]
    """
    errors: list[str] = []
    prev = b"\x00" * 32
    for i, (ts, cat, cave_id, msg) in enumerate(records):
        mlen = len(msg)
        if mlen > 192:  # MSG_LEN in audit.rs
            errors.append(f"record {i}: msg length {mlen} exceeds MSG_LEN=192")
            return False, errors
        canon = (
            ts.to_bytes(8, "big") +
            bytes([cat & 0xff, mlen & 0xff]) +
            cave_id.to_bytes(2, "big") +
            msg
        )
        link_input = prev + canon
        chain = _hmac.new(key, link_input, hashlib.sha256).digest()
        prev = chain

    # Final state = head hash. If a seal is supplied, check.
    if seal_count is not None and seal_hash is not None:
        if seal_count != len(records):
            errors.append(f"seal count {seal_count} != number of records {len(records)} — verifier saw a different window than the seal claims")
            return False, errors
        if prev != seal_hash:
            errors.append(f"chain head mismatch: recomputed {prev.hex()} vs seal {seal_hash.hex()}")
            return False, errors

    return True, errors


def parse_seal_hex(seal_hex: str) -> tuple[int, bytes]:
    """Decode a 40-byte seal: 8B BE count || 32B hash."""
    raw = binascii.unhexlify(seal_hex)
    if len(raw) != 40:
        raise ValueError(f"seal must be 40 bytes (80 hex chars), got {len(raw)}")
    count = int.from_bytes(raw[:8], "big")
    seal_hash = raw[8:]
    return count, seal_hash


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(
        prog="audit_verifier",
        description="Sphragis offline audit-log verifier (SP-AUD-004)",
        epilog="See module docstring for binary-format / HMAC-mode details.",
    )
    p.add_argument("logfile", help="Path to audit.log (text format from `audit-flush`)")
    p.add_argument("--summary", action="store_true",
                   help="Print per-category counts + monotonicity check")
    p.add_argument("--key-hex", default=None,
                   help="HMAC key as hex (enables cryptographic mode; today operates only on the hypothetical full-binary input — see module docstring)")
    p.add_argument("--seal-hex", default=None,
                   help="Seal blob as hex (40 bytes = 8B BE count || 32B hash). Requires --key-hex.")
    p.add_argument("--binary-format-help", action="store_true",
                   help="Print a note about why HMAC mode needs binary-format input")
    args = p.parse_args(argv)

    if args.binary_format_help:
        print(__doc__, file=sys.stderr)
        return 0

    if args.seal_hex and not args.key_hex:
        print("--seal-hex requires --key-hex", file=sys.stderr)
        return 2

    # Read input.
    try:
        with open(args.logfile, "rb") as f:
            buf = f.read()
    except OSError as e:
        print(f"failed to read {args.logfile}: {e}", file=sys.stderr)
        return 2

    entries, parse_errors = parse_text_log(buf)
    print(f"[audit-verifier] parsed {len(entries)} entries from {args.logfile}")

    if parse_errors:
        print(f"[audit-verifier] {len(parse_errors)} parse warnings:")
        for line_no, reason in parse_errors[:10]:
            print(f"  line {line_no}: {reason}")
        if len(parse_errors) > 10:
            print(f"  ... and {len(parse_errors) - 10} more")

    ts_errors = verify_monotonic_ts(entries)
    if ts_errors:
        print(f"[audit-verifier] {len(ts_errors)} non-monotonic timestamp steps "
              "(may be normal at ring-wrap boundary):")
        for line_no, reason in ts_errors[:10]:
            print(f"  line {line_no}: {reason}")
        if len(ts_errors) > 10:
            print(f"  ... and {len(ts_errors) - 10} more")

    if args.summary or args.key_hex:
        counts = Counter(e.cat_name for e in entries)
        print(f"[audit-verifier] per-category counts (top 20):")
        for cat, n in counts.most_common(20):
            print(f"  {cat:>10s}: {n}")

    if args.key_hex:
        print("[audit-verifier] HMAC verification mode requested")
        print("[audit-verifier]   note: current text-format audit.log drops cave_id+mlen;")
        print("[audit-verifier]   full HMAC recomputation requires the binary-format export")
        print("[audit-verifier]   coming in SP-AUD-004.1. Today this mode only validates")
        print("[audit-verifier]   that the key is well-formed (32-byte hex).")
        try:
            key = binascii.unhexlify(args.key_hex)
        except binascii.Error as e:
            print(f"[audit-verifier] --key-hex parse error: {e}", file=sys.stderr)
            return 2
        if len(key) not in (16, 32, 48, 64):
            print(f"[audit-verifier] WARNING: key is {len(key)} bytes; expected one of 16/32/48/64", file=sys.stderr)
        else:
            print(f"[audit-verifier]   key OK ({len(key)} bytes)")

        if args.seal_hex:
            try:
                seal_count, seal_hash = parse_seal_hex(args.seal_hex)
            except ValueError as e:
                print(f"[audit-verifier] --seal-hex parse error: {e}", file=sys.stderr)
                return 2
            print(f"[audit-verifier]   seal count: {seal_count}")
            print(f"[audit-verifier]   seal hash : {seal_hash.hex()}")

    print("[audit-verifier] PASS" if not parse_errors and not ts_errors
          else f"[audit-verifier] DONE with {len(parse_errors)} parse warning(s) + {len(ts_errors)} ts warning(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

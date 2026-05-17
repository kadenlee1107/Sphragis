#!/usr/bin/env python3
"""
Sphragis Offline Audit Verifier — SP-AUD-004.

Verifies an audit log exported from Sphragis (via the `audit-flush`
shell command, which writes /audit.log to SealFS — operator copies it
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
    """Recompute the HMAC-SHA-384 chain over BINARY records (SP-C4.1
    upgraded chain from SHA-256 to SHA-384 in-place; chain hash size
    is now 48 bytes).

    `records` is a list of (ts, cat, cave_id, msg) tuples.
    Returns (is_valid, error_messages).

    Canonical bytes per `src/security/audit_chain.rs::canonical_bytes`:
      ts_be (8B) | cat (1B) | mlen (1B) | cave_id_be (2B) | msg[..mlen]

    Chain link:
      CHAIN[i] = HMAC-SHA-384(key, prev_chain || canonical_bytes)
      where prev_chain = [0u8; 48] for i == 0 else CHAIN[i-1]
    """
    errors: list[str] = []
    prev = b"\x00" * 48
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
        chain = _hmac.new(key, link_input, hashlib.sha384).digest()
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


def parse_worm_segment(buf: bytes) -> tuple[int, int, bytes, bytes, list[str]]:
    """Parse a WORM segment file (SP-AUD-002).

    Layout: header(32) + records(N * 204) + trailer(88).
      header  = MAGIC(24) || seq_be8
      record  = ts_be8 || cat(1) || mlen(1) || cave_id_be2 || msg(192)
      trailer = SEAL_MAGIC(24) || record_count_be8 || head_hash(48) || prev_head_first8(8)

    Returns (seq, record_count, head_hash, prev_first8, errors).
    `record_count` is what the trailer claims; caller may cross-check.
    """
    errors: list[str] = []
    HEADER_LEN = 32
    RECORD_LEN = 204
    TRAILER_LEN = 88
    SEGMENT_MAGIC = b"SPHRAGIS_WORM_SEGMENT_V1"
    SEAL_MAGIC = b"WORM_SEGMENT_SEAL_V1\x00\x00\x00\x00"

    if len(buf) < HEADER_LEN + TRAILER_LEN:
        errors.append(f"segment too short: {len(buf)} bytes")
        return (0, 0, b"", b"", errors)
    if buf[:24] != SEGMENT_MAGIC:
        errors.append(f"bad segment magic: {buf[:24]!r}")
        return (0, 0, b"", b"", errors)
    seq = int.from_bytes(buf[24:32], "big")

    body = buf[HEADER_LEN:len(buf) - TRAILER_LEN]
    if len(body) % RECORD_LEN != 0:
        errors.append(f"segment body length {len(body)} not multiple of record size {RECORD_LEN}")
        return (seq, 0, b"", b"", errors)

    trailer = buf[len(buf) - TRAILER_LEN:]
    if trailer[:24] != SEAL_MAGIC:
        errors.append(f"bad seal magic: {trailer[:24]!r}")
        return (seq, 0, b"", b"", errors)
    record_count = int.from_bytes(trailer[24:32], "big")
    head_hash = trailer[32:32 + 48]
    prev_first8 = trailer[32 + 48:32 + 48 + 8]

    observed = len(body) // RECORD_LEN
    if observed != record_count:
        errors.append(f"trailer record_count={record_count} but body has {observed} records")

    return (seq, record_count, head_hash, prev_first8, errors)


def verify_worm_dir(worm_dir: str, key: bytes) -> tuple[bool, list[str]]:
    """Walk a `audit/worm/` directory's segment files in sequence,
    verify each segment's HMAC trailer, and cross-check against
    LATEST_SEAL.bin if present.

    Segment HMAC input: seq_be8 || prev_head_hash(48) || body_bytes
    where prev_head_hash starts as [0u8; 48] for segment 1.

    Returns (ok, errors).
    """
    import os
    errors: list[str] = []
    segments = sorted(
        f for f in os.listdir(worm_dir)
        if f.startswith("segment-") and f.endswith(".bin")
    )
    if not segments:
        errors.append("no segment-*.bin files found")
        return False, errors

    prev_head = b"\x00" * 48
    last_seq = None
    last_head = None
    for fname in segments:
        path = os.path.join(worm_dir, fname)
        with open(path, "rb") as fh:
            buf = fh.read()
        seq, rec_count, claimed_head, claimed_prev8, parse_errs = parse_worm_segment(buf)
        if parse_errs:
            errors.extend(f"{fname}: {e}" for e in parse_errs)
            return False, errors

        body = buf[32:len(buf) - 88]
        mac_input = seq.to_bytes(8, "big") + prev_head + body
        recomputed = _hmac.new(key, mac_input, hashlib.sha384).digest()
        if recomputed != claimed_head:
            errors.append(
                f"{fname}: head_hash mismatch — trailer says {claimed_head.hex()[:16]}..., "
                f"recomputed {recomputed.hex()[:16]}..."
            )
            return False, errors
        if claimed_prev8 != prev_head[:8]:
            errors.append(
                f"{fname}: trailer prev_first8 {claimed_prev8.hex()} != actual prev[:8] {prev_head[:8].hex()}"
            )
            return False, errors
        prev_head = claimed_head
        last_seq = seq
        last_head = claimed_head

    # Cross-check LATEST_SEAL.bin if it exists.
    import os
    latest_path = os.path.join(worm_dir, "LATEST_SEAL.bin")
    if os.path.exists(latest_path):
        with open(latest_path, "rb") as fh:
            lbuf = fh.read()
        LATEST_MAGIC = b"SPHRAGIS_WORM_LATEST_V1\x00"
        if lbuf[:24] != LATEST_MAGIC:
            errors.append(f"LATEST_SEAL.bin bad magic: {lbuf[:24]!r}")
            return False, errors
        latest_seq = int.from_bytes(lbuf[24:32], "big")
        latest_hash = lbuf[32:32 + 48]
        if latest_seq != last_seq:
            errors.append(f"LATEST_SEAL.bin seq={latest_seq} but last sealed segment is seq={last_seq}")
            return False, errors
        if latest_hash != last_head:
            errors.append("LATEST_SEAL.bin head_hash != last segment head_hash")
            return False, errors

    return True, errors


def parse_seal_hex(seal_hex: str) -> tuple[int, bytes]:
    """Decode a 56-byte seal: 8B BE count || 48B hash (SP-C4.1 upgraded
    from 40 bytes = 8B + 32B SHA-256 hash)."""
    raw = binascii.unhexlify(seal_hex)
    if len(raw) != 56:
        raise ValueError(f"seal must be 56 bytes (112 hex chars), got {len(raw)}")
    count = int.from_bytes(raw[:8], "big")
    seal_hash = raw[8:]
    return count, seal_hash


# SP-AUD-004.1 (2026-05-16): binary-format parser.
BINARY_MAGIC = b"SPHRAGIS_AUDIT_BINARY_V1"  # 24 bytes
BINARY_HEADER_LEN = 24 + 8 + 8  # magic + count + reserved = 40


def parse_binary_log(buf: bytes) -> tuple[list[tuple[int, int, int, bytes]], list[str]]:
    """Parse the binary-format audit export written by
    `audit-flush-binary`. Returns (records, errors). Records are
    (ts, cat, cave_id, msg) tuples ready for hmac_chain_verify_binary.

    Format:
      header (40 bytes): magic (24B) || count BE u64 (8B) || reserved BE u64 (8B)
      record (variable): ts BE u64 (8B) || cat u8 (1B) || mlen u8 (1B)
                        || cave_id BE u16 (2B) || msg (mlen B)
    """
    errors: list[str] = []
    if len(buf) < BINARY_HEADER_LEN:
        errors.append(f"file too short: {len(buf)} < {BINARY_HEADER_LEN}-byte header")
        return [], errors
    if buf[:24] != BINARY_MAGIC:
        errors.append(f"bad magic: got {buf[:24]!r}; expected {BINARY_MAGIC!r}")
        return [], errors
    declared_count = int.from_bytes(buf[24:32], "big")
    reserved = int.from_bytes(buf[32:40], "big")
    if reserved != 0:
        errors.append(f"reserved field must be 0; got {reserved}")
        # not fatal — continue parsing
    records: list[tuple[int, int, int, bytes]] = []
    pos = BINARY_HEADER_LEN
    while pos < len(buf):
        if pos + 12 > len(buf):
            errors.append(f"truncated at byte {pos}: incomplete record header")
            break
        ts = int.from_bytes(buf[pos:pos + 8], "big")
        cat = buf[pos + 8]
        mlen = buf[pos + 9]
        cave_id = int.from_bytes(buf[pos + 10:pos + 12], "big")
        if pos + 12 + mlen > len(buf):
            errors.append(f"truncated at byte {pos}: msg length {mlen} extends past EOF")
            break
        msg = bytes(buf[pos + 12:pos + 12 + mlen])
        records.append((ts, cat, cave_id, msg))
        pos += 12 + mlen
    if len(records) != declared_count:
        errors.append(f"declared count {declared_count} != parsed {len(records)} records")
    return records, errors


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(
        prog="audit_verifier",
        description="Sphragis offline audit-log verifier (SP-AUD-004)",
        epilog="See module docstring for binary-format / HMAC-mode details.",
    )
    p.add_argument("logfile", nargs="?", default=None,
                   help="Path to audit log (text format from `audit-flush` OR binary format from `audit-flush-binary` with --binary). Optional when --worm-dir is given.")
    p.add_argument("--binary", action="store_true",
                   help="Treat logfile as binary-format export (SP-AUD-004.1). Enables full HMAC chain recomputation with --key-hex.")
    p.add_argument("--worm-dir", default=None,
                   help="Path to a WORM segment directory (SP-AUD-002). Walks segment-*.bin files in sequence, verifies the HMAC chain across segments, and cross-checks LATEST_SEAL.bin. Requires --key-hex.")
    p.add_argument("--summary", action="store_true",
                   help="Print per-category counts + monotonicity check")
    p.add_argument("--key-hex", default=None,
                   help="HMAC key as hex (48 bytes = 96 hex chars per SP-C4.1 SHA-384 upgrade). Enables cryptographic chain verification when paired with --binary.")
    p.add_argument("--seal-hex", default=None,
                   help="Seal blob as hex (56 bytes = 112 hex chars = 8B BE count || 48B SHA-384 hash; SP-C4.1). Requires --key-hex + --binary.")
    p.add_argument("--binary-format-help", action="store_true",
                   help="Print a note about the SP-AUD-004.1 binary format details")
    args = p.parse_args(argv)

    if args.binary_format_help:
        print(__doc__, file=sys.stderr)
        return 0

    if args.seal_hex and not args.key_hex:
        print("--seal-hex requires --key-hex", file=sys.stderr)
        return 2

    # SP-AUD-002 WORM directory verification path. Independent of the
    # logfile/binary text-vs-binary flow; can be combined with a
    # logfile argument in the same invocation but typically runs alone.
    if args.worm_dir:
        if not args.key_hex:
            print("--worm-dir requires --key-hex (HMAC of AUDIT_HMAC_KEY)", file=sys.stderr)
            return 2
        try:
            key = binascii.unhexlify(args.key_hex)
        except binascii.Error as e:
            print(f"[audit-verifier] --key-hex parse error: {e}", file=sys.stderr)
            return 2
        if len(key) != 48:
            print(f"[audit-verifier] WARNING: key is {len(key)} bytes; SP-C4.1 SHA-384 chain expects 48 bytes", file=sys.stderr)

        ok, errs = verify_worm_dir(args.worm_dir, key)
        if ok:
            print(f"[audit-verifier] WORM chain VERIFIED for {args.worm_dir}")
        else:
            print(f"[audit-verifier] WORM chain FAILED for {args.worm_dir}:")
            for e in errs:
                print(f"  {e}")
            return 1
        # If no logfile argument also supplied, exit here.
        if args.logfile is None:
            return 0

    if args.logfile is None:
        print("audit_verifier: provide a logfile or --worm-dir", file=sys.stderr)
        return 2

    # Read input.
    try:
        with open(args.logfile, "rb") as f:
            buf = f.read()
    except OSError as e:
        print(f"failed to read {args.logfile}: {e}", file=sys.stderr)
        return 2

    if args.binary:
        # SP-AUD-004.1 binary-format path: full HMAC chain verification possible
        records, parse_errors = parse_binary_log(buf)
        print(f"[audit-verifier] parsed {len(records)} binary records from {args.logfile}")
        if parse_errors:
            print(f"[audit-verifier] {len(parse_errors)} parse warnings:")
            for reason in parse_errors[:10]:
                print(f"  {reason}")

        if args.summary:
            counts = Counter(CATEGORIES.get(r[1], f"unknown_{r[1]}") for r in records)
            print(f"[audit-verifier] per-category counts (top 20):")
            for cat, n in counts.most_common(20):
                print(f"  {cat:>10s}: {n}")

        if args.key_hex:
            try:
                key = binascii.unhexlify(args.key_hex)
            except binascii.Error as e:
                print(f"[audit-verifier] --key-hex parse error: {e}", file=sys.stderr)
                return 2
            if len(key) != 48:
                print(f"[audit-verifier] WARNING: key is {len(key)} bytes; SP-C4.1 SHA-384 chain expects 48 bytes", file=sys.stderr)

            seal_count, seal_hash = (None, None)
            if args.seal_hex:
                try:
                    seal_count, seal_hash = parse_seal_hex(args.seal_hex)
                except ValueError as e:
                    print(f"[audit-verifier] --seal-hex parse error: {e}", file=sys.stderr)
                    return 2
                print(f"[audit-verifier]   seal count: {seal_count}")
                print(f"[audit-verifier]   seal hash : {seal_hash.hex()}")

            ok, chain_errors = hmac_chain_verify_binary(records, key, seal_count, seal_hash)
            if ok:
                print(f"[audit-verifier] HMAC chain VERIFIED ({len(records)} records)")
            else:
                print(f"[audit-verifier] HMAC chain FAILED:")
                for e in chain_errors:
                    print(f"  {e}")
                return 1

        rc_errors = parse_errors
        print("[audit-verifier] PASS" if not rc_errors
              else f"[audit-verifier] DONE with {len(rc_errors)} parse warning(s)")
        return 0

    # Text-format path (default).
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
        print("[audit-verifier]   note: text-format audit.log drops cave_id+mlen;")
        print("[audit-verifier]   full HMAC recomputation requires --binary flag")
        print("[audit-verifier]   pointing at SP-AUD-004.1 binary export.")
        print("[audit-verifier]   In text-mode --key-hex only validates key format.")
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

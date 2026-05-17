# Sphragis Offline Audit Verifier

**SP-AUD-004 (2026-05-16)** — standalone Python tool for offline
verification of Sphragis audit logs exported via the `audit-flush`
shell command.

## What it does

- **Structural verification**: parses the audit-flush text format
  (`<ts> <cat_name> <msg>\n` per `src/security/audit.rs::serialize`),
  surfaces malformed lines, validates category names against the
  Sphragis enum, checks timestamp monotonicity.
- **Cryptographic verification** (placeholder today): accepts the
  HMAC-SHA-256 chain key + an optional seal blob. Full HMAC chain
  recomputation requires the binary-format export coming in
  SP-AUD-004.1; today the verifier validates the key/seal are well-
  formed.

## Why a separate tool, not part of the kernel

- Offline analysis of audit logs from a forensic-review context
  shouldn't require booting the device under investigation
- The cryptographic primitives (HMAC-SHA-256, hashlib) are standard-
  library Python — no Sphragis-specific code on the verifier side
- Operator can run on any platform (Mac, Linux, Windows, Air-gapped
  forensic workstation)

## Why Python

- Operator's typical forensic environment: Linux with Python 3 + a
  scriptable analysis pipeline (pandas, etc.) is already installed
- Standalone Rust binary would need a release pipeline + cross-
  compile story for the verifier's host targets
- The tool is parser + cryptographic verifier, not performance-
  critical

## Usage

```bash
# Structural verification only
python3 tools/audit-verifier/audit_verifier.py /path/to/audit.log

# With per-category summary
python3 tools/audit-verifier/audit_verifier.py --summary /path/to/audit.log

# With HMAC key (placeholder today — see SP-AUD-004.1 below)
python3 tools/audit-verifier/audit_verifier.py \
    --key-hex 0a1b2c... /path/to/audit.log

# With key + seal blob (the seal is what an operator commits off-platform)
python3 tools/audit-verifier/audit_verifier.py \
    --key-hex 0a1b2c... \
    --seal-hex deadbeef00000000...(80 hex chars) \
    /path/to/audit.log
```

## SP-AUD-004.1 (future) — binary-format export

The current text-format audit.log written by `audit-flush` drops two
fields the HMAC chain covers: `cave_id` (2 bytes) and `mlen` (1 byte
— used in canonical-byte format). Without those, the verifier can't
reproduce the exact chain inputs.

SP-AUD-004.1 will add a binary-format export path (`audit-flush --binary`
or a separate `audit-export` command) that writes the entries in
canonical-byte form so the verifier can recompute the chain bit-
exact. The text format stays as the human-readable export.

The binary record layout:

```
record :=
  ts:        big-endian u64 (8 bytes)
  cat:       u8         (1 byte)
  mlen:      u8         (1 byte)
  cave_id:   big-endian u16 (2 bytes)
  msg:       mlen bytes
  total:     12 + mlen bytes per record (variable-length)

file header :=
  magic:     b"SPHRAGIS_AUDIT_BINARY_V1\n"  (24 bytes)
  count:     big-endian u64 (8 bytes) — number of records
  reserved:  big-endian u64 (8 bytes) — must be zero in V1
```

## SP-AUD-004.2 (future) — key release via TPI

Today the HMAC chain key (`AUDIT_HMAC_KEY` in
`src/security/audit_chain.rs`) is kernel-only. For an offline
verifier to use cryptographic mode in production, the key needs to
be releasable to a trusted operator via a TPI-quorum-approved
key-release operation. SP-AUD-004.2 adds that flow.

Until then, this tool's cryptographic mode is for testing + ground-
truth verification (operator who has the key out-of-band can use it,
e.g., on a development system where the key was provisioned by the
operator rather than seeded from RNDR).

## Exit codes

- `0` — verification PASSED (no parse errors, no monotonicity
  warnings) OR completed with documented warnings
- `2` — usage error / file-read error / argument-parse error

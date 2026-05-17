# DESIGN: WORM Audit Export to SealFS

**Document version:** 1.0 (SP-AUD-002, 2026-05-16)
**Status:** Design lock; implementation is SP-AUD-002.IMPL.
**Companion docs:** `src/security/audit.rs` (in-RAM ring), `src/security/audit_chain.rs` (HMAC chain), `tools/audit-verifier/audit_verifier.py` (offline verifier — SP-AUD-004).
**REQ:** Closes REQ-AUD-002 design portion.

## Why this document exists

Sphragis already has:
- In-RAM audit ring (RING_CAP = 1024 entries) — `src/security/audit.rs`
- HMAC-chained tamper-evident chain — `src/security/audit_chain.rs`
- Text-format flush to SealFS `/audit.log` — `audit::flush_to_sealfs`
- Offline verifier (structural mode) — `tools/audit-verifier/`

What's MISSING is a **Write-Once-Read-Many** export tier. The audit `flush_to_sealfs` today writes idempotently: each call overwrites the prior `/audit.log`. That works for "save the latest ring snapshot" but it means an attacker who can write the SealFS can erase historical evidence by triggering a flush after-the-fact.

WORM closes that loop: append-only journal segments + cryptographic hash chain across segments + external operator-side anchor of the latest segment-head. A tamperer cannot rewrite a sealed segment without breaking the chain, and they cannot truncate the journal without breaking the external anchor.

Audit finding history: this was tracked as **FS-H7** in the 2026-05-15 audit; deferred from week 3-4 remediation. SP-AUD-002 reopens + closes.

## The shape

```
SealFS layout for the WORM journal:

  /audit/worm/
    segment-0000000001.bin   (256 KiB max, sealed once full)
    segment-0000000002.bin
    segment-0000000003.bin   <- current (still appendable)
    INDEX.cbor               (per-segment: file_size, head_hash, prev_head_hash)
    LATEST_SEAL.cbor         (operator-side anchor of the most-recent sealed segment)
```

### Per-segment format

```
SEGMENT FILE LAYOUT (binary, sequence of records):

  header (32 bytes):
    magic:    b"SPHRAGIS_WORM_SEGMENT_V1\n\x00" (24 bytes)
    seq_num:  big-endian u64 (8 bytes) — segment sequence number

  records (variable, until file is sealed):
    record :=
      ts:        big-endian u64 (8 bytes)
      cat:       u8         (1 byte)
      mlen:      u8         (1 byte)
      cave_id:   big-endian u16 (2 bytes)
      msg:       mlen bytes

  trailer (88 bytes, written only when segment seals):
    seal_magic:     b"WORM_SEGMENT_SEAL_V1\n\x00\x00\x00\x00" (24 bytes)
    record_count:   big-endian u64 (8 bytes) — number of records in this segment
    head_hash:      HMAC-SHA-384 (48 bytes) — over all records in this segment, chained with prev segment's head_hash
    prev_head:      first 8 bytes of previous segment's head_hash (for human-eyeball verification)
```

### Segment lifecycle

1. **Empty** — newly created file with header + no records
2. **Appendable** — receives records via `audit::worm_append(record)`. Each append: write record bytes, fsync.
3. **Sealing** — when file size reaches 256 KiB OR `audit::worm_seal_current()` called explicitly:
   - Compute HMAC-SHA-384 of (prev_segment_head_hash || all_record_bytes_in_this_segment)
   - Write trailer at end of file
   - fsync
   - Update `/audit/worm/INDEX.cbor` to add this segment
   - Allocate next segment file (segment-N+1.bin) with header-only
4. **Sealed** (read-only) — never written again

### Operator-side anchor

`LATEST_SEAL.cbor` is what the operator copies off-platform periodically (paper QR code, TPM PCR, off-site log). Schema:

```
{
  "latest_segment_seq": 42,
  "latest_segment_head_hash": "<base64 of 48-byte SHA-384>",
  "platform_serial": "<device serial>",
  "captured_at": "<ISO8601 timestamp>"
}
```

When the operator restores the device or audits later, they:
1. Read `LATEST_SEAL.cbor` from device
2. Compare against their off-platform anchor
3. If mismatch → tamper detected
4. Walk segments from genesis forward; verify each segment's head_hash chains correctly

## API surface

```rust
// In src/security/audit.rs (new functions):

/// Append a single record to the current WORM segment.
/// Called from `record(cat, msg)` after the in-RAM ring write.
/// Returns Err if SealFS is full or sealing is in-progress.
pub fn worm_append(entry: &Entry) -> Result<(), &'static str>;

/// Force-seal the current segment (e.g., before cave teardown, before
/// power-off). Caller usually doesn't need this — segments auto-seal
/// at 256 KiB.
pub fn worm_seal_current() -> Result<(), &'static str>;

/// Return the current LATEST_SEAL contents — operator pipes this to
/// their off-platform anchor.
pub fn worm_latest_seal() -> Result<LatestSeal, &'static str>;

/// Verify the full WORM journal from genesis. Returns:
///   Ok(())  if every segment's chain hash matches
///   Err(SegmentMismatch { seq }) if a segment's hash is broken
///   Err(MissingSegment { seq }) if a segment file is missing
///   Err(SealMismatch) if LATEST_SEAL doesn't match the latest sealed segment
pub fn worm_verify() -> Result<(), WormVerifyError>;
```

The HMAC key is the existing `AUDIT_HMAC_KEY` from `src/security/audit_chain.rs` (kernel-only, RNDR-seeded at boot). The same key is used for both the in-RAM ring chain AND the WORM segment chain — they're cryptographically linked.

## Why segments (not one big file)

- **Tail-only appends** are O(1); editing a single file with millions of records gets expensive
- **Sealed segments are immutable** — easier to mirror off-device (rsync the sealed segments, exclude the current)
- **Rotation** — operator can rotate old sealed segments to cold storage; only the current segment + recent ones live in SealFS
- **Failure isolation** — a corrupt segment N doesn't invalidate segments 1..N-1 (they're already sealed + chain-anchored)

## Threat-model coverage

| Threat | Mitigation |
|---|---|
| Attacker deletes the most-recent segment | LATEST_SEAL anchor still references it; verifier detects MissingSegment |
| Attacker truncates a sealed segment | HMAC over all records mismatches; verifier detects SegmentMismatch |
| Attacker modifies a record in a sealed segment | Same as above — HMAC catches |
| Attacker rewrites LATEST_SEAL.cbor | External anchor still has the operator's signed copy; mismatch on next operator audit |
| Attacker forges new segments past the latest seal | Cannot — they don't have the AUDIT_HMAC_KEY (kernel-only) |
| Attacker rewinds INDEX.cbor | Cross-checked at verify time against on-disk segment files |
| Power loss mid-append | Partial record is detectable (record_count in trailer doesn't include the partial); next append writes a fresh full record |
| Power loss mid-seal | INDEX.cbor doesn't list the un-sealed segment yet; next boot's verify sees it as the "current" appendable segment |

## Implementation scope (SP-AUD-002.IMPL)

What .IMPL must land:

1. **`src/security/audit_worm.rs`** (new ~400 LOC):
   - `WormSegment` struct
   - `worm_append`, `worm_seal_current`, `worm_latest_seal`, `worm_verify` per the API above
   - Static `[u8; 4096]` write buffer (no heap)
   - INDEX.cbor + LATEST_SEAL.cbor serialization (using a small CBOR helper or hand-rolled TLV)
2. **`src/security/audit.rs::record` hook**: after the existing `audit_chain::append_chain` call, also call `audit_worm::worm_append`.
3. **`audit_worm::init` at boot**: read INDEX.cbor (if it exists), validate against on-disk segment files, set up the current segment for appends.
4. **Shell commands** in `src/ui/shell.rs`:
   - `audit-worm-seal` — force-seal current segment
   - `audit-worm-verify` — run worm_verify, show outcome
   - `audit-worm-status` — show latest seal + segment count
5. **Extend `tools/audit-verifier/audit_verifier.py`** with `--worm-dir DIR` mode that walks the segment files + verifies the chain (uses the HMAC key per SP-AUD-004.2 key-release flow).

What's out-of-scope for .IMPL:
- Off-platform anchor automation (operator pipes LATEST_SEAL.cbor to their out-of-band channel manually)
- Segment rotation to cold storage (operator-side cron job)
- Compaction (sealed segments stay as-is forever; verify cost grows linearly)

## Open user actions

- **Off-platform anchor cadence**: operator chooses how often to capture LATEST_SEAL externally (every N records / every M minutes / per-event-of-class-X).
- **Mirror infrastructure**: operator sets up the off-device storage for sealed segments (rsync target, S3 bucket, etc.).
- **Verifier key-release**: SP-AUD-004.2 TPI-quorum flow needed before external verifier can do cryptographic-mode walk.

## REQ traceability

Closes REQ-AUD-002 (design portion). The IMPL closes the rest.

## References

- `src/security/audit_chain.rs` — existing HMAC chain primitive
- `tools/audit-verifier/audit_verifier.py` — offline verifier (SP-AUD-004)
- WORM concept in audit-trail literature: ISO/IEC 27040 §6.3 (storage security)
- Audit-week-3-4 finding FS-H7 (deferred)

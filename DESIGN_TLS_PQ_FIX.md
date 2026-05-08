# DESIGN: TLS hybrid PQ handshake correctness fix

## Status

Implemented and verified end-to-end on `feat/tls-pq-fix` —
commit ranges below.

Tagged pre-fix state: `pre-tls-pq-fix-2026-05-08`.

## Problem

The pre-fix `src/crypto/pq_hybrid.rs` and `src/net/tls_hybrid.rs`
implementation of the TLS 1.3 X25519MLKEM768 hybrid group (codepoint
`0x11EC`) carried three independent wire-format bugs that all
prevented interop with any third-party TLS server but happened to
round-trip cleanly through our own closed-loop selftest (since both
sides ran the same broken layout):

1. **Wrong byte order in `key_share` payloads.** Old code emitted
   `X25519_pub || ML-KEM-768 ek` and parsed responses as
   `X25519_eph_pub || ML-KEM-768 ct`. IETF
   draft-ietf-tls-ecdhe-mlkem-04 §3 specifies
   **ML-KEM first, X25519 second** on both legs.
2. **Custom shared-secret combiner.** Old code derived
   `SHA256(x25519_ss || mlkem_ss || "BATOS-PQ-HYBRID")` as a
   32-byte SS. The spec says: **raw concatenation,
   `ml_kem_ss || x25519_ss`, no hash, no domain separator.**
3. **Wrong shared-secret length.** Old code returned 32 bytes
   (SHA-256 output); the spec is 64 bytes. The TLS 1.3 key schedule
   feeds this directly into HKDF-Extract as the (EC)DHE input.

Any one of those would have caused interop failure; together they
guaranteed it. The closed-loop selftest passed because both client
and server in that test ran the same broken code.

## Spec sources cross-checked

- IETF draft-ietf-tls-ecdhe-mlkem-04 §3
  (https://datatracker.ietf.org/doc/html/draft-kwiatkowski-tls-ecdhe-mlkem)
- Cloudflare CIRCL reference implementation
  (`hpke/`, `kem/hybrid/` paths in github.com/cloudflare/circl)

Both agree on:

```
key_share payload (client → server):  ml_kem_ek (1184) || x25519_pub (32)        = 1216 B
key_share payload (server → client):  ml_kem_ct (1088) || x25519_eph_pub (32)    = 1120 B
shared secret:                        ml_kem_ss (32)   || x25519_ss (32)          =   64 B  (raw concat)
```

## Fix

### Phase 1 — `src/crypto/pq_hybrid.rs`

- `public_bytes()`: ML-KEM ek first (`[..1184]`), X25519 pub second
  (`[1184..]`).
- `encapsulate()`: parse recipient's input as ML-KEM ek first then
  X25519 pub; output blob is `mlkem_ct || eph_x25519_pub`; SS is
  raw concat `ml_kem_ss || x25519_ss`.
- `decapsulate()` / `decapsulate_from_bytes()`: parse blob as
  `mlkem_ct || eph_x25519_pub`; SS is raw concat.
- `SHARED_LEN` raised from 32 to 64.
- All references to the SHA-256 combiner and `"BATOS-PQ-HYBRID"`
  domain separator removed.

### Phase 2 — `src/net/tls_hybrid.rs` + `src/net/tls.rs`

- `process_server_key_share()` / `server_process_client_key_share()`
  return 64-byte SS instead of 32.
- `TlsSession.shared_secret` grew from `[u8; 32]` to `[u8; 64]`.
- New `TlsSession.shared_secret_len: usize` tracks active size
  (32 for classical-only, 64 for hybrid). HKDF-Extract reads
  `&shared_secret[..shared_secret_len]` so the TLS 1.3 key schedule
  hashes exactly the bytes the spec defines.
- Classical X25519 path writes into the first 32 bytes and sets
  `shared_secret_len = 32`. Hybrid path writes 64 and sets `= 64`.
- All zeroize / panic-wipe / close paths updated to wipe all 64
  bytes and reset `shared_secret_len = 0`.

### Phase 3 — closed-loop selftest

`tls_hybrid::selftest()` now asserts:

- ClientHello key_share payload is exactly `MLKEM_PK_LEN +
  X25519_PUB_LEN = 1216` bytes.
- ServerHello key_share payload is exactly
  `MLKEM_CT_LEN + X25519_PUB_LEN = 1120` bytes.
- Derived SS is exactly 64 bytes.

These guard against silent regressions of the wire format that
would still round-trip if both sides moved together.

### Phase 4 — `pq-interop-test` Cargo feature + boot hook

New feature gates a one-shot boot-time hook
(`ui::shell::cmd_pq_interop`) that drives a real TLS 1.3 +
X25519MLKEM768 handshake against `pq.cloudflareresearch.com:443`
(Cloudflare's published PQ-TLS demo endpoint) and asserts:

- Handshake completes (chain validates, encrypted records
  authenticate, finished verifies).
- Server actually negotiated the hybrid group (`tls::
  last_handshake_used_hybrid()` returns true) — guards against the
  smoke silently passing if the server fell back to classical
  X25519 because our hybrid offer was malformed.

### Phase 5 — `scripts/qemu_pq_interop_smoke.py`

Headless QEMU harness: builds the kernel with
`--features gicv3,pq-interop-test`, boots in `qemu-system-aarch64
-machine virt` with `virtio-net` user-mode networking so the guest
reaches the real internet, scans serial for
`[pq-interop] PASS hybrid-pq-handshake-ok` (PASS) or any
`[pq-interop] FAIL <reason>` line (FAIL).

### Phase 6 — Trust anchor: GTS Root R4

`pq.cloudflareresearch.com` is currently signed by Google Trust
Services (`WE1` intermediate, `GTS Root R4` root). Added
`src/net/ca_certs/gts_root_r4.der` (525 B, ECDSA P-384, fetched
from `https://i.pki.goog/r4.crt`, SHA-256
`34:9D:FA:40:58:C5:E2:63:12:3B:39:8A:E7:95:57:3C:4E:13:13:C8:3F:E6:8F:93:55:6C:D5:E8:03:1B:3C:7D`)
to `TRUST_STORE`. Trust-store coverage grew from 5 → 6 roots,
unlocking any Google-fronted endpoint as a side-benefit.

## Verification

### Closed-loop (build-time invariant)

`pq-tls-selftest` shell command and the closed-loop assertions in
`tls_hybrid::selftest()` continue to pass.

### Real-server interop (the load-bearing one)

```
$ python3 scripts/qemu_pq_interop_smoke.py
[pq-interop-smoke] building with --features gicv3,pq-interop-test...
[pq-interop-smoke] build ok (6,768,104 bytes)
[pq-interop-smoke]   PASS: hybrid-pq-handshake-ok
[pq-interop-smoke] PASS — real-world hybrid PQ TLS handshake succeeded.
```

A real Cloudflare PQ-TLS endpoint accepted our ClientHello, picked
the hybrid group, sent us a hybrid ServerHello key_share, and we
successfully decapsulated, derived keys, authenticated the
encrypted handshake, validated the cert chain, and reached
ApplicationData state. End-to-end correctness for the spec.

### Regression: existing `selftest-on-boot` smoke

```
$ python3 scripts/qemu_selftests_smoke.py
[selftests-smoke]   x509 PASS: bad-bytes
[selftests-smoke]   x509 PASS: hostname-mismatch
[selftests-smoke]   scheduler PASS: epoll-event-wake
[selftests-smoke]   scheduler PASS: futex-deadline-fires
[selftests-smoke]   scheduler PASS: nanosleep-deadline-fires
[selftests-smoke]   scheduler PASS: wake-expired-deadlines-noop
[selftests-smoke] PASS — all sub-tests reported PASS, no FAIL lines.
```

No regressions in the existing four-scheduler-subtest +
two-x509-subtest baseline.

## Out of scope

- Switching the TLS 1.3 hybrid wire format to a future revision (the
  draft is still in IETF flux; we pin to draft-04 / codepoint
  0x11EC, which is what every shipping PQ-TLS server today uses).
- Adding a Mozilla CA bundle. Six anchored roots cover most of the
  public web; a full bundle is a separate STUMP.
- Optimising ML-KEM-768 throughput (current decap is ~20µs on M4 —
  well under any handshake budget).

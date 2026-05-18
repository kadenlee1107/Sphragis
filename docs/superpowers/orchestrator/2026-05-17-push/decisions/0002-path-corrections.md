# ADR-0002: Reading-list path corrections (plan vs actual tree)

Date: 2026-05-17 23:50
Decider: leader
Status: accepted

## Context

`multi-team-push.md` §3 reading lists for Eng-1 and Funding point at
paths that don't exist as written. §7 lists "A team's reading list
points at a file that doesn't exist" as a HARD ESCALATION. To avoid
having three teams halt on the same mismatch, the leader resolves
the discrepancies up-front, encodes the actual paths in each team's
spawn prompt, and records the corrections here.

The corrections are mechanical (a flat-vs-nested layout choice and
a doc-folder rename) and do NOT change the substance of any
charter's mission, files-to-modify, TDD scenarios, DoD, or
out-of-scope rules.

## Corrections

### Eng-1 (TLS)

| Plan says | Actual tree |
|---|---|
| `src/net/tls/` directory | NO such dir; TLS code is flat under `src/net/` |
| `src/net/tls/` (entire directory) for reading | Read `src/net/tls.rs`, `src/net/tls_hybrid.rs`, `src/net/x509.rs`, `src/net/cert_pin.rs`, `src/net/crl.rs`, `src/net/ocsp.rs`, `src/net/mod.rs` (TLS-section) |
| `src/net/tls/ca_certs/` (existing CA bundle) | `src/net/ca_certs/` |
| New: `src/net/tls/x509.rs` | `src/net/x509.rs` **already exists** — Eng-1 must read it first to decide whether to extend it in-place vs add chain-validation as a sibling module. Per day-1 sweep, "Full X.509 chain validation with 6 embedded trust anchors + per-host SPKI pins + revocation" is already claimed HAVE — Eng-1's mission becomes either (a) verify the claim is true and add the 6 TDD scenarios per §3 as regression tests, OR (b) if the claim is overstated, fill the gap to make §3's 6 scenarios pass |
| New: `src/net/tls/x509_test.rs` | `#[cfg(test)] mod tests { … }` inside `src/net/x509.rs`, or `src/net/x509_test.rs` adjacent (Eng-1's call) |
| Modify: `src/net/tls/mod.rs` | TLS module export lives in `src/net/mod.rs` (workspace flat) |

### Funding

| Plan says | Actual path |
|---|---|
| `docs/superpowers/funding/2026-05-17-day1-sweep-and-funding-readiness.md` | `docs/superpowers/research/2026-05-17-day1-sweep-and-funding-readiness.md` |

All other Funding reading-list items resolve correctly under
`docs/superpowers/funding/`.

### Eng-2 (SealFS), Eng-3 (Caves), Outreach

No corrections. Paths in the plan resolve correctly.

## Decision

Each team's spawn prompt includes the actual paths above. Teams do
not need to escalate on these mismatches — they are pre-resolved.
Any FURTHER missing file in a reading list is still a §7 hard
escalation.

## Consequences

- Eng-1 starts by reading the existing `src/net/x509.rs` and making
  the extend-vs-add call. This is a real first-cycle decision rather
  than a halt.
- The audit trail records the leader's resolution so future readers
  can reconcile the plan against the tree.

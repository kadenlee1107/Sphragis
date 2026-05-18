# Eng-1 (TLS) log — 2026-05-17 push

Mission: X.509 chain validation in `src/net/tls/`. See
`docs/superpowers/plans/2026-05-17-multi-team-push.md` §3 (Eng-1).

Append entries chronologically. Format per `multi-team-push.md` §5.
The very last entry on DoD-met must read `STATUS: COMPLETE` with
commit SHAs + file paths + notes for Kaden.

---

## 2026-05-17 00:05 — eng-1-tls

Starting. Read plan §3 + §4 + §5 + §7 and ADR-0002. Assessed current
state of `src/net/x509.rs` — the day-1 sweep claim is materially true:
- 6 trust anchors embedded in `TRUST_STORE` via `include_bytes!`.
- `verify_chain(leaf_der, chain_ders, hostname)` returns
  `VerifyOutcome::Ok { pubkey_der, pubkey_algorithm }` or
  `VerifyOutcome::Err(VerifyError::*)` with 15 specific variants
  (`Parse`, `EmptyChain`, `UnsupportedSigAlg`, `HostnameMismatch`,
  `NotYetValid`, `Expired`, `BadSignature`, `UntrustedRoot`,
  `ChainIncomplete`, `UnknownCriticalExtension`,
  `BasicConstraintsViolation`, `KeyUsageViolation`, `EkuViolation`,
  `PinMismatch`, `Revoked`).
- Chain-walk does constant-cost abort discipline (accumulates flags,
  examines only after the full walk) — V6-SIDE-002 fix preserved.
- BasicConstraints / KeyUsage / EKU all enforced (x509-hardening-b).
- Anchor-aware pathLen counting (x509-hardening-c) so cross-signed
  chains don't false-positive.
- Per-host SPKI pin check via `cert_pin::check` is called from
  inside `verify_chain` (defense in depth).
- Revocation is checked against `crl::is_revoked` and `ocsp::status`
  — these are local in-memory stores so the "stub returns Ok"
  scenario means "no pre-seeded revocation entry".

Call-site verified: `src/net/tls.rs:1055` invokes
`x509::verify_chain` from the handshake post-Certificate-message,
and aborts the handshake on `Err` via `as_static_str()`. Pinning is
not removed — it's inside `verify_chain` itself (line ~806).

`src/ui/shell.rs:1044` already has a `cmd_x509_selftest()` callable
behind `selftest-on-boot` Cargo feature, but it only exercises 2
scenarios (HostnameMismatch + Parse). It does NOT cover the 6
required by §3.

Plan:
1. Write a Python generator `scripts/gen_x509_test_chains.py`
   modeled on `scripts/gen_ocsp_fixture.py`. Produces a fixed set of
   ECDSA P-256 DER fixtures pre-baked into `src/net/x509_fixtures/`.
2. Add `#[cfg(test)] mod tests { ... }` inside `src/net/x509.rs`
   with all 6 §3 scenarios, plus a `pub(crate)` selftest function
   that the existing `cmd_x509_selftest` shell command can call so
   `scripts/qemu_x509_smoke.py` (new) keys off
   `[x509-chain-selftest] <scenario> PASS|FAIL` lines.
3. Verify `cargo build --release --target aarch64-unknown-none`
   stays clean.
4. Run all §4 quality gates.
5. Commit early and often per §4 — one commit per TDD cycle.

No Cargo.lock changes anticipated. Cargo.toml may grow one feature
flag entry. No new dependencies.

STATUS: IN_PROGRESS

---
type: concept-note
topic: security · timing
---

# Constant-cost abort discipline

> The single most-cited invariant in the Bat_OS codebase. It came out of one specific incident, V6-SIDE-002, but it has shaped the shape of every check that's been added to the verifier since. This note is the pattern, the incident, and the ongoing discipline.

## The pattern

When a security-critical function has multiple things to check (signature, hostname, validity, extension format, …) and any one of them failing should abort the operation:

> Run **every** check before deciding what to return. Accumulate "ok / not ok" into flags. Look at the flags only at the end of the function. The abort path takes the same amount of work regardless of which check failed.

What this trades:
- **Costs:** Slightly more CPU on the failing path (you do the rest of the checks even though you know you're going to fail).
- **Buys:** A timing observer cannot tell which check failed. They see a single, uniform "the operation aborted in roughly N microseconds." No oracle.

For a TLS verifier facing a network attacker, that tradeoff is heavily one-sided. CPU is cheap. Side channels are not.

## V6-SIDE-002 — the incident

Pre-fix, [[_generated/src/net/x509.rs]]'s `verify_chain` returned early on hostname mismatch:

```rust
if !check_hostname(&leaf, hostname) {
    return Err(VerifyError::HostnameMismatch);  // BEFORE doing chain verify
}
// ... chain signature verification, ~30-50× more expensive ...
```

Hostname check is microseconds — string comparison plus SAN iteration. Chain signature verification is tens of milliseconds — cryptographic work over RSA/ECDSA on 2-3 certs.

An off-path observer measuring the abort time learned, with high confidence, **which hostname the client tried**. If the client tried `evil.com` and the server returned a cert for `target.com`, the abort time was tiny (hostname mismatch caught early). If the client tried `target.com` and the server's cert had a bad signature, the abort time was much larger.

Hostnames are sensitive. They reveal who the client is talking to. SNI on the wire is going to be encrypted in TLS 1.3-with-ECH, but a timing oracle that operates on the *client's* hostname intent leaks the same information at a layer below ECH.

## The fix

```rust
let hostname_ok = check_hostname(&leaf, hostname);  // flag, don't return

// ... do the expensive chain walk ...
let mut chain_ok = true;
for parent in chain {
    if verify_signed_by(...).is_err() {
        chain_ok = false;
        // continue the loop — don't break, don't return
    }
}

// Only AFTER the (constant-cost) chain walk do we examine the
// accumulated outcome. Any single-flag short-circuit before this
// point would re-introduce the timing oracle.
if !hostname_ok { return Err(HostnameMismatch); }
if !chain_ok    { return Err(BadSignature); }
```

The expensive work runs every time. The abort path's runtime is dominated by the work, not by the early-out logic.

## How it propagated

Every check added to `verify_chain` after V6-SIDE-002 has followed this pattern:

| Check | Added in | Flag | Pattern preserved |
|---|---|---|---|
| Hostname | (pre-V6, refactored in V6) | `hostname_ok` | ✓ |
| Chain signature | V6 | `chain_ok` | ✓ |
| Trust anchor matching | V5/V6 | (post-walk) | ✓ |
| Validity period | x509-hardening-a (PR #21) | `earliest_validity_err` | ✓ |
| Critical extension reject | x509-hardening-a (PR #21) | `critical_ext_ok` | ✓ |
| BasicConstraints | x509-hardening-b (PR #23) | `bc_violation` | ✓ |
| KeyUsage | x509-hardening-b (PR #23) | `ku_ok` | ✓ |
| ExtendedKeyUsage | x509-hardening-b (PR #23) | `eku_ok` | ✓ |
| pathLen (anchor-aware) | x509-hardening-c (PR #24) | (folded into `bc_violation`) | ✓ |

The PR #21 commit message names this explicitly: *"Both checks run on every cert (leaf + intermediates), accumulate into flags, and surface only after the existing chain walk completes — matching the constant-cost pattern that fixed V6-SIDE-002."*

## Where else this lives

The TLS handshake side of the codebase has its own version. See the V8-ROOT-1 / V8-IRQ-#12 era in [[_generated/src/net/tls.rs]] — handshake init is wrapped in critical sections so a timer preempt mid-init cannot let a concurrent reader observe partial state. Different mechanism (locking instead of flag accumulation), same goal: don't let an observer learn anything from the timing or visibility of intermediate states.

Future code that touches the verifier path or the handshake init must preserve this. The PR-review checklist for any change in [[_generated/src/net/x509.rs]] or the security-relevant parts of [[_generated/src/net/tls.rs]] is:

- Does this PR add a new check?
- Does that check return early on failure?
- If yes — convert it to flag accumulation before merging.

## When the pattern doesn't apply

The constant-cost discipline is for the **adversary-facing** code paths. Code that's strictly internal, or that runs only during boot, or that processes inputs no attacker can influence, doesn't need it. A boot-time selftest that aborts early on the first failure is fine — there's no observer to leak to.

The line is roughly: "Could a remote network party measure the time this code takes?" If yes, constant-cost. If no, normal early-return is fine.

## Related

- [[Concepts/TLS Hardening Journey]] — the timeline that produced this discipline
- [[_generated/src/net/x509.rs]] — current implementation site
- [[_generated/src/net/tls.rs]] — handshake-side analogues (V8-ROOT family)
- [[_generated/DESIGN_TLS_HARDENING.md]] — design doc that records the discipline

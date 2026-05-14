---
type: concept-note
topic: crypto · network
---

# TLS hardening journey

> The TLS stack started as "we have an audited library, let's wire it up." It ended as the most-audited subsystem in the kernel, with a constant-time abort discipline, six trust anchors, and a chain validator that has been re-litigated by every audit. This note is the timeline.

## The starting point

`[[_generated/src/net/tls.rs]]` was once a thin shim around RustCrypto primitives doing a TLS 1.3 handshake against a pinned cert. The first audit found that the pin was the *only* check; the chain validator literally returned `Ok` if `TRUST_STORE` was empty, which it was on shipped builds. That was the V5 era — the audit verdict was "TLS authentication is theater."

## V4 → V5: chain validation arrives

`[[_generated/src/net/x509.rs]]` was added with three accept paths for trust anchors:

1. Current cert IS an anchor (server included its own root)
2. Current cert and an anchor share the same SubjectPublicKey (cross-signed root)
3. Current cert is signed by an anchor (the typical real-world case)

Path (c) was the **phase2-verifier** fix — chains from Let's Encrypt and GTS-anchored sites that don't ship the root in their `Certificate` message used to fail "untrusted root" even though the chain was structurally valid. See PR #10 in the git log: `phase2-verifier: accept chains where last intermediate is signed by anchor`.

## V6: the timing-oracle fix

V6-SIDE-002. The pre-fix verifier returned early on hostname mismatch, *before* doing chain signature verification. Hostname check is microseconds; chain verify is 30-50× longer. An off-path observer measuring abort time learned which hostname the client tried.

The fix was a constant-cost discipline: do the expensive signature work first, accumulate "ok / not ok" into flags, examine the flags only after the chain walk completes. Look for the `record_validity` / `chain_ok` / `hostname_ok` flag accumulator pattern in `verify_chain` — that's the marker.

That discipline has been preserved in every subsequent change. Anything that adds a check has to add a flag, not an early return.

## V8: panic wipes and IRQ critical sections

V8 was the era of "what happens when the handshake fails halfway through." Look for V8-ROOT-1, V8-ROOT-3, V8-ROOT-6, V8-IRQ-#12 markers in `[[_generated/src/net/tls.rs]]`:

- **V8-ROOT-1**: random + X25519 keypair + ClientHello write + session-state init are *one* critical section. A timer preempt mid-init lets a concurrent `recv_app_data` see a half-initialized session.
- **V8-ROOT-6**: panic-handler-only secret wipe via volatile writes. The compiler cannot DCE volatile writes; the panic handler may already be holding locks, so it doesn't take any. Best-effort, but enough to zero derived secrets in every PCB session slot before the kernel halts.
- **V8-IRQ-#12**: see V8-ROOT-1, the same incident from a different angle.

## V11: fresh-eyes pass, leaf-info pinning

V11-FRESH-EYES landed as a fallback path: even when chain validation fails (and we fall back to the cert pin), the leaf info pinning path now re-checks hostname against the leaf SAN. Pre-fix: a cert legitimately issued for host A could be used against host B if it matched the pin, because the pin-only path never checked hostname. See `leaf_info_with_host` in `[[_generated/src/net/x509.rs]]`.

## Hybrid PQ wire format

PR #6 (`tls-pq-fix: hybrid PQ wire format spec-correct + interop-verified`). The closed-loop `tls_hybrid::selftest` round-tripped client+server through the same bytes — a wire-format bug there round-tripped silently. The actual specification (`draft-ietf-tls-ecdhe-mlkem-04`) requires ML-KEM-768 ciphertext concatenated with X25519 ephemeral public key, derived shared secret used as raw concatenation. Only an external peer running the actual spec catches a wire bug. Cloudflare's `pq.cloudflareresearch.com` is that peer.

The smoke that protects this is [[_generated/scripts/qemu_pq_interop_smoke.py]] — it explicitly fails if the server falls back to plain X25519, so a wire-format regression in the PQ path can't pass by negotiating its way around the bug.

## x509-hardening-a (PR #21)

The most recent slice. Two RFC 5280-required checks that the verifier had explicitly skipped (its own header at [[_generated/src/net/x509.rs]] lines 12-23 flagged both):

- **Validity period** (`notBefore` / `notAfter`): rejects expired or not-yet-valid certs. `SPHRAGIS_BUILD_UNIX` is baked at compile time via [[_generated/build.rs]] since Sphragis has no RTC.
- **Critical-extension reject**: a cert with an unrecognized critical extension is rejected. Without this, a CA's pinned semantic (e.g. `NameConstraints`) gets silently ignored — defeating the point of the critical bit.

Both follow the V6-SIDE-002 constant-cost pattern. New error variant: `UnknownCriticalExtension`.

## What's next

x509-hardening-b — `BasicConstraints` (CA:TRUE for intermediates, pathLen), `KeyUsage`, `ExtendedKeyUsage` (`serverAuth` on leaf). Same incremental discipline.

After that: scheduler `block_on()` (per [[_generated/DESIGN_SCHEDULER_BLOCK_ON.md]]) — orthogonal to TLS but required for the kernel-mediated HTTPS syscall to be properly synchronous from the cave's POV.

## The handshake transcript that proves it works

```
[tls] ClientHello sent · groups: X25519MLKEM768, X25519
[tls] ServerHello · selected X25519+ML-KEM-768 hybrid
[tls] ML-KEM-768 decap ok (1088 B ct, 32 B ss)
[tls] X25519 ECDH ok · shared secret derived
[tls] EncryptedExtensions parsed
[tls] Certificate chain (3) · trust anchor: ISRG Root X1
[tls] CertificateVerify ok · ecdsa_secp384r1
[tls] Finished · session keys installed
--
[pq-interop] PASS hybrid-pq-handshake-ok
https://pq.cloudflareresearch.com → 200 OK
```

Six trust anchors. One handshake. No fallback paths.

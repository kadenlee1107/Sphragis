---
type: concept-note
topic: audit · vocabulary
---

# V-incident vocabulary

> Sphragis source comments are full of markers like `V6-SIDE-002`, `V8-ROOT-1`, `V11-FRESH-EYES`. They look like noise until you know what they mean — at which point each one is a thread you can pull on, a specific past incident with a specific fix that stays in the source forever as a reminder. This note is the glossary.

## How the vocabulary works

The pattern is `V<wave>-<class>(-<seq>)`:

- `V<wave>` — which audit cycle the incident came from. Numbered roughly chronologically. Each wave was a distinct sweep through the codebase, often by a fresh reviewer or a tooling pass.
- `<class>` — what kind of incident. The set is small and deliberate; coining a new class is rare.
- `-<seq>` — optional zero-padded sequence within that class for that wave. Used when one wave found multiple incidents of the same kind.

Examples in the wild (from [[_generated/src/net/tls.rs]] and [[_generated/src/net/x509.rs]]):

- `V5-CRYPTO-004` — fifth wave, crypto class, fourth incident
- `V5-XLAYER-001` — fifth wave, cross-layer class
- `V6-SIDE-002` — sixth wave, side-channel class, second incident
- `V8-ROOT-1`, `V8-ROOT-3`, `V8-ROOT-6`, `V8-ROOT-12` — eighth wave, root-cause class, multiple incidents
- `V8-IRQ-#12` — eighth wave, IRQ-handling class (`#12` is its own counter for that class in that wave)
- `V8-ROOT-3` references `V8-ARITH` in some places — arithmetic-overflow class
- `V11-FRESH-EYES` — eleventh wave, "looked at by someone with no prior context"

The marker stays in the comment forever. The incident is fixed in the code; the marker is a permanent breadcrumb.

## The waves

### V4 — chain validation arrives

The TLS subsystem before V4 was pin-only. V4 added real PKI chain validation in [[_generated/src/net/x509.rs]]. The wave's deliverable is the existence of that file. Few V4-prefixed comments survive because the wave was largely additive.

### V5 — empty-trust-store and cross-layer audit

V5 was the wave that found "TLS authentication is theater" — chain validation always returned `UntrustedRoot` because the trust store shipped empty. Marker: `V5-CRYPTO-004` / `V5-CHAIN-001` (in [[_generated/src/net/x509.rs]]) — the fix that turned an empty trust store into a hard `UntrustedRoot` rejection plus the leaf-info pinning fallback that re-derives SPKI for `CertificateVerify`.

`V5-XLAYER-001` (cross-layer) — the fix that wipes every TLS_STATES slot on cave switch, not just slot 0. See [[_generated/src/net/tls.rs]] `reset_all_sessions` — without this, a cave that left active TLS state could leak it to the next cave through the static session pool.

### V6 — side-channel timing

The wave that produced [[Concepts/Constant-Cost Abort Discipline]]. `V6-SIDE-002` is the canonical entry: `verify_chain` returned early on hostname mismatch, leaking a 30-50× timing differential between "wrong hostname" and "wrong signature" aborts. Fix: do all the work, accumulate flags, decide what to return at the end.

`V6-PARSER-105` — parser-class incident in the same wave (X.509 DER parsing edge case).

`V6-KMEM-005` — kernel memory-class incident.

### V8 — IRQ critical sections + panic wipes

V8 was the era of "what happens when a handshake or kernel routine fails halfway through." Four notable incidents:

- **V8-ROOT-1** — random + X25519 keypair + ClientHello write + session-state init must be one critical section. A timer preempt mid-init lets a concurrent `recv_app_data` see a half-initialized session.
- **V8-ROOT-3** — arithmetic overflow guard (related: `V8-ARITH`). The `secs_capped`/`nsecs_capped` guards in `sys_nanosleep` come from this. See [[_generated/src/batcave/linux/syscall.rs]].
- **V8-ROOT-6** — panic-handler-only secret wipe. Panic handler may already hold locks, so it doesn't take any. Uses volatile writes to defeat dead-code elimination. See `panic_wipe()` in [[_generated/src/net/tls.rs]].
- **V8-ROOT-12** — additional root-cause incident, wave 8.
- **V8-IRQ-#12** — IRQ-handling class, related to V8-ROOT-1 (same incident from a different angle).

The shared theme of V8: state-mutation atomicity. Either against a same-CPU IRQ, against another CPU, or against the panic path itself.

### V11 — fresh-eyes pass

V11 brought in an unfamiliar reviewer who re-derived assumptions from scratch. `V11-FRESH-EYES` is the marker; the named fix is in [[_generated/src/net/x509.rs]] `leaf_info_with_host` — even when chain validation fails and the code falls back to cert pinning, the leaf-info path now re-checks hostname against the leaf SAN.

Pre-V11: a cert legitimately issued for host A could be used against host B if it matched the pin, because the pin-only path never checked hostname. Found by re-asking "what does this guarantee?" against an existing test that passed.

## Why this vocabulary persists in source

Two reasons.

**One: a permanent breadcrumb beats a git-blame.** `git blame` will tell you when a line was last touched, but it won't tell you *why* it has the shape it has. `// V6-SIDE-002 fix: do the EXPENSIVE signature verification FIRST` is a line a future reader can read in 30 seconds and understand. Tracing the same fact through git blame, audit reports, and PR descriptions takes much longer.

**Two: it primes future audits.** When `V12` lands and the reviewer is looking at `verify_chain`, the markers tell them "these lines have been re-examined before; here's what came out of it." A change that touches a `V6-SIDE-` line gets extra scrutiny because the reader knows that line embodies a hard-won fix.

The cost: source files have a few extra comment lines. The benefit: every relitigation of that code starts informed, not from scratch.

## STUMPs

Adjacent vocabulary: `STUMP #<N>`. STUMPs are *project-internal* tracker entries — non-blocking work items the team agreed to come back to. Not the same as a V-incident: a STUMP is a planned-future, a V is a fixed-past.

Both show up in source comments. Both are searchable in the auto-generated [[_generated/_index|vault]] via Obsidian's full-text search. Searching `V8-ROOT` finds every comment that mentions any V8-ROOT incident; searching `STUMP #87` finds the linker.ld rebuild-cache caveat in `[[_generated/build.rs]]`.

## Related

- [[Concepts/TLS Hardening Journey]] — chronological walkthrough of V4 → V11 → x509-hardening
- [[Concepts/Constant-Cost Abort Discipline]] — V6-SIDE-002 in detail
- [[_generated/src/net/tls.rs]] / [[_generated/src/net/x509.rs]] — where the V markers cluster
- [[_generated/DESIGN_TLS_HARDENING.md]] — design-doc-level recap

## Open

- A V12 hasn't landed yet. The next wave is open; usually triggered by a new reviewer or a new threat-model surface.

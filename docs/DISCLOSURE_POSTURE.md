# Public Disclosure Posture

**Status as of 2026-05-13.** Internal strategy doc. Lives in the
private repo. NOT a public document. Reviewed before every public
post / README update / grant application / press touch.

## Why this exists

Public disclosure has consequences that are easy to forget while
writing a README:

1. **US patent law: 1-year grace period.** If we publicly describe
   how a mechanism works (in code, blog post, paper, conference
   talk, README, social-media thread, *anything*), we have **365
   days from that disclosure** to file a US utility patent. After
   365 days the invention enters the public domain and is
   unpatentable. Most countries (EU, China, Japan) have NO grace
   period at all — public disclosure forecloses non-US patents
   immediately.
2. **Trade-secret framing.** Information not publicly disclosed
   can be protected as trade secret indefinitely (cf. Coca-Cola
   formula). Once it's disclosed, trade-secret protection
   evaporates instantly.
3. **Competitive runway.** A specific implementation detail might
   be standard once described, but "discovering it for yourself"
   can take a competitor weeks or months. That asymmetry is real
   value.

We are NOT filing patents right now (a single utility patent
costs $5-15k in attorney + USPTO fees, and we have no money).
This doc is about *preserving the option* to file later, and
about not leaking trade secrets unnecessarily on day one.

## Three disclosure tiers

| Tier | Examples | Public stance |
|---|---|---|
| **Tier 1 — disclose fully.** Standard primitives we re-use from public knowledge. | AEAD (ChaCha20-Poly1305), Bell-LaPadula, Biba, SHA-256, hash-chained audit logs, heap canaries (per `hardened_malloc`), Spectre `sb` barriers, AGPL+commercial dual-license, Apple Silicon MMU model. | README, blog posts, conference talks. No restraint. |
| **Tier 2 — disclose at high level only.** The *concept* is fine to mention; the specific *implementation*, *byte layout*, or *invariant set* is held back. | Existence of a TPI quorum; existence of cave-private MMU isolation; existence of off-platform audit seal; existence of taint propagation. | "We use a two-officer quorum to gate destructive operations" — yes. "Our grant ring stores (op_id, role, public_key, nonce, ts, signature)[…] with a 60-second TTL and one-shot consume semantics" — keep close. |
| **Tier 3 — keep close.** Specific mechanisms with potential novelty, or hardware facts that took real RE work to discover. | M4 PMGR gate-enable sequence + IDs; ATC PHY tunable map; AIC2 base address on M4; dockchannel UART address on M4; specific SealVerify state machine; exact cave-as-unified-isolation-primitive coupling between audit ring + mount ns + mem quota + MMU L1 + IPC mailbox + per-cave WireGuard state. | NOT mentioned by mechanism. Demonstrated only by black-box behaviour. Documented in private docs (e.g. `M4_GROUND_TRUTH.md`) shared only with people under NDA or grant-fund evaluators with confidentiality terms. |

## Specific mechanisms — categorisation

The numbered list below is the working assessment. Each item
should be revisited before its first public disclosure.

### Hold close (Tier 3)

1. **M4 hardware ground truth** (`docs/M4_GROUND_TRUTH.md`).
   Every hex address, PMGR sequence, ATC PHY tunable that took
   real RE effort to discover. *NOT publicly disclosed in detail.*
   Public framing: "Bat_OS boots on Apple M4 (Mac16,1)" + boot
   photos. Mechanism: under NDA only.
2. **Specific TPI implementation invariants.** Grant-ring TTL
   value, canonical-bytes layout, one-shot-consume semantics,
   role-separation table, the audit-record-before-consume
   ordering. Public framing: "Two-officer Ed25519 quorum with
   replay-resistance and TTL." Detail: under NDA only.
3. **Cave-as-unified-isolation-primitive coupling.** The exact
   choice to bundle audit-ring isolation + mount namespace +
   memory quota + per-cave L1 page tables + IPC mailbox +
   per-cave WireGuard state under a single `cave_id` is a
   non-obvious architectural decision that took us several
   weeks to refine. Public framing: "Caves are the unit of
   security isolation." Mechanism + the integration design: hold.
4. **Audit chain off-platform seal protocol.** The specific
   `SealVerify` state machine (`Truncated{missing}`,
   `Mismatch`, `SealAboveRingTail`, `SealAheadOfHead`) and the
   handshake for verifying a seal against the live ring is
   non-obvious and arguably patentable as a method. Hold for
   now. Public framing: "Audit log is sealed off-platform on
   a regular cadence."

### Disclose at high level only (Tier 2)

5. **AEAD-bound MLS labels.** Concept ("file labels are bound
   into AEAD AAD so tampering with the label invalidates
   decryption") is disclosable; the exact AAD byte layout
   `filename || sens || integ` is implementation detail and
   not standard enough to volunteer. Public framing: "MLS
   labels are cryptographically bound to file contents."
6. **Taint propagation model.** Existence + 32-bit width +
   monotonic OR + propagation through ns_read/ns_create is
   disclosable. The fact that the bitmap is operator-defined
   semantically (vs the kernel imposing taxonomy) is a
   design choice worth not over-explaining publicly. Public
   framing: "Information-flow taint follows reads and writes."
7. **Cave-quota enforcement coupling to allocators.** Existence
   is disclosable; the specific charge-pages/release-pages
   API surface and the rollback discipline on AEAD failure is
   non-obvious. Hold the API detail; disclose the property.

### Disclose fully (Tier 1)

8. **Bell-LaPadula + Biba lattices.** Textbook.
9. **SELinux-style Type Enforcement (subject domains + object
   types + DENY matrix).** Textbook.
10. **CIPSO IPv4 + CALIPSO IPv6.** RFCs 2401 / 5570, public.
11. **Hash-chained audit log with off-platform seal.** Concept
    is textbook; the *specific* seal verify state machine is
    Tier 3 (see #4).
12. **Heap guard canaries.** Per-allocation HMAC-derived
    canaries are the `hardened_malloc` model, already public.
13. **ARMv8.5 `sb` speculation barrier at cross-domain
    boundaries.** Linux kernel does this, ARM publishes the
    pattern.
14. **AGPL-3.0 + commercial dual-license posture.** MongoDB /
    Sentry / GitLab playbook. Public knowledge.
15. **First non-Apple OS booted on Apple M4.** The *claim*
    discloses freely; the *how* (PMGR sequences, etc.) is #1.

## Operational rules

1. **Before any public post**, scan it against this doc. Tier 3
   content gets removed or paraphrased to Tier 2 wording.
2. **The repo README, when made public, may freely describe
   Tier 1 and Tier 2 items.** It may *mention* Tier 3 items by
   name + outcome but must not disclose mechanism.
3. **Grant applications** may go deeper than the public README
   *if* the grant programme has confidentiality terms (NLnet
   evaluators have an internal review process; Sovereign Tech
   Fund has confidentiality language in their NDA). Read the
   programme's confidentiality posture before submitting.
4. **Code itself, when open-sourced under AGPL,** is necessarily
   fully disclosed — there's no way to ship binaries under AGPL
   while hiding mechanism. This means: by the time we open the
   repo, anything still in Tier 3 must either be moved to Tier 2
   (and accepted as disclosed) or patented (filed) or extracted
   into a separately-licensed proprietary module.
5. **Conference talks / blog posts** — Tier 3 content is OK to
   mention abstractly ("we use a state machine that distinguishes
   four failure modes") but never described in enough detail
   that a competent reader can reimplement.
6. **Re-evaluate this doc every 60 days** or before any major
   public milestone.

## Patent posture summary

We are not filing patents *now*. We are *preserving the option*
to file later by keeping Tier 3 mechanisms out of public
disclosure. Each Tier 3 item triggers a 365-day US filing clock
the moment it goes public; the clock for the EU/JP/CN starts
immediately.

If at any point a grant or partner pays $50k+, the first $15-20k
of that should arguably go to filing a utility patent on the
single most defensible Tier 3 mechanism (current best candidate:
**#4, the audit-chain off-platform seal protocol** — it's
self-contained, novel, and has clear commercial application in
compliance-regulated industries).

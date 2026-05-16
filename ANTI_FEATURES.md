# Sphragis Anti-Features (Explicit Non-Goals)

Sphragis is a security-first bare-metal Rust microkernel for gov / high-assurance use. This document lists things Sphragis explicitly will NOT do. Knowing what we won't build is as important as knowing what we will.

## ANTI-001: No full functional-correctness proof of the whole kernel

**What we don't do:** Attempt to prove every line of the kernel correct end-to-end (the seL4 model).

**Why:** seL4 has a 15-year, ~25-person-year head start. We cede that lane. We claim **information-flow non-interference on critical subsystems** (capability dispatcher, IPC, scheduler invariants) via Verus or Kani — a more tractable proof effort that still produces a defensible "verified" claim for gov procurement.

## ANTI-002: No AI/LLM/ML in the kernel critical path

**What we don't do:** Run language models, neural nets, or reinforcement-learning policies inside the kernel TCB.

**Why:** Non-deterministic, hard to certify, expands the attack surface, and inference latency makes scheduler integration counterproductive. AI features can ship as user-mode caves in the community build, but the `sphragis-gov` build excludes them entirely.

## ANTI-003: No QKD integration as a featured capability

**What we don't do:** Market quantum key distribution as a Sphragis differentiator.

**Why:** NSA's stated preference is post-quantum cryptography (PQC), not QKD. Sphragis maintains a key-plane abstraction that *could* swap in a QKD-derived link key for a specific tactical comms scenario, but we don't lead with it.

## ANTI-004: No Linux binary compatibility promise

**What we don't do:** Promise to run arbitrary Linux binaries.

**Why:** A full Linux ABI compatibility surface drags in the same TCB shape we're trying to avoid. Sphragis ships a narrow Linux ABI shim (`src/caves/linux/`) sufficient to host an analyst-toolbox (vim, git, python, ssh, tmux) under heavy capability restrictions, but binary compat is not a goal.

## ANTI-005: No weak cryptography in the gov build

**What we don't do:** Allow AES-128, SHA-1, MD5, RSA-2048, ECDSA-256, plain ChaCha20-Poly1305 (without CNSA-grade context), DH-2048, or other below-CNSA-2.0 algorithms in the `sphragis-gov` build profile.

**Why:** CNSA 2.0 deadlines (2027-01-01 for new NSS acquisitions; 2033 for exclusive use) require modern algorithms. Weak algorithms are accepted in `sphragis-community` only for legacy-interop scenarios; the gov SKU rejects them at the policy layer.

## ANTI-006: No closed-source kernel components

**What we don't do:** Ship binary blobs in the Sphragis kernel or first-party drivers.

**Why:** Auditable from sand to syscall. Apache-2.0 source for every line we own. Hardware-vendor firmware (M4 SEP firmware is Apple-signed) exists at the boundary; we attest TO it but don't sign on its behalf.

## ANTI-007: No GPL/AGPL/copyleft dependencies

**What we don't do:** Accept GPL-2.0, GPL-3.0, AGPL, LGPL, SSPL, BUSL, or Commons-Clause dependencies.

**Why:** Apache-2.0 license requires compatibility. Prime-integration friction is real — primes (Lockheed, Northrop, etc.) will not embed copyleft code into their proprietary product lines. Enforced in CI via `deny.toml`.

---

These anti-features are part of Sphragis's strategic identity, not arbitrary limits. Changing one of them is a category-redefinition and requires explicit project-leadership approval.

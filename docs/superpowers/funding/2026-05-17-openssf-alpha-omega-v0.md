# OpenSSF Alpha-Omega Application — Draft v0

**STATUS: DRAFT v1**
**Date drafted:** 2026-05-17
**Author:** Funding Team (Mac Claude, vault-mediated)
**Target program:** OpenSSF Alpha-Omega Project (Linux Foundation)
**Submission path:** `https://share.hsforms.com/1sZmUUNQLQ0SwlMhrcOs7ww4tvhy`
(linked from `https://alpha-omega.dev/grants/how-to-apply/`)
**Mode:** drafting only; founder submits.

> **Founder action — fill before submit:**
> - Legal name, email, GitHub handle, country of residence
> - Sphragis Inc. incorporation status (in-flight as of 2026-05-17 →
>   include the Stripe Atlas filing receipt # if Atlas has issued
>   one; if not, declare "individual applicant; Delaware C-Corp in
>   formation")
> - If Alpha-Omega's intake call surfaces a different scope or
>   budget than what's drafted below, that's expected — the
>   published process explicitly says proposals are co-developed
>   with the A-O team via a meeting after initial submission

---

## §0 — Why we're applying to Alpha-Omega

Alpha-Omega's stated mission (per `alpha-omega.dev`) is to
"catalyze sustainable security improvements across open source,
from the largest global projects to the smallest but essential
components." The program has distributed >$20M across >70 grants
since 2022 with individual project grants documented in the
$300K-$500K range for established projects (e.g. Rust Foundation
$460K, Eclipse $400K, Node.js $300K).

Sphragis is a smaller, emerging Apache-2.0 open-source security
project — a security-first Rust microkernel for Apple Silicon —
that fits Alpha-Omega's "smallest but essential components"
framing. Sphragis sits at the OS-substrate layer beneath
applications and libraries Alpha-Omega already funds, and the
security properties Sphragis enforces (memory safety,
post-quantum crypto, kernel-mediated TLS, capability isolation,
audit-logged egress) become foundational guarantees that
downstream OSS security projects can rely on rather than
re-implement.

**Ask:** $150,000 over 9 months for **specific deliverable
work-packages** scoped below. This is calibrated as a "proof
project" ask — between A-O's documented small-grant baseline and
the multi-hundred-K grants made to mature established projects.
The work packages produce outputs directly usable by every other
Rust security project and every operating-system project pursuing
similar guarantees.

---

## §1 — Project identification (Alpha-Omega proposal section 1)

> **Project name:** Sphragis
>
> **Repository:** `https://github.com/kadenlee1107/Sphragis`
>
> **License:** Apache License 2.0 (relicensed from AGPL-3.0 on
> 2026-05-16; license hygiene enforced by `cargo-deny` in CI; no
> GPL, LGPL, SSPL, Commons-Clause, or BUSL dependencies anywhere in
> the dependency graph).
>
> **Project type:** Operating system — bare-metal microkernel
> written in Rust (`#![no_std] #![no_main]` against
> `aarch64-unknown-none`). Boots on real Apple M4 hardware
> (Mac16,1 / J604 / T8132 "Donan") and on QEMU virt aarch64.
>
> **Maintainer:** Kaden Lee — sole maintainer as of 2026-05-17.
> GitHub: `@kadenlee1107`.
>
> **Organization:** Sphragis Inc. (Delaware C-Corporation, in
> formation as of 2026-05-17 via Stripe Atlas). For
> Alpha-Omega purposes the applicant is the project; legal
> recipient of any grant award is Sphragis Inc. once incorporation
> completes (3-7 business days per Atlas timeline).
>
> **Codebase scale:** approximately 96,000 lines of Rust across
> 199 source files. 143 commits in the most recent 24-hour
> productization push leading into this application. Bit-identical
> reproducible build verified (SHA-256
> `f4b12add37d44d4ae031a0bc5db83739a15c2d54d7d8096e1fcb667ca7e5ad03`).
>
> **Public history:** 14 weeks of mechanical-trace security audit
> activity with traceable git commits closing 32 Priority-0 audit
> findings. Audit history is visible in the repository commit log
> and in the strategic planning documents under
> `docs/superpowers/`.

---

## §2 — Current state and why this is critical
   (Alpha-Omega proposal section 2)

### What Sphragis does today

Per the public capability statement, marketing site, and
end-of-day-1 sweep (`docs/superpowers/research/2026-05-17-day1-sweep-and-funding-readiness.md`):

**Memory safety + isolation.** Pure-Rust kernel with no C/C++ in
the trusted computing base. Capability-isolated processes
("caves") each holding their own L1 page table, IPC namespace,
mount namespace, AF_UNIX namespace, mem quota, and sensitivity +
integrity labels. Per-cave ASIDs (TTBR0_EL1 bits 63:48 with
TCR.AS=1) and TLBI safety net. Cave-policy gate on every
cross-cave syscall (network/file/IPC/shm/ptrace/signal).
Bell-LaPadula + Biba dual-lattice multi-level security label
enforcement.

**Post-quantum crypto, CNSA-2.0 native.** ML-KEM-1024 (FIPS 203)
for non-TLS contexts; ML-KEM-768+X25519 hybrid for TLS
(`draft-ietf-tls-ecdhe-mlkem-04`, verified end-to-end interop
with `pq.cloudflareresearch.com`); ML-DSA-87 (FIPS 204) for
signatures; LMS (NIST SP 800-208, RFC 8554) for software-firmware
signing; full classical algorithm coverage (AES-256-GCM/
GCM-SIV/XTS, ChaCha20-Poly1305, X25519, Ed25519, SHA-384, SHA-512,
Argon2id). `gov-strict` build profile rejects AES-128 / SHA-1 /
SHA-256-for-sig / RSA / ECDSA / plain-ChaCha20 at the policy
layer. Fail-closed RNG variant. Boot-time KATs for every
critical primitive.

**Encrypted filesystem (SealFS).** AES-256-GCM-SIV at rest
(misuse-resistant AEAD), per-file AEAD, per-cave keys derived from
Argon2id (8 MiB / 3 iter / 1 par). MLS labels bound into AEAD AAD —
tampering with classification invalidates decryption.

**Kernel-mediated TLS 1.3.** Processes never see TLS. The kernel
performs the handshake, chain validation, and pinning; the process
gets a plaintext file descriptor. No process can ship broken TLS,
skip cert validation, or downgrade to HTTP.

**HMAC-SHA-384 chained audit ring** (CNSA-aligned) with RNDR-seeded
kernel-only key, WORM segment export to SealFS, and an offline
verifier tool (`tools/audit-verifier/`). NIAP FAU_GEN.1 emit sites
covering Authentication, PrivilegeEscalation, Crypto, Net, Fs,
KeyRotate, TpiOp, LoadableMod, UpdateApply, FileAccess, Cave.

**Attestation primitive.** `attest::quote(nonce, claims) -> Quote`
produces CBOR-encoded ML-DSA-87-signed quotes binding cave
measurement, kernel measurement, and audit-log digest. External
verifier ships at `tools/attest-verifier/`.

**Reproducible builds, supply-chain scaffold.** Bit-identical
build verified across two passes. In-toto attestation chain
designed (`scripts/build_intoto_attestation.py`). SBOM generation
scripts (`scripts/gen_sbom.py`, `scripts/generate_sbom.py`).
`deny.toml` + `cargo-audit` block GPL/LGPL/SSPL/Commons-Clause
licenses and RustSec advisories at the CI gate. DCO sign-off
required on every commit.

### Why it's critical now

The 2027-2030 procurement-cliff window simultaneously imposes five
constraints on critical-infrastructure operating systems:

1. **NSA CNSA 2.0** (May 2025 issuance) mandates ML-KEM-1024 +
   ML-DSA-87 + AES-256 + SHA-384 for new National Security System
   cryptographic acquisitions by 2027-01-01. The EU rapidly aligns.
   RSA, ECDSA, and SHA-256 become procurement-ineligible.
2. **US ONCD memory-safety policy** (2024-2025) explicitly
   identifies Rust as the canonical path for new
   critical-infrastructure software, with the EU Cyber Resilience
   Act mirroring this position.
3. **SLSA Level 4 reproducible builds**, sigstore-signed releases,
   and full source-bootstrap are increasingly required by NIS2 and
   the CRA.
4. **Capability-safe hardware** (ARM Morello pure-capability,
   CHERIoT-Ibex) reaches production in 2026; operating systems
   that cannot leverage capability hardware forfeit material
   security advantage.
5. **Formal proofs of non-interference** become procurement
   expectations rather than research curiosities.

No current open-source operating system simultaneously satisfies
all five constraints. Closed proprietary alternatives (Green Hills
INTEGRITY-178B, Wind River VxWorks 653, Lynx LynxOS-178) are
US-vendor-locked, classical-crypto-only, and cannot retrofit
without abandoning their certifications. General-purpose Linux
distributions resist formal verification at scale (~30M-line C
TCB).

**Sphragis is being built to close exactly this gap, as foundational
open-source security infrastructure.** Closing the gap benefits the
broader OSS security ecosystem in three concrete ways:

- Every downstream library and application running on Sphragis
  inherits memory safety, post-quantum crypto, kernel-mediated
  TLS, and audit-logged egress for free — properties OSS projects
  currently bolt on per-project at varying quality.
- The Verus formal-methods proof patterns Sphragis publishes
  become reusable by every other Rust microkernel project
  (Hubris/Oxide, Asterinas/Ant Group, Theseus/Rice University,
  RedoxOS).
- The FIPS 140-3 module boundary documentation, CAVP submission
  preparation, and SLSA Level 4 reference implementation Sphragis
  produces become reusable templates for every Rust
  cryptographic-library project pursuing CMVP validation.

### Current security challenges (project-perspective)

1. **Single-maintainer bus factor.** The 14-week velocity is real
   but solo work is fragile. Alpha-Omega funding directly mitigates
   bus-factor risk by enabling consultant capacity on
   formal-methods work.
2. **Formal proof completeness.** Verus toolchain is installed and
   two non-interference proof specifications are written
   (`verification/cap_dispatch/SPEC.md`,
   `verification/ipc_flow/SPEC.md`). The proofs themselves are
   incomplete. This is the highest-leverage open work for
   Sphragis's procurement story.
3. **FIPS 140-3 lab engagement.** Module boundary doc is complete
   (`docs/FIPS_140_3_MODULE_BOUNDARY.md`); CMVP queue times in
   2025-2026 ran 6-18 months. Alpha-Omega funding accelerates
   pre-engagement scoping with at least one accredited lab.
4. **Supply-chain provenance hardening.** Reproducible build is
   verified; sigstore signing, Rekor entries, and in-toto
   envelopes are designed but not yet wired into the release
   pipeline (workflow YAMLs are staged at
   `.github-workflows-pending/`; OAuth-blocked from auto-push;
   awaiting maintainer review-and-add via the GitHub web UI).
5. **License hygiene drift risk.** Apache-2.0-only policy is
   enforced by `cargo-deny` today, but rapid development means
   continued vigilance is required.

---

## §3 — Desired outcomes and direct benefit to end-users
   (Alpha-Omega proposal section 3)

### Three outcome work-packages

**WP1 — Verus non-interference proofs of the capability dispatcher
and the IPC subsystem. (Months 1-6, $70,000)**

*Specific deliverables:* (a) production-grade Verus toolchain
integration into the Sphragis continuous-integration pipeline so
proofs are continuously verified on every kernel pull request;
(b) completion of the non-interference proof of the capability
dispatcher specified at `verification/cap_dispatch/SPEC.md` — the
property: given two caves A and B with no explicit information-flow
permission, no system-call sequence from A can cause B's observable
state to differ from a baseline execution; (c) analogous
non-interference proof of the IPC subsystem
(`verification/ipc_flow/SPEC.md`); (d) reusable proof patterns
abstracted to a `verification/patterns/` directory with
documentation suitable for adaptation by other Rust microkernel
projects (Hubris, Asterinas, Theseus, RedoxOS); (e) submitted
methodology paper to USENIX Security 2027 or IEEE Symposium on
Security and Privacy 2027.

*Direct benefit to end-users:* every OSS project running on
Sphragis inherits formally-verified non-interference. The proof
patterns themselves become an open-source artifact reusable by
every other Rust microkernel — the work multiplies across the
ecosystem.

**WP2 — SLSA Level 4 supply-chain provenance pipeline.
(Months 3-9, $50,000)**

*Specific deliverables:* (a) production sigstore cosign signing of
release artifacts (kernel image, SBOM, documentation bundles) with
Rekor transparency-log entries — every released artifact becomes
externally verifiable via the public Rekor log;
(b) production in-toto attestation envelopes for the complete
source → build → release chain, building on
`scripts/build_intoto_attestation.py` and wiring it into the
GitHub Actions release pipeline; (c) LMS-signed kernel image —
kernel binary signed with the LMS post-quantum signature scheme
(already implemented in `src/crypto/lms.rs`), with the boot stub
verifying signature before jumping to Rust kernel entry;
(d) bootstrappable build from documented seed following the
Guix-project model, enabling independent reproducibility
verification by any auditor without trusting the Rust toolchain
distribution channel; (e) independent reproducibility verification
by at least one third-party auditor producing a public
verification report.

*Direct benefit to end-users:* downstream consumers of Sphragis
release artifacts can independently verify provenance offline
using only Apache-2.0 tools. The reference implementation patterns
become a SLSA-L4 template for every other Rust project at every
assurance level.

**WP3 — FIPS 140-3 cryptographic module boundary refinement and
CMVP pre-engagement. (Months 4-9, $30,000)**

*Specific deliverables:* (a) refinement of
`docs/FIPS_140_3_MODULE_BOUNDARY.md` to CMVP pre-engagement
quality — complete enumeration of public API, Sensitive Security
Parameter (SSP) management per FIPS 140-3 §7.8, role definitions
with separation enforcement, self-test policy mapping, key
destruction protocols, and critical-security-parameter flow
diagrams; (b) NIST Cryptographic Algorithm Validation Program
(CAVP) submission preparation for SHA-384, SHA-512, HMAC-SHA-384,
AES-256-GCM, AES-256-GCM-SIV, AES-256-XTS, ML-KEM-1024, ML-DSA-87,
and LMS; (c) engagement letter from at least one accredited CMVP
lab (Atsec, Leidos, or InfoGard) scoping cost and timeline for a
full Level 1 validation.

*Direct benefit to end-users:* the FIPS 140-3 boundary
documentation template and CAVP submission patterns are directly
reusable by every open-source Rust cryptographic library project
seeking CMVP validation. The Rust ecosystem currently lacks a
freely-available reference implementation of this work.

### Success metrics

- WP1: two completed Verus proofs visible in `verification/`,
  CI-verified on every kernel pull request; methodology paper
  submitted to USENIX Security 2027 or IEEE S&P 2027.
- WP2: every release artifact signed in Rekor; in-toto envelope
  attached to every release; LMS-signed kernel boots successfully
  on M4 hardware; at least one third-party reproducibility audit
  report published.
- WP3: revised FIPS 140-3 boundary doc accepted by an accredited
  lab as pre-engagement-ready; CMVP cost-and-timeline letter on
  file.
- Cross-cutting: monthly public progress reports (per Alpha-Omega
  recipient policy); blog posts at funding start, midpoint, and
  end; Alpha-Omega acknowledgment in README, release notes, and
  any peer-reviewed publication.

### Direct benefit to end-users — concrete

| End-user category | Direct benefit |
|---|---|
| Other Rust microkernel projects (Hubris, Asterinas, Theseus, RedoxOS) | Adoptable Verus proof patterns; FIPS 140-3 boundary template |
| Rust cryptographic-library projects (rustls, ring, RustCrypto) | CAVP submission patterns; reference for FIPS validation in Rust |
| Open-source supply-chain security projects (sigstore, in-toto, SLSA) | A working Rust kernel reference implementation of SLSA Level 4 |
| Critical-infrastructure operators (automotive, medical, energy) | An OSS substrate that meets CNSA 2.0, FIPS 140-3, and SLSA-L4 mandates with formal proofs |
| Allied-government cybersecurity authorities | A sovereign-deployable substrate not US-vendor-locked |
| Academic kernel-verification research community | Citable Verus-on-production-Rust comparable, plus published methodology paper |
| End-users of any software built on Sphragis | Memory-safety, post-quantum crypto, kernel-mediated TLS, and audit-logged egress as foundational guarantees, not bolted-on conventions |

---

## §4 — Implementation approach (Alpha-Omega proposal section 4)

### Activities + timeline

| Months | Work package | Key activities |
|---|---|---|
| 1-2 | WP1 ramp | Verus CI integration; spec validation by external consultant; proof attempts on simplest dispatcher subset |
| 3-4 | WP1 core | Complete dispatcher non-interference proof; begin IPC proof |
| 3-4 | WP2 ramp | Sigstore + Rekor wiring; in-toto envelope generation in CI |
| 4-5 | WP3 ramp | FIPS 140-3 boundary doc refinement; lab outreach |
| 5-6 | WP1 finish | Complete IPC non-interference proof; pattern publication |
| 5-6 | WP2 core | LMS-signed kernel boot path; third-party reproducibility audit kickoff |
| 6 | WP3 mid | CAVP submission package preparation |
| 6 | WP1 publish | USENIX/S&P methodology paper submitted |
| 7-8 | WP2 finish | Third-party reproducibility audit completed and published |
| 8-9 | WP3 finish | Lab engagement letter signed; final FIPS 140-3 boundary doc |
| Monthly | Cross-cutting | Public progress report; Alpha-Omega strategy roundtable attendance; blog posts at start/mid/end |

### Staffing

- **Maintainer (Kaden Lee), ~75% allocation across 9 months.**
  Drives WP1 specification + WP2 implementation + WP3 documentation.
- **External Verus consultant, ~200 hours across Months 1-6.**
  Preferred pool: Microsoft Research / CMU Verus team contributors,
  or European formal-methods groups (IMDEA, MPI-SWS, INRIA,
  Cambridge, KTH). Identified within Month 1.
- **Third-party reproducibility auditor, ~80 hours in Months 6-8.**
  Independent reviewer (academic researcher or allied OSS project
  maintainer) producing a public verification report.

### Budget breakdown ($150,000 total)

| Line | Amount | Justification |
|---|---|---|
| Maintainer time (1,200 hours @ $75/hr loaded) | $90,000 | 75% allocation × 9 months × 173 hrs/month |
| Verus consultant (200 hours @ $150/hr) | $30,000 | Specialist rate; sourced as above |
| Third-party reproducibility auditor (80 hours @ $125/hr) | $10,000 | Independent audit deliverable |
| CMVP lab pre-engagement scoping fee | $5,000 | One-time engagement charge |
| Conference / paper open-access fee | $2,000 | USENIX Security or IEEE S&P open-access |
| Cloud / CI / infrastructure (9 months) | $3,000 | CI runners for proof verification + reproducibility checks |
| Travel (one academic conference for paper presentation) | $5,000 | USENIX or S&P conference travel |
| Contingency | $5,000 | Held back for unforeseen lab fees, additional consultant time |

### Risk + mitigation

| Risk | Mitigation |
|---|---|
| Verus toolchain regression mid-project | Parallel Kani fallback for memory-safety regression tests; reduce proof scope if dispatcher-wide proofs prove intractable |
| Verus consultant identification delay | Multiple candidate institutions identified up-front; outreach starts Week 1 |
| CMVP lab timeline outside our control | WP3 deliverable is the *engagement letter*, not the certificate — bounded scope |
| Solo-maintainer bandwidth | Consultant + 75% allocation (not 100%) explicitly leaves slack for the rest of the project |
| Specification iteration | Open spec at `verification/SPEC.md`; community review during Month 1-2 |
| Third-party auditor availability | Identified up-front via Alpha-Omega network (referral path: monthly strategy roundtables) |

### Why this work is right-sized for Alpha-Omega

The three work packages are **independent and parallelizable** —
A-O can de-scope to a subset if budget pressure requires (e.g.
fund WP1 + WP2 only at $120K, defer WP3). The deliverables are
**concrete and externally verifiable** — every milestone produces
an artifact a reviewer can independently inspect (a Verus proof
that the toolchain accepts, a Rekor entry, a signed lab letter).
The work directly **multiplies across the OSS security ecosystem**
— each output benefits not just Sphragis but every other Rust
systems project pursuing similar guarantees.

---

## §5 — Public-claim ceiling check

Every claim in this draft is grounded in primary sources:

- Code claims (M4 boot, ~96K LoC, 199 files, Apache-2.0,
  cryptographic algorithm coverage, reproducible build SHA) →
  `marketing-site/index.html` lines 1180-1810 +
  `docs/superpowers/research/2026-05-17-day1-sweep-and-funding-readiness.md`
  §1
- Procurement-cliff framing → mirrored from the STF and NLnet
  drafts which were drafted from the same source: the master plan
  + day-1 sweep §3
- Verus harness state, FIPS 140-3 boundary doc state, sigstore
  scaffold state → day-1 sweep §3 "What's live but skeletal vs
  production-quality"
- Pre-existing primary funding drafts:
  `docs/superpowers/funding/2026-05-17-stf-form-field-answers.md`
  + `docs/superpowers/funding/2026-05-17-nlnet-form-field-answers.md`
  + `docs/superpowers/funding/2026-05-17-sbir-phase-1-afwerx-open-v0.md`
  (technical-narrative source)
- Alpha-Omega program facts (grant size range, application
  framework, monthly reporting expectations) → `alpha-omega.dev`
  WebFetch on 2026-05-17

No claim in this draft exceeds the marketing site's public ceiling.

---

## §6 — What Kaden does next

1. **Open the submission form** at
   `https://share.hsforms.com/1sZmUUNQLQ0SwlMhrcOs7ww4tvhy` (linked
   from `alpha-omega.dev/grants/how-to-apply/`). The Alpha-Omega
   process is: form → intake call with A-O team → co-developed SOW.
2. **Fill the form's identifying fields** (name, email, GitHub
   handle, country, project, license, repo URL).
3. **Paste content from §1, §2, §3, §4 above** into the
   corresponding free-text fields. If the form has tighter
   character limits than this draft, compress §2 and §3 first
   (they have the most narrative slack; §1 and §4 are tighter).
4. **Note the budget ask: $150,000 over 9 months.** Justify with
   the breakdown in §4.
5. **Submit.** Expect a response within 2-4 weeks per typical A-O
   intake cadence. The response will likely be an invitation to a
   scoping call with the A-O team to co-develop the final SOW.
6. **Prepare for the scoping call** by re-reading the master plan
   (`docs/superpowers/plans/2026-05-16-sphragis-gov-os-master-plan.md`)
   and refreshing your mental model of which work packages are
   most-urgent vs most-leveragable.
7. **If awarded:** be ready to publish a monthly progress report
   (A-O requires public reporting); attend the monthly A-O
   strategy roundtable; publish kick-off, midpoint, and end blog
   posts.
8. **Coordinate with other parallel applications** — STF, NLnet,
   SBIR — to ensure no duplicate-funding overlap. The Alpha-Omega
   work packages above are scope-distinct from STF's WP1
   (CNSA 2.0 module completion) and NLnet's narrower Verus-only
   focus; partial overlap on WP3 (FIPS 140-3) is acceptable if A-O
   funds only the pre-engagement scoping and STF funds the broader
   CMVP path. Be transparent in the intake call about parallel
   asks.

**Estimated total founder time:** ~90 minutes for the form
submission + 60 minutes for the scoping call (when scheduled).

---

## §7 — Primary sources cited

- `docs/superpowers/research/2026-05-17-day1-sweep-and-funding-readiness.md`
  — capability + status truth-source
- `docs/superpowers/funding/2026-05-17-stf-form-field-answers.md`
  — model for tone, length, and technical-narrative compression
- `docs/superpowers/funding/2026-05-17-nlnet-form-field-answers.md`
  — model for stating Verus scope without overclaiming
- `docs/superpowers/funding/2026-05-17-sbir-phase-1-afwerx-open-v0.md`
  — technical narrative source for cross-domain framing
- `docs/superpowers/funding/2026-05-17-founder-action-checklist.md`
  — incorporation status (Sphragis Inc., in formation)
- `marketing-site/index.html` lines 1180-1810 — public-claim
  ceiling
- `docs/superpowers/plans/2026-05-16-sphragis-gov-os-master-plan.md`
  — strategic context
- `docs/superpowers/plans/2026-05-17-multi-team-push.md` §3
  (Funding, draft #1) — charter directive
- `alpha-omega.dev/grants/how-to-apply/` (WebFetch, 2026-05-17)
  — proposal framework + submission URL
- `alpha-omega.dev` (WebFetch, 2026-05-17) — mission statement and
  grant-history context (>70 grants, >$20M cumulative)
- `openssf.org/press-release/2026/03/17/...` (WebSearch result,
  2026-05-17) — $12.5M March 2026 grant infusion context

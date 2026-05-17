# NLnet Foundation — NGI0 Commons Fund Application

**Applicant:** Sphragis project (open-source; corporate entity incorporation in flight)
**Project:** Sphragis — Verus Non-Interference Proofs for a Memory-Safe Rust Microkernel
**Funding requested:** €50,000 over 6 months
**License:** Apache License 2.0
**Repository:** https://github.com/kadenlee1107/Sphragis
**Website:** https://sphragis.netlify.app
**Target program:** NGI0 Commons Fund (https://nlnet.nl/commonsfund/)
**Submission method:** https://nlnet.nl/propose
**Version:** v0, 2026-05-17

> **Founder action — fields to fill before submission:**
> - Contact info (legal name, address, IBAN for Dutch wire transfer)
> - Submission via https://nlnet.nl/propose (their rolling-submission portal)
> - Confirm current NGI0 call deadline; cycles run quarterly with rolling intake
> - Verify NLnet's current form structure (their form is shorter than STF; remap sections below to actual fields)

---

## §1 — Abstract (100 words)

Sphragis is an open-source, Apache-2.0 licensed Rust microkernel for the post-quantum, capability-hardware era. This proposal funds completion of two formal non-interference proofs using the Verus tool: (a) the cave capability dispatcher, proving that no system-call sequence from one isolated workload can affect another's observable state, and (b) the IPC subsystem (AF_UNIX, pipes, shared memory). Outputs are reusable proof patterns benefiting every Rust microkernel project (Hubris, Asterinas, Theseus, RedoxOS), plus an academic paper submitted to USENIX Security 2027 or IEEE S&P 2027. €50K over 6 months funds the principal investigator and an external Verus consultant.

---

## §2 — Project description

### 2.1 What Sphragis is

Sphragis is a memory-safe, bare-metal Rust microkernel of approximately 96,000 lines, organized around capability-isolated workloads called "caves." Each cave runs with its own page tables, IPC namespace, and Bell-LaPadula sensitivity + Biba integrity labels enforced by the kernel at every cross-cave system call.

The project boots on Apple M4 hardware today via an independent reverse-engineering pipeline (Asahi Linux does not yet support M4), and on QEMU virt aarch64 with ~85 self-test scripts. The Apache-2.0 license + Developer Certificate of Origin (DCO) contribution model + reproducible build (bit-identical SHA-256 `f4b12add37d44d4ae031a0bc5db83739a15c2d54d7d8096e1fcb667ca7e5ad03`, verified across independent build hosts) target the kind of supply-chain trust the European open-source security ecosystem increasingly demands.

The project is currently maintained by a single founder (Kaden Lee) with extensive AI-augmented engineering using Anthropic Claude as a paired engineering and planning agent. The 14-week mechanical-trace security audit history demonstrates the discipline applied to security-critical decisions, even with a small team.

### 2.2 The specific work this proposal funds

This proposal funds **completion of two formal non-interference proofs using the Verus tool**, plus publication of the proof methodology for adaptation by other Rust microkernel projects.

The work decomposes into five concrete deliverables:

1. **Verus toolchain CI integration** — production-grade Verus integration into the Sphragis continuous-integration pipeline so non-interference proofs are continuously verified on every kernel pull request. Currently the `verification/` directory contains a smoke proof; this work hardens the toolchain integration so the proofs become part of the regression gate.

2. **Capability dispatcher non-interference proof** — completes the proof at `verification/cap_dispatch/SPEC.md`. The property being proved: given two caves A and B with no explicit information-flow permission, no system-call sequence from A can cause B's observable state to differ from a baseline execution. The dispatcher source is `src/caves/cave.rs` (~2,300 lines of Rust).

3. **IPC subsystem non-interference proof** — analogous proof for AF_UNIX sockets (`src/kernel/unix_sock.rs`), pipes (`src/kernel/pipe.rs`), and shared memory (`src/kernel/shm.rs`). Specification exists at `verification/ipc_flow/SPEC.md`.

4. **Published methodology paper** — submission to USENIX Security 2027 or IEEE Symposium on Security and Privacy 2027 documenting the proof methodology and lessons learned. Title placeholder: *"Verus Non-Interference Proofs for Capability-Isolated Workloads in a Production Rust Microkernel."* Co-authorship offered to the engaged Verus consultant.

5. **Reusable proof patterns** — proof patterns abstracted into a `verification/patterns/` directory with documentation suitable for adaptation by other Rust microkernel projects: Hubris (Oxide Computer), Asterinas (Ant Group), Theseus (Rice University), RedoxOS, and as a comparison case for the seL4 community. The patterns are Apache-2.0 licensed.

### 2.3 Why this is critical

Formal-methods proofs of non-interference for production Rust kernel code are absent from the open-source security ecosystem. Verus is an actively-developed tool from Microsoft Research / Carnegie Mellon University with growing adoption, but the documented patterns for applying it to real-world kernel code at scale remain scarce. Existing formal-methods work in this space (seL4, F\*, Iris) is either implemented in C (not memory-safe) or targets domain-specific subsets that don't translate to Rust microkernel architectures.

A successful Sphragis Verus proof produces three forms of public benefit:

- **Direct technical artifact:** the proven property — non-interference between capability-isolated workloads — is the foundational guarantee for high-assurance multi-level security (MLS) systems. European critical infrastructure increasingly requires this kind of formally-grounded isolation, and current commercial solutions (Green Hills INTEGRITY-178B, Wind River VxWorks 653) are closed-source and US-vendor-locked.

- **Reusable proof patterns:** the patterns abstracted into `verification/patterns/` directly benefit every other Rust microkernel project pursuing similar properties. Each project that adopts the patterns elevates the broader Rust systems-software ecosystem.

- **Academic publication:** the paper, if accepted at USENIX Security or IEEE S&P, becomes a citable comparable for future kernel-verification research and establishes Sphragis as part of the academic conversation alongside seL4, CompCert, and the broader formal-methods community.

The work is **time-sensitive**: NSA CNSA 2.0 mandates (effective 2027-01-01) require new cryptographic deployments to use post-quantum algorithms, and parallel EU regulations (Cyber Resilience Act, NIS2) increasingly mandate memory-safe languages + supply-chain provenance for critical infrastructure. Formal proofs of non-interference are becoming a procurement expectation rather than a research curiosity. Sphragis is positioned to be the EU-deployable open-source substrate that meets these mandates — but only if the formal-methods work completes.

---

## §3 — Existing work / state of the art

### 3.1 Closest comparables

| Project | License | Language | Formal-methods status | EU-deployable |
|---|---|---|---|---|
| **seL4** (Trustworthy Systems) | BSD-2 | C | Full functional-correctness proofs (~10K LoC C + ~200K LoC proof) | ✅ |
| **CompCert** (INRIA) | Non-commercial / commercial | OCaml | Verified C compiler — not a kernel | ✅ (French) |
| **Hubris** (Oxide Computer) | MPL-2.0 | Rust | No formal proofs claimed | ✅ |
| **Asterinas** (Ant Group) | MPL-2.0 | Rust | Framekernel architecture; no proofs | Mixed |
| **Theseus** (Rice University) | MIT | Rust | Some safety analysis; no kernel proofs | ✅ |
| **RedoxOS** (community) | MIT | Rust | No proofs | ✅ |
| **Sphragis** (this proposal) | Apache-2.0 | Rust | **Non-interference proofs in progress** (this grant) | ✅ |

### 3.2 How this work differs from existing efforts

**seL4** has the most rigorous formal-verification posture in the field — full functional-correctness proofs over ~10K lines of C code, with proof corpus 20× the source size. The seL4 team has spent ~25 person-years building this. We **explicitly cede this lane** to seL4: Sphragis claims narrower information-flow non-interference proofs on critical subsystems via Verus, not whole-kernel functional correctness. This is a more tractable proof scope for our staffing model and complements rather than competes with seL4's work.

**Hubris, Asterinas, Theseus, RedoxOS** are Rust microkernel projects with no formal-verification track record. The proof patterns produced by this grant would directly benefit them.

**Verus itself** (the tool we will use) is actively developed at Microsoft Research and Carnegie Mellon University. It is open-source (MIT licensed) and has documented success on smaller proofs. Scaling Verus to production-kernel non-interference properties is genuinely novel work and would benefit the Verus community as well.

### 3.3 Why nobody else is doing this

The combination — Rust microkernel + Verus non-interference proofs + production kernel scale — does not exist today. Each individual ingredient exists:

- Rust microkernels exist (Hubris, Asterinas, Theseus, RedoxOS, Sphragis)
- Verus exists and is maturing rapidly
- Non-interference is a well-understood property from the formal-methods literature

But integrating all three at production scale, on a microkernel deployable to European critical infrastructure, with the resulting patterns Apache-2.0 licensed for free reuse — that combination is what this grant funds.

---

## §4 — Technical challenges + risks

**Risk 1 — Verus toolchain immaturity.** Verus is research-grade software actively developed; toolchain regressions are possible mid-engagement. *Mitigation:* parallel Kani (AWS-supported, more mature) fallback for memory-safety regression tests; reduce proof scope to single-function granularity if dispatcher-level proofs prove intractable.

**Risk 2 — Proof complexity outstrips budget.** Non-interference proofs at production-kernel scale are inherently uncertain. *Mitigation:* the proposal scope already accommodates a "partial proof + analysis" deliverable if the full dispatcher proof exceeds budget. The reusable patterns + paper are still publishable from a partial result.

**Risk 3 — Solo-founder bandwidth.** Sphragis is currently single-maintainer. *Mitigation:* the grant explicitly funds an external Verus consultant (≈25% allocation, ~200 hours) to supplement principal investigator capacity specifically on the formal-methods work.

**Risk 4 — Sphragis project maintenance during the grant.** Six months of focused formal-methods work means less time on other Sphragis subsystems. *Mitigation:* the project's existing 85-script QMP-driven self-test harness catches regressions; cargo-deny + cargo-audit CI enforces dependency hygiene autonomously.

---

## §5 — Ecosystem

### 5.1 What this project depends on

**Direct technical dependencies:** all permissively licensed (MIT / Apache-2.0 / BSD), zero copyleft. Enforced by `cargo-deny` in CI. Key dependencies:
- Rust toolchain (Rust Foundation, MIT/Apache-2.0)
- LLVM (LLVM Foundation, Apache-2.0)
- Verus (Microsoft Research + CMU, MIT)
- ml-kem, ml-dsa (RustCrypto family, MIT/Apache-2.0)
- aes, aes-gcm-siv, sha2, sha3, blake3, argon2, chacha20poly1305 (RustCrypto family, MIT/Apache-2.0)
- ed25519-compact, x25519-dalek, p256/p384 (various, MIT/Apache-2.0)
- x509-cert, spki, der (RustCrypto, MIT/Apache-2.0)
- QEMU (for test infrastructure only, GPL — not linked into the kernel)

**No vendored binaries in the kernel-relevant tree.** Total transitive Cargo dependency graph: under 200 crates, all permissively licensed.

### 5.2 What depends on this project

Sphragis is in pre-1.0 status with no established downstream dependents yet. The proof patterns funded by this grant are designed to be adopted by:

- **Other Rust microkernel projects** seeking similar formal-methods backing: Hubris, Asterinas, Theseus, RedoxOS
- **The Verus tool itself** — production-scale proof patterns inform Verus development
- **The seL4 community** as a comparison case (different language, different proof scope)
- **Academic kernel-verification researchers** at CMU, MPI-SWS (Germany), IMDEA Software Institute (Spain), INRIA (France), and similar groups

European institutions specifically interested in Sphragis as a substrate: BSI (German federal cybersecurity agency), ANSSI (French equivalent), and the broader EUCC certification track (entered force February 2026). These are stakeholders in the formal-methods completion, not yet active deployers.

---

## §6 — Duration + budget

### 6.1 Duration: 6 months

The work packages are designed to be sequential within the 6-month period:

| Month | Work |
|---|---|
| 1 | Verus toolchain CI integration; capability-dispatcher proof scoping |
| 2 | Capability-dispatcher proof: first invariants |
| 3 | Capability-dispatcher proof completion |
| 4 | IPC subsystem proof: AF_UNIX + pipes |
| 5 | IPC subsystem proof: shared memory; paper draft |
| 6 | Paper submission; reusable patterns documented; final report |

### 6.2 Budget: €50,000

| Line item | Amount (€) |
|---|---|
| Principal investigator labor (~500 hours @ €70/hr loaded) | 35,000 |
| External Verus consultant (~200 hours @ €60/hr) | 12,000 |
| Paper-publication open-access fee | 1,000 |
| Hosting / cloud / domain (6 months) | 800 |
| Conference travel (FOSDEM 2027 to present results) | 1,200 |
| **Total** | **50,000** |

**Cost share:** the principal investigator's nominal full-time salary at market rates would be approximately €100K/year loaded. The €70/hour rate accepted under this grant represents an in-kind contribution of approximately 30% of fair-market compensation, indicating committed cost-sharing.

---

## §7 — Have you worked on this before?

The principal investigator has built the entirety of Sphragis from a standing start over approximately 14 weeks, including:

- Independent reverse-engineering of the Apple M4 boot path (Asahi Linux does not yet support M4)
- ~96,000 lines of memory-safe Rust kernel code
- 85 QMP-driven self-test scripts
- The `verification/` directory scaffolding with smoke proofs and two written non-interference specifications (`verification/cap_dispatch/SPEC.md`, `verification/ipc_flow/SPEC.md`)
- Comprehensive design + threat-modeling + security-target documentation (~6,000 lines of strategic planning material in `docs/superpowers/`)

The Verus proof completion is the natural next step on the formal-methods axis. The proof specifications already authored represent the project-specific learning curve being already complete; the grant funds the actual proof writing.

---

## §8 — Prior funding

**No funding received to date** from any source. The principal investigator has self-funded the work from personal savings.

**Parallel applications planned or in flight:**

- Sovereign Tech Fund (Germany) — €120K, separate scope covering CNSA 2.0 cryptographic module + SLSA Level 4 supply-chain provenance + this Verus work as part of a larger package; submitted 2026-05-17
- OpenSSF Alpha-Omega Project — $10-50K, different scope (security-infrastructure broadly defined)
- GitHub Accelerator — $40K + mentorship if current cohort window is open
- US Federal SBIR Phase I (DoD SBIR 26.2 + AFWERX Open + DARPA SBIR) — $75K, entirely different US-federal scope

**No duplicate-funding risk between this NLnet application and any other.** The €50K NLnet scope is narrowly focused on the Verus non-interference proofs (Work Package 2 of the larger Sphragis project). If both NLnet and STF award funding, the NLnet portion would fund the Verus work specifically; STF would fund the parallel cryptographic + supply-chain work. Transparent coordination would be reported to NLnet per standard grant terms.

---

## §9 — License + open source compliance

All Sphragis source code is licensed under **Apache License 2.0** (`SPDX-License-Identifier: Apache-2.0`). Verified at the repository root `LICENSE` file. All grant-funded deliverables — the Verus proofs, the reusable proof patterns, the publication paper, the technical documentation — will be Apache-2.0 licensed.

Dependency policy: `cargo-deny` CI gate explicitly denies GPL-2.0, GPL-3.0, AGPL-3.0, LGPL-2.1, LGPL-3.0, SSPL-1.0, Commons-Clause, BUSL-1.1. Only permissive licenses (Apache-2.0, MIT, BSD, ISC, Unicode, CC0, Zlib) are permitted.

Contribution model: Developer Certificate of Origin (DCO) v1.1 — the same lightweight model used by the Linux kernel. No CLA required.

---

## §10 — Team

> **[FOUNDER TO FILL]**

### 10.1 Principal investigator: [Kaden Lee]

- Founder + sole maintainer of Sphragis
- Reverse-engineered the Apple M4 boot path independently (Asahi Linux does not yet support M4)
- 14 weeks of full-time Sphragis development; ~96K LoC + ~6K LoC of strategic planning documentation
- Public artifact: https://github.com/kadenlee1107/Sphragis
- [Education: degree, institution, year]
- [Prior professional roles relevant to systems software, security, or open-source]
- [Citizenship + country of residence: required for grant administration]

### 10.2 External consultant — Verus formal-methods expertise

Approximately 25% allocation for the 6-month grant period. Specific consultant to be identified within Month 1; candidate pool includes:
- Verus development team (Microsoft Research / Carnegie Mellon University, US)
- Verified Rust Group (University of California system, US)
- **European formal-methods groups (preferred):** IMDEA Software Institute (Madrid, Spain), MPI-SWS (Kaiserslautern/Saarbrücken, Germany), INRIA (Paris/Saclay, France), University of Cambridge Computer Lab (UK), KTH Royal Institute of Technology (Sweden)

European consultant preferred for NLnet's NGI ecosystem alignment — keeps grant funds within the European research ecosystem and seeds EU-based formal-methods expertise.

---

## §11 — Compare with similar (already-funded) NLnet projects

NLnet has historically funded multiple projects in adjacent areas:

- **Sequoia PGP** — Rust-based OpenPGP implementation (analogous to Sphragis in being a memory-safe Rust security infrastructure)
- **rust-tls** — Rust TLS implementation
- **NTRU / PQ-related projects** — post-quantum cryptography
- **OpenPGP CA** — operator-CA tooling
- **Various formal-methods + verification tooling projects** under NGI Assure

Sphragis's Verus proof work continues this trajectory: memory-safe Rust security infrastructure with formal-methods backing, public benefit through reusable patterns. The Apache-2.0 license + DCO contribution model match NLnet's stated preferences for open-source license posture.

---

## §12 — Contact

> **[FOUNDER TO FILL]**

- **Name:** [Kaden Lee]
- **Email:** sphragis-os@proton.me
- **GitHub:** https://github.com/kadenlee1107
- **Address:** [residential or business address for grant administration]
- **Country:** [United States of America / wherever you are]
- **IBAN (for Dutch wire transfer):** [bank account details — needed if awarded; ideally a Mercury or Brex account opened post-incorporation]
- **Native language:** [for understanding-of-application purposes]

---

## §13 — Submission notes

**Submission method:** https://nlnet.nl/propose (NLnet's rolling-submission portal)

**Form structure:** NLnet's form is typically much shorter than this document. Their UI usually presents ~10 fields with character limits per field. Adapt sections above to fit the actual form — most fields will compress to the short version while preserving the technical substance.

**Cycle timing:** NLnet runs roughly quarterly calls (Feb, May, Aug, Nov are common deadlines). Rolling intake means submissions arriving after a deadline simply enter the next cycle. Check https://nlnet.nl/ for current call status.

**Response time:** typically 4-12 weeks after the cycle deadline. NLnet sends a personal response either accepting or declining; declined applications receive constructive feedback.

**Tactical note:** NLnet explicitly welcomes EARLY-STAGE projects with "bright ideas." Their stated mission: "We fund people with bright ideas — and the technical know-how to make them happen." Sphragis matches this profile exactly. This is the highest-probability shot of the parallel-grants portfolio.

---

**End of NLnet application draft v0.**

*Apply early, the cycle is rolling — submissions are evaluated as they arrive.*

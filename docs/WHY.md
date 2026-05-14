# Why Sphragis

**High-assurance computing rests on a kernel nobody designed for it.**

Linux is the de facto operating system for everything from public cloud to embedded medical devices to military communications. Its design is a triumph of openness, but it was built for general-purpose desktops in 1991 and has grown into a codebase exceeding thirty million lines, maintained by thousands of contributors across hundreds of organizations. The result is an enormous attack surface and a CVE landing every few days. For environments where a kernel compromise is catastrophic — defense, intelligence, critical infrastructure, compliance-regulated systems — *"we run Linux because everyone runs Linux"* is a choice, not a defense.

The conditions for a credible alternative have only recently aligned. Three shifts converged in the last twenty-four months. **First, memory safety has consequences.** Microsoft and Google have independently published data showing that roughly 70% of security vulnerabilities trace to memory-safety bugs — a class that Rust eliminates by construction. A kernel written in Rust isn't an aesthetic choice; it is a structural reduction of the attack surface by the largest single category of historical CVEs. **Second, Apple Silicon has become the most capable consumer-accessible AArch64 hardware**, and the security-research community has been mostly locked out of it. Asahi Linux opened the door on M1 and M2. Sphragis walks through it on M4 and builds for security as the primary goal rather than as a feature added on. **Third, AI-augmented development is real.** A single architect equipped with strong AI tooling can now ship what previously required a team of fifteen. Fewer hands on the keyboard is itself a security property — smaller supply-chain surface, fewer review boundaries, no contributor whose key got phished last quarter.

## What Sphragis does differently

- **No ambient authority.** There is no `root`, no `sudo`. Every destructive privileged operation — wiping the audit log, downgrading a file's classification, rotating the master key, flushing an off-platform audit seal — requires a fresh M-of-2 Ed25519 quorum from two pre-registered officers. A single compromised key does not get you privileged operations.
- **Defense-in-depth by construction, not by accretion.** Bell-LaPadula sensitivity, Biba integrity, SELinux-style type enforcement, AEAD-bound classification labels, and information-flow taint propagation are stacked together as primitives the kernel was designed around — not bolted onto a kernel that was already shipping.
- **Memory safety end-to-end.** Sphragis is written in Rust. Every transitive dependency is permissive-licensed and continuously audited against the OSV.dev vulnerability feed.
- **Reproducible and transparency-logged.** Every release artifact is signed with a project Ed25519 key and appended to a Rekor-compatible Merkle transparency log, with in-toto v0.9 attestations for each build step.

## Evidence

Sphragis boots on real Apple M4 hardware (Mac16,1 / J604 / T8132 "Donan") via an independent reverse-engineering pipeline — no Asahi base, no upstream Linux fork. Over the past months it has also shipped approximately fifty distinct security primitives, each backed by a headless QEMU selftest that anyone can run in under a minute. Every claim in this document is independently verifiable by anyone with a Rust toolchain and an hour. See [`RECEIPTS.md`](RECEIPTS.md) for the full mapping from claim to commit to selftest.

## Where this goes

Near-term work stabilizes the cave isolation primitive, completes the off-platform audit anchor (HSM / TPM / Apple Secure Enclave integration), and opens the repository under AGPL-3.0 with a commercial license tier for closed-source integration. Medium-term, the goal is pilot deployments in compliance-regulated environments — defense, intelligence, healthcare cryptography — paired with formal threat modeling and a third-party security audit. Long-term, Sphragis aims to become the default substrate for high-assurance computing on non-x86 hardware.

---

Built and maintained by Kaden Lee with paired-programming assistance from Anthropic's Claude (Sonnet 4.6 / Opus 4.7). Architectural decisions, threat-model judgements, and the responsibility for what ships are human; the AI is a productivity multiplier, not the author.

Contact: GitHub [@kadenlee1107](https://github.com/kadenlee1107). Commercial license inquiries welcome.

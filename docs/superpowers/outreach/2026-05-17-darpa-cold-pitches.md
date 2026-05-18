# Sphragis — DARPA PM Cold-Pitches (Draft v1)

**Status:** DRAFT v1 — Kaden reviews + sends. Outreach team only drafts.
**Date:** 2026-05-17
**Author:** Outreach team (2026-05-17 multi-team push)
**Source:** `docs/superpowers/funding/2026-05-17-darpa-pm-prep-v1.md` (DARPA PM Prep v1) §1–§8. Public anchor: `https://github.com/kadenlee1107/Sphragis`.

**Targets (per charter §3 Outreach):**
1. **PROVERS PM** — Pipelined Reasoning of Verifiers Enabling Robust Systems (I2O). HIGHEST-fit program per DARPA prep §2. PM identity rotates; addressee is role-based pending verification.
2. **RSSC PM** — Resilient Software Systems Capstone (I2O). HIGH-fit, lowest-friction entry. PM identity per DARPA prep §2.
3. **TRACTOR PM** — Translate All C to Rust (I2O). LOW-MEDIUM fit per DARPA prep §2 §TRACTOR, but a real angle exists ("Sphragis as a Rust target codebase for TRACTOR-translated C drivers"). Chosen as the third per charter "one more if identified in `darpa-pm-prep-v1.md`" because INSPECTA is counselled in DARPA prep §2 as "wait for PROVERS or RSSC engagement first" rather than a direct cold-pitch target.

**Tone note (DARPA prep §8 red flag #5):** "PM 1-on-1s reward technical honesty more than salesmanship. If asked 'what doesn't work yet?' lead with the truth (Verus proofs are spec'd not complete; x86_64 is designed not built; FIPS not yet certified)." These emails are more technical and more honest about gaps than the VC or prime emails.

**Common framing all three emails reuse (paragraph 1 template, derived from DARPA prep §6):**
*"I lead Sphragis Inc., a single-founder startup building a memory-safe Rust microkernel targeted at the 2027-01-01 NSA CNSA 2.0 acquisition mandate. The kernel boots today on Apple M4 hardware (independent reverse-engineering pipeline — Asahi Linux doesn't yet support M4); the cryptographic module is CNSA-2.0-native (ML-KEM-1024, ML-DSA-87, AES-256, SHA-384); we've shipped 96K lines of Rust with a 14-week mechanical-trace security audit history."*

Each email below: subject line, addressee, body. Tailoring lives in paragraphs 2 + 3 per charter §3 DoD. Founder signature block omitted — Kaden fills before send.

---

## Email 1 — PROVERS PM

**To:** DARPA I2O Program Manager — PROVERS (Pipelined Reasoning of Verifiers Enabling Robust Systems). DARPA prep §4 instructs: verify the current PM at `https://www.darpa.mil/program/pipelined-reasoning-of-verifiers-enabling-robust-systems`; if the program page does not name the PM directly (some are pseudonymous for OPSEC), use the COR / TPOC named on the most recent SAM.gov BAA listing. Kaden must verify before send.
**Addressee line:** "Dear [PROVERS PM name],"
**Subject:** `Sphragis — Rust microkernel as a PROVERS application substrate (request for 30-min meeting)`

---

Dear [PROVERS PM name],

I lead Sphragis Inc., a single-founder startup building a memory-safe Rust microkernel targeted at the 2027-01-01 NSA CNSA 2.0 acquisition mandate. The kernel boots today on Apple M4 hardware (independent reverse-engineering pipeline — Asahi Linux doesn't yet support M4); the cryptographic module is CNSA-2.0-native (ML-KEM-1024, ML-DSA-87, AES-256, SHA-384, LMS); we've shipped 96K lines of Rust with a 14-week mechanical-trace security audit history. I'm writing because I believe Sphragis is a natural fit for PROVERS — not as another verification-tool performer, but as a production-scale Rust kernel where PROVERS verification tooling lands publication-grade evidence at deployable-system scale.

Technical fit specifics relevant to PROVERS' research agenda:

- **Verus harness on the capability dispatcher + IPC paths.** Two written non-interference proof specifications (`verification/cap_dispatch/SPEC.md` and the IPC information-flow spec) are committed; partial Verus proofs are in flight. The intended end-state is mechanized information-flow non-interference on the critical-subsystem boundary — exactly the deliverable PROVERS exists to advance, but on a real kernel with real drivers and a real boot path rather than a textbook example. Existing PROVERS performers focus on tool development (Verus, F\*, Iris); Sphragis is the application substrate that lets a PROVERS performer claim publication-grade scale evidence.
- **What does NOT work yet (per DARPA prep §8 red-flag-5 honesty discipline).** The Verus proofs are spec'd, not complete. The x86_64 port is designed, not built. FIPS 140-3 is module-boundary-documented, not certified. The kernel TCB is 96K LoC of Rust but the formal-proof coverage today is partial-spec on two subsystems. A PROVERS engagement would let us close those gaps with PM-aligned milestones and tooling support, and would produce a deliverable artifact (mechanized proof + reproducibility evidence) defensible at a DARPA program-review level.
- **Reproducibility + provenance discipline.** Bit-identical reproducible build verified end-to-end (SLSA-L4 chain artifacts at `scripts/check_reproducible_build.sh`); Apache-2.0 licensed; DCO sign-off on every commit. The substrate is publishable in full as program-evaluation evidence with no closed-source obstructions.

I'd value 30 minutes of your time in the next four-to-six weeks for a video walk-through of (a) a live M4 boot, (b) the `verification/cap_dispatch/SPEC.md` proof spec + the partial Verus harness, (c) the `attest::quote()` flow with the external verifier in `tools/attest-verifier/`. I can travel to Arlington, can do video at your convenience, and can have a 1-page PROVERS-specific program-fit memo (per DARPA prep §5 template) plus the master implementation plan in your inbox within 24 hours of mutual interest. Public evidence chain: https://github.com/kadenlee1107/Sphragis (Apache-2.0, DCO sign-off, bit-identical reproducible build verified). If 30 minutes is too long for a first conversation I'd value any shorter pointer — including a pointer to the closest-fit PM if PROVERS scope is not a match. Many thanks for your time and consideration.

[Founder signature block — Kaden to fill]

---

## Email 2 — RSSC PM

**To:** DARPA I2O Program Manager — Resilient Software Systems Capstone. DARPA prep §4 instructs: verify the current PM at `https://www.darpa.mil/program/resilient-software-systems-capstone`; if the program page does not name the PM directly, use the COR / TPOC named on the most recent SAM.gov BAA listing.
**Addressee line:** "Dear [RSSC PM name],"
**Subject:** `Sphragis — substrate the red team can't escape (RSSC performer inquiry)`

---

Dear [RSSC PM name],

I lead Sphragis Inc., a single-founder startup building a memory-safe Rust microkernel targeted at the 2027-01-01 NSA CNSA 2.0 acquisition mandate. The kernel boots today on Apple M4 hardware (independent reverse-engineering pipeline — Asahi Linux doesn't yet support M4); the cryptographic module is CNSA-2.0-native (ML-KEM-1024, ML-DSA-87, AES-256, SHA-384, LMS); we've shipped 96K lines of Rust with a 14-week mechanical-trace security audit history. I'm writing because RSSC's capstone framing — "demonstrate resilience under realistic adversary" — maps unusually cleanly to what Sphragis produces as evidence: kernel-enforced classification isolation between caves, an attestation primitive that survives operator turnover, an HMAC-SHA-384 append-only audit chain, and a substrate small enough (~96K LoC, ~300× smaller TCB than Linux) that a red team can actually reason about every code path it might attack.

Technical fit specifics relevant to RSSC's red-team-survival evaluation rubric:

- **Bell-LaPadula + Biba MLS labels enforced at every cross-cave syscall.** Per-cave (per-process) classification labels with kernel-mediated gates — "no read up" and "no write up" are kernel invariants, not userspace conventions. The red-team bypass test bench is already partially written (per SBIR Phase I Objective 1) and Sphragis is built to be tested — we'd welcome RSSC red-team adversarial evaluation as performer milestone evidence.
- **HMAC-SHA-384 audit chain with WORM segment export to SealFS and an offline Python verifier.** Tamper-evident, append-only, cap-aware reads, forensic-grade — this is the audit artifact RSSC red-team-survival evaluation needs to ratify post-engagement claims. The offline Python verifier (`tools/audit-verifier/audit_verifier.py`) runs against the WORM export with no kernel involvement, which is the property a third-party assessor wants.
- **What does NOT work yet (per DARPA prep §8 red-flag-5 honesty discipline).** Persistent label-history audit-log integration is a future-session item, not done. x86_64 port is designed not built. The kernel is live on M4 only today. The right RSSC scoping is a feasibility-study deliverable (~$500K / 6 months) that hardens the substrate against your specific threat model and documents red-team-survival metrics in your evaluation rubric.

I'd value 30 minutes of your time in the next four-to-six weeks for a video walk-through of (a) a live M4 boot, (b) the cross-cave label-enforcement test bench, (c) the audit-chain WORM export and offline-verifier flow, (d) the `attest::quote()` flow demonstrating per-cave attestable identity. I can travel to Arlington, can do video at your convenience, and can have a 1-page RSSC-specific program-fit memo plus the red-team-ready posture statement (DARPA prep §2 RSSC entry) in your inbox within 24 hours of mutual interest. If RSSC's first-time-performer threshold is unfriendly to startup-stage proposers, I would value any pointer to the right BAA cycle or partnering performer to approach. Public evidence chain: https://github.com/kadenlee1107/Sphragis (Apache-2.0, DCO sign-off, bit-identical reproducible build verified). Many thanks for your time and consideration.

[Founder signature block — Kaden to fill]

---

## Email 3 — TRACTOR PM

**To:** DARPA I2O Program Manager — TRACTOR (Translate All C to Rust). DARPA prep §4 instructs: verify the current PM via the DARPA staff roster + the most recent SAM.gov BAA listing for TRACTOR awards. Per DARPA prep §2 §TRACTOR, fit is "LOW-MEDIUM" — this email is honest about the modest fit and proposes a specific, narrow angle (Sphragis as a Rust target codebase for translated drivers) rather than a Phase I bid.
**Addressee line:** "Dear [TRACTOR PM name],"
**Subject:** `Sphragis — Rust kernel as a TRACTOR translation target (modest-fit inquiry)`

---

Dear [TRACTOR PM name],

I lead Sphragis Inc., a single-founder startup building a memory-safe Rust microkernel targeted at the 2027-01-01 NSA CNSA 2.0 acquisition mandate. The kernel boots today on Apple M4 hardware (independent reverse-engineering pipeline — Asahi Linux doesn't yet support M4); the cryptographic module is CNSA-2.0-native (ML-KEM-1024, ML-DSA-87, AES-256, SHA-384, LMS); we've shipped 96K lines of Rust with a 14-week mechanical-trace security audit history. I'm reaching out with a candid framing: Sphragis is greenfield Rust (TRACTOR's mission is *translating existing C codebases*), so the fit is modest at best. But I think there's a narrow, useful angle that may serve TRACTOR's evaluation needs, and I'd value 30 minutes to test that hypothesis with you directly rather than guess wrong.

The narrow angle:

- **Sphragis as a Rust target codebase for TRACTOR-translated C drivers, demonstrating "translated C running in a memory-safe Rust kernel context."** Today our driver coverage is: dockchannel UART, AIC, PMGR, ATC PHY, DCP framebuffer, DWC3 XHCI USB — all Rust-from-scratch. The natural integration pattern would be: TRACTOR-translates a representative legacy C driver (e.g. a Linux network driver or a BSD storage driver), Sphragis hosts it as a guest-cave driver under our cross-cave IPC isolation, and the joint artifact is a published demonstration that TRACTOR output runs in a real memory-safe Rust kernel with kernel-enforced isolation rather than only in a synthetic harness. Apache-2.0 license means the demonstration artifact is publishable in full as program-evaluation evidence.
- **What's already in place that lowers the integration cost.** Memory-safe Rust microkernel TCB is ~96K LoC (~300× smaller than Linux); the cave model gives natural isolation for an untrusted-driver pattern; the kernel is `no_std`, edition 2024, with 0 warnings and 0 clippy lints on the pinned-nightly toolchain (toolchain pin documented). The mechanical-trace audit history + DCO sign-off + reproducible-build chain (SLSA-L4) gives a program-review-defensible joint artifact.
- **Honest about what doesn't fit.** Sphragis is not itself a C-to-Rust translation effort. We won't pitch ourselves as a TRACTOR performer. The right framing is "we're a useful target for your translator output to land in" — a comparable position to a friendly downstream consumer rather than a peer performer. If that framing is uninteresting at the current TRACTOR phase I'd value a redirect to the closest-fit DARPA PM (PROVERS or RSSC are the higher-fit programs per our own analysis at `docs/superpowers/funding/2026-05-17-darpa-pm-prep-v1.md` §2).

I'd value 30 minutes of your time in the next four-to-six weeks for a video walk-through, even if the outcome is a clean "no fit — try PROVERS or RSSC." I can travel to Arlington, can do video, and can have a 1-page TRACTOR-specific program-fit memo in your inbox within 24 hours of mutual interest. Public evidence chain: https://github.com/kadenlee1107/Sphragis (Apache-2.0, DCO sign-off, bit-identical reproducible build verified). Many thanks for your time and consideration.

[Founder signature block — Kaden to fill]

---

## What Kaden does next

1. **Verify each PM name before send.** DARPA prep §4: the program pages may not name a PM directly; if not, the COR / TPOC on the most recent SAM.gov BAA listing is the right addressee. PMs rotate (3–5 year terms) — the names here must be current.
2. Fill the founder signature block on each email (DARPA prep §6 template: founder name, email, phone, "Founder & CEO, Sphragis Inc.", public evidence chain URL).
3. **Sequencing** — per DARPA prep §7: send cold-outreach to PROVERS PM + RSSC PM in M2–M3 (i.e. now), TRACTOR is opportunistic and can be paced 2–4 weeks behind. Do NOT fire all three the same day — DARPA PMs sometimes compare notes; staggered outreach is more professional.
4. **Prepare the 1-page program-fit memos** before send (DARPA prep §5 template). Each PM cold-pitch promises "1-page program-fit memo in your inbox within 24 hours of mutual interest" — that memo needs to be ready to send the moment a reply arrives. Outside Outreach team's scope to author the program-fit memos themselves (those are technical artifacts Kaden authors); but the promise needs to be deliverable.
5. **DARPA Forecast to Industry (fall 2026)** is the single highest-leverage event per DARPA prep §3. If a 1-on-1 portal opens before any of these cold-pitches lands a meeting, reserve slots there as the primary channel and treat the cold emails as secondary.
6. **If TRACTOR PM declines or doesn't reply, that's expected.** The honest-fit framing in Email 3 is designed to either (a) land a useful narrow conversation or (b) generate a redirect referral to a higher-fit PM. Either outcome is a win.

**Do NOT send these emails until Kaden has reviewed personally and verified each PM name + email address.** Outreach team only drafts.

# Sphragis — DARPA PM Identification + Meeting Prep

**Version:** v1, 2026-05-17
**Purpose:** Identify the highest-leverage DARPA PMs to approach, what to bring, what to say, in what order.
**Target programs (in priority order):**
1. **PROVERS** (Pipelined Reasoning of Verifiers Enabling Robust Systems) — I2O
2. **Resilient Software Systems Capstone** (RSSC) — I2O
3. **INSPECTA** (formal methods on seL4; risk of competitor positioning)
4. **TRACTOR** (Translate All C to Rust) — I2O
5. **V-SPELLS** (formal methods on legacy software) — I2O

---

## §1 — Why DARPA before all others

DARPA programs offer the best **technical fit + funding tier + procurement gate** combination for Sphragis. Per the Phase-1 procurement research:

- **Funding tier:** $3-15M per performer over 3-4 years (extrapolated from HACMS / CASE program patterns; INSPECTA specifics not public).
- **Procurement gate:** A DARPA program performer gets visibility into the gov-cyber-substrate community that gates ACT 3 task orders + NSA CSfC components-list inclusion + FFRDC (MITRE / MIT LL / JHU/APL / GTRI) relationships.
- **Strategic positioning:** DARPA performer status converts to In-Q-Tel pitchability and Series A credibility for defense-focused VCs.

The DARPA ecosystem is the gateway to the rest of the federal funnel. **A 30-minute meeting with the right PM is more strategically valuable than a $1M commercial seed check** at this stage.

---

## §2 — Program-by-program fit analysis

### PROVERS — Pipelined Reasoning of Verifiers Enabling Robust Systems

**Office:** I2O (Information Innovation Office)
**Status:** Execution phase (launched April 2023; multi-year). Specific PM identity rotates.
**Fit for Sphragis:** **HIGHEST**
- PROVERS exists specifically to advance formal-methods tooling and integration for software assurance in critical systems.
- Sphragis's Rust microkernel + Verus harness + IPC non-interference proof spec are direct fit deliverables.
- Existing PROVERS performers focus on tool development (Verus, F*, Iris); Sphragis is the **application substrate** that makes their tools relevant to a deployable product.
- A "Sphragis-on-PROVERS" performer pitch is "we apply PROVERS tooling to a real-world kernel; you get publication-grade evidence that your tools work at scale."

**Likely PM channel:** DARPA Forecast to Industry (annual fall, DC); request 1-on-1 with current PROVERS PM.

**What to bring:**
- Live demo of `verification/cap_dispatch/SPEC.md` + partial Verus proof
- Boot-evidence photo set (M4)
- Threat model + Security Target (CC:2022 Part 1 conformant)
- 1-page program-fit memo (see §5 below)

### Resilient Software Systems Capstone (RSSC)

**Office:** I2O
**Status:** Dec 2025 start; March 2026 first red team; ~24-month projects per service.
**Fit:** **HIGH** (lowest-friction entry)
- RSSC is newer than PROVERS; more open BAA cycles; more accessible to first-time DARPA performers.
- Capstone framing means "demonstrate resilience under realistic adversary"; Sphragis's audit chain + attestation + MLS isolation produce evidence that maps directly to red-team-survival metrics.
- Pitch angle: "Sphragis is the substrate the red team can't escape from — we'd like a performer slot to harden the substrate against your specific threat model."

**Likely PM channel:** Same as PROVERS (DARPA Forecast). RSSC PM is more likely to take a first-time performer meeting than PROVERS.

**What to bring:**
- Live Quote() demo with external verifier
- Audit-chain WORM-export walkthrough
- Per-cave MLS bypass-attempt test bench (from SBIR Phase I Objective 1)
- Red-team-ready posture statement (we want to be tested; here's what we'd defend)

### INSPECTA — Integration of formal methods on seL4

**Lead performer:** Collins Aerospace (with CMU + UNSW + U Kansas)
**Status:** Funded; in execution.
**Fit:** **MEDIUM (with risk)**
- INSPECTA already has a performer team. Pitching ourselves as a "Rust alternative to the seL4 stack INSPECTA is funding" risks being framed as competition rather than collaboration.
- Better positioning: **partnership rather than performer**. "We use the formal-methods tooling INSPECTA produces; we offer a Rust-substrate-comparison dataset for INSPECTA's evaluation."
- Caveats: Collins Aerospace is a Lockheed competitor in some markets; INSPECTA selection process may not welcome a startup-stage proposer.

**Likely PM channel:** Indirect — reach via academic co-PI (CMU's Frank Pfenning lab or UNSW's seL4 team) rather than direct DARPA PM.

**What to bring:** Don't lead with INSPECTA. Wait for PROVERS or RSSC engagement to establish DARPA-performer credibility, then approach INSPECTA in a comparative-evaluation framing.

### TRACTOR — Translate All C to Rust

**Status:** ~$5M July 2025 contract; in execution.
**Fit:** **LOW-MEDIUM**
- TRACTOR is about translating *existing* C codebases to Rust. Sphragis is *greenfield* Rust.
- However: TRACTOR may want a Rust target codebase to demonstrate "translated C running in a memory-safe Rust kernel context." Sphragis could be that target.
- Pitch angle: "we'll integrate TRACTOR-translated C drivers into Sphragis as proof-of-deployment for your translator." Modest fit.

**Likely PM channel:** Lower priority; mention to TRACTOR PM only if a natural opening arises at DARPA Forecast.

### V-SPELLS — Verified Security and Performance Enhancements for Large Legacy Software

**Status:** Multi-year; specific PM rotation.
**Fit:** **LOW**
- V-SPELLS focuses on legacy-software enhancement. Sphragis is greenfield. Limited overlap.

**Skip unless a clear opening emerges.**

---

## §3 — DARPA Forecast to Industry — the single most important event

**DARPA Forecast to Industry** is the annual fall meeting in DC where DARPA PMs publicly preview upcoming programs and accept short meetings with prospective performers. This is **the single highest-leverage event** for Sphragis in 2026.

**When:** Typically early fall (Sept-Nov); check DARPA's events page for 2026 date.
**Where:** Washington DC
**Format:** Plenary sessions in the morning; one-on-one PM meetings (15-30 min slots) in the afternoon, by reservation through DARPA's portal.
**Cost to attend:** Free attendance; travel ~$1.5K for one founder.

**Prep timeline (assuming Forecast in October 2026):**

- **8 weeks before:** Identify target PMs from public DARPA PM roster (https://www.darpa.mil/staff). Reserve 1-on-1 slots through the portal as soon as it opens.
- **4 weeks before:** Finalize 1-page program-fit memos for each PM you're meeting (see §5). Confirm capability statement is on the public marketing site.
- **2 weeks before:** Practice the 5-minute pitch. Rehearse Q&A. Print 5 capability statements + 5 program-fit memos for handouts.
- **Day of:** Arrive early. Attend plenaries to take notes on program directions. Pitch 1-on-1s with PM. Take business cards from EVERY meeting (FFRDC reps, prime BD, academic researchers — all valuable downstream).
- **48 hours after:** Send personalized follow-up email to every PM met, with the 1-page memo as PDF attachment + GitHub link.

---

## §4 — How to identify the current PMs

DARPA PMs rotate (typical 3-5 year terms). The current PM for any given program is published on the DARPA program page. Check:

- https://www.darpa.mil/program/pipelined-reasoning-of-verifiers-enabling-robust-systems (PROVERS)
- https://www.darpa.mil/program/resilient-software-systems-capstone (RSSC)
- https://www.darpa.mil/staff (full staff roster — search by program assignment)

If the program page doesn't list a PM directly (some are pseudonymous for OPSEC reasons), the BAA listing on SAM.gov for the most recent program announcement will identify the contracting officer's representative (COR) and the technical point of contact (TPOC). Both are valid meeting targets.

Also useful:

- **DARPA's I2O office director** — currently a public-facing figure; their public talks indicate program priorities.
- **Recent BAA awards** (announced via DARPA news releases) identify which performers are already in each program, which informs partnership-vs-competition framing.

---

## §5 — Program-fit memo template (1 page per program)

```
─────────────────────────────────────────────────────────────────
SPHRAGIS — PROGRAM FIT MEMO
[Program name]
Prepared for: [PM name, if known]
Prepared by: [Founder name], Founder & CEO, Sphragis Inc.
Date: [date]
─────────────────────────────────────────────────────────────────

ONE-LINE PITCH
[Program-specific value prop. For PROVERS: "Sphragis is the
production-scale Rust kernel where PROVERS verification
tooling lands evidence." For RSSC: "Sphragis is the substrate
the red team can't escape from."]

WHY US, WHY NOW
- [Program technology gap Sphragis fills]
- [Procurement window urgency — typically the 2027 CNSA cliff]
- [Demonstrated technical credibility — link to public evidence]

WHAT WE'D PROPOSE
- [3-4 bullets describing a Phase I-style ~6-month / ~$500K
   feasibility study scoped to the program's evaluation criteria]
- [Specific deliverables that map to the program's BAA language]

WHAT WE BRING
- 96K-line Rust microkernel, live on Apple M4 hardware
- CNSA-2.0-native crypto (ML-KEM-1024, ML-DSA-87, AES-256, SHA-384)
- Verus harness + 2 written non-interference proof specifications
- Attestation primitive with external verifier tool
- 14-week mechanical-trace security audit history
- Apache-2.0 licensed; reproducible build verified
- Public evidence chain: github.com/kadenlee1107/Sphragis

WHAT WE'D WANT FROM THIS MEETING
- 30 minutes of your time to demonstrate the substrate
- Feedback on whether the program's scope can accommodate our
  approach
- If yes: pointer to the next BAA cycle or open RFI we should
  respond to
- If no: pointer to the closest-fit program or PM in DARPA

CONTACT
[Founder] · [email] · [phone]
─────────────────────────────────────────────────────────────────
```

Print 5 copies of each program-specific memo + bring 10 generic capability statements for ad-hoc conversations.

---

## §6 — Cold outreach template (if Forecast is months away)

Subject: `Sphragis — Rust microkernel for the 2027 CNSA cliff`

```
Dear [PM name],

I lead Sphragis Inc., a single-founder startup building a memory-safe
Rust microkernel targeted at the 2027-01-01 NSA CNSA 2.0 acquisition
mandate. The kernel boots today on Apple M4 hardware (independent
RE pipeline — Asahi doesn't yet support M4); the cryptographic
module is CNSA-2.0-native (ML-KEM-1024, ML-DSA-87, AES-256,
SHA-384); we've shipped 96K lines of Rust with a 14-week
mechanical-trace security audit history.

I'm writing because I believe Sphragis is a natural fit for
[PROGRAM NAME] as [PERFORMER / PARTNER / EVALUATION TARGET].
Specifically:

- [Program-specific fit bullet 1]
- [Program-specific fit bullet 2]
- [Program-specific fit bullet 3]

I'd value 30 minutes of your time in the next 4-6 weeks to walk
through a live demonstration and discuss whether the substrate
maps to your program's research agenda. I can travel to DC, Arlington,
or arrange a video call at your convenience.

Public evidence chain: https://github.com/kadenlee1107/Sphragis
(Apache-2.0 licensed, DCO sign-off, bit-identical reproducible builds)

Many thanks for your time and consideration.

Best regards,
[Founder name]
Founder & CEO, Sphragis Inc.
[email] · [phone]
```

---

## §7 — Sequencing

| Step | Timing | Outcome target |
|---|---|---|
| Confirm incorporation + SAM.gov in flight | Now | Required for any contracting conversation |
| Register on https://www.darpa.mil/work-with-us/contracts and SAM.gov | M0-M2 | Eligibility baseline |
| Send cold-outreach emails to PROVERS PM + RSSC PM | M2-M3 | 1-on-1 video meetings before Forecast |
| Submit DARPA SBIR Phase I (parallel to DoD SBIR + AFWERX) | M3 | Phase I award maximizes Forecast meeting credibility |
| Attend DARPA Forecast to Industry (fall 2026) | M4-M6 (event-dependent) | 3-5 PM 1-on-1s, business-card harvest |
| Submit BAA response if invited from PM meeting | M6-M9 | Formal performer candidacy |
| Performer selection (if successful) | M9-M15 | Award $3-15M over 3-4 years |

---

## §8 — Red flags + reality checks

**1. DARPA SBIR rejection is the norm.** Submit to multiple programs in parallel; don't anchor expectations.

**2. PM 1-on-1s at Forecast are competitive.** Slot reservations fill within hours of portal opening. Reserve as soon as possible.

**3. PM rotation can derail a relationship.** A PM championing your work may rotate out mid-engagement. Cultivate relationships across multiple PMs in the same office.

**4. DARPA performer selection is opaque.** Don't expect timeline transparency. Plan financial runway assuming a 9-18 month performer-decision delay.

**5. Be honest about what you don't have.** PM 1-on-1s reward technical honesty more than salesmanship. If asked "what doesn't work yet?" lead with the truth (Verus proofs are spec'd not complete; x86_64 is designed not built; FIPS not yet certified).

---

**End of DARPA PM prep v1.**

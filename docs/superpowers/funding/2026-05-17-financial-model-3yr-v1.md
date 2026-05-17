# Sphragis — 3-Year Financial Model

**Version:** v1, 2026-05-17
**Horizon:** Q3 2026 → Q2 2029 (Months 0-36)
**Purpose:** Companion to the master plan, SBIR Phase I proposal, and VC pitch deck. Drives capital strategy + use-of-funds tables. Numbers track the master plan in `docs/superpowers/plans/2026-05-16-sphragis-gov-os-master-plan.md`.

> **Founder action:**
> - Replace `[FOUNDER SALARY]` placeholders with target take-home if you want a custom number (default below: $150K Y1 loaded, $175K Y2, $200K Y3)
> - Adjust hire dates if cash flow demands later starts (Eng4 Y2 Q1 → Y2 Q2 saves ~$50K of burn)
> - Adjust the conservative/base/aggressive scenario if you want a different VC ask anchor

---

## §1 — Headcount plan

| Period | Headcount | Composition | New hires this period |
|---|---|---|---|
| M0-9 (pre-Phase-I-award) | **1** | Founder + agent-augmented | — |
| M9-15 (Phase I + first hires) | **2** | + Eng1 Verus specialist | Eng1 @ M9 |
| M15-24 (Phase II ramp) | **4** | + Eng2 hw/x86_64 + Eng3 UX | Eng2 @ M15, Eng3 @ M18 |
| M24-36 (Phase II → III + commercial) | **5** | + Eng4 cert eng + Eng5 customer eng | Eng4 @ M24, Eng5 @ M30 |

**Loaded salary assumptions (US-based, fully remote, no office):**

| Role | Base | Loaded factor | Loaded annual |
|---|---|---|---|
| Founder/CEO | $150K Y1 / $175K Y2 / $200K Y3 | 1.25× | $187K / $219K / $250K |
| Senior engineer (Eng1 Verus, Eng2 hw) | $180K-$220K | 1.30× | $234K-$286K |
| Mid engineer (Eng3 UX, Eng4 cert) | $150K-$180K | 1.30× | $195K-$234K |
| Customer engineer (Eng5) | $160K-$190K | 1.30× | $208K-$247K |

> Loaded factor covers payroll tax (FICA + FUTA + SUTA), 401(k) match if offered, health insurance (~$15-25K/yr/head for individual coverage), workers comp, training/conference budget per head, equipment depreciation.

---

## §2 — Year-by-year P&L (base case)

### Year 1 (M0-12)

| Line item | Q1 | Q2 | Q3 | Q4 | **Y1 total** |
|---|---|---|---|---|---|
| **Revenue** | | | | | |
| SBIR Phase I | — | — | $25K | $50K | **$75K** |
| Commercial pilot | — | — | — | — | $0 |
| Total revenue | — | — | $25K | $50K | **$75K** |
| **COGS** (~negligible Y1) | — | — | — | — | **$0** |
| **Gross profit** | — | — | $25K | $50K | **$75K** |
| **Operating expenses** | | | | | |
| Salaries (founder + Eng1 from Q4) | $47K | $47K | $47K | $59K | $200K |
| Eng1 onboarding M9 | — | — | $58K | $58K | $117K |
| Conference + travel (AFCEA WEST, DARPA Forecast) | $15K | $5K | $5K | $5K | $30K |
| Legal + IP (incorp, BIS, trademark Y2) | $7K | $3K | $3K | $3K | $16K |
| Hardware (NUC, dev kits, M4 spare) | $3K | $2K | $5K | $2K | $12K |
| CMVP lab pre-engagement | — | $10K | $20K | $20K | $50K |
| Insurance (GLI + E&O) | $1K | $1K | $1K | $1K | $4K |
| Cloud + domains + SaaS tooling | $1K | $1K | $1K | $1K | $4K |
| Marketing / website / content | $3K | $2K | $2K | $3K | $10K |
| Conference + USENIX submission | — | $5K | $5K | $3K | $13K |
| Contingency / G&A | $4K | $4K | $4K | $6K | $18K |
| **Total OpEx** | **$81K** | **$80K** | **$151K** | **$161K** | **$474K** |
| **Operating income** | -$81K | -$80K | -$126K | -$111K | **-$398K** |

**Y1 burn (excluding founder personal funds):** $398K
**Y1 SBIR offset:** $75K
**Y1 net cash needed:** **$323K** before VC seed closes

### Year 2 (M13-24)

| Line item | Q1 | Q2 | Q3 | Q4 | **Y2 total** |
|---|---|---|---|---|---|
| **Revenue** | | | | | |
| SBIR Phase II ($1.25M / 21mo straight-line) | $180K | $180K | $180K | $180K | $720K |
| Commercial design partner (LOI Y1Q4 → contract Y2Q2) | — | $50K | $100K | $150K | $300K |
| ACT 3 task-order sub | — | — | $25K | $50K | $75K |
| Total revenue | $180K | $230K | $305K | $380K | **$1,095K** |
| **OpEx** | | | | | |
| Salaries (Founder + Eng1 + Eng2 from M15 + Eng3 from M18) | $115K | $128K | $182K | $195K | $620K |
| FIPS 140-3 lab testing in earnest | $40K | $50K | $50K | $50K | $190K |
| Conference + travel | $10K | $15K | $5K | $10K | $40K |
| Hardware (CHERIoT board, ARM server testbed) | $5K | $3K | $3K | $2K | $13K |
| Legal + IP (trademark + counsel + new hires) | $5K | $5K | $5K | $5K | $20K |
| Marketing + biz dev (capability stmt update, deck refresh) | $5K | $5K | $5K | $5K | $20K |
| Insurance | $1K | $1K | $1K | $1K | $4K |
| Cloud + SaaS | $2K | $2K | $2K | $3K | $9K |
| Contingency / G&A | $9K | $10K | $10K | $11K | $40K |
| **Total OpEx** | **$192K** | **$219K** | **$263K** | **$282K** | **$956K** |
| **Operating income** | -$12K | $11K | $42K | $98K | **$139K** |

**Y2 net cash positive!** $139K operating income. But cumulative free cash flow still needs covering Y1's $323K shortfall.

### Year 3 (M25-36)

| Line item | Q1 | Q2 | Q3 | Q4 | **Y3 total** |
|---|---|---|---|---|---|
| **Revenue** | | | | | |
| Phase II tail (M25-30) | $180K | $180K | $60K | — | $420K |
| Phase III sole-source / commercial pilot 1 conversion | $100K | $200K | $400K | $600K | $1,300K |
| ACT 3 task-order grown | $75K | $100K | $125K | $150K | $450K |
| Commercial Design Partner 2 (confidential AI) | — | $50K | $150K | $300K | $500K |
| STRATFI bridge if needed | $750K (1x mid-year) | — | — | — | $750K |
| Total revenue | $1,105K | $530K | $735K | $1,050K | **$3,420K** |
| **OpEx** | | | | | |
| Salaries (5-headcount full year) | $215K | $228K | $240K | $253K | $936K |
| FIPS 140-3 cert close-out + STIG DISA process | $40K | $30K | $20K | $20K | $110K |
| Conference + travel | $15K | $15K | $15K | $15K | $60K |
| Hardware + cloud | $10K | $10K | $10K | $10K | $40K |
| Legal + IP + contracts | $10K | $10K | $10K | $10K | $40K |
| Marketing + biz dev | $15K | $15K | $15K | $15K | $60K |
| Insurance + ops | $5K | $5K | $5K | $5K | $20K |
| Contingency / G&A | $15K | $15K | $15K | $15K | $60K |
| **Total OpEx** | **$325K** | **$328K** | **$330K** | **$343K** | **$1,326K** |
| **Operating income** | $780K | $202K | $405K | $707K | **$2,094K** |

**Y3 operating income: $2.09M.** Company is cash-flow positive on operations, has the STRATFI bridge if needed, and is on track to a Series A discussion at end of Y3.

---

## §3 — Cumulative cash flow

| Period | OpInc | Cumulative OpInc | External capital raised | Cash balance |
|---|---|---|---|---|
| Y1 start | $0 | $0 | Founder $25K | $25K |
| Y1 end | -$398K | -$398K | + Seed VC $2.0M (Y1 mid-Q3) | **$1,627K** |
| Y2 end | +$139K | -$259K | — | **$1,766K** |
| Y3 end | +$2,094K | +$1,835K | + STRATFI $750K (or skip if not needed) | **$3,860K + STRATFI** |

**Cash never below ~$1.0M after seed close.** Even in the conservative scenario (see §5), runway is comfortable through Y3.

---

## §4 — Use of seed funds ($1.5M-$3M target ask)

**Target raise: $2.0M (median of range)**

| Bucket | Amount | % | Outcome |
|---|---|---|---|
| Engineer hires Y1 (Eng1) + Y2 (Eng2, Eng3) | $800K | 40% | 3 engineers @ ~$250K loaded for 12mo each (some overlap covered by SBIR Phase II) |
| FIPS 140-3 cert lab + CMVP queue | $400K | 20% | Buys the full L1 cert through CMVP queue + algorithm CAVPs |
| Hardware acquisition (CHERIoT board, ARM testbed, x86 NUCs, FPGA dev board for Caliptra) | $50K | 2.5% | Multi-hardware capability story |
| Conference + biz dev + travel | $200K | 10% | AFCEA WEST, TechNet Cyber, AUSA, Sea-Air-Space, DARPA Forecast, AFRL briefings, RSA Gov, prime in-person meetings |
| Legal + IP (corporate counsel, federal contracting counsel, trademark in 4 classes, insurance, FCL prep) | $200K | 10% | Through Y2 |
| Marketing + content + USENIX paper | $100K | 5% | Site polish, capability statement refresh, peer-reviewed publication |
| Contingency + runway buffer | $250K | 12.5% | ~3 months extra burn |
| **Total** | **$2,000K** | 100% | |

---

## §5 — Scenarios (sensitivity analysis)

### Conservative (probability ~25%): SBIR Phase I gets 0 awards, commercial slow

- Y1: SBIR rev $0, commercial $0 → burn $398K, no offset → **need $400K extra capital**
- Y2: Re-submit SBIR; first commercial design partner LOI Q3 → revenue $200K → still need $250K extra
- Y3: Phase II if reattempt awarded; commercial revenue $600K → revenue $1.0M → **operating breakeven**
- 3-year cumulative cash needed: **$3.0M seed → $3.2M needed including buffer**

### Base (probability ~50%): As modeled above

- 1 SBIR Phase I award (out of 3 submissions, ~70% odds)
- Phase II at M9 → completes M27
- 1 commercial design partner Y2, 1 more Y3
- ACT 3 sub starts Y2 mid
- 3-year cumulative: $2.0M seed sufficient, ends Y3 with $3.9M cash

### Aggressive (probability ~25%): Multiple SBIR awards, IQT engages

- 2 SBIR Phase I awards (DoD + AFWERX or DoD + DARPA)
- Phase II awarded at M9
- IQT pitches Y2 → strategic check + IC intro → 1 IC pilot Y2 Q4
- 3-year cumulative: $2.0M seed sufficient + Y3 IQT strategic round optional

**Investor read:** The deal works at $2.0M base. The conservative scenario requires +$1.2M which could come from STRATFI bridge or a Series Seed extension. Aggressive scenario is upside not modeled into base ask.

---

## §6 — Cap table impact

Assumptions (base case $2.0M seed at $8M post-money):

| Stakeholder | Pre-seed | Post-seed | Notes |
|---|---|---|---|
| Founder common | 100% | 75% | Standard founder dilution at seed |
| Seed VC preferred | 0% | 25% | $2M / $8M post = 25% |
| Option pool | (not yet) | post-pool to be set by lead | Typically 10% expansion pre-Series-A |

**Y3 Series A consideration:**
At the end of Y3 with $5-15M ARR and FIPS 140-3 cert in hand, expect a Series A at $30-80M post-money. Founder retained equity ~50-60% pre-Series-A, ~35-45% post depending on round size.

This is a **deeply optimistic** Series A scenario. Realistic Series A might be smaller ($5-10M at $25-40M post) earlier in Y3 if commercial revenue doesn't ramp as projected.

---

## §7 — Comparable financial benchmarks

| Comparable | Stage | Capital raised | Status |
|---|---|---|---|
| **Anjuna** (confidential computing wrapper for Linux) | Series B $30M (2021) | ~$45M total | Acquired by Anjuna parent ops |
| **Edgeless Systems** (confidential AI / confidential containers) | Series A $5M (2022) | ~$8M total | Active |
| **Fortanix** (confidential computing + data security) | Series C $90M (2022) | ~$135M total | Active |
| **Cylance** (endpoint, ML-based — different sector but defense-adjacent IPO trajectory) | IPO acquired by BlackBerry $1.4B (2018) | — | Exited |
| **Hubris OS** (Oxide Computer, internal use) | Part of Oxide's $44M Series A | n/a (internal) | Not sold separately; pattern proof |
| **CHERI / Capabilities Limited** | DARPA-funded; commercial spin-out under SCI Semiconductor | ~$10M+ from CHERIoT silicon partners | Silicon shipping 2026 |
| **seL4 Foundation** | Linux Foundation member project | Non-profit grants | Not investable; reference only |

**The defense+confidential-computing seed-to-Series-B path is well-precedented.** Anjuna and Fortanix took the most analogous arcs. Their TAMs were Linux-wrapped TEE substrates; Sphragis's TAM is the same plus the federal CDS refresh tailwind.

---

## §8 — Honest red flags

**1. Founder concentration risk.** Single founder until Y1 Q4. If founder is incapacitated, the project halts. Mitigation: post-seed Eng1 hire is partly a bus-factor reduction. Investor will want at least co-founder LOI on file before close.

**2. Federal-revenue concentration risk.** Y1+Y2 revenue is ~70% federal. SBIR Phase I/II rejections concentrated would force pivot to commercial sooner. Mitigation: dual-track GTM is in plan; commercial design partner Y1 Q4 is the earliest hedge.

**3. FIPS 140-3 timeline risk.** CMVP queue is 6-18 months; if Y2 cert slips to Y3, gov sales gate slips. Mitigation: lab pre-engagement starts Y1 Q2; commercial design partner doesn't require FIPS cert.

**4. Apple platform risk.** If Apple changes M4 firmware significantly, our independent RE pipeline may need rebuilding. Mitigation: x86_64 port at M15 reduces single-platform exposure.

**5. Competitive entry risk.** A well-funded competitor (e.g., a Rust-OS team inside an existing prime) could enter the lane. Mitigation: our 14-week audit history + Apache-2.0 license + Verus proof artifacts create switching costs and credibility moats. First-mover into CNSA-2.0-native lane is durable through 2027 cliff.

**6. Founder salary self-imposed.** Burn model assumes modest founder salary. If founder needs to take market rate ($300K+ for senior eng founder), Y1 burn jumps ~$150K. Investor may push for either market-rate salary (capital-intensive) or a more aggressive milestone-based vesting.

---

## §9 — One-page summary for the deck

**Capital:** $2.0M seed → 18-24 months runway, $8M post-money
**Use of funds:** 40% engineering / 20% certification / 25% biz dev + GTM / 15% legal + ops + buffer
**Milestones:**
- M9: SBIR Phase I awarded; first engineer hired
- M12: x86_64 port shipped; CMVP lab in test
- M18: Commercial design partner LOI; pre-FIPS pilot deployment
- M24: FIPS 140-3 L1 cert in hand; STIG submitted; ACT 3 sub
- M30-36: First non-SBIR gov revenue ($500K-$5M ARR)
**Series A scenario:** Y3-end at $30-80M post depending on ARR ramp and FIPS cert status
**Capital efficiency proof:** 47 P0 reqs out of MISSING in 24 hours with $0 spend pre-incorporation

---

**End of financial model v1.**

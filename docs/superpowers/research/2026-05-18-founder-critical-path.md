STATUS: COMPLETE | date 2026-05-18 | author researcher-founder

# Founder Paperwork Critical Path — 60-Day Gantt (2026-05-18 → 2026-07-17)

## TL;DR

- **Longest pole:** SAM.gov entity validation. Current 2026 reality is **10–15 business days when clean, 3–4 weeks with any flag** ([SAM.gov registration guide 2026, accessed 2026-05-18](https://blogs.usfcr.com/sam-registration-2025)). CAGE assignment adds **another 5–7 business days** after SAM activates ([GovCon Giants CAGE guide 2026](https://govcongiants.com/guides/cage-code)). Realistic worst-case SAM+CAGE: **6 weeks (≈Jul 1)**.
- **Cheapest-quickest first move:** File Stripe Atlas TODAY (2026-05-18). $500 + ~$240 Delaware fees. Incorporation completes in **1–2 business days** ([Stripe Atlas pricing 2026, accessed 2026-05-18](https://stripe.com/atlas)). EIN typically arrives same week → SAM.gov clock starts at Week 1, not Week 2.
- **Total 60-day one-time cost:** **~$1,650–$3,400** (incorporation, domains, trademark optional, founder insurance optional). **Recurring:** ~$184–$284/yr (registered agent + Delaware franchise + domains + Google Workspace).
- **Material finding (overrides v0 checklist):** Under 15 CFR §742.15(b)(2) as amended by the 2021 BIS rule, **email notification is only required if the source code implements "non-standard cryptography."** Sphragis ships only standardised algorithms (FIPS 197/202/203/204, NIST SP 800-208, RFC 7748/8032/8439/8554/9180, etc.) → **notification is no longer mandatory** ([15 CFR 742.15 — Cornell LII, accessed 2026-05-18](https://www.law.cornell.edu/cfr/text/15/742.15); [BIS 2021 final rule analysis — Wilson Sonsini, accessed 2026-05-18](https://www.wsgr.com/en/insights/us-department-of-commerces-bureau-of-industry-and-security-relaxes-several-classification-and-reporting-requirements-for-encryption-items.html)). Filing remains a **cheap defensive courtesy** and is kept in the Gantt as Week-1 work but downgraded from "legally required" to "audit-defensive optional."
- **Critical SBIR window:** AFWERX/DoD 26.2 monthly cycle closes **2026-06-24** ([Granted AI SBIR/STTR calendar 2026, accessed 2026-05-18](https://grantedai.com/blog/sbir-sttr-deadlines-calendar-2026)). With SAM.gov submitted Week 1, SAM-active around Jun 12–22, CAGE Jun 19–29 → **26.2 is plausible but tight**. The robust target is **26.3 (closes 2026-07-22)** — last submission within the 60-day window.

---

## §1 — Week-by-Week Gantt

All dates calendar 2026. Owner: Kaden, on every row. "GC4N" = gating criterion for next step.

| Week | Days | Action | Start | End | Cost | GC4N |
|---|---|---|---|---|---|---|
| **W1** | May 18 (Mon) → May 24 (Sun) | A. File Stripe Atlas: Delaware C-Corp + EIN + Mercury + Clerky cap table (`Sphragis Inc.`). Verify name on Delaware ICIS + USPTO search first. | 05-18 | 05-19 (CoI back in 1–2 biz days) | $500 Atlas + ~$240 DE state fees | Certificate of Incorporation + EIN issued |
| **W1** | May 18 → May 24 | B. **Day 0 of 83(b) clock** is the founder-stock issuance date on the Stripe Atlas docs — file 83(b) certified-mail (or IRS online filing, accepted since 2025) within 30 calendar days, **no extensions** ([Beancount.io 83(b) guide May 2026](https://beancount.io/blog/2026/05/02/83b-election-founders-startup-employees-restricted-stock-guide); [Miller Nash on IRS online 83(b)](https://www.millernash.com/firm-news/news/founders-wishes-granted-irs-allows-online-83b-election-filings)). | 05-18 | within 30d of issuance | $0 (Atlas handles, founder verifies certified-mail receipt) | 83(b) receipt saved permanently |
| **W1** | May 18 → May 24 | C. Buy domains via Cloudflare or Porkbun: `sphragis.com` + `.org` + `.dev`. Avoid GoDaddy. | 05-18 | 05-19 | ~$50/yr recurring | Domain DNS active |
| **W1** | May 18 → May 24 | D. (Optional, defensive) Send BIS+NSA §742.15(b) courtesy notification using the corrected v1 template (`enc@nsa.gov`, not `web_site@nsa.gov`; cite §742.15(b), not §740.17). See §2.D below — **no longer mandatory** for Sphragis's standard-crypto-only codebase but worth the 10 minutes for audit-defensive record. | 05-18 | 05-22 | $0 | Email sent + auto-ack saved |
| **W1** | May 22–24 (after EIN) | E. **Submit SAM.gov entity registration the day EIN arrives.** UEI auto-issues during the workflow; CAGE code is requested on the same form. Validate that the business address is NOT a UPS Store. | 05-22 (target) | 05-23 | $0 | SAM submission status = "Submitted" |
| **W2** | May 25 → May 31 | F. Open Mercury business banking account (Atlas bundles this automatically; verify it actually provisions). $0 monthly fee, no minimum ([Brex vs Mercury 2026 — Relay](https://relayfi.com/blog/brex-vs-mercury/)). Atlas-routed wires already work for SBIR Phase II milestones. | 05-25 | 05-28 | $0 | Mercury account ACH/wire ready |
| **W2** | May 25 → May 31 | G. Provision Google Workspace Business Starter for `kaden@sphragis.com` + `security@sphragis.com` + `info@sphragis.com`. **$7.00/user/mo annual, $8.40/user/mo monthly** ([Google Workspace pricing 2026](https://workspace.google.com/pricing); [Name.com 2026 plan guide](https://www.name.com/blog/google-workspace-pricing)). 3 users = $252/yr annual plan. | 05-26 | 05-27 | $252/yr (annual) | Email sending + DKIM/SPF/DMARC pass |
| **W2** | May 25 → May 31 | H. Update `README.md` export-control section to cite the §742.15(b) standard-crypto carve-out (no notification required) — replaces v0 text that asserted notification was filed. | 05-29 | 05-31 | $0 | PR merged on main |
| **W3** | Jun 1 → Jun 7 | I. Register at DSIP (DoD SBIR/STTR Innovation Portal); fill company profile + key personnel; doesn't gate on SAM-active. | 06-01 | 06-03 | $0 | DSIP profile complete |
| **W3** | Jun 1 → Jun 7 | J. Register at SBIR.gov (covers non-DoD agencies — NSF/DOE/HHS); keep as optionality, no immediate proposal. | 06-04 | 06-05 | $0 | SBIR.gov account active |
| **W3** | Jun 1 → Jun 7 | K. Monitor SAM.gov status daily. If stuck on "Submitted" >25 business days, call FSD 866-606-8220 ([SAM.gov 2026 timeline — Federal Processing Registry](https://federalprocessingregistry.co/sam-gov-registration-timeline-what-every-federal-contractor-needs-to-know/)). | ongoing | — | $0 | SAM moves to "Active" |
| **W4** | Jun 8 → Jun 14 | L. **83(b) deadline lands somewhere in Week 4** (Day 30 from issuance). Verify Atlas certified-mail tracking + IRS receipt confirmation. No extensions, none. | varies | varies | $0 | Tracking confirms receipt |
| **W4** | Jun 8 → Jun 14 | M. SAM.gov clean-application median window — if everything was correct on Week 1 submission, activation lands around now (10–15 biz days from submit). | 06-08 | 06-12 | $0 | UEI displayed in SAM profile |
| **W5** | Jun 15 → Jun 21 | N. SAM goes Active → CAGE assignment queue 5–7 biz days behind ([Federal Processing Registry CAGE timeline](https://federalprocessingregistry.co/cage-code-issuance-timeline-sam/)). | 06-15 | 06-22 | $0 | CAGE email received |
| **W5** | Jun 15 → Jun 21 | O. (Conditional, contingent on SAM-active) **File SBIR Phase I to DoD/AFWERX 26.2 — closes Jun 24.** Use the v0 proposal `2026-05-17-sbir-phase-1-afwerx-open-v0.md` as the technical-volume draft; fill cover sheet with UEI+CAGE+EIN+banking. Skip if SAM/CAGE not active in time. | 06-15 | 06-23 | $0 (no proposal fee) | Submitted via DSIP |
| **W6** | Jun 22 → Jun 28 | P. CAGE code arrives → update sphragis.com capability statement footer + GitHub Sponsors/OpenSSF drafts' identity blocks (`sed`-pass per session-report §funding). | 06-22 | 06-26 | $0 | All 4 funding drafts have real entity identifiers |
| **W6** | Jun 22 → Jun 28 | Q. Apply to OpenSSF Alpha-Omega (rolling intake call) using draft `2026-05-17-openssf-alpha-omega-v0.md`. | 06-22 | 06-28 | $0 | Intake call scheduled |
| **W6** | Jun 22 → Jun 28 | R. Submit GitHub Sponsors profile from draft `2026-05-17-github-sponsors-profile.md`; submit GitHub Secure Open Source Fund application (rolling) from draft `2026-05-17-github-accelerator-v0.md`. | 06-22 | 06-28 | $0 | Sponsors profile published; SOS Fund submission acked |
| **W7** | Jun 29 → Jul 5 | S. (Optional but recommended) USPTO trademark `Sphragis` (and `SealFS`) Class 9 + Class 42 via the **new single-application system** ($350/class base — TEAS Plus/Standard collapsed in Jan 2025, [USPTO 2025 fee changes](https://www.uspto.gov/trademarks/fees-payment-information/summary-2025-trademark-fee-changes); [WTR specialist chapter 2026](https://www.worldtrademarkreview.com/review/the-trademark-prosecution-review/2026/article/specialist-chapter-understanding-the-new-2025-us-trademark-filing-system-and-fee-increases)). Self-file to avoid the $500–2000 counsel premium. | 06-29 | 07-05 | $700 (2 classes × $350) for `Sphragis`; +$700 for `SealFS` optional | TM serial numbers issued |
| **W7** | Jun 29 → Jul 5 | T. Quote-request from Vouch (https://vouch.us) for general-liability + E&O bundle. Foundational GL **starts ~$500–$1,000/yr**, E&O **$2,000–$4,000/yr** for tech startups ([Embroker E&O guide 2026](https://www.embroker.com/blog/insurance-explained/professional-liability-eo-insurance/)). Bind only when first contract or prime teaming requires it; otherwise defer to August. | 06-29 | 07-05 | $0 (quote only) | Quote in inbox |
| **W8** | Jul 6 → Jul 12 | U. (Conditional) **DoD/AFWERX SBIR 26.3 monthly cycle closes Jul 22** ([SBIR 2026 deadlines — Team 80](https://team-80.com/blog/sbir-deadlines/); [Precision Federal 2026 SBIR calendar](https://precisionfederal.com/resources/sbir-agency-calendar-2026)). If 26.2 was missed, submit here. Refine cover sheet + cost volume. | 07-06 | 07-12 | $0 | Proposal submitted via DSIP |
| **W8** | Jul 6 → Jul 12 | V. Initial 1-hour consult with FAR-familiar counsel (PilieroMazza, Smith Pachter, or Holland & Knight). Budget $300–500/hr. Topic: prime-sub teaming agreement boilerplate + IP-rights-in-data clauses for SBIR. | 07-08 | 07-10 | $300–500 | First-engagement memo on file |
| **W9** | Jul 13 → Jul 17 | W. Capability statement v1 + sphragis.com publication. Use NAICS 541511 / 541512 / 541519. Master plan §A4 + §G6 deliverable. Required for ACT 3 prime cold-pitches per session-report Outreach. | 07-13 | 07-17 | $0 (Cloudflare Pages free tier) | Site reachable; capability-statement PDF linked |
| **W9** | Jul 13 → Jul 17 | X. **IWRP or C5 consortium membership inquiry** (~$10–25K to actually join; this week is just the inquiry/membership-fee confirmation, not the binding payment). Master plan §G5. Defer the actual payment until Month 4 unless an OTA prototype RFS lines up. | 07-13 | 07-17 | $0 (inquiry only) | Consortium application packet received |
| **W9** | Jul 13 → Jul 17 | Y. Day-60 retrospective. Confirm: entity legal, EIN, SAM, UEI, CAGE, DSIP, SBIR.gov, banking, email, domain, 83(b) receipted, at least one SBIR submission filed, ≥3 funding-track drafts submitted, capability site live. | 07-17 | 07-17 | $0 | Punch list filed in session journal |

---

## §2 — Per-Step Detail (external state, May 2026)

### A. Stripe Atlas — Delaware C-Corp

- **Price:** $500 USD, covers Delaware incorporation + first-year registered-agent + EIN + Clerky cap-table + Mercury referral; recurring $100/yr registered-agent thereafter ([Stripe Atlas docs](https://stripe.com/atlas), accessed 2026-05-18).
- **Timeline:** 1–2 business days from form submission to Certificate of Incorporation, per Atlas's own documentation in May 2026. EIN usually issued same week if SSN is provided on the founder form.
- **Atlas perk:** $2,500 in Stripe product credits for the first year (not relevant to Sphragis runtime but useful for any future commerce work) — confirmed [Sparklaunch Atlas 2026 review](https://sparklaun.ch/compare/stripe-atlas), accessed 2026-05-18.
- **Alternative considered + rejected:** Firstbase ($399 base but more fragmented integrations) and Doola ($297 base, weaker EIN handling) — both cheaper headline price but require more founder time. See [GlobalSolo comparison 2026](https://www.globalsolo.global/blog/stripe-atlas-vs-firstbase-vs-doola-pricing-comparison-2026), accessed 2026-05-18.

### B. 83(b) election

- **30 calendar days, no extensions, no hardship relief.** ([Carta 83(b) founder guide](https://carta.com/learn/equity/stock-options/taxes/83b-election/); [Beancount.io 2026-05 founder guide](https://beancount.io/blog/2026/05/02/83b-election-founders-startup-employees-restricted-stock-guide), accessed 2026-05-18).
- **IRS accepts online filings as of 2025** ([Miller Nash news, 2025](https://www.millernash.com/firm-news/news/founders-wishes-granted-irs-allows-online-83b-election-filings)). Atlas will queue the paper form by default; founder must independently verify either certified-mail tracking OR online-filing confirmation.

### C. Domains

- Cloudflare Registrar at-cost (≈$10/yr per `.com`, $15–20 for `.dev`), Porkbun similar. Avoid GoDaddy renewal escalation tactics (well-documented).
- DNSSEC + CAA records set immediately to lock issuance to a known CA.

### D. BIS / NSA encryption notification (status correction)

**Material correction to v0 checklist + v1 template:** Under **15 CFR §742.15(b)(1)**, "publicly available encryption source code classified under ECCN 5D002 is not subject to the EAR" subject to §742.15(b)(2). §742.15(b)(2) states notification is required "**for publicly available encryption source code classified under ECCN 5D002 that provides or performs 'non-standard cryptography' as defined in part 772**" ([Cornell LII §742.15 text](https://www.law.cornell.edu/cfr/text/15/742.15), accessed 2026-05-18).

Sphragis ships only standard cryptography per `MARKETING SITE specifications panel` and `2026-05-17-day1-sweep-and-funding-readiness.md §1`: AES-128/256 (FIPS 197), SHA-2/3 family (FIPS 180-4/202), ML-KEM (FIPS 203), ML-DSA (FIPS 204), LMS (NIST SP 800-208), X25519/Ed25519 (RFC 7748/8032), ChaCha20-Poly1305 (RFC 8439), HKDF (RFC 5869), Argon2id (RFC 9106), HMAC (FIPS 198-1), HOTP/TOTP (RFC 4226/6238). **All on the "standard cryptography" side of part 772.**

- **2021 BIS final rule** explicitly eliminated email notification for publicly available source code that uses standard cryptography ([Wilson Sonsini summary](https://www.wsgr.com/en/insights/us-department-of-commerces-bureau-of-industry-and-security-relaxes-several-classification-and-reporting-requirements-for-encryption-items.html); [Jones Day summary](https://www.jonesday.com/en/insights/2021/04/commerce-reduces-requirements-relating-to-massmarket-encryption-items-and-publicly-available-software); [Baker McKenzie sanctions blog](https://sanctionsnews.bakermckenzie.com/bis-updates-reporting-requirements-relating-to-mass-market-encryption-items-and-publicly-available-software-and-also-updates-certain-classifications/) — all accessed 2026-05-18).
- BIS estimated the rule eliminated ~80% of notifications it had been receiving.
- **Recommendation:** Send the §742.15(b) email anyway as a 10-minute defensive courtesy. Many AOs / counsel still expect to see the audit-trail; the email is harmless. **Do not** claim in `README.md` that notification was "filed because required" — claim instead that notification was sent as a courtesy and that Sphragis qualifies for the §742.15(b)(1) standard-crypto carve-out.
- Submission addresses unchanged: `crypt@bis.doc.gov` + `enc@nsa.gov` ([§742.15(b)(2) text](https://www.law.cornell.edu/cfr/text/15/742.15)).

### E. SAM.gov

- **Free.** Median activation 10–15 business days when application is clean; 3–4 weeks when any field flags ([SLED.AI SAM.gov guide 2026](https://www.sledai.com/blog/sam-registration-guide/); [USFCR blog](https://blogs.usfcr.com/sam-registration-2025); [Funding Landscape 2026](https://fundinglandscape.com/answers/sam-gov-registration-guide-2026) — all accessed 2026-05-18).
- IRS validation step adds **1–3 days on its own**; EBiz POC self-confirmation can stall things for days if Kaden doesn't watch his inbox.
- Worst-case "stuck on Submitted >25 biz days" → call **866-606-8220** (Federal Service Desk).
- **The 10-week worst case** ([E.B. Howard Consulting 2026](https://www.ebhoward.com/sam-gov-may-take-up-to-10-weeks-to-approve/), accessed 2026-05-18) is generally validation-failure-driven, not normal queue.

### F. Mercury (banking)

- $0 monthly fees, no minimum balance, no personal guarantee. **Mercury Treasury** (sweep into MMF for yield) requires $250K total balance, irrelevant pre-Phase-II ([Relay comparison 2026](https://relayfi.com/blog/brex-vs-mercury/); [Aspire 2026 comparison](https://aspireapp.com/us/blog/brex-vs-mercury) — accessed 2026-05-18).
- **Brex dropped bootstrapped/pre-revenue accounts in June 2022**; current Brex targets VC-backed only. For Sphragis pre-SBIR, Mercury is the only sensible pick.

### G. Google Workspace

- **Business Starter $7/user/mo annual, $8.40/user/mo flexible** ([Google Workspace pricing page](https://workspace.google.com/pricing); [Name.com 2026 plan guide](https://www.name.com/blog/google-workspace-pricing) — accessed 2026-05-18).
- 30 GB pooled storage/user, Gmail + Drive + Meet + Calendar. Sufficient for a 1–5 person founder team for Y1.
- 3 founder addresses × $7 × 12 = **$252/yr.**

### I/J. DSIP + SBIR.gov

- Both free and ~5-minute account creations. DSIP is the DoD-side portal (gating SBIR submissions); SBIR.gov covers non-DoD agencies (NSF, DOE, HHS, NIH). Worth maintaining both even if Y1 strategy is DoD-only — DOE has shown interest in secure-OS work historically.

### N. CAGE code

- DLA assigns automatically after SAM activates; 5–7 biz days typical, can stretch to 3 weeks if entity-validation flags ([Federal Processing Registry CAGE timing](https://federalprocessingregistry.co/cage-code-issuance-timeline-sam/); [GovCon Giants 7-day guide](https://govcongiants.com/guides/cage-code); [Amerifusion 2026 guide](https://amerifusiongovcon.com/how-to-get-a-cage-code/) — all accessed 2026-05-18).
- Code arrives via email; update sphragis.com capability statement + all funding drafts in the same `sed` pass.

### O / U. SBIR submission windows

External research output 2026-05-18:
- **DoD SBIR 26.2 / AFWERX open topic:** Pre-release Apr/May 2026; open ~May 5–6; **closes 2026-06-24** ([Granted AI SBIR/STTR calendar 2026](https://grantedai.com/blog/sbir-sttr-deadlines-calendar-2026); [team-80 SBIR 2026 deadlines](https://team-80.com/blog/sbir-deadlines/) — accessed 2026-05-18).
- **DoD SBIR 26.3:** closes **2026-07-22** (next monthly cycle).
- **DoD SBIR 26.4 / 26.5 / 26.6:** monthly closes **08-19, 09-23, 10-21** per same sources.
- **DARPA SBIR FY26:** First wave (SWiFT, BARK, EXPOSITION, PEPI) pre-released 2026-04-30, closes 2026-06-03 — **not relevant to Sphragis topic-wise**. DPA26BZ01-NV005 Microsystems Technology closes 2026-06-24 — also weak fit. DARPA's SBIR topic cadence is monthly; the substantive Sphragis pitch is direct-PM (PROVERS / RSSC / TRACTOR) per `2026-05-17-darpa-pm-prep-v1.md`, not SBIR.
- **AFWERX cadence change:** the historical "fully open" topic is gone — current open topics are **"focused open" / mission-driven** ([Sciencedocs SBIR 2026 calendar](https://www.sciencedocs.com/sbir-grant-calendar/); [BWCo AFWERX explainer](https://www.bwcoconsulting.com/fod/afwerx) — accessed 2026-05-18). The Sphragis Phase I draft `2026-05-17-sbir-phase-1-afwerx-open-v0.md` needs to be retargeted at a specific 26.2 / 26.3 topic listed on DSIP **before** Week 5 submission.

### S. USPTO trademark — fee-schedule update

**Important schedule change** ([USPTO 2025 fee changes summary](https://www.uspto.gov/trademarks/fees-payment-information/summary-2025-trademark-fee-changes); [WTR 2026 review](https://www.worldtrademarkreview.com/review/the-trademark-prosecution-review/2026/article/specialist-chapter-understanding-the-new-2025-us-trademark-filing-system-and-fee-increases); [Quarles fee-change brief](https://www.quarles.com/newsroom/publications/trademark-fee-changes-at-the-uspto-what-you-need-to-know); [Stinson summary](https://www.stinson.com/newsroom-publications-usptos-new-trademark-fees-expected-to-impact-filing-costs) — all accessed 2026-05-18):

- **TEAS Plus / TEAS Standard were collapsed into a single application on 2025-01-18.**
- New base fee: **$350/class** if the application is complete and uses the ID Manual.
- Surcharges: **+$100/class** for insufficient info; **+$200/class** for non-ID-Manual goods/services free-text; **+$200** for every 1,000 chars beyond 1,000 in goods/services description.
- USPTO expects most applicants pay only the base fee.
- For `Sphragis` Class 9 + Class 42: **$700** budgeted (assume clean ID-Manual entries; counsel optional). Same again for `SealFS` if Kaden wants the secondary mark — recommended to file `Sphragis` only in 60-day window, defer `SealFS` to Month 6 unless a brand-risk flag appears.
- Wait time for examination: still 8–12 months end-to-end (separate from filing-window urgency).

### T. Insurance

- Foundational GL: **$500–$1,000/yr** for a pre-revenue tech startup; E&O / professional liability **$2,000–$4,000/yr** ([Embroker E&O cost guide 2026](https://www.embroker.com/blog/product-guides/professional-liability-eo-insurance-cost/); [Vouch startup-insurance pricing primer](https://www.vouch.us/insurance101/start-up-insurance-costs-how-much-to-pay) — accessed 2026-05-18).
- Both Vouch and Embroker decline to quote on website; both require a 5-minute questionnaire.
- **Recommendation:** request quotes Week 7 but **don't bind** until the first signed contract / teaming agreement / DoD subcontract requires it. Cash drag is otherwise wasted at pre-revenue.

### V. FAR counsel intake

- Hourly rate **$300–500/hr** ([master plan §A3]; market-rate confirmed in cold-pitch outreach research).
- 1-hour intake covers: prime-sub teaming agreement structure, IP-rights-in-data clause posture for SBIR Phase I (DFARS 252.227-7018 small-business protections), assignment of inventions agreement boilerplate.
- Budget $300–500 single-engagement, defer ongoing retainer to Month 4–6.

### Delaware franchise tax (background recurring cost)

- **Annual minimum $175** (Authorized Shares method) or **$400** (Assumed Par Value Capital method) plus **$50 annual-report filing fee**. **Due March 1 each year** ([Delaware Division of Corporations annual report guide](https://corp.delaware.gov/paytaxes/); [FileForms 2026 Delaware franchise deadlines](https://fileforms.com/delaware-franchise-tax-2026-deadlines/) — accessed 2026-05-18).
- First payment due **2027-03-01** for incorporation year 2026. Outside the 60-day window but tracked for runway planning.

---

## §3 — Cumulative Cost Summary

### One-time (within Day 1 → Day 60)

| Item | Cost | Mandatory? |
|---|---|---|
| Stripe Atlas filing | $500 | Yes |
| Delaware state filing fees (incl. in Atlas) | ~$240 | Yes |
| Domains × 3 (yr 1) | ~$50 | Yes |
| BIS notification | $0 | **No (defensive courtesy)** |
| SAM.gov + UEI + CAGE | $0 | Yes |
| DSIP + SBIR.gov registrations | $0 | Yes |
| Google Workspace year 1 (3 users × $84) | $252 | Recommended |
| USPTO trademark `Sphragis` × 2 classes | $700 | Recommended; defer if cash-tight |
| USPTO trademark `SealFS` × 2 classes (optional) | $700 | Optional (defer to Month 6) |
| FAR counsel 1-hour intake | $300–500 | Recommended |
| GL + E&O insurance binding (if required by contract) | $0–4,000/yr | Conditional |
| **Total cash out in 60 days (typical)** | **~$1,650–$3,400** | Sphragis-only branding |
| **Total if `SealFS` TM + counsel + insurance binding all happen in 60d** | **~$5,640–$9,140** | Aggressive case |

### Recurring (year-2+)

| Item | Cost/yr |
|---|---|
| Stripe Atlas registered agent (year 2+) | $100 |
| Delaware franchise tax + annual report | $175 (min, AS method) + $50 = $225 |
| Domains (Cloudflare/Porkbun) | ~$50 |
| Google Workspace × 3 users | $252 (Business Starter) → ~$500 (Business Standard if Drive growth) |
| **Subtotal (no insurance)** | **~$877–$1,125** |
| Vouch / Embroker GL + E&O (when bound) | $2,500–$5,000 |
| **Subtotal with insurance bound** | **~$3,377–$6,125/yr** |

These match the master plan §A3 + financial model §burn-Y1 envelope of $30–100K legal/admin per year before salaries.

---

## §4 — Contingencies

### Contingency 1: SAM.gov slips past Day 60 (~Jul 17)

**Trigger:** SAM.gov still "Submitted" or in validation loop on Jun 22 (Day 35) — the 25-business-day FSD-call threshold.

**Actions, in priority order:**
1. **Call FSD 866-606-8220** the morning of Day 36 if no movement. Confirm whether it's an IRS-validation, EBiz-POC, or DLA bottleneck. Document the case number.
2. **Continue Track B/C/D engineering work** — none of it depends on SAM activation. The block is only on SBIR submissions + GSA MAS path (master plan §G1 — defer-to-Month-9 anyway).
3. **Skip SBIR 26.2 (Jun 24).** Aim for 26.3 (Jul 22) or 26.4 (Aug 19). Each monthly cycle is a 30-day slip, not a year-long miss.
4. **Apply to OpenSSF Alpha-Omega and GitHub Secure OSS Fund anyway** — neither requires CAGE/UEI (they take individual-applicant filings; the funding drafts already include "Sphragis Inc. — in formation" language per session-report §funding).
5. **Apply to Sovereign Tech Fund and NLnet** — both are German/EU non-profit programs that accept individual or pre-incorporated applicants per `2026-05-17-sovereign-tech-fund-application-v0.md` and `2026-05-17-nlnet-application-v0.md`. Neither cares about SAM.gov status.
6. **Engage Federal Processing Registry or USFCR as a paid SAM.gov registration consultant** if the validation block is opaque after Week 7 — last-resort, ~$500–1,500 for resolution.

### Contingency 2: SBIR 26.2 (Jun 24) is missed

**Already accommodated in the Gantt** — Week 8 row "U" captures fallback to 26.3 (Jul 22, still within the 60-day window). The 26.2 → 26.3 slip is **1 cycle / 28 days**, not a year.

If 26.3 is also missed (most likely cause: SAM/CAGE still inactive):
1. Slip to **26.4 (Aug 19)** or **26.5 (Sep 23)**. Each is a new monthly cycle.
2. Use the slip time to **improve the proposal**: strengthen the AFWERX-topic alignment, sharpen the cost volume, get one or two external "expert advisor" letters into Volume 5.
3. Consider **NSF SBIR** (Phase I window ~Sep–Oct annually; quasi-commercial reviewers, less DoD-specific topic fit) as a non-DoD parallel — uses SBIR.gov registration (Week 3 step J).
4. Aggressive bridge: **Sovereign Tech Fund** (decision in ~3 months, up to €500K — see draft) or **NLnet NGI0 Core** (~€50K decision in ~6 weeks).

### Contingency 3: Stripe Atlas filing rejected or stalled

**Trigger:** Delaware filing rejected (rare, usually entity-name conflict) or Atlas hits an EIN-issuance delay (IRS backlog).

1. Re-confirm `Sphragis Inc.` availability on the Delaware ICIS search; fall back to `Sphragis Corp.` or `Sphragis Systems Inc.` if needed. **Trademark filing changes if entity name changes** — re-do the USPTO knock-out search.
2. If EIN delayed >2 weeks, file Form SS-4 directly with the IRS by fax to keep the SAM clock alive — Atlas's SLA is best-effort, not guaranteed.

### Contingency 4: BIS/NSA reply asks for additional info

(Low-probability since notification is technically optional, but if Kaden sends the courtesy email and BIS replies):

- Reply within the deadline they cite, citing **15 CFR §742.15(b)(1) standard-cryptography carve-out** as the regulatory basis for why no further submission is required.
- Save all correspondence in the compliance archive (`compliance/bis-742-15b/` as the template specifies).
- If BIS asserts non-standard cryptography is involved, request the specific algorithm they flag — Sphragis can argue every algorithm is FIPS/NIST/RFC standardised. This becomes a counsel-intake question (Step V).

### Contingency 5: 83(b) certified-mail receipt fails

**Trigger:** Within 30 days of stock issuance, no IRS certified-mail return receipt arrives.

- File a **second copy via IRS-online filing** (allowed since 2025 per Miller Nash news cite above) before Day 30 ends. Belt-and-suspenders is acceptable; duplicate 83(b) filings are harmless.
- If Day 30 passes without filing: **call counsel immediately**. Some private letter rulings have granted relief in limited circumstances ([etonvs missed-deadline analysis](https://etonvs.com/409a-valuation/remedies-for-missed-83b-deadline/), accessed 2026-05-18) but the case law is unforgiving — the deadline is treated as jurisdictional.

---

## Sources cited (in order of first appearance)

- [SAM.gov Registration in 2026 — USFCR blog](https://blogs.usfcr.com/sam-registration-2025) — accessed 2026-05-18
- [CAGE Code Free Lookup — GovCon Giants 2026](https://govcongiants.com/guides/cage-code) — accessed 2026-05-18
- [Stripe Atlas pricing page](https://stripe.com/atlas) — accessed 2026-05-18
- [15 CFR 742.15 — Cornell LII](https://www.law.cornell.edu/cfr/text/15/742.15) — accessed 2026-05-18
- [Wilson Sonsini — BIS 2021 encryption rule relaxations](https://www.wsgr.com/en/insights/us-department-of-commerces-bureau-of-industry-and-security-relaxes-several-classification-and-reporting-requirements-for-encryption-items.html) — accessed 2026-05-18
- [Granted AI — SBIR/STTR 2026 calendar](https://grantedai.com/blog/sbir-sttr-deadlines-calendar-2026) — accessed 2026-05-18
- [Beancount.io — 83(b) founder guide May 2026](https://beancount.io/blog/2026/05/02/83b-election-founders-startup-employees-restricted-stock-guide) — accessed 2026-05-18
- [Miller Nash — IRS online 83(b) filings](https://www.millernash.com/firm-news/news/founders-wishes-granted-irs-allows-online-83b-election-filings) — accessed 2026-05-18
- [Federal Processing Registry — SAM.gov timeline 2026](https://federalprocessingregistry.co/sam-gov-registration-timeline-what-every-federal-contractor-needs-to-know/) — accessed 2026-05-18
- [Google Workspace pricing](https://workspace.google.com/pricing) — accessed 2026-05-18
- [Name.com — Google Workspace plans 2026](https://www.name.com/blog/google-workspace-pricing) — accessed 2026-05-18
- [Federal Processing Registry — CAGE timing](https://federalprocessingregistry.co/cage-code-issuance-timeline-sam/) — accessed 2026-05-18
- [GovCon Giants CAGE 7-day guide](https://govcongiants.com/guides/cage-code) — accessed 2026-05-18
- [Amerifusion — How to get a CAGE Code 2026](https://amerifusiongovcon.com/how-to-get-a-cage-code/) — accessed 2026-05-18
- [Team-80 — 2026 SBIR Deadlines updated](https://team-80.com/blog/sbir-deadlines/) — accessed 2026-05-18
- [Precision Federal — 2026 SBIR Agency Calendar](https://precisionfederal.com/resources/sbir-agency-calendar-2026) — accessed 2026-05-18
- [BWCo — AFWERX & SpaceWERX Open Topic Program](https://www.bwcoconsulting.com/fod/afwerx) — accessed 2026-05-18
- [Sciencedocs — SBIR 2026 calendar](https://www.sciencedocs.com/sbir-grant-calendar/) — accessed 2026-05-18
- [USPTO — Summary of 2025 trademark fee changes](https://www.uspto.gov/trademarks/fees-payment-information/summary-2025-trademark-fee-changes) — accessed 2026-05-18
- [World Trademark Review — 2026 Specialist Chapter on USPTO 2025 fee changes](https://www.worldtrademarkreview.com/review/the-trademark-prosecution-review/2026/article/specialist-chapter-understanding-the-new-2025-us-trademark-filing-system-and-fee-increases) — accessed 2026-05-18
- [Quarles — Trademark Fee Changes at the USPTO](https://www.quarles.com/newsroom/publications/trademark-fee-changes-at-the-uspto-what-you-need-to-know) — accessed 2026-05-18
- [Stinson — USPTO new trademark fees](https://www.stinson.com/newsroom-publications-usptos-new-trademark-fees-expected-to-impact-filing-costs) — accessed 2026-05-18
- [Embroker — E&O insurance cost guide 2026](https://www.embroker.com/blog/product-guides/professional-liability-eo-insurance-cost/) — accessed 2026-05-18
- [Vouch — startup insurance cost primer](https://www.vouch.us/insurance101/start-up-insurance-costs-how-much-to-pay) — accessed 2026-05-18
- [Embroker — Professional Liability (E&O) Insurance Pricing 2026](https://www.embroker.com/blog/insurance-explained/professional-liability-eo-insurance/) — accessed 2026-05-18
- [Relay — Brex vs Mercury 2026](https://relayfi.com/blog/brex-vs-mercury/) — accessed 2026-05-18
- [Aspire US — Brex vs Mercury 2026](https://aspireapp.com/us/blog/brex-vs-mercury) — accessed 2026-05-18
- [Delaware Division of Corporations — Annual Report & Tax Instructions](https://corp.delaware.gov/paytaxes/) — accessed 2026-05-18
- [FileForms — Delaware Franchise Tax 2026 deadlines](https://fileforms.com/delaware-franchise-tax-2026-deadlines/) — accessed 2026-05-18
- [GlobalSolo — Stripe Atlas vs Firstbase vs Doola 2026 comparison](https://www.globalsolo.global/blog/stripe-atlas-vs-firstbase-vs-doola-pricing-comparison-2026) — accessed 2026-05-18
- [Sparklaunch — Stripe Atlas cost 2026](https://sparklaun.ch/compare/stripe-atlas) — accessed 2026-05-18
- [E.B. Howard Consulting — SAM.gov may take 10 weeks](https://www.ebhoward.com/sam-gov-may-take-up-to-10-weeks-to-approve/) — accessed 2026-05-18
- [SLED.AI — SAM.gov Registration Guide 2026](https://www.sledai.com/blog/sam-registration-guide/) — accessed 2026-05-18
- [Funding Landscape — SAM.gov 2026](https://fundinglandscape.com/answers/sam-gov-registration-guide-2026) — accessed 2026-05-18
- [Jones Day — BIS reduces encryption requirements 2021](https://www.jonesday.com/en/insights/2021/04/commerce-reduces-requirements-relating-to-massmarket-encryption-items-and-publicly-available-software) — accessed 2026-05-18
- [Baker McKenzie — BIS updates encryption reporting requirements](https://sanctionsnews.bakermckenzie.com/bis-updates-reporting-requirements-relating-to-mass-market-encryption-items-and-publicly-available-software-and-also-updates-certain-classifications/) — accessed 2026-05-18
- [Carta — 83(b) election guide](https://carta.com/learn/equity/stock-options/taxes/83b-election/) — accessed 2026-05-18
- [etonvs — missed 83(b) deadline remedies](https://etonvs.com/409a-valuation/remedies-for-missed-83b-deadline/) — accessed 2026-05-18

— End of file —

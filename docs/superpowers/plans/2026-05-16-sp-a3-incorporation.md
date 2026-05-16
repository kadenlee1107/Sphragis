# SP-A3: Incorporation + Procurement On-Ramp

**Type:** Founder-led checklist (not engineering work — this is administrative / legal / regulatory).

**Goal:** Stand up the legal + regulatory infrastructure so Sphragis can receive federal contract awards, comply with crypto export controls, and engage with the gov procurement ecosystem.

**Architecture:** Sequential dependency chain. Incorporate → SAM.gov registration → CAGE/UEI assignment → DSIP registration → BIS encryption classification filing. Some items can parallelize; the timeline below shows true dependencies.

**Tech Stack:** None (no code). Stripe Atlas or local legal counsel + SAM.gov + DSIP + email forms.

**Requirements closed:** PRC-001, PRC-002, CRT-007. Partial: LIC-004 (trademark deferred to month 6).

**Depends on:** Nothing technical. Can run in parallel with SP-A1, SP-A2 engineering work.

**Estimated duration:** 60-90 days end-to-end. Funded items + waiting times below.

---

## Cost Summary

| Item | Cost | Notes |
|---|---|---|
| Delaware C-Corp formation | $500-2,000 | Stripe Atlas $500; local counsel $1-2K |
| Registered agent (annual) | $50-200 | Required for Delaware entity |
| Delaware franchise tax | $400-450/year | Minimum |
| EIN application | $0 | Free via IRS |
| Business bank account | $0 (Mercury / Brex) | Skip if using personal until first revenue |
| SAM.gov registration | $0 | Free, 30-60 day wait |
| CAGE code | $0 | Free, ~14 day wait |
| UEI assignment | $0 | Free, automatic via SAM.gov |
| DSIP registration | $0 | Free, instant |
| BIS encryption classification notification | $0 | Free email submission |
| Trademark application (USPTO) | $250-350 | Federal filing fee per class |
| Trademark counsel | $500-2,000 | Optional but recommended |
| **Total Y1** | **~$1,500-7,000** | Most of this is one-time |

---

## Checklist

### Phase 1: Entity formation (Weeks 1-2)

- [ ] **Decide entity type**
  - **Recommendation:** Delaware C-Corp. Standard for tech companies, prime contractors expect it, In-Q-Tel + VC require it, qualified small business stock (QSBS) tax treatment.
  - Alternatives considered: LLC (worse for investor/M&A), S-Corp (limits future investor classes), single-member LLC then convert (delays the choice without gain).

- [ ] **Choose company name**
  - Match the Sphragis branding. Common patterns: "Sphragis Inc.", "Sphragis Corp.", "Sphragis Systems Inc."
  - Run name search on Delaware Division of Corporations entity search: https://icis.corp.delaware.gov/Ecorp/EntitySearch/NameSearch.aspx
  - Run USPTO trademark search: https://www.uspto.gov/trademarks/search

- [ ] **Form the C-Corp**
  - **Option A:** Stripe Atlas (https://stripe.com/atlas) — $500 flat, handles Delaware incorporation + EIN + Mercury banking + Clerky cap-table tooling.
  - **Option B:** Clerky (https://www.clerky.com) — ~$799-$1,049, more lawyer-style structure, no banking bundled.
  - **Option C:** Local counsel — $1,500-3,000, white-glove but slower.
  - **Pick one.** Stripe Atlas is the path-of-least-resistance for a small founding team.

- [ ] **Receive incorporation documents**
  - Certificate of Incorporation (DE)
  - Bylaws
  - Stock Purchase Agreement / Founder Stock issuance
  - 83(b) election (file within 30 days of stock issuance — DO NOT MISS THIS)
  - EIN letter from IRS

- [ ] **File 83(b) election within 30 days**
  - Required if founder receives restricted stock. Failing to file = massive tax bill on future appreciation.
  - Stripe Atlas / Clerky handle this; verify it was actually filed.

### Phase 2: Federal registrations (Weeks 2-8)

- [ ] **Apply for SAM.gov registration** (https://sam.gov)
  - Free. Requires: legal entity name, EIN, business address (cannot be a UPS Store), DUNS history (now UEI), banking info for electronic payment.
  - **Timeline: 7-60 days for activation.** Expect closer to 30-60. Cannot receive federal contracts until activated.
  - Apply ASAP — this is the longest-pole item.

- [ ] **Receive UEI (Unique Entity Identifier)**
  - Automatic via SAM.gov registration. Replaces the old DUNS number.

- [ ] **Apply for CAGE code** (Commercial and Government Entity)
  - Free, via SAM.gov registration flow.
  - Timeline: ~14 days after SAM.gov approval.
  - Required for DoD contracts.

- [ ] **Register at DSIP** (DoD SBIR/STTR Innovation Portal: https://www.dodsbirsttr.mil)
  - Free. Required for any DoD SBIR submission.
  - Add company profile + key personnel + technical capabilities.

- [ ] **Register at SBIR.gov** (https://www.sbir.gov)
  - Free. Required for SBIR submissions to non-DoD agencies (NSF, DOE, HHS, etc.).

- [ ] **Register at eOffer** (https://eoffer.gsa.gov)
  - Free. Required to submit GSA MAS offers (SP-G1 dependency, not needed until Month 9-12).

### Phase 3: Export control compliance (Week 4)

This is **legally required** for any US-developed software with crypto. Filing protects you from BIS enforcement action; failing to file is the #1 export-control compliance miss.

- [ ] **Determine your ECCN classification**
  - Sphragis ships AES-256, ML-KEM-1024, ML-DSA-87, X25519, ChaCha20-Poly1305, SHA-384/512, etc.
  - **ECCN: 5D002** (encryption software, network infrastructure).

- [ ] **File initial classification with BIS (Bureau of Industry and Security)**
  - **License Exception ENC** covers most commercial encryption software.
  - **Initial classification notification** required at first publication / first export.
  - Email `crypt@bis.doc.gov` AND `web_site@nsa.gov` with:
    - Company name + address + point of contact
    - Product name (Sphragis)
    - Brief description (security-first bare-metal Rust microkernel)
    - List of cryptographic algorithms used (AES-256, ML-KEM-1024, ML-DSA-87, X25519, ChaCha20-Poly1305, SHA-256/384/512, LMS, XMSS, HMAC, HKDF)
    - URL to public source repository
    - Statement: "Submitted under License Exception ENC §740.17(b)(1)"
  - Keep the sent-confirmation email forever; it's the proof of compliance.

- [ ] **Add license header to README and project docs**

  Add to README.md a short notice:
  ```markdown
  ## Export Control

  Sphragis includes cryptographic software and is classified under ECCN 5D002.
  This software is publicly available and made available outside the U.S. under
  License Exception ENC §740.17(b)(1). The U.S. Department of Commerce, BIS,
  and NSA have been notified of the source code's publication.
  ```

### Phase 4: Operational setup (Weeks 4-8, parallelizable)

- [ ] **Business bank account**
  - Mercury (https://mercury.com) and Brex (https://www.brex.com) both fast and free for startups.
  - Required for Phase II SBIR (~$1.25M wires).

- [ ] **Domain name registration**
  - Buy `sphragis.com`, `sphragis.dev`, `sphragis.org`. Together ~$50/yr.
  - Use Cloudflare or Porkbun. Avoid GoDaddy.

- [ ] **Email infrastructure**
  - Google Workspace or Fastmail. ~$6-12/user/mo.
  - At minimum: `kaden@sphragis.com` for founder, `security@sphragis.com` for vuln reports, `info@sphragis.com` for catch-all.

- [ ] **Project management / CRM (optional Y1)**
  - Linear / Notion for issues + docs. ~$10-15/user/mo.
  - Defer Salesforce-type CRM until first paid pilot.

### Phase 5: Trademark (Month 6, deferable)

- [ ] **USPTO trademark application** (https://www.uspto.gov)
  - **Class 9** (computer software) — required for the brand on the product.
  - **Class 42** (computer services) — required for SaaS / consulting.
  - $250-350 per class. Self-file possible but trademark counsel recommended.
  - Timeline: 8-12 months for examination, longer for any objection.

- [ ] **Begin using ® symbol once registered** (until then, use ™ as common-law mark).

### Phase 6: Insurance + counsel (Month 3-6)

- [ ] **General liability + errors & omissions insurance**
  - Required by some IDIQs / SBIR agreements.
  - Vouch (https://www.vouch.us) or Embroker for startup-friendly policies.
  - ~$1-3K/yr for minimum coverage.

- [ ] **Engage outside counsel for federal contracting**
  - At least one consultation. Counsel familiar with FAR (Federal Acquisition Regulation) + small-business rules.
  - Smith Pachter McWhorter, Holland & Knight, Reed Smith all have gov-contracts practice groups. Or smaller specialty firms (Howe & Associates, PilieroMazza for small business set-asides).
  - $300-500/hr; budget for 5-10 hours up-front consultation.

---

## Decision Gates

- **End of Week 2:** Entity formed. EIN received. 83(b) filed.
- **End of Week 4:** SAM.gov submitted. BIS encryption notification filed.
- **End of Week 8:** SAM.gov approved. CAGE + UEI assigned. DSIP + eOffer registered. Bank account open.
- **End of Month 3:** Insurance + legal counsel relationship established. **Eligible to receive federal contracts.**

If SAM.gov approval slips past Week 8 (common): no action needed; just delays SBIR submission start.

---

## Risk Items

| Risk | Mitigation |
|---|---|
| 83(b) deadline missed (30 days post-stock-issuance) | Calendar reminder Week 1. Stripe Atlas / Clerky usually handle automatically. Verify manually. |
| SAM.gov fraud-prevention block (e.g., shared address) | Use a real business address, not UPS Store. If using residential, confirm IRS doesn't flag. |
| BIS notification not filed pre-publication | File BEFORE merging the first crypto-containing release tag. SP-A1 SP-A2 are non-crypto so safe to merge; SP-B1 crypto work must wait for BIS confirmation. |
| Wrong NAICS code on SAM.gov | Use NAICS: 541511 (Custom Computer Programming Services), 541512 (Computer Systems Design Services), 541519 (Other Computer Services). Add all three. |
| Picking wrong-state incorporation | Stick with Delaware. Don't optimize for the founder's home state. |

---

## Output

After SP-A3 completes:
- ✅ Delaware C-Corp formed
- ✅ EIN, 83(b), founders' stock issued
- ✅ SAM.gov registered + CAGE + UEI
- ✅ DSIP + eOffer + SBIR.gov registered
- ✅ BIS encryption classification notification filed
- ✅ Business bank account open
- ✅ Domains registered + email infrastructure
- ✅ Insurance + counsel relationships established (or in flight)

Output to Track G: **SBIR Phase I submissions (SP-G2) can begin** once SAM.gov + DSIP both active.

Output to Track B: **First crypto-bearing release tag (after SP-B1 lands) requires BIS notification to be on-file FIRST.**

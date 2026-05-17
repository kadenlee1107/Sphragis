# Founder Action Checklist — Track A (Paperwork)

**Purpose:** What you (the founder) personally need to do, in what order, with what cost and what links. This is the only-you work that runs in parallel while I draft Track B (proposals, decks, financial models, capability briefs).

**Estimated wall-clock:** 60-90 days end-to-end (SAM.gov is the longest pole)
**Estimated cost:** $1,500-$7,000 (most one-time; recurring is ~$50-200/yr registered agent + $400/yr DE franchise tax)
**Founder time:** ~20-30 hours total spread over 90 days; most weeks are 1-2 hours of admin once initial filings are in flight

---

## Phase 1 — Entity formation (Week 1-2)

### ☐ 1. Decide entity type
**Action:** Pick Delaware C-Corp.
**Why:** Standard for tech companies, prime contractors expect it, In-Q-Tel + VC require it, QSBS tax treatment, easy to fundraise into.
**Alternatives considered + rejected:**
- LLC — worse for investor / M&A
- S-Corp — limits future investor classes
- Single-member LLC then convert later — delays the choice without gain

### ☐ 2. Choose entity name
**Action:** Confirm "Sphragis Inc." or "Sphragis Corp." is available.
**Verify:**
- Delaware Division of Corporations: https://icis.corp.delaware.gov/Ecorp/EntitySearch/NameSearch.aspx
- USPTO trademark search: https://www.uspto.gov/trademarks/search (Class 9 Computer Software + Class 42 Computer Services)
**Time:** 15 minutes
**Cost:** $0

### ☐ 3. Form the C-Corp
**Action:** Use Stripe Atlas (recommended).
- https://stripe.com/atlas
- $500 flat. Handles Delaware incorporation + EIN application + Mercury banking + Clerky cap-table tooling all bundled.
**Alternative:** Clerky (~$799-$1,049) or local counsel ($1,500-3,000).
**Time:** ~30 min to fill in the form, then 3-7 days for Delaware to process.
**Cost:** $500 (Atlas) or $1,500-3,000 (local counsel)

### ☐ 4. Receive incorporation documents
**Wait for:**
- Certificate of Incorporation (Delaware)
- Bylaws
- Stock Purchase Agreement / Founder Stock issuance documents
- 83(b) election form
- EIN letter from IRS

### ☐ 5. File 83(b) election — CRITICAL, 30-day window
**Action:** File the 83(b) election with IRS within **30 days** of founder stock issuance.
**Why:** Required if you receive restricted stock. Missing this deadline = massive tax bill on future appreciation. There is no extension available.
**Where:** Stripe Atlas / Clerky handles this automatically; **verify they actually filed it** by checking the certified-mail tracking + the IRS receipt confirmation email.
**Time:** ~10 minutes to verify
**Cost:** $0

---

## Phase 2 — Federal registrations (Week 2-8)

### ☐ 6. Apply for SAM.gov registration — START IMMEDIATELY
**Action:** Submit SAM.gov application as soon as the EIN arrives.
- https://sam.gov
- Free.
**Wait time:** 7-60 days for activation. Expect closer to 30-60. **This is the bottleneck for everything else.**
**Required fields:** legal entity name, EIN, business address (not a UPS Store), DUNS history (auto-converted to UEI), banking info for electronic payment.
**Time:** ~1-2 hours to complete the form initially; intermittent emails to respond to over the ~60 days.

### ☐ 7. Receive UEI (Unique Entity Identifier)
**Auto-issued** by SAM.gov during registration. Replaces the old DUNS number.

### ☐ 8. Apply for CAGE code
**Action:** Apply via SAM.gov registration flow.
**Wait time:** ~14 days after SAM.gov approves.
**Free.**
**Required for:** any DoD contract.

### ☐ 9. Register at DSIP (DoD SBIR/STTR Innovation Portal)
- https://www.dodsbirsttr.mil
- Free.
- Required for any DoD SBIR submission.
**When:** After SAM.gov is active. Takes ~5 minutes once registered.
**Action:** Add company profile + key personnel + technical capabilities.

### ☐ 10. Register at SBIR.gov
- https://www.sbir.gov
- Free.
- Required for SBIR submissions to non-DoD agencies (NSF, DOE, HHS).
**When:** Anytime after incorporation. Skip if you're only targeting DoD.

### ☐ 11. Register at GSA eOffer
- https://eoffer.gsa.gov
- Free.
- Required to submit a GSA MAS Schedule offer.
**When:** Defer to Month 9-12 (need 2 years of past performance, which the first SBIR Phase I will provide).

---

## Phase 3 — Export control compliance (Week 4)

### ☐ 12. File ECCN 5D002 initial classification with BIS — LEGALLY REQUIRED
**Action:** Email both addresses below with the notification template at the bottom of this section. **Must be filed at first publication / first export.** Sphragis is already public on GitHub — the notification should have been filed when the repo went public; filing now is acceptable as a catch-up but should be done before any further crypto-bearing release tag.

**Recipients:**
- `crypt@bis.doc.gov` (Bureau of Industry and Security)
- `web_site@nsa.gov` (NSA)

**Subject:** `License Exception ENC §740.17(b)(1) notification — Sphragis Inc.`

**Body template:**
```
TO: BIS Encryption Notification Officer + NSA Web Site Encryption Notification

Sphragis Inc., a Delaware C-Corporation (EIN [your EIN]),
hereby submits the following notification under License Exception
ENC §740.17(b)(1):

Product:           Sphragis (security-first bare-metal Rust microkernel)
Source repository: https://github.com/kadenlee1107/Sphragis
License:           Apache License 2.0
ECCN:              5D002
Description:       Open-source operating system kernel and supporting
                   user-space tools, distributed publicly under a
                   permissive open-source license.

Cryptographic algorithms used:
  AES-128, AES-256 (GCM, GCM-SIV, XTS, CTR variants)
  ML-KEM-768, ML-KEM-1024 (FIPS 203)
  ML-DSA-65, ML-DSA-87 (FIPS 204)
  LMS (NIST SP 800-208, RFC 8554)
  X25519 (Curve25519 ECDH)
  Ed25519 (signatures)
  RSA-2048, RSA-3072 (verify-only, legacy interop)
  ECDSA P-256, ECDSA P-384 (verify-only, legacy interop)
  ChaCha20-Poly1305, XChaCha20-Poly1305
  SHA-1 (verify-only, legacy interop)
  SHA-256, SHA-384, SHA-512
  SHA-3 family
  BLAKE2s, BLAKE3
  HMAC, HKDF, Argon2id, PBKDF2
  HOTP (RFC 4226), TOTP (RFC 6238)

This notification is being submitted in accordance with EAR
§740.17(b)(1) for publicly available encryption source code.

Contact:
  [Founder name]
  [Founder email]
  [Founder phone]
  Sphragis Inc.
  [Business address]

Filed: [today's date]
```

**Required:** keep the sent-confirmation email FOREVER. It is the proof of compliance under any future BIS audit.
**Time:** 10 minutes
**Cost:** $0

### ☐ 13. Add export-control notice to README
**Action:** After SAM is active, add a public-facing notice to `README.md`:

```markdown
## Export Control

Sphragis includes cryptographic software and is classified under
ECCN 5D002. This software is publicly available under the Apache
License 2.0 and made available outside the U.S. under License
Exception ENC §740.17(b)(1). The U.S. Department of Commerce, BIS,
and NSA have been notified of the source code's publication.
```

---

## Phase 4 — Operational setup (Week 4-8, parallelizable)

### ☐ 14. Business bank account
**Action:** Open via Mercury or Brex.
- https://mercury.com (recommended — best fit for SBIR wires)
- https://www.brex.com (alternative)
**Cost:** $0
**Time:** ~30 min application, ~3 days approval
**Required for:** Phase II ($1.25M+ wires).

### ☐ 15. Domain registration
**Action:** Buy `sphragis.com`, `sphragis.org`, `sphragis.dev` (~$50/yr total).
- Cloudflare or Porkbun. **Avoid GoDaddy.**
**Time:** 10 minutes
**Cost:** ~$50/yr recurring

### ☐ 16. Email infrastructure
**Action:** Google Workspace ($6-12/user/mo) or Fastmail.
**Initial addresses:**
- `kaden@sphragis.com` — founder
- `security@sphragis.com` — vulnerability reports
- `info@sphragis.com` — general inbox / catch-all
- `bd@sphragis.com` — business development (optional, deferable)

### ☐ 17. Project management / CRM
**Recommended Y1:** Linear + Notion (~$10-15/user/mo each).
**Defer Y1:** Salesforce / HubSpot until first paid pilot.

---

## Phase 5 — Trademark (Month 6, deferable)

### ☐ 18. USPTO trademark application
**Action:** File trademark for "Sphragis" and "SealFS" via https://www.uspto.gov
**Classes to claim:**
- Class 9 — computer software
- Class 42 — computer services
**Cost:** $250-350 per class (self-file via TEAS Plus / TEAS Standard)
**With counsel:** add $500-2,000 for trademark counsel (recommended for first-time)
**Wait time:** 8-12 months for examination
**Usage:** ™ until registered; ® once registered.

---

## Phase 6 — Insurance + legal counsel (Month 3-6)

### ☐ 19. General liability + E&O insurance
**Action:** Apply via Vouch or Embroker — startup-friendly carriers.
- https://www.vouch.us
- Required by some IDIQs and SBIR agreements.
**Cost:** ~$1,000-3,000/yr for minimum coverage

### ☐ 20. Outside counsel for federal contracting
**Action:** Schedule an initial 1-2 hour consultation with FAR-familiar counsel.
**Firms to consider:**
- Smith Pachter McWhorter (gov-contracts practice)
- Holland & Knight (broader; FCC-cleared on classified work)
- Reed Smith
- PilieroMazza (small-business set-aside specialty)
- Howe & Associates (smaller specialty firm)
**Cost:** $300-500/hr; budget 5-10 hours of initial consultation.

---

## Decision gates

| Week | Gate | Pass criteria | Pivot if delayed |
|---|---|---|---|
| 2 | Entity formed | EIN + 83(b) filed | Email Stripe Atlas support |
| 4 | SAM submitted + BIS filed | Pending registration confirmation | None — keep working other tracks |
| 8 | SAM active + CAGE + UEI | Eligible to receive federal contracts | If SAM.gov takes longer, delay G2 by however long it takes; not a hard stop |
| 12 | All Phase-1-through-4 items done | Bank, domains, email, insurance, counsel all sorted | None — minor items can slip |

---

## What I'm doing in parallel (Track B)

While you work this checklist, I'm drafting:

1. ✅ **SBIR Phase I Volume 2 Technical Proposal** — `docs/superpowers/funding/2026-05-17-sbir-phase-1-afwerx-open-v0.md`
2. ⏭️ **Defense seed VC pitch deck v1** — refined from the demo deck v0
3. ⏭️ **3-year financial model** — P&L, headcount, burn, runway, revenue ramp
4. ⏭️ **ACT 3 capability brief** — targeted version for AIS / CNF / Global InfoTek
5. ⏭️ **DARPA PM identification + meeting prep**
6. ⏭️ **VC target list + warm-intro paths**

By the day SAM.gov clears, every Track B artifact is ready for you to send.

---

## When to ping me

- ✅ Entity formed + EIN received → **ping me, I'll update the SBIR proposal cover sheet placeholders**
- ✅ SAM.gov approved + UEI/CAGE assigned → **ping me, I'll update all proposal cover sheets**
- ✅ BIS notification sent (got receipt) → **ping me, I'll update the README export-control section**
- ⚠️ Any registration rejection or unclear instruction → **ping me, I'll help debug**
- 🚨 Anything taking >2× the listed wait time → **ping me, we'll escalate**

---

## Honest reality check

This checklist is mechanical, not strategic. The strategy is settled (master plan). The questions left are paperwork-speed questions, not technology or market questions. The longest-pole item (SAM.gov, 30-60 days) is **out of your control once submitted** — it'll clear when GSA's queue clears.

Founder-time budget for this whole checklist: ~20-30 hours spread over 90 days. The first week is the densest (entity formation + SAM application + BIS notification). After that you're mostly waiting on government processing and responding to occasional email inquiries from SAM / IRS / your registered agent.

**Start Phase 1 (Stripe Atlas) today if possible.** Every day of delay shifts the SBIR submission timeline by the same day.

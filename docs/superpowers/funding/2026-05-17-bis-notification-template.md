# BIS / NSA Encryption Source Code Notification — Email Template

**STATUS: DRAFT v1 — KADEN TO SEND**
**Date drafted:** 2026-05-17
**Author:** Funding Team (Mac Claude, vault-mediated)
**Mode:** drafting only; founder sends the email.

**Primary regulatory basis:** 15 CFR §742.15(b) — notification
requirement for *publicly available* encryption source code classified
under ECCN 5D002.
**Cross-reference:** 15 CFR §734.7 — "published" information /
fundamental research exclusion from the EAR.

---

## §0 — Why this exists (read once, then send the email)

Sphragis ships with cryptographic functionality (AES-256, ML-KEM-1024,
ML-DSA-87, LMS, ChaCha20-Poly1305, X25519, Ed25519, SHA family,
Argon2id, HMAC, HKDF — full enumeration below). The kernel is
distributed publicly under Apache-2.0 from `github.com/kadenlee1107/
Sphragis`.

Under 15 CFR §742.15(b), publicly available encryption source code
classified under ECCN 5D002 is **not subject to the EAR** *provided
that* a one-time email notification has been sent to:

1. BIS (Bureau of Industry and Security) — `crypt@bis.doc.gov`
2. NSA ENC Encryption Request Coordinator — `enc@nsa.gov`

The notification must contain either (a) the URL where the source
code is published, OR (b) a copy of the source code. Most open-source
projects send the URL (option a). The notification is a one-time
filing per URL; you only need to re-notify if (i) the URL changes
location, or (ii) you transmit copies rather than a URL and the
cryptographic functionality is updated.

**Once the email is sent and you receive any reply (auto-ack counts),
keep both sent + received copies forever.** They are the compliance
record under any future BIS audit.

Time to send: ~10 minutes. Cost: $0.

---

## §1 — Email to send (copy this into your mail client)

**FROM:** `kaden@sphragis.com` *(or your personal address if the
Sphragis Inc. domain mail isn't yet provisioned — BIS does not care
which address you send from; what matters is the content and the
fact that it was sent)*
**TO:** `crypt@bis.doc.gov`
**CC:** `enc@nsa.gov`
**SUBJECT:** `Notification under 15 CFR 742.15(b) — publicly available encryption source code — Sphragis (ECCN 5D002)`

**BODY:**

```
To: BIS Encryption Notification Officer
Cc: NSA ENC Encryption Request Coordinator

This email constitutes a notification pursuant to 15 CFR
§742.15(b) of the Export Administration Regulations regarding
publicly available encryption source code classified under
ECCN 5D002.

PROJECT INFORMATION

  Project name:        Sphragis
  Source repository:   https://github.com/kadenlee1107/Sphragis
  License:             Apache License 2.0
  ECCN:                5D002
  Brief description:   Open-source security-first bare-metal
                       operating system kernel, written in Rust,
                       targeting Apple Silicon hardware. Includes
                       in-kernel cryptographic primitives,
                       post-quantum hybrid TLS, encrypted
                       filesystem, audit logging, and attestation.

The complete source code (kernel and supporting user-space tools)
is publicly available without restriction at the URL above under
the Apache License 2.0. The project does not require a license,
registration, or any access control to download, build, modify, or
redistribute.

CRYPTOGRAPHIC ALGORITHMS IMPLEMENTED

Symmetric ciphers + AEAD:
  AES-128 / AES-256 (CTR, GCM, GCM-SIV, XTS variants)
  ChaCha20-Poly1305, XChaCha20-Poly1305

Post-quantum key encapsulation:
  ML-KEM-768 (FIPS 203) — used in hybrid TLS
  ML-KEM-1024 (FIPS 203) — used in non-TLS contexts

Post-quantum digital signatures:
  ML-DSA-65, ML-DSA-87 (FIPS 204)
  LMS (NIST SP 800-208, RFC 8554) — verify-only

Classical asymmetric:
  X25519 (Curve25519 ECDH)
  Ed25519 (signatures)
  RSA-2048, RSA-3072 (verify-only, legacy interop)
  ECDSA P-256, ECDSA P-384 (verify-only, legacy interop)

Hash + MAC + KDF:
  SHA-256, SHA-384, SHA-512
  SHA-3 family
  BLAKE2s, BLAKE3
  HMAC (with SHA-256, SHA-384, SHA-512)
  HKDF
  Argon2id (memory-hard passphrase KDF)
  PBKDF2

One-time-password:
  HOTP (RFC 4226)
  TOTP (RFC 6238)

Legacy / verify-only:
  SHA-1 (legacy interop only — verify path)

The cryptographic algorithms above are all standard, well-known
algorithms specified in publicly available standards documents
(FIPS, NIST SP, RFC). The project does not implement non-standard
cryptography as defined in 15 CFR Part 772.

CONTACT INFORMATION

  Person submitting notification:
    [FOUNDER: enter your real legal name]
  Email:
    [FOUNDER: enter the email you want BIS to reply to]
  Phone:
    [FOUNDER: enter your phone — optional but recommended]
  Organisation:
    Sphragis Inc. (Delaware C-Corporation, in formation as of 2026-05-17)
  Business address:
    [FOUNDER: enter the address you registered for incorporation;
     if Atlas provides a registered-agent address, that is acceptable]

DATE OF FIRST PUBLICATION

  The source code repository at the URL above has been publicly
  accessible on the internet since [FOUNDER: enter the actual date
  the repository was first made public — check your GitHub repo
  "Insights → Activity" history for the earliest commit/push date].

Please confirm receipt of this notification at the reply address
above. Thank you.

Regards,

[FOUNDER: your name]
[FOUNDER: your title — e.g. "Founder & Maintainer"]
Sphragis Inc.
[FOUNDER: your email]
```

---

## §2 — Fields the founder must fill before sending

| Field in the template | What to substitute |
|---|---|
| `[FOUNDER: enter your real legal name]` | Your legal name as it will appear on the C-Corp incorporation docs (e.g. `Kaden Lee`). |
| `[FOUNDER: enter the email you want BIS to reply to]` | Either `kaden@sphragis.com` (recommended once provisioned) or your personal address. |
| `[FOUNDER: enter your phone]` | Direct line. Optional but recommended — BIS sometimes calls back rather than emails. |
| `[FOUNDER: enter the address you registered for incorporation]` | Whatever you put on the Stripe Atlas / Clerky filing. A registered-agent address is fine. |
| `[FOUNDER: enter the actual date the repository was first made public]` | Check `https://github.com/kadenlee1107/Sphragis` Insights → Activity → earliest push to `main`. **This is a factual claim; do not guess.** |

---

## §3 — Differences from the v0 template in the founder-action-checklist

The earlier draft in
`docs/superpowers/funding/2026-05-17-founder-action-checklist.md`
(§Phase 3, item 12) had three issues this v1 template fixes:

1. **Wrong CFR citation.** The v0 template cited
   "EAR §740.17(b)(1)." That section governs encryption *items*
   eligible for License Exception ENC (a different regulatory
   pathway). The correct citation for one-time notification of
   *publicly available* encryption source code under ECCN 5D002 is
   **15 CFR §742.15(b)**. (Some legal practitioners also reference
   §734.7 "published information" exclusion as the legal basis for
   why notified source code is then "not subject to the EAR" — the
   §742.15(b) email is the trigger.)

2. **Wrong NSA email address.** The v0 template addressed
   `web_site@nsa.gov`. The correct address per current BIS guidance
   and §742.15(b) is **`enc@nsa.gov`** (the ENC Encryption Request
   Coordinator inbox). EFF's published explainer of EAR encryption
   notification (2019) and current BIS guidance both confirm this
   pair: `crypt@bis.doc.gov` + `enc@nsa.gov`.

3. **Form-letter style mismatch.** The v0 was structured as a
   memo. This v1 is structured as the literal email body the
   founder will paste; uses a plain-text monospace-friendly layout
   that survives any mail client; clearly delimits founder-edit
   fields so nothing ships with a `[FOUNDER: ...]` placeholder.

This v1 template should be used in preference to the v0 in the
founder-action-checklist. The founder-action-checklist remains the
authoritative *paperwork roadmap*; this file is the authoritative
*email body*.

---

## §4 — What Kaden does next

1. **Verify the first-public-push date** of the repository at
   `https://github.com/kadenlee1107/Sphragis`. Use GitHub Insights →
   Activity. Write the actual date into the `DATE OF FIRST
   PUBLICATION` field.
2. **Decide which "from" address to send from.** Personal email is
   acceptable. `kaden@sphragis.com` is cleaner once the Google
   Workspace tenant is provisioned (founder-action-checklist
   Phase 4 item 16). Either works for BIS.
3. **Confirm the Sphragis Inc. business address** matches what was
   submitted on the Delaware C-Corp incorporation. If incorporation
   is still pending (the action checklist says "in flight"),
   sending now from a personal address with a personal address
   field is acceptable — BIS notification is about the *source
   code being publicly available*, not about corporate status. If
   you wait, you delay protection.
4. **Fill the five `[FOUNDER: ...]` fields** in the email body.
5. **Send the email.** TO=`crypt@bis.doc.gov`,
   CC=`enc@nsa.gov`. Subject and body as in §1.
6. **Save both the sent copy and any auto-acknowledgment** to a
   permanent location — recommended: a `compliance/bis-742-15b/`
   folder in your encrypted archive. This is your audit-proof
   forever.
7. **Add the export-control notice to README.md** (founder-action
   checklist item 13) referencing this notification by date.

**Estimated total founder time:** ~10 minutes (including the date
lookup).

---

## §5 — Primary sources cited

- `docs/superpowers/funding/2026-05-17-founder-action-checklist.md`
  §Phase 3 item 12 (v0 template — this v1 supersedes it on CFR
  citation + NSA email)
- `marketing-site/index.html` lines 1769-1810 (Specifications panel —
  confirms the algorithm list shipping in the public build)
- `docs/superpowers/research/2026-05-17-day1-sweep-and-funding-readiness.md`
  §1 ("Crypto suite — CNSA 2.0 ready") — authoritative list of
  algorithms implemented
- `docs/superpowers/plans/2026-05-17-multi-team-push.md` §3 (Funding,
  draft #3) — charter directive

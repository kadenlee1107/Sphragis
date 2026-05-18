# Sphragis — LinkedIn Announcement (STRETCH Draft v1)

**STATUS:** STRETCH — NOT REQUIRED FOR PUSH DoD. Kaden reviews + posts. Outreach team only drafts.
**Date:** 2026-05-17
**Author:** Outreach team (2026-05-17 multi-team push)
**Source:** Marketing site (`marketing-site/index.html`) public-claim ceiling; `CLAUDE.md` project description; `docs/superpowers/funding/2026-05-17-vc-pitch-deck-v1.md` Appendix B elevator pitch for tone.

**LinkedIn posting discipline:**
- LinkedIn audience is different from HN / Lobsters: more professional / less technical, friendlier to "what does it do" framing, more receptive to founder-narrative posts.
- LinkedIn favors **3–6 short paragraphs separated by line breaks**, opening with a hook in the first 2 lines (because the "see more" cutoff hides the rest of the post on mobile). The first 2 lines are everything.
- A photo / image attachment dramatically improves reach. Recommend attaching the M4 boot photo set (or one curated shot from `marketing-site/img/m4-boot/`).
- **Acceptable to mention founder-stage activity here** — LinkedIn is a professional network where the "I'm building a thing" narrative is welcomed. Still keep it grounded and avoid hype words.
- One variant for general professional network + one variant for defense / security / sovereign-tech network. Kaden picks one or posts both spaced 2–3 weeks apart.

---

## Variant A — General professional / engineering network

**Hook (first 2 lines — these are the only lines visible above the fold on mobile):**

```
Today I'm sharing Sphragis publicly for the first time:
a memory-safe Rust microkernel that boots on Apple M4 hardware.
```

**Full post body:**

```
Today I'm sharing Sphragis publicly for the first time:
a memory-safe Rust microkernel that boots on Apple M4 hardware.

This has been my work over the last 14 weeks — a single-author
project to see how small a secure workstation OS could be if you
deleted everything that wasn't load-bearing. The answer: about
96,000 lines of Rust, zero third-party kernel code, and a hardware
target (Apple Silicon M4) that no other non-Apple OS currently
supports.

Three properties are enforced in the kernel, not by convention:

— SealFS encrypts every block before it touches disk
  (AES-256-GCM-SIV with Argon2id key derivation). No file ever
  lives unencrypted.

— TLS 1.3 with hybrid post-quantum key agreement
  (X25519+ML-KEM-768) runs in the kernel. Processes get plaintext
  file descriptors and write HTTP. They cannot ship broken TLS,
  skip certificate validation, or downgrade to HTTP.

— Default-deny network egress per process, with every denial
  audit-logged. A process with no policy entry cannot reach the
  network.

What it deliberately omits: no web browser (browsing belongs on a
separate device), no package manager, no telemetry. It is a
workstation, not a laptop.

The boot-evidence photos on the site are from April 17, 2026 —
phone-camera shots of the M4 internal display because power went
out before host capture. Not pretty. Durable.

Source under Apache-2.0: https://github.com/kadenlee1107/Sphragis
Marketing site with full specifications and the boot photos:
[https://sphragis.com — Kaden to confirm actual URL]

Happy to talk to engineers interested in Rust at the kernel layer,
post-quantum TLS implementations, encrypted filesystems, or the
M4 reverse-engineering work. DMs open.

#Rust #Security #Cryptography #PostQuantum #OS #AppleSilicon
```

---

## Variant B — Defense / security / sovereign-tech network

**Hook:**

```
Two procurement cliffs converge in 18 months: FIPS 140-3 in
September, and the NSA CNSA 2.0 mandate in January 2027.
```

**Full post body:**

```
Two procurement cliffs converge in 18 months: FIPS 140-3 in
September, and the NSA CNSA 2.0 mandate in January 2027.

After 2027-01-01, every new National Security System acquisition
must use ML-KEM-1024 + ML-DSA-87 + AES-256 + SHA-384. RSA and
ECDSA become forbidden for new deployments. The installed-base
operating systems — Green Hills INTEGRITY-178B (certified in 2008,
frozen), VxWorks 653, LynxOS-178, RHEL — cannot meet that bar
without multi-year retrofit work.

Layered on top: CISA / NSA / ONCD memory-safety policy guidance
explicitly identifies Rust as the canonical path for new
greenfield systems software. Linux is 30 million lines of C.
Rewriting it to Rust is barely started.

I've spent the last 14 weeks building Sphragis: a memory-safe
Rust microkernel positioned for exactly this procurement-refresh
window. CNSA-2.0-native from day one (not retrofitted). Boots on
real Apple M4 hardware today via an independent reverse-
engineering pipeline (Asahi Linux doesn't yet support M4). Apache-
2.0 license so defense primes can integrate without copyleft
contamination. 14 weeks of mechanical-trace security audit
history. Bit-identical reproducible builds.

This is the substrate the procurement vacuum needs filled. The
public evidence is at github.com/kadenlee1107/Sphragis (Apache-
2.0, DCO sign-off on every commit, reproducible build verified).

Honest about gaps: FIPS 140-3 module-boundary documented but not
yet certified (CMVP lab pre-engagement is on the work plan).
x86_64 port is designed but not built. Verus formal-proof work
on the capability dispatcher + IPC paths is partial-spec rather
than complete.

If you're in the defense / IC / sovereign-tech ecosystem and any
of this is in your decision space — whether as integrator, prime,
PM, evaluator, or capital allocator — I'd value a conversation.
DMs open.

#NationalSecurity #PostQuantum #Cryptography #CNSA20 #Rust
#MemorySafety #DefenseTech #AmericanDynamism
```

---

## What Kaden does next

1. **Pick a variant.** Recommendation: post **Variant A first** (engineering / professional network) and let it settle 2–3 weeks before posting Variant B. Variant B has a more pointed defense / commercial framing and will pull a different (smaller, more targeted) audience that benefits from being preceded by the broader-network credibility build of Variant A. Posting both same-day will look like a launch campaign and dilute both.
2. **Attach a photo.** Strongly recommend pulling one shot from `marketing-site/img/m4-boot/` (the `IMG_7118.jpg` "microkernel shell on M4" photo is the strongest first-impression image — boot prompt visible, real-hardware-evidence framing).
3. **Edit the first 2 lines until they fit above the mobile fold** (~140 chars / 2 short lines). LinkedIn cuts off everything after that on mobile until the reader taps "see more." The first 2 lines are 80% of the post's reach.
4. **Replace the marketing-site URL placeholder** with the actual URL before posting.
5. **Time the post for Tuesday–Thursday morning US Eastern** for best engagement. Avoid posting during major news cycles or industry conferences (your post drowns).
6. **Engage with comments for 24–48h.** LinkedIn comment-engagement compounds reach more than HN / Lobsters do. Respond to every substantive comment.
7. **Hashtag discipline.** LinkedIn rewards 3–5 well-chosen hashtags. Avoid `#OS` / `#Tech` / `#Innovation` (too broad — low signal). Variant A's tags are engineering-focused; Variant B's tags are defense-vertical-focused.

**Do NOT post until Kaden has reviewed personally.** Outreach team only drafts.

---

## Out-of-scope

- Posting on LinkedIn (Kaden posts).
- Engaging with LinkedIn comments / DMs (Kaden engages).
- Coordinating with HN / Lobsters posts (those have their own drafts; spacing is recommended above but Kaden decides).
- Editing the marketing site (out of scope for Outreach).

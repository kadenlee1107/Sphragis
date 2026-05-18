# Sphragis — Lobsters Launch Post (STRETCH Draft v1)

**STATUS:** STRETCH — NOT REQUIRED FOR PUSH DoD. Kaden reviews + posts. Outreach team only drafts.
**Date:** 2026-05-17
**Author:** Outreach team (2026-05-17 multi-team push)
**Source:** Marketing site (`marketing-site/index.html`) public-claim ceiling; `CLAUDE.md` project description.

**Lobsters posting discipline:**
- Lobsters is invite-only, smaller and more technical than HN. The audience is comparatively friendly to systems work, low-level engineering, Rust, security, reproducible builds.
- Lobsters is **strict on tagging** and on self-promotion. Authors submitting their own work must check the "I am the author" box; this is a community norm enforced by moderators.
- **Allowed tags relevant to Sphragis:** `rust`, `security`, `cryptography`, `osdev`, `apple`, `release` (if released), `show` (if "look at this thing"). Submissions need at least one tag. Recommended tag set below.
- Lobsters does NOT have a Show HN equivalent — there's a `show` tag with the same intent, but it's more relaxed.
- **No marketing speak.** Lobsters downvotes hype as quickly as HN. Even more allergic to "another OS" pitches than HN — but more receptive once they see real boot evidence and engineering substance.

---

## Submission

**Title:** `Sphragis: a Rust microkernel that boots on Apple M4 silicon`

**URL:** `https://github.com/kadenlee1107/Sphragis` (preferred — Lobsters audience clicks to source first; the marketing page is a credibility anchor accessible via the repo README) **OR** `https://sphragis.com` if Kaden prefers leading with the marketing page (less common on Lobsters but acceptable when boot-evidence is the visual hook).

**Tags:** `rust`, `osdev`, `security`, `apple`, `show`

**"I am the author" checkbox:** YES (required by community norms).

**Description (optional field; if used):**

```
A bare-metal Rust microkernel I've been building over the last 14
weeks. Boots on a stock Apple M4 MacBook Pro 14" via an independent
reverse-engineering pipeline (Asahi doesn't yet support M4 as of
May 2026). Kernel-mediated TLS 1.3 with hybrid PQ key agreement,
SealFS encrypted filesystem (AES-256-GCM-SIV + Argon2id), per-
process default-deny network egress. ~96K LoC of no_std Rust on
the pinned nightly, 0 warnings, 0 clippy lints. Apache-2.0.
```

---

## First comment from submitter (post within 60 seconds of submission)

```
Author here. Happy to answer technical questions about anything in
the repo — particularly relevant to a Lobsters audience:

1. The M4 boot path. m1n1 chainload from macOS Recovery
   (Permissive Security via `kmutil configure-boot`). Our vendored
   m1n1 has a `--skip-secondary-cpus` flag pre-added because the
   M4 P-cluster SErrors on RVBAR writes — different from M1, where
   that wasn't an issue. The boot-evidence photos on the marketing
   page (and in docs/photos/) are phone-camera shots of the M4
   internal display from April 17, 2026 — power went out before
   host frame capture, so phone photos are the durable record.

2. The TLS-as-kernel-syscall model. Processes never touch TLS.
   They open a hostname, get a plaintext fd, and write HTTP. The
   kernel handles handshake (including X25519MLKEM768 hybrid PQ
   key agreement per draft-ietf-tls-ecdhe-mlkem-04) + cert chain
   validation. Verified end-to-end against Cloudflare's public PQ
   research endpoint. Six trust anchors ship with the OS. No
   fallback paths, no pin-and-pray. Design doc:
   DESIGN_HTTPS_SYSCALL.md in the repo.

3. SealFS. AES-256-GCM-SIV (RFC 8452 misuse-resistant AEAD)
   on every block. Argon2id master key derivation (8 MiB / 3
   passes). On 2026-05-17 we did a magic-byte + version rotation
   (the project is pre-production; the only "users" are us, so we
   broke backwards compat deliberately rather than carry legacy
   format support forever). Pre-2026-05-17 disk images fail magic
   check by design.

4. The cave model — per-process isolation primitive, not Unix
   processes-with-namespaces. Each cave has a classification label
   (Bell-LaPadula + Biba) enforced at the kernel layer on every
   cross-cave call. No shared memory between caves except through
   explicit kernel-mediated IPC.

What's deliberately omitted: no web browser (browsing is on a
separate device), no package manager / app store, no third-party
kernel code, no telemetry. RustCrypto for the audited crypto
primitives is the only external dependency in the security path.

What doesn't work yet: not yet FIPS certified, not yet on x86_64
(designed but not built), Verus formal-proof work on the cap
dispatcher + IPC paths is partial-spec rather than complete. The
honest framing is "boots on real M4 hardware with real properties"
not "production-ready."

Repo: https://github.com/kadenlee1107/Sphragis (Apache-2.0, DCO
sign-off, bit-identical reproducible build).
```

---

## What Kaden does next

1. **Submit only after the HN post has settled** (24–48h delay minimum). Cross-posting Sphragis to HN + Lobsters on the same day is fine, but Lobsters mods sometimes flag obvious cross-post coordination. Spaced out feels less like a launch campaign.
2. **Tag set:** `rust`, `osdev`, `security`, `apple`, `show`. If Lobsters complains about tag count (max is usually 3), drop `show` first, then `apple`.
3. **Check the "I am the author" box.** Required.
4. **Be present in comments for the first 24 hours.** Lobsters comment threads run slower than HN but stay alive longer. Engaged authors get more upvotes.
5. **Do NOT mention fundraising, contracts, market sizes, or commercial intent.** Same allergy as HN, plus Lobsters has an explicit no-self-promotion-of-commercial-products norm — engineering substance only.
6. **Anticipate questions about M4 RE details, the TLS model, and the cave model.** Have repo links ready for design docs (`DESIGN.md`, `DESIGN_CAVES.md`, `DESIGN_TLS_HARDENING.md`, `DESIGN_HTTPS_SYSCALL.md`, `DESIGN_CRYPTO.md`).

**Do NOT post until Kaden has reviewed personally.** Outreach team only drafts.

---

## Out-of-scope

- Posting on Lobsters (Kaden posts).
- Replying to comments (Kaden replies).
- Acquiring a Lobsters invite if Kaden doesn't have one (out of scope; if no invite is available, route through HN-only and revisit later).

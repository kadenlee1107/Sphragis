# GitHub Accelerator / Secure Open Source Fund — Application Draft

**STATUS: DRAFT v1**
**Date drafted:** 2026-05-17
**Author:** Funding Team (Mac Claude, vault-mediated)
**Mode:** drafting only; founder submits.

---

## §0 — Cohort window status (READ FIRST)

The charter asked for a draft of "GitHub Accelerator" with a fallback
header if no cohort is open. After researching GitHub's current
programs as of 2026-05-17:

**GitHub Accelerator (the program by that name):**
- **STATUS: CLOSED.** The last cohort was 2024 (applications closed
  2024-03-05, kickoff 2024-04-22). No 2026 cohort has been
  announced as of 2026-05-17 per `accelerator.github.com` and
  `github.com/open-source/accelerator`.
- The 2024 cohort was **AI/ML focused** — a poor fit for Sphragis
  on principle (the Sphragis project explicitly excludes AI/LLM
  from the trusted computing base; see ANTI-002 in
  `ANTI_FEATURES.md`).
- Action: monitor `accelerator.github.com` quarterly. If a 2026
  or 2027 cohort with broader security/infrastructure scope
  reopens, use the application body below as the starting draft.

**GitHub Secure Open Source Fund:**
- **STATUS: OPEN — applications accepted on a rolling basis** per
  `github.com/open-source/github-secure-open-source-fund` (verified
  2026-05-17).
- Direct fit for Sphragis: security-focused, requires "clear
  open source license" (Sphragis is Apache-2.0) + "demonstrated
  community adoption" + "commitment to improving security"
  (Sphragis matches all three).
- Benefits: $10,000 cash ($6K at program start + $2K at 6-mo
  + $2K at 12-mo check-in) + $10K Azure credits + up to $150K
  additional Azure credits via Microsoft for Startups + 3-week
  intensive program + GitHub Security Lab office hours + GitHub
  Copilot/AutoFix access + Microsoft for Startups onboarding.

**The body of this draft (§1 onward) is written for the GitHub Secure
Open Source Fund — the program that is currently open and the
best programmatic fit for Sphragis.** When/if GitHub Accelerator
reopens with a security or infrastructure track, this same body can
be adapted with minor reframing (mostly the "what we'd do with the
$40K stipend / 10-week duration" sections).

**GitHub Fund (M12 equity vehicle):** flagged but out of scope for
this draft. M12 is equity investment, not a grant; the typical entry
point is "after first paying customer or first SBIR Phase I award."
Sphragis is pre-incorporation as of 2026-05-17; revisit M12 once
the C-Corp is live and a federal contract or Series Seed conversation
is in motion. The day-1 sweep §6 already identifies defense-focused
seed VCs (Shield, Lux, a16z American Dynamism) as the primary VC
target list; M12 would be additive at the Series Seed stage.

---

## §1 — Application body: GitHub Secure Open Source Fund

> **Application route:** `github.com/open-source/github-secure-open-source-fund`
> → "Apply now" button. The form is reportedly short and lightly
> structured; this body is sized to compress further if individual
> fields impose tighter limits. Founder fills in identifying fields
> from the action checklist below.

### §1.1 — Project identification

> **Project name:** Sphragis
>
> **Repository:** `https://github.com/kadenlee1107/Sphragis`
>
> **License:** Apache License 2.0 — pure permissive, no GPL/LGPL/
> SSPL/Commons-Clause/BUSL contamination in the dependency graph,
> enforced by `cargo-deny` in CI. License conversion from AGPL-3.0
> was completed on 2026-05-16.
>
> **Project type:** Security-first bare-metal Rust microkernel for
> Apple Silicon. `#![no_std] #![no_main]` against
> `aarch64-unknown-none`. Boots on real Apple M4 hardware
> (Mac16,1 / J604 / T8132 "Donan") via an independent
> reverse-engineering pipeline.
>
> **Maintainer:** Kaden Lee — sole maintainer.
> GitHub: `@kadenlee1107`.
>
> **Codebase scale:** ~96,000 lines of Rust across 199 source files.
> Bit-identical reproducible build verified (SHA-256
> `f4b12add37d44d4ae031a0bc5db83739a15c2d54d7d8096e1fcb667ca7e5ad03`).

### §1.2 — Community adoption / governance

> **Community traction:** Sphragis is pre-1.0 with no established
> downstream dependents. The project is explicit about this rather
> than overstating community traction. The relevant traction
> indicators today are:
>
> - 143 commits in the most recent 24-hour productization push;
>   14 weeks of mechanical-trace security audit history visible in
>   the git log
> - Sphragis has booted on real Apple M4 hardware (boot evidence in
>   `docs/photos/2026-04-17_first_m4_boot/`) — independent
>   capability, since Asahi Linux does not yet support M4
> - Verified hybrid post-quantum TLS interop with
>   `pq.cloudflareresearch.com` — interop test in the test scripts
> - Multiple parallel grant + procurement applications in flight
>   (Sovereign Tech Fund, NLnet, OpenSSF Alpha-Omega, US Federal
>   SBIR Phase I — drafts in `docs/superpowers/funding/`)
>
> **Governance:** Lightweight by design at this maturity stage —
> single-maintainer with DCO sign-off required on every commit
> (`CONTRIBUTING.md`). Code of Conduct + Privacy Statement adoption
> committed to as part of accepting the Secure Open Source Fund
> Code-of-Conduct requirement. Roadmap is public
> (`docs/superpowers/plans/2026-05-16-sphragis-gov-os-master-plan.md`).

### §1.3 — Why we need security investment

> Sphragis is foundational open-source security infrastructure:
> downstream consumers running on Sphragis inherit memory safety,
> post-quantum crypto, kernel-mediated TLS, audit-logged egress,
> and capability isolation as kernel-enforced guarantees rather
> than userspace conventions. The project sits at the operating
> system substrate layer, beneath the libraries and applications
> that GitHub Secure Open Source Fund already supports.
>
> The 2027 procurement-cliff window (NSA CNSA 2.0 + EU CRA + NIS2)
> simultaneously requires post-quantum cryptography + memory-safe
> systems languages + reproducible builds. Most operating systems
> shipping today do not meet that bar. Sphragis is being built
> specifically to close this gap as an Apache-2.0 OSS substrate
> that prime contractors and commercial integrators can adopt
> without copyleft concerns.
>
> The specific security workstreams the Fund would accelerate:
>
> 1. **CodeQL / SAST integration.** Sphragis has `cargo-deny` +
>    `cargo-audit` in CI today, blocking license violations and
>    known-CVE crates. Adding GitHub Advanced Security (CodeQL)
>    on top would catch a class of bugs that lint-and-clippy
>    discipline alone cannot.
> 2. **Dependabot tuning.** The dependency graph is small
>    (~200 transitive crates, all permissive) but kernel
>    cryptographic code is in the highest-risk class; tighter
>    Dependabot configuration with auto-PR routing for cryptography
>    crates specifically would catch the early signal of upstream
>    RustSec advisories.
> 3. **Security policy + advisory workflow hardening.** Sphragis
>    needs a published `SECURITY.md` with a disclosure timeline,
>    a `security@sphragis.com` reporter inbox (Phase 4 item 16 in
>    the founder-action checklist), and a GitHub Security Advisories
>    workflow ready for first-CVE handling.
> 4. **Supply-chain attestation.** Sigstore, in-toto, and Rekor
>    workflow YAMLs are staged at `.github-workflows-pending/`
>    (OAuth-blocked from auto-push; the Secure Open Source Fund's
>    Microsoft / GitHub workflow-add tooling would unblock these
>    in days).

### §1.4 — Plan for the 3-week intensive

> Sphragis's solo maintainer can dedicate the required 15 program
> hours over 3 weeks plus 2.5-hour check-ins at 6 and 12 months
> (20 hours total).
>
> **Week 1 — Inventory + harden the existing CI gate.** Add CodeQL
> to the GitHub Actions workflow. Tune Dependabot routing for
> cryptography-bearing crates (the `ml-kem`, `ml-dsa`,
> `aes-gcm-siv`, `argon2`, and `x25519-dalek` crates specifically).
> Publish `SECURITY.md` with disclosure timeline + PGP key.
> Stand up `security@sphragis.com` inbox.
>
> **Week 2 — Supply-chain attestation goes live.** Wire the staged
> sigstore + in-toto + Rekor workflow YAMLs into the actual
> GitHub Actions release pipeline (`.github-workflows-pending/` →
> `.github/workflows/`). LMS-signed kernel image released and
> verified on M4 hardware boot. SBOM auto-generated and attached
> to every release tag.
>
> **Week 3 — Validate + publish.** Run the GitHub Advanced Security
> CodeQL pass on the entire codebase, triage findings, file
> remediation issues. Publish a public blog post documenting the
> hardening work and inviting third-party reproducibility
> verification. Commit to monthly security retrospective posts
> for the 12-month follow-on window.
>
> **6-month check-in deliverable:** evidence that the hardening
> workstreams above remain green (CodeQL clean, Dependabot
> auto-PRs being merged on schedule, sigstore signing live on
> every release).
>
> **12-month check-in deliverable:** evidence of zero memory-safety
> CVEs (memory safety is a property of the language choice, which
> the program acknowledges) and demonstrable adoption signal
> (downstream projects citing Sphragis, third-party reproducibility
> verifications, conference presentations).

### §1.5 — Use of the $10,000 program cash + $10-150K Azure credits

> **$10,000 cash, distributed $6K + $2K + $2K** (per program rules):
>
> - $6,000 at program start covers ~80 hours of maintainer time
>   exclusively on security hardening work (Weeks 1-3 plus
>   immediate follow-up).
> - $2,000 at 6-month check-in covers ~25 hours of follow-on
>   security review, CVE response capacity, and the 6-month
>   retrospective post.
> - $2,000 at 12-month check-in covers ~25 hours of the same plus
>   the 12-month review.
>
> **$10,000 baseline Azure credits + up to $150,000 Microsoft for
> Startups Azure credits:**
>
> - CI runners for reproducible-build verification (the
>   bit-identical-build property requires sustained CI capacity).
> - Hosted CodeQL execution against the kernel codebase.
> - Marketing site + project blog hosting (currently scaffold-only
>   per the day-1 sweep §3).
> - Storage for SBOM + in-toto envelope + Rekor entry artifacts.
> - The $150K Microsoft for Startups tier (if Sphragis Inc.
>   qualifies post-incorporation) becomes a multi-year runway
>   contributor.

### §1.6 — Commitment to the program

> Sphragis Inc. (Delaware C-Corp in formation as of 2026-05-17 per
> the founder-action-checklist Phase 1 item 3) commits to:
>
> - The 15-hour intensive program over 3 weeks.
> - The 2.5-hour check-ins at 6 and 12 months.
> - The GitHub Code of Conduct + Privacy Statement.
> - Public monthly progress reports on the project blog covering
>   the security hardening workstreams.
> - GitHub Secure Open Source Fund acknowledgment in `README.md`,
>   release notes for every tagged release during the
>   program window, and any blog or conference presentation
>   directly covering the funded workstreams.
> - 40-hours-per-week effort on Sphragis is NOT promised
>   (Sphragis is currently solo-maintained without external
>   funding; the maintainer's allocation across the next 6 months
>   will be calibrated to the actual funding portfolio across STF,
>   NLnet, Alpha-Omega, SBIR, and this program — see the parallel
>   applications in `docs/superpowers/funding/`). What IS promised
>   is the 15 + 2.5 + 2.5 = 20 hours required by the program plus
>   the follow-on security work the program funds directly.

---

## §2 — Public-claim ceiling check

Every claim in this draft is grounded in primary sources:

- Code claims (M4 boot, ~96K LoC, 199 files, Apache-2.0,
  cryptographic algorithms, reproducible build SHA) →
  `marketing-site/index.html` lines 1180-1810
- Status of CI gates, staged workflow YAMLs, sigstore/in-toto
  design state → `docs/superpowers/research/2026-05-17-day1-sweep-and-funding-readiness.md`
  §3 ("What's live but skeletal vs production-quality") + §1
  ("Build chain + provenance")
- ANTI-002 (no AI in TCB) → repository `ANTI_FEATURES.md` (cited
  but not re-quoted)
- Parallel grant applications → `docs/superpowers/funding/`
  (STF, NLnet, Alpha-Omega, SBIR drafts dated 2026-05-17)
- Procurement-cliff context → master plan + day-1 sweep §3

No claim in this draft exceeds the marketing site's public ceiling.

---

## §3 — Alternative framing if GitHub Accelerator reopens

If a future GitHub Accelerator cohort opens with a security or
infrastructure track (broader than the 2024 AI-only scope), the
core narrative in §1.3 (why security investment), §1.4 (the
3-week intensive plan), and §1.6 (commitment) all transfer
directly. The differences would be:

- **Stipend:** $40,000 (Accelerator) vs $10,000 (Secure Open
  Source Fund). The $40K covers full-time maintainer engagement
  for the 10-week program duration; the $10K is structured as
  hourly capacity targeted specifically at the security-hardening
  workstreams.
- **Duration:** 10 weeks (Accelerator) vs 3 weeks (Secure Open
  Source Fund) — the Accelerator framing would expand each Week
  N item in §1.4 by ~3× scope.
- **Selection rate:** 10 projects (Accelerator) vs rolling
  intake (Secure Open Source Fund) — Accelerator is more
  competitive; the application would need a tighter "why this
  project, why now" hook.

For a 10-week Accelerator framing, the natural workstream
progression would be: Weeks 1-3 = security hardening (as in §1.4);
Weeks 4-6 = SLSA Level 4 supply-chain provenance (overlapping with
the Alpha-Omega WP2 ask); Weeks 7-9 = community + governance build
(SECURITY.md, CONTRIBUTING.md, code-of-conduct, security
disclosure protocol, contributor onboarding); Week 10 = capstone
public demo + retrospective.

If/when GitHub Accelerator reopens, fork this file to
`docs/superpowers/funding/YYYY-MM-DD-github-accelerator-vN.md` and
adapt §1 with the new program scope. Keep §0 (cohort window
status) as the authoritative pointer.

---

## §4 — What Kaden does next

1. **Decide which program(s) to apply to first.** Recommendation:
   apply to **GitHub Secure Open Source Fund first** (rolling
   applications, direct security fit, low time commitment, no
   AI-only scope filter). Hold GitHub Accelerator for if/when
   the next cohort window opens.
2. **Open the GitHub Secure Open Source Fund application form** at
   `https://github.com/open-source/github-secure-open-source-fund`
   → click "Apply now."
3. **Confirm GitHub Sponsors region eligibility.** The program
   requires the applicant be located in a GitHub-Sponsors-supported
   region. The US (founder's residence as of the day-1 sweep) is
   supported.
4. **Fill the identifying fields** (legal name, GitHub handle,
   project URL, license, brief description).
5. **Paste content from §1.1 through §1.6** into the corresponding
   free-text fields. If the form has tighter character limits than
   this draft, compress §1.3 first (it has the most narrative
   slack; §1.4 and §1.6 are concrete and harder to compress).
6. **Confirm the 20-hour program commitment** is feasible against
   parallel work (it should be — 15 hours over 3 weeks is 1 hour/
   workday, plus two 2.5-hour check-ins later).
7. **Submit.** Expect a virtual interview as the next step
   (mirrors the GitHub Accelerator process). Bring the public
   marketing site link, the day-1 sweep, and a 1-page security
   hardening plan if asked.
8. **If awarded:** the program-start check ($6K) and Azure
   credits trigger. Begin Week 1 of §1.4 immediately. Monthly
   public progress reports start the first month post-program-end.
9. **Bookmark `accelerator.github.com`** and re-check quarterly
   for GitHub Accelerator 2026 or 2027 cohort announcement.
10. **Bookmark `github.com/open-source/github-fund`** for the M12
    equity-fund path — but defer until after the C-Corp is live
    and there's a first-paying-customer or SBIR Phase I award to
    anchor the conversation. Per the day-1 sweep §6, defense-
    focused seed VCs (Shield, Lux, a16z American Dynamism) are
    the primary VC target list; M12 is additive at Series Seed.

**Estimated total founder time:** ~60 minutes for the Secure Open
Source Fund form submission + 45 minutes for the virtual interview
(when scheduled).

---

## §5 — Primary sources cited

- `docs/superpowers/research/2026-05-17-day1-sweep-and-funding-readiness.md`
  — capability + status truth-source, parallel-applications
  context
- `marketing-site/index.html` lines 1180-1810 — public-claim
  ceiling
- `docs/superpowers/funding/2026-05-17-founder-action-checklist.md`
  Phase 1 (incorporation status) + Phase 4 item 16 (email
  infrastructure including `security@sphragis.com`)
- `docs/superpowers/funding/2026-05-17-stf-form-field-answers.md`
  — model for compression discipline
- `docs/superpowers/funding/2026-05-17-openssf-alpha-omega-v0.md`
  — flagged overlap on supply-chain attestation work package
- `docs/superpowers/plans/2026-05-17-multi-team-push.md` §3
  (Funding, draft #2) — charter directive
- `accelerator.github.com` (WebFetch, 2026-05-17) — 2024 cohort
  status verification, AI-only scope filter
- `github.com/open-source/github-secure-open-source-fund`
  (WebFetch, 2026-05-17) — current program details, rolling
  applications, $10K cash + $10-150K Azure credits
- `github.com/open-source/github-fund` (WebFetch, 2026-05-17) —
  M12 equity-vehicle context (out of scope for this draft)

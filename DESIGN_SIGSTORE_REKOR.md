# DESIGN: Sigstore + Rekor Release Signing for Sphragis

**Document version:** 1.0 (SP-BLD-005, 2026-05-16)
**Status:** Design lock; implementation is SP-BLD-005.IMPL.
**Companion docs:** `DESIGN_SLSA_PROVENANCE.md` (SLSA L4 framing — sigstore is the signing infrastructure), `DESIGN_LMS_KERNEL_SIGNING.md` (boot-time LMS signing for the kernel itself — sigstore is for release-distribution integrity).
**REQ:** Closes REQ-BLD-005 design portion.

## Why sigstore + Rekor (not just LMS)

Sphragis already has two crypto signing stories in flight:
1. **ML-DSA-87** for attestation quotes (`src/crypto/pq_cnsa.rs`, SP-C1.x)
2. **LMS** for boot-time kernel-image verification (`src/crypto/lms.rs`, SP-BLD-008 design)

Why ALSO sigstore? Because they cover DIFFERENT TRUST RELATIONSHIPS:

| Layer | Signs what? | Verified by whom? | Trust anchor |
|---|---|---|---|
| **ML-DSA-87 attestation** | Cave-quote envelopes at runtime | External verifier validating "this device is genuinely Sphragis" | Operator-CA endorsement chain (HSM-bound) |
| **LMS kernel signing** | The kernel image itself at release time | Bootloader verifying "the kernel binary I'm about to run is operator-approved" | Pubkey embedded in bootloader |
| **Sigstore + Rekor** | Release artifacts (binaries, SBOMs, provenance) at publication time | Operator downloading "is this kernel image actually from the legitimate Sphragis project?" | Sigstore Fulcio root + Rekor transparency log |

Sigstore is the **release-distribution layer** — the operator who downloads `sphragis-aarch64-unknown-none.bin` from GitHub Releases needs to verify it actually came from the legitimate Sphragis project and hasn't been swapped. They use the sigstore signature + Rekor transparency-log entry, not the LMS or ML-DSA signature.

After verifying via sigstore (release-distribution trust), the operator installs the kernel on a device. From then on, LMS (boot-time trust) and ML-DSA (runtime attestation trust) take over.

## The sigstore signing flow (release time)

```
GitHub Actions release workflow                 sigstore Fulcio CA           sigstore Rekor log
        │                                              │                              │
        │ 1. Build produces release artifacts:         │                              │
        │    - sphragis-aarch64-unknown-none.bin       │                              │
        │    - sphragis-aarch64-unknown-none.intoto.   │                              │
        │      jsonl (SLSA provenance)                 │                              │
        │    - sphragis-sbom.spdx.json                 │                              │
        │                                              │                              │
        │ 2. Workflow obtains GitHub OIDC token        │                              │
        │    identifying the workflow + repo + ref     │                              │
        │                                              │                              │
        │ 3. cosign sign-blob over each artifact:      │                              │
        │    cosign --identity-token=<oidc> \          │                              │
        │           sign-blob --bundle=...  artifact   │                              │
        │                                              │                              │
        │    cosign internally:                        │                              │
        │      a. ephemeral X.509 keypair generation   │                              │
        │      b. CSR ──── 4. issue cert ───────────▶ │                              │
        │                                              │                              │
        │ 5. ◀── 6. cert with workflow-identity ────── │                              │
        │         SAN (subject alternative name)       │                              │
        │                                              │                              │
        │ 7. Sign artifact-hash with ephemeral priv    │                              │
        │                                              │                              │
        │ 8. ──── 9. submit (sig, cert, artifact hash) to Rekor ────────────────────▶ │
        │                                                                              │
        │ 10. ◀── 11. signed-entry-timestamp ──────────────────────────────────────── │
        │                                                                              │
        │ 12. Bundle (sig + cert + Rekor entry) saved alongside artifact for          │
        │     publication to GitHub Releases. Ephemeral private key is destroyed.     │
        │                                                                              │
```

## The verification flow (operator at download time)

```
Operator preparing to install Sphragis
        │
        │ 1. Download from GitHub Releases:
        │    - sphragis.bin
        │    - sphragis.bin.sigstore  (the cosign bundle from step 12 above)
        │    - sphragis.bin.intoto.jsonl  (SLSA provenance, signed similarly)
        │
        │ 2. cosign verify-blob:
        │    cosign verify-blob \
        │        --bundle sphragis.bin.sigstore \
        │        --certificate-identity-regexp "^https://github.com/kadenlee1107/Sphragis/\.github/workflows/release\.yml@refs/tags/.*$" \
        │        --certificate-oidc-issuer https://token.actions.githubusercontent.com \
        │        sphragis.bin
        │
        │    cosign internally:
        │      a. Verify cert chains to pinned Fulcio root
        │      b. Verify cert SAN matches the workflow identity
        │      c. Verify Rekor signed-entry-timestamp is in the log
        │      d. Verify the signature over SHA-256(sphragis.bin)
        │
        │ 3. If all pass → trust the artifact. If any fail → reject.
        │
        │ 4. Optionally (SLSA verification):
        │    slsa-verifier verify-artifact \
        │        --provenance-path sphragis.bin.intoto.jsonl \
        │        --source-uri github.com/kadenlee1107/Sphragis \
        │        --source-tag vX.Y.Z \
        │        sphragis.bin
```

## Why ephemeral keys + transparency log

Sigstore's design rejects long-lived signing keys (the traditional PGP approach):

1. **Ephemeral keys**: generated per-signature, destroyed after signing. There's no signing key sitting on a developer's laptop to steal.
2. **Identity-bound certificates**: Fulcio issues a short-lived X.509 cert tying the public key to an OIDC identity (GitHub workflow + repo). The cert is what proves "this build came from this workflow."
3. **Rekor transparency log**: every signature is recorded in a public append-only log. An attacker who somehow signs a malicious artifact has to either:
   - Publish to Rekor (and immediately give themselves away to the operator's verification check), OR
   - Skip Rekor publication (and the operator's verification fails because no Rekor entry exists)

The transparency log is the load-bearing security property. It's what makes "we revoked the dev's signing key after they left" obsolete — there are no long-lived signing keys.

## Threat-model coverage

| Threat | Mitigation |
|---|---|
| Attacker steals a developer's GitHub credentials + pushes a malicious release | Fulcio binds the signature identity to the GitHub workflow path + repo. Operator's verification checks identity matches `kadenlee1107/Sphragis/.github/workflows/release.yml` — a non-workflow push wouldn't match. |
| Attacker compromises the GitHub Actions runner | Sigstore signature identifies the WORKFLOW, not the runner. Runner-level compromise doesn't change the cert SAN. The provenance (via SLSA) shows the workflow_sha; a swapped workflow shows up there. |
| Attacker swaps the artifact after publication | Operator verifies SHA-256(downloaded_artifact) matches the signed hash. Mismatch = reject. |
| Attacker submits a fake Rekor entry | Rekor is signed by the log's monitor identity; entries are append-only with Merkle-tree-anchored consistency proofs. A fake entry wouldn't have a valid signed-entry-timestamp from the genuine Rekor monitor. |
| Operator compromises their own pinned Fulcio root | Operator-side problem — distribute the pinned root via your existing trusted channel (operator OS package manager, internal CA). |
| Sigstore Fulcio/Rekor service compromise | Catastrophic for the ecosystem. Sigstore's threat model mitigates via multi-witness Rekor logs + open-source Fulcio code + the option for operators to run their own private instances. Sphragis can pin a private Rekor for enterprise gov deployments. |

## Implementation scope (SP-BLD-005.IMPL)

What .IMPL must land:

1. **GitHub Actions step in `release.yml`** (lands alongside SP-BLD-001.IMPL.A):
   - `cosign sign-blob --bundle=...` for each release artifact (binary, SBOM, in-toto provenance)
   - Outputs `<artifact>.sigstore` bundle file
2. **Operator-side verifier doc** in `tools/release-verifier/`:
   - Wraps `cosign verify-blob` with the Sphragis-specific identity claims
   - `slsa-verifier` integration for the provenance check
3. **Operator runbook section** in `docs/OPERATOR_RUNBOOK.md` (future SP-DOC-001):
   - Pre-install verification checklist
   - What to do on verification failure

What's deliberately NOT in SP-BLD-005.IMPL:
- Private Fulcio / Rekor for enterprise gov — separate SP if customer demands
- ML-DSA-87 sigstore — sigstore still uses ECDSA today (sigstore's PQ migration is a sigstore-side roadmap item; we'll inherit when available)
- Air-gapped signing — out of scope; sigstore requires internet to reach Fulcio + Rekor. Air-gapped deployments use the LMS chain (SP-BLD-008) for kernel verification + customer-side internal CA for artifact distribution.

## Open user actions

- **Pin sigstore Fulcio + Rekor roots** at verifier install time. Operator distributes via their trusted channel.
- **GitHub Actions OIDC enablement**: settings change in repo admin. One-time.
- **Decide on transparency-log policy**: public Rekor (default) vs private enterprise Rekor instance.

## REQ traceability

Closes REQ-BLD-005 (design portion). The IMPL closes the rest.

## References

- Sigstore overview: https://docs.sigstore.dev/
- Fulcio architecture: https://docs.sigstore.dev/fulcio/overview/
- Rekor overview: https://docs.sigstore.dev/rekor/overview/
- cosign CLI: https://github.com/sigstore/cosign
- GitHub Actions OIDC: https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/about-security-hardening-with-openid-connect
- SLSA v1.1 (companion design): `DESIGN_SLSA_PROVENANCE.md` (SP-BLD-001)

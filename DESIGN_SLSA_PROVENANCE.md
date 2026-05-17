# DESIGN: SLSA Level 4 Build Provenance for Sphragis

**Document version:** 1.0 (SP-BLD-001, 2026-05-16)
**Status:** Design lock; implementation is SP-BLD-001.IMPL.
**Companion docs:** `DESIGN_LMS_KERNEL_SIGNING.md` (boot-time signature verify), `docs/FIPS_140_3_MODULE_BOUNDARY.md` (§7.4 software/firmware security).
**REQ:** Closes REQ-BLD-001 design portion.

## What SLSA L4 demands

Supply-chain Levels for Software Artifacts (SLSA) v1.1 levels:

| Level | Build platform requirements | Provenance |
|---|---|---|
| L1 | Documented build process; provenance available | Build script + tool versions |
| L2 | Build runs on a service; auth required; signed provenance | + service identity attests to build |
| L3 | Hardened, isolated, ephemeral build env | + build user has no influence over build steps once initiated |
| **L4** | All of L3 + **two-party review** of every change to build config + reproducible builds | + hermetic + dependency provenance |

L4 is the maximum SLSA level. It requires:

1. **Two-party code review** for every change to the build configuration (CI workflows, build scripts, Cargo.toml dependency changes)
2. **Hermetic builds** — no network access during build; all dependencies pre-fetched and content-addressed
3. **Reproducible builds** — bit-for-bit identical artifacts from the same source on independent machines
4. **Dependency provenance** — every dependency itself has SLSA provenance (recursive)
5. **Signed provenance** — the provenance statement is signed by the build platform's identity

## Sphragis's path to L4

### Where we are today

| Requirement | Status |
|---|---|
| Documented build process | ✅ `cargo build --target aarch64-unknown-none --release` documented in README, contributing.md |
| Provenance available | ⚠️ `scripts/gen_sbom.py` produces SBOM; `scripts/build_intoto_attestation.py` produces in-toto statement |
| Build runs on service | ⚠️ Local dev today; CI runs cargo build on every PR (`.github/workflows/license-check.yml`) but doesn't release-publish |
| Signed provenance | ❌ Not signed |
| Hardened, isolated, ephemeral build env | ⚠️ GitHub Actions runners are ephemeral; "hardened" depends on actor permissions |
| Two-party code review | ⚠️ Today this is operator-side discipline (the maintainer + DCO sign-off review); GitHub branch protection rules would formalize |
| Hermetic builds | ❌ Cargo build with network-allowed |
| Reproducible builds | ⚠️ `scripts/check_reproducible_build.sh` exists (SP-B3 verified some artifacts); needs CI integration |
| Dependency provenance | ❌ Not recursive; depends on upstream crates' SLSA posture |

### The path to L4

| SP | What lands |
|---|---|
| SP-BLD-001.IMPL.A (this design's MVP) | GitHub Actions reusable workflow that produces signed in-toto attestations on every tag-push, using GitHub's OIDC + sigstore Fulcio. Targets SLSA **L3** initially. |
| SP-BLD-001.IMPL.B | Hermetic build mode: pre-vendor all deps via `cargo vendor`; CI uses `--offline` flag; build runs in network-isolated container. Targets SLSA **L3+hermetic**. |
| SP-BLD-001.IMPL.C | Reproducible-build verification gate: independent CI runner rebuilds + diff-checks against the primary build. Targets SLSA **L4** (full). |
| SP-BLD-001.IMPL.D | Branch-protection rules: require two approvals on any change to `.github/workflows/`, `Cargo.toml`, `Cargo.lock`, `deny.toml`, `audit.toml`. |
| SP-BLD-001.IMPL.E (out of scope for OS — depends on Cargo ecosystem) | Recursive dependency provenance — wait for crates.io to add SLSA support uniformly. |

## The signed-provenance artifact

Per SLSA v1.1 §schema, the provenance statement is an `in-toto` predicate:

```json
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [
    {
      "name": "sphragis-aarch64-unknown-none.bin",
      "digest": {
        "sha256": "<hex>",
        "sha384": "<hex>"
      }
    }
  ],
  "predicateType": "https://slsa.dev/provenance/v1.1",
  "predicate": {
    "buildDefinition": {
      "buildType": "https://github.com/sphragis/release-build@v1",
      "externalParameters": {
        "ref": "refs/tags/v0.1.0",
        "repository": "kadenlee1107/Sphragis"
      },
      "internalParameters": {
        "actor_id": "<GitHub user ID>",
        "workflow_path": ".github/workflows/release.yml",
        "workflow_sha": "<sha>"
      },
      "resolvedDependencies": [
        // Per-crate digests from Cargo.lock
        {"name": "aes", "version": "0.8.4", "digest": {"sha256": "<hex>"}}
        // ... ~150 crates
      ]
    },
    "runDetails": {
      "builder": {
        "id": "https://github.com/actions/runner@v2"
      },
      "metadata": {
        "invocationId": "<GitHub Actions run ID>",
        "startedOn": "2026-MM-DDTHH:MM:SSZ",
        "finishedOn": "2026-MM-DDTHH:MM:SSZ"
      },
      "byproducts": [
        {"name": "build-log", "digest": {"sha256": "<hex>"}}
      ]
    }
  }
}
```

Signed via sigstore (see `DESIGN_SIGSTORE_REKOR.md` companion design) with a Fulcio-issued certificate identifying the GitHub Actions workflow + identity that produced the build. The signature is recorded in Rekor (transparency log) so any attempt to swap the provenance is detectable.

## Verification flow

```
Verifier (operator preparing to install Sphragis)
        │
        │ 1. Download sphragis-aarch64-unknown-none.bin + sphragis-aarch64-unknown-none.bin.intoto.jsonl
        │
        │ 2. Verify the sigstore signature on .intoto.jsonl
        │    - Fulcio cert chains to Fulcio root (pinned)
        │    - Rekor entry exists for this signature
        │    - Signing identity matches: workflow=.github/workflows/release.yml + repo=kadenlee1107/Sphragis + ref=refs/tags/vX.Y.Z
        │
        │ 3. Verify the provenance subject matches the binary
        │    - SHA-256(binary) == provenance.subject[0].digest.sha256
        │
        │ 4. Verify the build platform claim
        │    - provenance.runDetails.builder.id == "https://github.com/actions/runner@v2"
        │
        │ 5. (For L4) Verify reproducibility
        │    - Re-fetch source at provenance.buildDefinition.externalParameters.ref
        │    - Re-build via the same workflow on an independent machine
        │    - Compare SHA-256(rebuilt_binary) == SHA-256(downloaded_binary)
        │
        │ 6. (For L4) Verify dependency provenance
        │    - For each provenance.buildDefinition.resolvedDependencies[]:
        │      - Confirm that crate's own SLSA provenance (if available)
        │      - Confirm crate digest matches Cargo.lock entry
```

## Threat-model coverage

| Threat | Mitigation |
|---|---|
| Attacker modifies CI workflow to substitute a malicious build | Two-party code review (L4 requirement); signed-provenance fingerprint includes workflow_sha |
| Attacker compromises a GitHub Actions runner | Ephemeral runners + minimal scope; provenance signed by Fulcio with workflow identity, not runner identity |
| Attacker swaps the provenance after publication | Sigstore Rekor transparency log — any swap is detectable by comparing on-disk provenance to Rekor entry |
| Attacker compromises a build dependency | Recursive dependency provenance (L4 ideal); cargo-deny + cargo-audit policy gates flag known-bad deps in CI |
| Attacker publishes a malicious version of the Sphragis kernel | Verifier check #2 catches: signature identity doesn't match the legitimate workflow |

## Implementation scope (SP-BLD-001.IMPL)

What .IMPL.A must land:

1. `.github/workflows/release.yml` — GitHub Actions workflow triggered on tag-push. Steps:
   - Checkout
   - Build (`cargo build --release --target aarch64-unknown-none`)
   - Generate in-toto provenance via `scripts/build_intoto_attestation.py` (existing — extend to SLSA v1.1 schema)
   - Sign via sigstore cosign (`cosign sign-blob --output-signature=...` against the binary; OIDC identity from GitHub Actions)
   - Upload binary + .intoto.jsonl to GitHub Releases
2. Verifier script `tools/slsa-verifier/verify.sh` — wraps `slsa-verifier` CLI tool with Sphragis-specific identity claims (repository, workflow path).

Out of scope for .IMPL.A (each is its own follow-on):
- B (hermetic), C (reproducible-build CI gate), D (branch protection rules), E (recursive dep provenance).

## Open user actions

- **Set up GitHub branch protection** (.IMPL.D): repo admin configures Required-Approvals + Required-Status-Checks on `main`. Cannot be done from a non-admin PR.
- **Sigstore Fulcio root pinning**: distribute the Fulcio root cert chain to deployment-side verifiers (operator's package-distribution pipeline).
- **GitHub Actions OIDC setup**: enable OIDC for the repo so sigstore can verify the runner identity. Repo settings change.

## REQ traceability

Closes REQ-BLD-001 (design portion). The IMPL closes the rest.

## References

- SLSA v1.1: https://slsa.dev/spec/v1.1/
- in-toto: https://in-toto.io/
- sigstore: https://sigstore.dev/
- Rekor: https://docs.sigstore.dev/rekor/overview/
- GitHub Actions OIDC: https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/about-security-hardening-with-openid-connect
- slsa-verifier: https://github.com/slsa-framework/slsa-verifier

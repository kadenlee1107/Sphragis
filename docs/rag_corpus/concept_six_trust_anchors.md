---
type: concept-note
topic: crypto · pki
---

# Six trust anchors, no system trust store

> Most operating systems ship a Mozilla CA bundle (~150 root certificates) as their trust store. Sphragis ships exactly six. This note is why that's a feature.

## The set, in source

[[_generated/src/net/x509.rs]]'s `TRUST_STORE` is a `&[&[u8]]` of six DER-encoded CA roots, embedded via `include_bytes!`:

| # | CA root | Algorithm | Anchors |
|---|---|---|---|
| 1 | **ISRG Root X1** | RSA 4096 | Let's Encrypt's primary root. Cloudflare-fronted sites, GitHub Pages, basically every site that auto-issues with LE. |
| 2 | **ISRG Root X2** | ECDSA P-384 | Let's Encrypt's modern ECDSA root. Sites that opted into ECDSA leaf certs. |
| 3 | **Amazon Root CA 1** | RSA 2048 | AWS-hosted services, anything fronted by Amazon.com. |
| 4 | **DigiCert Global Root CA** | RSA 2048 | A large fraction of enterprise + financial sites. |
| 5 | **DigiCert Global Root G2** | RSA 2048 | DigiCert's modern root. Anchors Google's intermediate CA chain among others. |
| 6 | **GTS Root R4** | ECDSA P-384 | Google Trust Services' modern ECDSA root. Required for `pq.cloudflareresearch.com` (used by our PQ-interop smoke). |

Six. Adding a seventh requires recompiling the kernel image.

## Why not the Mozilla bundle

The Mozilla CA bundle is a community-curated list of ~150 roots that browsers trust by default. It's the de-facto standard for "sites you can validate without prompting the user." Most every Linux distro ships it; Apple has its own equivalent.

Sphragis doesn't, for a deliberate set of reasons:

1. **Smaller list, easier to audit.** Six certs you can list on one screen and re-derive trust for. 150 certs you take on faith from whoever curates the bundle.
2. **Harder to subvert.** Adding a CA to the kernel image requires editing source and recompiling. Adding a CA to a system trust store on Linux is `cp foo.crt /usr/local/share/ca-certificates/ && update-ca-certificates` — privileged, but a smaller surface to compromise.
3. **No bundle-version question.** "Whose Mozilla CA bundle version are we on?" is a question with multiple wrong answers. With six embedded roots, the question is "is the kernel image that's running the one we built?" — same question we already answer for everything else.
4. **Smaller blast radius for CA misissuance.** If one of the 150 roots in a Mozilla bundle gets caught misissuing certs (this happens — Symantec lost their roots in 2018, the Trustwave story, the WoSign story), you have a kernel that trusts them by default until you update. With six hand-picked roots, the surface for "we trust this CA when we shouldn't have" is small.

The tradeoff: some sites won't validate. A Sphragis cave trying to reach a site whose chain ends at, say, `Sectigo` will get `UntrustedRoot`. That's a feature, not a bug — the cave's policy presumably names which hosts it's allowed to talk to ([[Concepts/Cave Isolation Model]]), and adding a host whose chain we don't anchor means making a deliberate decision to add the CA.

## How a chain reaches an anchor

[[_generated/src/net/x509.rs]]'s `verify_chain` accepts three paths to a trust anchor (a / b / c in the source comments):

**(a)** The current cert (the topmost cert the server sent) **IS** an anchor — exact-bytes equality with one of the six. Uncommon but legal: some servers ship their root in the chain.

**(b)** The current cert and an anchor share the same `SubjectPublicKey` — a **cross-signed root**. Common: GTS Root R4 was once cross-signed by GlobalSign; the chain ships the cross-signed cert, but the public key is the same as the anchor we ship. Equality on bytes fails; equality on SPKI succeeds.

**(c)** The current cert is **signed by** an anchor — the typical real-world case. The server sends `[leaf, intermediate]` and stops short of the root because RFC 5246 says clients should have the root locally. Verifies by treating the anchor as a virtual parent and re-running the signature check.

Pre-PR-#10 (`phase2-verifier`), only (a) and (b) were implemented. Chains from Let's Encrypt-anchored sites that don't ship the root failed at "untrusted root" even though the chain was structurally valid. PR #10 added (c).

## Why this list specifically

These six were chosen by working through the smokes the project actually uses and noting which CAs anchored them:

- **ISRG X1 + X2** — Let's Encrypt issues most of public HTTPS today. If you only had these two, you'd validate most of the internet.
- **GTS R4** — required for `pq.cloudflareresearch.com`, which is the project's PQ-TLS interop test endpoint. Hard requirement.
- **Amazon Root CA 1** — AWS-hosted endpoints (S3, CloudFront) sit under this. Anything that might end up serving an evaluation build's hash file.
- **DigiCert CA / G2** — enterprise / financial / government endpoints. Procurement audience may need to point caves at these.

The six covers most of what's likely to actually be reached. It's not a research-grade trust store; it's a hand-picked one for this project's actual workflows.

## The signature-algorithm coverage that follows

Because these six roots are a mix of RSA-2048, RSA-4096, and ECDSA P-384, the verifier needs to support:

- **For cert chain signatures**: ECDSA-P256/P384, RSA-PKCS1v15 (SHA-256/384/512), RSA-PSS — every self-signature on a root above plus the chains they typically anchor.
- **For TLS-1.3 CertificateVerify**: ECDSA-P256, ECDSA-P384, RSA-PSS (SHA-256/384/512). PKCS#1v1.5 is **not valid** for CertVerify per RFC 8446 §4.4.3 — it's only for cert chain sigs.

This is documented as "Signature algorithm coverage" inline in [[_generated/src/net/x509.rs]]. If a future seventh anchor uses an algorithm not in the list, the verifier will need extending — adding a new root is not just "drop a DER and recompile."

## The refresh procedure

Each CA publishes its root via a stable URL (also documented inline):

| Root | Source |
|---|---|
| ISRG X1 | letsencrypt.org/certs/isrgrootx1.der |
| ISRG X2 | letsencrypt.org/certs/isrg-root-x2.der |
| Amazon CA 1 | amazontrust.com/repository/AmazonRootCA1.cer |
| DigiCert Global CA | cacerts.digicert.com/DigiCertGlobalRootCA.crt |
| DigiCert Global G2 | cacerts.digicert.com/DigiCertGlobalRootG2.crt |
| GTS Root R4 | i.pki.goog/r4.crt |

Re-fetch, drop into [[_generated/src/net]] under `ca_certs/`, rebuild. The DER bytes are versioned in git so a refresh is a visible diff.

## Open

- **A full Mozilla CA bundle (~150 roots) is a follow-up STUMP.** This six-entry set is enough to verify the most common chains and move the audit's "TLS authentication is theater" verdict; a full bundle is a separate scope question.
- **Bring-your-own-CA** is explicitly NOT supported. The trust store is hard-coded. If an operator needs to anchor a custom CA, they edit source — no runtime configuration path exists.
- **Revocation** (OCSP / CRL) is the operator's job. The kernel doesn't fetch revocation status for any anchor, which means if one of the six is later compromised, the operator is responsible for noticing and rebuilding.

## Related

- [[Concepts/Cryptography Stack]] — full primitive catalog this fits inside
- [[Concepts/TLS Hardening Journey]] — how chain validation became real (V4 onward)
- [[_generated/src/net/x509.rs]] — `TRUST_STORE`, the three accept paths, the algorithm coverage notes

# DESIGN: HSM-Backed Operator-CA for Sphragis Attestation

**Document version:** 1.0 (SP-C1.6, 2026-05-16)
**Status:** Design lock; PKCS#11 client implementation is SP-C1.6.IMPL.
**Companion docs:** `src/security/attest.rs` (SP-C1.1/C1.2/C1.3 — the API surface this design feeds), `docs/FIPS_140_3_MODULE_BOUNDARY.md` (§7.8 CSP table where the operator-CA key sits).
**REQ:** Closes REQ-ATT-006 design portion.

## Why this document exists

The SP-C1.1 attestation surface is in place: caves can produce ML-DSA-87-signed Quotes attesting to their identity + the kernel measurement. Today's attestation key (`ATTEST_KEY` in `src/security/attest.rs`) is generated at first use and lives in RAM. That's fine for development; it's NOT fine for any production gov deployment, because:

1. A kernel exploit that reads RAM extracts the attestation key and forges quotes.
2. Without an external "endorsement chain," a verifier has no way to distinguish "this is the genuine Sphragis attestation key for this device" from "an attacker generated their own key."

The HSM-backed operator-CA pattern fixes both. This document specifies how.

## The four-actor model

| Actor | Role | Key material held |
|---|---|---|
| **Operator CA** (offline signing host) | Issues endorsement certificates for each Sphragis device's attestation key | Operator-CA root **private** key (HSM-bound) |
| **Sphragis device** | Holds a per-device attestation keypair; presents Quotes signed by it | Per-device attestation **private** key (in-kernel — SP-C1.4/1.5 future moves to SEP/Caliptra-bound) |
| **HSM** (YubiHSM 2, AWS CloudHSM, Azure Dedicated HSM, on-prem Thales Luna, etc.) | Cryptographic boundary holding the operator-CA private key. Never releases the key; signs over PKCS#11 / KMIP request | Operator-CA root **private** key |
| **External verifier** | Validates a Quote by chaining the per-device attestation pubkey through the operator-CA endorsement to a trusted operator-CA root | Operator-CA root **public** key (pinned) |

## Provisioning flow (one-time per device)

```
device                    operator-CA host                  HSM
  │                                │                          │
  │  1. Generate per-device         │                          │
  │     attestation keypair          │                          │
  │     (kernel calls Dsa87Key::    │                          │
  │      generate)                  │                          │
  │                                 │                          │
  │ ───── 2. Send pubkey + ────▶                              │
  │       device measurement                                   │
  │       + device serial                                      │
  │       over operator-authenticated channel                  │
  │                                 │                          │
  │                                 │  3. Construct endorsement│
  │                                 │     statement (CBOR):    │
  │                                 │       device-pubkey,     │
  │                                 │       device-meas,       │
  │                                 │       serial,            │
  │                                 │       issue-time,        │
  │                                 │       validity-window    │
  │                                 │                          │
  │                                 │ ─── 4. PKCS#11 C_Sign ──▶│
  │                                 │      over endorsement    │
  │                                 │      bytes               │
  │                                 │                          │
  │                                 │ ◀─ 5. ML-DSA-87 signature│
  │                                 │      (HSM never released │
  │                                 │      private key)        │
  │                                 │                          │
  │ ◀──── 6. Endorsement cert ────  │                          │
  │       (statement || sig)        │                          │
  │                                 │                          │
  │  7. Store in BatFS as           │                          │
  │     /attest/endorsement.cbor    │                          │
  │     for inclusion in Quotes     │                          │
```

## Quote-time inclusion of the endorsement

Once provisioned, every Quote produced by `attest::quote(...)` includes the endorsement bytes alongside the device-side signature. The verifier checks BOTH signatures:

```rust
struct Quote {
    // Existing SP-C1.1 fields...
    pub kernel_measurement: KernelMeasurement,
    pub cave_identity:      CaveIdentity,
    pub nonce:              [u8; NONCE_LEN],
    pub claims:             Claims,
    pub signature:          Vec<u8>,  // ML-DSA-87 over signed_payload
    pub verifying_key:      Vec<u8>,  // per-device pubkey

    // SP-C1.6 additions:
    pub endorsement:        Vec<u8>,  // operator-CA-signed endorsement cert
}
```

Verifier flow:

1. Parse the endorsement cert. Confirm it's signed by the trusted operator-CA root (which the verifier holds pinned).
2. Extract the device-pubkey + device-meas + validity-window from the endorsement.
3. Confirm `Quote.verifying_key` matches the endorsement's device-pubkey.
4. Confirm `Quote.kernel_measurement` matches the endorsement's device-meas (allows operator-controlled list of approved measurements — see "Approved-measurement registry" below).
5. Confirm current time is within the endorsement's validity-window.
6. Verify `Quote.signature` over the canonical signed_payload using `Quote.verifying_key`.

A Quote without a valid endorsement is rejected — that's the property that defeats the "rogue attacker generates their own attestation key" attack.

## HSM API surface

The operator-CA host runs the signing on-demand. It needs:

- **PKCS#11** as the standard interface (any HSM that speaks PKCS#11 works — YubiHSM 2 v2.4+, AWS CloudHSM, Azure Dedicated HSM, Thales Luna 7).
- **KMIP 2.x** as a secondary interface for HSMs that don't expose PKCS#11 directly.
- The library on the operator-CA host: `pkcs11-rs` (Apache-2.0) or vendor-specific bindings.

For ML-DSA-87: PKCS#11 v3.1 (Sep 2024) added ML-DSA OIDs to the mechanism list. Older HSMs may not support ML-DSA natively — operator falls back to a software ML-DSA implementation that uses the HSM only for KEY WRAP / UNWRAP of a master secret that derives the ML-DSA key on-host. Less ideal but still gives HSM-bound root.

## Approved-measurement registry

The endorsement names a SPECIFIC kernel measurement. When the device boots a new kernel (e.g., after an update), the measurement changes — the endorsement no longer matches.

Two approaches:

**Approach A (strict):** endorsement names ONE measurement. Re-provisioning required after every kernel update. Highest security, highest operational overhead.

**Approach B (registry):** endorsement names a "measurement-registry-pubkey" instead of a specific measurement. The registry is a separately-signed list of allowed measurements; the verifier checks the Quote's kernel_measurement against the registry. The registry is updated by the operator on every kernel release. Slightly more complex but matches how real organizations operate.

Sphragis ships with Approach A as the default; gov customers can opt into Approach B by deploying a measurement-registry-signer alongside the operator-CA.

## Audit-trail

Every endorsement issuance and every endorsement use is auditable:

- **Operator-CA side**: HSM logs every C_Sign call (operator + timestamp + pubkey signed over). HSM-side log is the authoritative record.
- **Device side**: `attest::quote(...)` emits a `Category::Attest` audit record (SP-AUD-003.1 — already landed). The log shows "quote produced" with the cave_id; cross-reference with the endorsement's device-id to confirm it's the right device.

## Threat-model coverage

| Threat | Mitigation |
|---|---|
| Attacker generates own attestation key | Endorsement check fails — attacker's key isn't in any operator-CA-signed endorsement |
| Attacker steals device's attestation key (kernel exploit) | Still bound to a specific (device-id, kernel-measurement) by the endorsement; can't be used for OTHER devices or OTHER kernel versions. SP-C1.4 (SEP) / SP-C1.5 (Caliptra) close the kernel-exploit gap by moving the key out of RAM. |
| Operator-CA root key compromise | Catastrophic — attacker can issue endorsements for arbitrary keys. Mitigated by HSM-binding the operator-CA root: extraction requires physical HSM theft + tamper-detection bypass. Operator should rotate the root periodically (e.g., yearly) and publish revocation. |
| Endorsement replay | Endorsement carries a validity-window. Verifier checks current time against the window. Operator can revoke compromised endorsements out-of-band. |
| Verifier doesn't have operator-CA pubkey | Pinned at verifier install time. Distribution is the operator's problem (out of band). |

## Implementation scope (SP-C1.6.IMPL)

What SP-C1.6.IMPL must land:

1. **Sphragis-side endorsement storage**: load `/attest/endorsement.cbor` from BatFS at boot; cache parsed endorsement; include in every Quote.
2. **Sphragis-side endorsement field in Quote struct**: add `endorsement: Vec<u8>` field; bump Quote wire format version.
3. **Operator-CA host script**: Python or Rust tool that takes (device-pubkey, device-meas, serial), constructs the endorsement CBOR, calls HSM via PKCS#11 for signing, writes endorsement out as a file the operator copies to the device.
4. **External verifier**: extend `tools/audit-verifier/` (or new `tools/attest-verifier/`) to validate the endorsement chain.

What's deliberately NOT in SP-C1.6.IMPL:

- ML-DSA-87 native HSM support (Approach B fallback if HSM doesn't speak ML-DSA — separate SP)
- Measurement-registry-signer for Approach B — separate SP
- HSM-side key generation (operator does this once, out of band)
- Operator-CA root revocation infrastructure (separate SP)

## REQ traceability

Closes REQ-ATT-006 (design portion) — design lock. The IMPL closes the rest.

## References

- PKCS#11 v3.1: https://docs.oasis-open.org/pkcs11/pkcs11-base/v3.1/pkcs11-base-v3.1.html
- YubiHSM 2 ML-DSA support: https://developers.yubico.com/YubiHSM2/
- AWS CloudHSM: https://docs.aws.amazon.com/cloudhsm/
- FIPS 203 (ML-KEM): https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.203.pdf
- FIPS 204 (ML-DSA): https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.204.pdf
- IETF RATS RFC 9334 (Remote Attestation Procedures): https://www.rfc-editor.org/rfc/rfc9334

# Sphragis — Cryptographic Architecture

**Status:** design document, reflects reality as of 2026-04-22.
**Owner:** Kaden + the Claudes.
**Update discipline:** when a primitive lands or changes, update the
`Current state` column the same commit. When a new use case appears,
add a row.

## Philosophy

> Every crypto choice is a **match between threat model and primitive**.
> No single cipher fits every use case. OTP is unbreakable for 32-byte
> secrets you can pre-distribute, useless for 1 TB filesystems. AES-GCM
> is ideal for network records, wrong for disk blocks. The goal is to
> always use the primitive that's *actually* best for *this specific
> threat model*, and to be honest when we haven't yet.

## Use-case matrix

Legend:
- ✅ = live and correct
- 🟡 = live but using a weaker primitive than ideal
- 🔒 = design decision, intentional tradeoff
- ❌ = gap, tracked for implementation
- 🧭 = post-quantum consideration

---

### 1. Passphrase → master key derivation

**Threat:** operator's passphrase is a relatively low-entropy secret.
Attacker who exfiltrates the on-disk KDF state (kernel image, device
salt) can run offline brute-force / dictionary attacks on GPUs/ASICs.

| Aspect | Value |
|---|---|
| Best primitive | **Argon2id**, m=256 MiB, t=3, p=1 |
| Why | Memory-hard → GPU/ASIC cost is ~linear in memory. Best KDF since 2015 (winner of PHC). NIST SP 800-132 recommends it. |
| Current state | ✅ Argon2id at 8 MiB × 3 passes × 1 lane in BOTH the auth-gate KDF (`security/auth.rs::kdf`, salt `sphragis-auth-v2`) AND the BatFS master KDF (`main.rs::derive_batfs_key`, salt `sphragis-batfs-v3`, STUMP #138). Per-cave `fs_key` is HMAC-SHA256 keyed by the BatFS master (STUMP #111 audit C011) — cheap because it's not a passphrase derivation. |
| Gap | The 256 MiB target is unreachable in our 32 MB kernel heap. Bumping the heap is a separate STUMP. 8 MiB still moves the bar by ~6 orders of magnitude vs the pre-#138 16-round SHA. Argon2id 256 MiB once we have heap space; "Phase B" target. |
| PQ | N/A — symmetric, 256-bit output. |

---

### 2. Filesystem encryption (BatFS files)

**Threat:** device theft, cold-boot attack, disk-image exfiltration.
Adversary has the raw encrypted blocks and wants to read / tamper.

| Aspect | Value |
|---|---|
| Best primitive | **AES-256-GCM-SIV** per file (nonce-misuse resistant) OR **XChaCha20-Poly1305** (longer nonce, no wraparound risk) |
| Why | AEAD → confidentiality + authentication in one pass. GCM-SIV survives nonce-reuse (CTR mode does not). XChaCha20 with 192-bit nonces never reuses in practice. |
| Current state | ✅ ChaCha20-Poly1305 AEAD per file. Per-file key via `sha256::derive_key(MASTER_KEY, filename)`. AAD = filename so an attacker can't rename ciphertext to a different slot. Tag stored in the on-disk inode (since STUMP #136). _(Doc-drift caught by STUMP #144 audit; previous text said AES-256-CTR + HMAC, which described an even-earlier construction.)_ |
| Gap | None at this layer. Memory-encryption + a per-file additional-data binding to the BatFS inode is a future hardening pass. |
| PQ | Grover attack → effective 128-bit security. Still fine for 2040+. |

---

### 3. Disk-block random access (future: BatFS block device)

**Threat:** same as above but we need to read/write 4 KB blocks without
rewriting the whole file.

| Aspect | Value |
|---|---|
| Best primitive | **AES-256-XTS** |
| Why | Industry standard for disk encryption (LUKS, FileVault, BitLocker). Block-number doubles as tweak; no nonce needed; no integrity but we do it at the filesystem layer separately. |
| Current state | N/A — BatFS is per-file today, not block-based. |
| Gap | If/when we build a real block device abstraction. Not a priority. |

---

### 4. Cave audit log encryption (at rest)

**Threat:** daemon runs on the Mac host; Mac disk could be imaged. The
audit log contains per-cave exec history — sensitive (what tools were
run, what targets, what stdout/stderr said).

| Aspect | Value |
|---|---|
| Best primitive | **ChaCha20-Poly1305** with counter nonce, or AES-256-GCM with counter nonce. Merkle-chain framing for tamper-evidence. |
| Why | AEAD detects tampering, counter nonce prevents reuse, chain detects truncation. |
| Current state | ✅ ChaCha20-Poly1305 with prev-tag chained as AAD (`batcaved.py:386-444`). Confidentiality + integrity + tamper-evident truncation. _(STUMP #144 audit: previous text claimed AES-256-CTR; the daemon was already on AEAD with chained AAD.)_ |
| Gap | None at this layer. Future: replicate to a remote witness for non-repudiation. |
| PQ | Same as #2 — 128-bit Grover margin is ample. |

---

### 5. File integrity MAC (BatFS content verification)

**Threat:** attacker flips bits in ciphertext hoping we decrypt to
attacker-chosen bytes (malleability). HMAC makes this detectable.

| Aspect | Value |
|---|---|
| Best primitive | **Poly1305** (keyed) — 128-bit tag, proven security, very fast |
| Why | Poly1305 is one-time-MAC, paired with ChaCha20 it forms the AEAD. We'd drop HMAC-SHA256 once we move to AEAD. |
| Current state | ✅ HMAC-SHA256 in `fs/batfs.rs::compute_file_mac`. Correct, just heavier than Poly1305. |
| Gap | Folded into gap #2 above (when we adopt AEAD, the separate MAC goes away). |

---

### 6. Transport Layer Security (outbound TCP → HTTPS, DoH, etc.)

**Threat:** wire-tapping, MITM, downgrade attacks, replay.

| Aspect | Value |
|---|---|
| Best primitive | **TLS 1.3** with **X25519** key agreement + **ChaCha20-Poly1305 or AES-256-GCM** record encryption + **Ed25519 or ECDSA-P256** certificate signatures. |
| Why | TLS 1.3 removed all the legacy foot-guns (no CBC, no RC4, no SHA-1, no RSA key exchange, no compression). Mandatory forward secrecy. |
| Current state | ✅ `src/net/tls.rs` — our own TLS implementation doing TLS 1.3 records with AES-256-GCM, X25519 for ECDHE, Ed25519 + P-256 for certs. X.509 chain validation landed (see security/PENTEST_V4_FIX_SUMMARY.md). |
| Gap | 🧭 Add **hybrid key exchange** (X25519 + ML-KEM-768) for post-quantum forward secrecy. Data captured today could be decrypted by a future CRQC against stored X25519 public keys. Mandatory for high-value long-term traffic. |
| PQ | **This is where PQ matters most.** Captured-now-decrypt-later threat is real for long-lived ciphertext. |

---

### 7. Digital signatures (initrd blob, Chromium blob, code signing)

**Threat:** supply-chain attack swaps a Sphragis-shipped binary for a
malicious one.

| Aspect | Value |
|---|---|
| Best primitive | **Ed25519** for now + **ML-DSA-65 (Dilithium-3)** for PQ hybrid |
| Why | Ed25519 — small (64 B sig), fast, deterministic (no RNG-failure foot-gun), no parameter pitfalls. ML-DSA NIST-standardized 2024 as the PQ replacement. |
| Current state | ✅ `ed25519-compact` for initrd + Chromium blob signature check. X.509 chain verify uses p256/ECDSA. |
| Gap | 🧭 Hybrid sig for long-term-trust artifacts (e.g., signed kernel releases). Less urgent than TLS key-exchange because forgery requires a CRQC today, not a stored-ciphertext attack. |
| PQ | Shor's algorithm breaks Ed25519 under CRQC. PQ needed for durable trust roots. |

---

### 8. Hashing (content addressing, Merkle tree, checksums)

**Threat:** collision attacks (chosen-prefix), preimage for integrity.

| Aspect | Value |
|---|---|
| Best primitive | **BLAKE3** (faster, tree-hashable, parallel) OR **SHA-256** for compat |
| Why | BLAKE3 is 2–3× faster than SHA-256 on modern CPUs, tree structure means it parallelizes and supports incremental verification. SHA-256 is still secure; decision is performance vs universality. |
| Current state | ✅ SHA-256 via `sha2` crate everywhere (MerkleTree, file hash, KDF input). |
| Gap | Optional: swap to BLAKE3 if perf matters. Not a security upgrade. |
| PQ | 256-bit → Grover-128. Fine. |

---

### 9. Random number generation

**Threat:** predictable nonces, weak ISN, low-entropy keys.

| Aspect | Value |
|---|---|
| Best primitive | Hardware RNG (ARMv8.5 RNDR) seeded into a CSPRNG (ChaCha20-based DRBG) |
| Why | Hardware RNG gives true entropy; CSPRNG stretches it to unlimited output; resistant to RNG prediction attacks even if HW RNG has microstructure. |
| Current state | ✅ `src/crypto/rng.rs` — ARMv8.5 RNDR mixed with boot-time entropy (cntpct_el0, boot cookie). Boot log confirms `[rng] ARMv8.5 RNDR available — mixing HW entropy`. |
| Gap | Verify we NEVER block on RNG during userspace syscalls (affects getrandom latency). Otherwise solid. |
| PQ | N/A. |

---

### 10. Session keys / ephemeral key agreement (in-kernel IPC auth)

**Threat:** cross-cave impersonation, stolen IPC credentials.

| Aspect | Value |
|---|---|
| Best primitive | **X25519** for key agreement + **BLAKE3-keyed** or **HKDF-SHA-256** to derive session keys + **Noise protocol framework** as the session construction |
| Why | X25519 is tiny + fast + safe. Noise (used by WireGuard, Signal) is the best practice for building session-authenticated channels from primitives. |
| Current state | ❌ No per-cave keypairs today. IPC is via `batpipe` with no authentication. |
| Gap | Each Cave gets an Ed25519 identity + X25519 ephemeral on `caves enter`; IPC establishes a Noise-XX session. Bigger project; tracked. |
| PQ | 🧭 Hybrid X25519 + ML-KEM for post-quantum. |

---

### 11. Panic / duress / deadman emergency tokens

**Threat:** attacker coerces operator to unlock; operator wants a
distinguishable "wipe everything" code.

| Aspect | Value |
|---|---|
| Best primitive | **One-time pad (OTP)** — legitimately! Plus **HMAC-SHA-256** authentication of the token. |
| Why | Token is short (32 bytes), pre-distributable (print 10 on paper, tear off as used), never reused. Exactly the niche OTP was designed for. Combined with HMAC the signal is unforgeable by anyone without the pad. |
| Current state | ✅ `security/otp.rs` ships: 8 duress tokens + 24 deadman tokens, RNDR-seeded, single-use enforced (zeroed on consume), constant-time scan (`otp.rs:130-173`). Discovered by STUMP #144 audit — earlier "🟡" entry was stale. |
| Gap | Pre-distribution / paper-backup workflow not yet built — the strip lives in RAM only. Future STUMP: print QR strip on first-boot for offline backup. |
| PQ | N/A — single-use info-theoretically secure. |

---

### 12. Deadman proof-of-life (durability across offline periods)

**Threat:** operator is forced offline; automated deadman triggers
before operator can re-authenticate. Operator wants pre-generated
"proof of life" tokens they can send from another channel.

| Aspect | Value |
|---|---|
| Best primitive | **OTP-backed HMAC tokens** (operator keeps a small strip, sends one per day via offline channel — phone SMS, email, satellite) |
| Why | Same argument as #11: short secrets, single-use, pre-distributable. OTP's info-theoretic security means even if the token is intercepted it can't forge future tokens. |
| Current state | ✅ The 24 deadman-arm tokens in `security/otp.rs` cover this — same OTP infrastructure as #11. STUMP #144 audit caught this entry as stale. |
| Gap | Same as #11: paper / out-of-band distribution flow not built yet. Operator currently can't take a strip with them on travel. |

---

### 13. Inter-Cave IPC authentication

**Threat:** compromised cave tries to impersonate another cave during
IPC (read its files via `batpipe`, etc.).

| Aspect | Value |
|---|---|
| Best primitive | Mutual Ed25519 authentication + Noise-XX session |
| Why | Each cave has a long-lived Ed25519 identity + ephemeral X25519; Noise-XX gives mutual auth + forward-secret session keys in 1.5 round trips. |
| Current state | ❌ batpipe is unauthenticated. |
| Gap | Covered in #10. |

---

## Implementation priority (by impact × effort)

1. **#1 Argon2id KDF** — **highest security ROI**. Replaces 16-round SHA-256 which is a known weak point. The `argon2` crate exists for no_std. Probably 1–2 hours of work.

2. **#11 + #12 OTP duress/deadman tokens** — real and novel use of OTP that matches Kaden's instinct. Medium effort. Genuine security upgrade for the most sensitive codepath (the one that *wipes the whole system*).

3. **#4 Audit-log AEAD + Merkle chain** — finishes Phase 3's story. Moves from "confidential" to "confidential + integrity + tamper-evident". 1–2 hours.

4. **#2 BatFS → ChaCha20-Poly1305** — cleans up the file-encryption path. Mostly a migration, not new crypto. 2–3 hours.

5. **#6 TLS post-quantum hybrid (X25519+ML-KEM)** — biggest *future* protection for long-term data. Real work (need a PQ crate) but significant. Half a day.

6. **#7 Ed25519 → hybrid ML-DSA** — signatures for durable trust roots. Similar effort to #6.

7. **#10 + #13 Noise-session IPC auth** — enables secure inter-cave comms. Bigger project (days).

## What this doc is NOT

- Not a promise that all 13 land this week.
- Not a claim that current crypto is broken — it's a claim that for
  each use case there's a best-in-class primitive, and we should
  converge on those deliberately rather than accidentally.
- Not a replacement for the pentest rigor in `security/PENTEST_*.md`;
  it's orthogonal: that doc audits what's there, this doc says what
  *should* be there.

## References

- NIST SP 800-131A (algorithm transitions)
- NIST FIPS 203 (ML-KEM / Kyber)
- NIST FIPS 204 (ML-DSA / Dilithium)
- RFC 8439 (ChaCha20-Poly1305)
- RFC 9420 (Argon2)
- Password Hashing Competition (2015) winner: Argon2
- The Noise Protocol Framework (noiseprotocol.org)
- BLAKE3 paper + reference impl (github.com/BLAKE3-team/BLAKE3)

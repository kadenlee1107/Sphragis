# Sphragis Receipts

Concrete evidence for every claim in the [README](../README.md). One section per claim. For each: the commit(s) that landed it, the headless QEMU selftest that exercises it, and what passing the selftest actually proves.

If a stranger reads the README and asks "is this real or marketing copy" — this is the answer. Every claim below is reproducible by anyone with `qemu-system-aarch64` and a Rust nightly toolchain.

> Disclosure-tier note: this doc respects the same tiers as the README. Mechanism-level detail appears for Tier 1 primitives. Tier 2 items are described by concept + property tested, not byte layout. Tier 3 items appear by name + selftest only — anyone who wants the design detail can run the code, but we don't volunteer it in prose.

---

## 1. Hardware bring-up (Apple M4)

**Claim:** Sphragis is the first known non-Apple OS booted on Apple M4 hardware (Mac16,1 / J604 / T8132 "Donan").

- **Evidence:** [`docs/photos/2026-04-17_first_m4_boot/`](photos/2026-04-17_first_m4_boot/) — sixteen photos with [`INDEX.md`](photos/2026-04-17_first_m4_boot/INDEX.md) describing each one. Reaches an interactive shell with status bar. ADT discovery, DWC3 USB-3 controller bring-up, ATC PHY tunables, dockchannel UART — all confirmed running on real silicon.
- **Reproducibility:** The boot path is documented in [`UBUNTU_QUICKSTART.md`](../UBUNTU_QUICKSTART.md). The chainload uses the m1n1 proxyclient with our pre-patched [`external/m1n1/proxyclient/tools/chainload.py`](../external/m1n1/proxyclient/tools/chainload.py) (carries the `--skip-secondary-cpus` flag that M4's P-cluster needs to avoid an RVBAR SError).
- **Note on disclosure:** The hardware reverse-engineering specifics (PMGR sequences, ATC PHY tunable map, AIC2 base, dockchannel address) live in `docs/M4_GROUND_TRUTH.md` and are not publicly mirrored — this is the project's most valuable trade secret.

---

## 2. Cave isolation primitive

**Claim:** Workloads run inside per-cave isolation domains — each with its own L1 page table, mount namespace, IPC mailbox, memory quota, and security labels. Cave boundaries enforce TLB invalidation; cave-private state never crosses.

- **Per-cave L1 page tables.** `feat/per-cave-kernel-partition` → `dd5bfda6` (cross-cave isolation verified). [`src/batcave/linux/mmu.rs::switch_to_cave`](../src/batcave/linux/mmu.rs).
  - **Selftest:** `python3 scripts/qemu_cave_private_selftest.py` — proves that cave A cannot read or write memory mapped in cave B's TTBR0 window.
- **Mount namespace per cave.** `feat/mount-ns-auto-apply` → `28875714`. [`src/batcave/cave.rs::active_mount_prefix`](../src/batcave/cave.rs).
  - **Selftest:** `python3 scripts/qemu_mount_ns_selftest.py` — proves that two caves can each create a file named "config" without colliding; cave A cannot list cave B's filenames.
- **Memory quota per cave.** `feat/batfs-quota-enforcement` → `aab7630c`; expansion to ELF runner via `feat/quota-cave-private-elf` → `28b53783`.
  - **Selftest:** `python3 scripts/qemu_batfs_quota_selftest.py` — proves quota is charged on `ns_create`, released on `ns_delete`, and that a cave exceeding its quota fails the write before reaching the encryption stage.
- **IPC mailbox per cave (sys-wg).** Arc 3 slice 3 → `767306d0`. [`src/batcave/sys_wg_ipc.rs`](../src/batcave/sys_wg_ipc.rs).
  - **Selftest:** `python3 scripts/qemu_sys_wg_ipc_selftest.py`.

---

## 3. MLS / Biba / TE / Information-flow stack

### Bell-LaPadula sensitivity lattice
- **Claim:** No read-up. A cave at sensitivity level `L_s` can read a file at level `L_o` only when `L_s ≥ L_o`.
- **Commit:** `feat/mls-labels-batfs` → `43d298a4`. Labels stamped at `ns_create`, enforced at `ns_read`.
- **Selftest:** `python3 scripts/qemu_mls_selftest.py` — proves no-read-up against the four-level lattice (Unclassified / Confidential / Secret / TopSecret).

### Biba integrity lattice (dual to BLP)
- **Claim:** No read-DOWN. A cave at integrity `I_s` can read a file at integrity `I_o` only when `I_s ≤ I_o`. A high-integrity cave refuses to be tainted by low-integrity input.
- **Commit:** `feat/biba-integrity-lattice` → `52449fda`.
- **Selftest:** `python3 scripts/qemu_biba_selftest.py`.

### AEAD-bound MLS labels
- **Claim:** A file's MLS labels are bound into the AEAD's AAD (`filename || sens || integ`). Any in-place tamper of the label invalidates decryption and the read fails closed.
- **Commits:** `feat/mls-labels-aead-bound` → `ec497e0e` (BatFS files); `feat/mls-ipc-aead-bound` → `d747bb75` (IPC messages).
- **Selftest:** `python3 scripts/qemu_mls_binding_selftest.py` (BatFS) + `python3 scripts/qemu_mls_ipc_binding_selftest.py` (IPC) — both prove a flipped label byte produces an AEAD verification failure rather than silent label confusion.

### SELinux-style Type Enforcement (subjects + objects)
- **Claim:** Per-(domain, type, op) DENY matrix on filesystem access; per-(from_cave, to_cave) allow-list on cave transitions.
- **Commits:** `feat/type-enforcement-transitions` → `11d8e12c` (subject transitions); `feat/te-on-objects` → `19a48383` (object types).
- **Selftests:** `python3 scripts/qemu_te_selftest.py` (transitions) + `python3 scripts/qemu_te_obj_selftest.py` (object-type matrix).

### Exec-time domain auto-transition (`domain_auto_trans`)
- **Claim:** A BatFS file can be tagged with a target cave id; running it via `exec-file` swaps the active cave to that target, gated by the existing TE allow-list. Matches SELinux execve semantics (lookup in caller namespace, transition before run).
- **Commit:** `feat/exec-domain-trans` → `faaebc1b`.
- **Selftest:** `python3 scripts/qemu_exec_trans_selftest.py` — proves (a) rule round-trip through lookup, (b) policy denies a non-admin transition without a rule and permits it after `add_transition_rule`, (c) admin caller successfully fires the transition.

### Information-flow taint propagation
- **Claim:** 32-bit operator-defined taint bitmap per cave and per file. Monotonic OR propagation: `ns_read` OR's file → cave; `ns_create` OR's cave → file. `ns_delete` zeros the slot.
- **Commit:** `feat/taint-propagation` → `a91437b9`.
- **Selftest:** `python3 scripts/qemu_taint_selftest.py` — proves stamp persistence, read-side propagation, write-side propagation, and monotonicity (untainted read does not regress a tainted cave).

### CIPSO IPv4 + CALIPSO IPv6 (SECMARK)
- **Claim:** Outbound packets carry the active cave's MLS sensitivity in a CIPSO IP option (RFC 2401) or CALIPSO IPv6 hop-by-hop option (RFC 5570). Inbound packets honor a receiver-side enforcement gate.
- **Commits:** `feat/secmark-cipso-emit` → `8e33e002` (IPv4 emit); `feat/secmark-receiver-enforcement` → `bf342d3f` (receiver gate); `feat/calipso-format` → `15c0ecc5` (IPv6 encode/parse).
- **Selftests:** `python3 scripts/qemu_secmark_selftest.py`, `python3 scripts/qemu_secmark_recv_selftest.py`, `python3 scripts/qemu_calipso_selftest.py`.

---

## 4. Audit log + accountability

### Tamper-evident audit ring (hash chain)
- **Claim:** Every audit entry's chain hash is `sha256(prev_chain_hash || canonical(entry))`. A verifier walks the resident ring from the oldest entry and returns either OK or the index of the first mismatch — the offset of the tamper.
- **Commit:** `feat/audit-tamper-evident-chain` → `6053d362`.
- **Selftest:** `python3 scripts/qemu_audit_chain_selftest.py` — proves that flipping a single byte of any past audit entry is detected, with the correct mismatch index reported.

### Off-platform audit seal
- **Claim:** The chain head + entry count can be serialized as a 40-byte seal, written to BatFS (and, in production, sealed off-platform via TPM / Apple SE / paper QR). On re-mount the verifier walks `start..seal.count` and either confirms `OK`, reports `Truncated{missing}`, or reports `Mismatch`.
- **Commit:** `feat/audit-chain-seal` → `7e2f4507`.
- **Selftest:** `python3 scripts/qemu_audit_seal_selftest.py` — proves the four-state verification outcome (`Ok`, `Truncated`, `Mismatch`, `SealAheadOfHead`).
- **Disclosure note:** The specific `SealVerify` state machine is held as Tier 3 (see [`DISCLOSURE_POSTURE.md`](DISCLOSURE_POSTURE.md)). Code is verifiable; design is not described in prose here.

### Two-person integrity (TPI) on destructive operations
- **Claim:** Every destructive privileged op requires a fresh M-of-2 Ed25519 quorum from two pre-registered officers (audit + crypto). Replay-resistant, TTL-bounded, role-separated, one-shot consumed.
- **Commits:** `feat/tpi-quorum` → `09bb70a8` (the primitive); `feat/tpi-wired-audit-seal` → `2ded4ec5` (wired to audit-seal); `feat/tpi-wired-ops` → `4667969c` (wired to audit-wipe + mls-declassify).
- **Selftests:**
  - `python3 scripts/qemu_tpi_selftest.py` — proves M-of-2 quorum + role separation + replay-resistance + TTL.
  - `python3 scripts/qemu_audit_seal_tpi_selftest.py` — proves audit-seal is TPI-gated end-to-end.
  - `python3 scripts/qemu_tpi_wired_ops_selftest.py` — proves audit-wipe + mls-declassify are TPI-gated end-to-end.
- **Disclosure note:** The grant-ring TTL value, canonical-bytes layout, and consume-vs-record ordering are Tier 3 implementation invariants.

---

## 5. Memory + speculative-execution hardening

### Hardened heap (canary frames)
- **Claim:** Every kernel allocation gets a 16-byte canary frame front and back, keyed by a boot-random secret. On deallocation we verify both frames; mismatch = panic. Freed blocks get a POISON pattern stamped on the front canary for double-free detection across reallocation.
- **Commit:** `feat/heap-guard` → `01460a66`. [`src/kernel/mm/guard.rs`](../src/kernel/mm/guard.rs).
- **Selftest:** `python3 scripts/qemu_heap_guard_selftest.py` — exercises all three detection paths (`Overflow`, `UnderflowOrAlien`, `DoubleFree`) without panicking the kernel. Non-destructive `inspect_user_ptr` lets the test run, then `repair_for_test` restores the canary so the eventual real `dealloc` proceeds cleanly.

### Spectre v1/v2/BHB barriers at cross-domain boundaries
- **Claim:** ARMv8.5 FEAT_SB (`sb`, NOP on older cores via `.inst 0xd50330ff`) emitted at three cross-domain transitions:
  - EL1→EL0 `eret` (in [`src/arch/aarch64/exceptions.s`](../src/arch/aarch64/exceptions.s) `RESTORE_REGS` macro, inlined into every vector)
  - Scheduler task switch ([`src/kernel/scheduler.rs::schedule`](../src/kernel/scheduler.rs))
  - TTBR0 cave swap ([`src/batcave/linux/mmu.rs::switch_to_cave`](../src/batcave/linux/mmu.rs))
- **Commit:** `feat/spectre-barriers` → `2a46e307`.
- **Evidence:** binary inspection — `llvm-objdump --triple=aarch64-unknown-none -d target/aarch64-unknown-none/release/sphragis | grep 'sb\b'` shows 8 `sb` instructions at the expected addresses (6 inlined from `RESTORE_REGS` × every vector that returns + 2 from sched/mmu).

---

## 6. Supply chain + reproducibility

### CVE feed ingestion against `Cargo.lock`
- **Claim:** `scripts/cve_audit.py` queries OSV.dev for every locked dependency. Suppressions live in `cve_audit.ignore` with rationale. Found and triaged RUSTSEC-2023-0071 (Marvin attack on `rsa` crate) — suppressed because Sphragis only uses `rsa` for public-key verify, not private-key decrypt, where the timing leak applies.
- **Commit:** `feat/cve-audit` → `14c786fe`.

### Permissive-only dep stack
- **Claim:** Every transitive dependency is licensed permissively (MIT / Apache-2.0 / BSD / CC0 / Unlicense). No GPL / AGPL / LGPL / MPL / EUPL contamination.
- **Re-audit one-liner** (from [`LICENSING.md`](LICENSING.md)):
  ```sh
  cargo tree --target aarch64-unknown-none --no-default-features \
    --features gicv3 -e normal --prefix none --format '{p} | {l}' \
    | sort -u | grep -iE 'gpl|agpl|copyleft|mpl|eupl'
  ```
  Empty output = clean. Last verified 2026-05-13.

### Reproducible builds + SBOM
- **Claim:** `scripts/repro_build.sh` produces deterministic kernel images; SBOM in CycloneDX format is emitted alongside.
- **Commit:** `feat/sbom-repro-build` → `d72f57dc`.

### Sigstore-compatible release signing + Rekor Merkle log
- **Claim:** `scripts/sign_release_artifacts.py` signs each release artifact with a project Ed25519 key, then appends a Rekor-compatible transparency log entry via `scripts/rekor_local.py` (RFC 6962 Merkle tree, audit paths, signed tree heads).
- **Commits:** `feat/release-signing-transparency` → `c6079796` (signing); `feat/sigstore-rekor-local` → `300cc834` (local Rekor); `feat/intoto-attestations` → `10041b5a` (in-toto v0.9 build-step attestations).

---

## 7. Networking

### WireGuard responder (Noise IK)
- **Claim:** Full Noise IK handshake, sliding-window replay protection, cave-private peer table, end-to-end verified against a Python initiator over a closed wire.
- **Commits:** `feat/wireguard-phase2-wire-framing` → `c47f9ad8` (wire encoders + mac1); `feat/wireguard-phase2.5-udp-dispatch` → `2abfd466`; replay protection → `1991c5a0`; full real-peer interop → `25b52b0e`.
- **Selftests:**
  - `python3 scripts/qemu_wg_dispatch_selftest.py` — UDP dispatch layer.
  - `python3 scripts/qemu_wg_replay_selftest.py` — replay protection window.
  - `python3 scripts/qemu_wg_initiator_e2e_selftest.py` — full handshake to a real peer.

### Stateful TCP/UDP firewall (conntrack)
- **Claim:** Stateful connection tracking with per-flow TCP state machine; firewall rules consult conntrack instead of stateless tuples. UDP flows tracked symmetrically.
- **Commits:** `feat/conntrack-foundation` → `20e782a2`; `feat/fw-conntrack-hardening` → `f10a71c5`; `feat/udp-conntrack-gating` → `78f40281`.
- **Selftests:** `python3 scripts/qemu_conntrack_selftest.py`, `python3 scripts/qemu_fw_hardening_selftest.py`.

### TLS 1.3 hardening
- **Claim:** TLS posture documented in [`DESIGN_TLS_HARDENING.md`](../DESIGN_TLS_HARDENING.md); X.509 chain validator with OCSP revocation cache; kernel-mediated HTTPS for caves per [`DESIGN_HTTPS_SYSCALL.md`](../DESIGN_HTTPS_SYSCALL.md).
- **Commits:** `feat/ocsp-revocation` → `e3f4b998`.
- **Selftests:** `python3 scripts/qemu_ocsp_selftest.py`, `python3 scripts/qemu_sni_selftest.py`.

---

## How to run every selftest

```sh
# build once
cargo build --release --target aarch64-unknown-none --features gicv3

# then run every selftest you want
for s in scripts/qemu_*_selftest.py; do
  echo "=== $s ==="
  python3 "$s"
done
```

A pass on every selftest produces a clean "all primitives operational" signal. Every selftest is headless, deterministic, and finishes in well under a minute on a developer Mac.

---

## What this doc is NOT

- **A threat model.** That belongs in its own doc (planned: `docs/THREAT_MODEL.md`).
- **An audit.** Sphragis has not been externally audited. The selftests prove that the implementation matches the design; they don't prove the design is free of higher-order weaknesses.
- **A claim of formal verification.** None of this code is machine-checked. Where we have invariants the compiler enforces (Rust type / borrow / lifetime system), we lean on that. Where we have invariants beyond what the compiler enforces (e.g. "the audit chain head is sealed within N entries"), we rely on tests and discipline.

The receipts say "this works as designed." Whether the design is sufficient for any particular threat model is a separate question.

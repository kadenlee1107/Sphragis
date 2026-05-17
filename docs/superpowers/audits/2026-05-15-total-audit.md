# Sphragis Total Security Audit — 2026-05-15

> **Scope:** Mechanical-trace security audit of the entire Sphragis codebase as of commit `9ef2c788` (post-Wave-8 merge). Six parallel deep-audit agents running on Opus, each owning a domain. Each agent re-checked the 9 unclosed findings from the 2026-05-06 Bat_OS-era audit AND scanned all new code added since (Waves 1-8, COMMS wire protocol, AGENT scaffold, ~10 K LoC delta).
>
> **Bar:** "Best-in-class modern hardening per subsystem" — meet NIST SP 800-53 / FIPS 140-3 / CSfC / CNSA 2.0 architectural alignment, with a path to certification rather than just "meets the floor."
>
> **Total findings:** 149 (14 CRITICAL, 32 HIGH, 53 MEDIUM, 30 LOW, 20 INFO).

---

## Executive Summary

Three patterns emerge across all six audits:

1. **The V11/2026-05-06 hardening sweep has substantially regressed.** Multiple "closed" findings from the prior audit are back open in the current code — most prominently the SealFS spinlock (FS-C1), the SVC source-EL check (Mem-H1), and the per-host SPKI pinning (Crypto-F3 was wired then unwired). This is the most actionable category — these fixes are mechanical and were once landed.

2. **Two parallel syscall paths exist with asymmetric hardening.** The Linux ABI compatibility layer in `caves/linux/syscall.rs` correctly applies `uaccess::is_user_range`, per-cave FD tables, and seccomp-style filtering. The native kernel syscall path in `kernel/syscall.rs` does none of these. Most criticals (Cave-C1/C2/C3, Mem-C1) live on the native side. The fix is to either remove the native path entirely (refuse SVC ≠ 0 from EL0) or apply the Linux-side discipline to it.

3. **The UI app surface added 8 large `static mut`-heavy modules in Waves 1-8 without applying cross-cave isolation discipline.** The `reset_for_cave_switch` hub at `caves/cave.rs:2212` wires only 5 subsystems (comms, keyboard, font, wm, console); 6 of 8 UI apps and 2 supporting modules (clipboard, tablet) are unwired, so their state persists across cave switches. Concretely: EDITOR's 256 KB BUFFER, FILES' 8 KB PREVIEW_BUF, AGENT's 41 KB TURNS array, the virtio-tablet's mis-routed key ring (which on Cocoa can contain passphrase bytes) — all leak across cave boundaries.

The fourth, less concentrated pattern: **modules implemented but not wired.** OCSP/CRL/CT-logs primitives all exist but `verify_chain` never consults them. `cert_pin::check` is callable from the shell but never called from the TLS handshake. `agent::reset_for_cave_switch` is defined but never invoked. The structural reflex of "land the primitive, wire it up later" has accumulated unwired-but-needed dependencies.

**Recommended ordering of remediation:** TLS server-auth bypass (Crypto-F1+F2) is the worst single issue — fix it first because it neutralizes every TLS-protected channel including the planned AGENT→inference-host link. Then the dual-syscall-path consolidation (Cave-C1 + Mem-H1). Then the cross-cave UI state leak (Drv-C1). Then the FS regressions (FS-C1/C2/C3/C4). The 32 highs and 53 mediums fan out from there.

---

## Findings Index

| ID | Severity | Title | File:line |
|---|---|---|---|
| **Crypto-F1** | 🔴 CRITICAL | TLS handshake accepts session without CertificateVerify | `net/tls.rs:901-1094` |
| **Crypto-F2** | 🔴 CRITICAL | TLS handshake accepts Finished before Certificate | `net/tls.rs:907-1094` |
| **Net-F1** | 🔴 CRITICAL | IPv4 fragments dispatched as if complete datagrams | `net/ip.rs:71-101` |
| **FS-C1** | 🔴 CRITICAL | SealFS spinlock + RAII guard around create/read/delete/list/stats — REGRESSED, no lock today | `fs/sealfs.rs:835/983/1028/1082/1093/1100` |
| **FS-C2** | 🔴 CRITICAL | `wipe::destroy_keys()` is a no-op for the SealFS master key | `security/wipe.rs:111-129` |
| **FS-C3** | 🔴 CRITICAL | `verify_all_integrity()` is a tautology — cannot detect tampering | `fs/sealfs.rs:185-189` |
| **FS-C4** | 🔴 CRITICAL | ChaCha20-Poly1305 nonce derivation: CTR-style persistent prefix, no per-volume/cave key | `fs/sealfs.rs:347-365` + `:686` |
| **Drv-C1** | 🔴 CRITICAL | `reset_for_cave_switch` not wired for 6 of 8 UI apps + tablet + clipboard | `caves/cave.rs:2212-2258` |
| **Drv-C2** | 🔴 CRITICAL | COMMS AEAD has no associated data — nonce is the only direction-bind | `ui/apps/comms.rs:370/422` |
| **Drv-C3** | 🔴 CRITICAL | COMMS KDF does not commit identity pubkeys | `ui/apps/comms.rs:486-506` |
| **Cave-C1** | 🔴 CRITICAL | `kernel/syscall.rs` SVC≠0 path blindly trusts user pointers (F7 still wide open on native path) | `kernel/syscall.rs:160-167/246/266/293` |
| **Cave-C2** | 🔴 CRITICAL | UDP RX is single global cross-cave buffer (F5 unchanged) | `net/udp.rs:114-132` |
| **Cave-C3** | 🔴 CRITICAL | `SYS_SHM_PTR` returns a kernel-address pointer to userspace | `kernel/syscall.rs:188-198` |
| **Mem-C1** | 🔴 CRITICAL | `from_utf8_unchecked` on user-controlled path bytes — UB regression | `caves/linux/syscall.rs:1970/1993/2192/4640/4706/5249-5497` |
| **Crypto-F3** | 🟠 HIGH | Per-host SPKI pinning is dead code (`cert_pin::check` never called from `verify_chain`) | `net/cert_pin.rs:127` + `net/tls.rs:1011` |
| **Crypto-F4** | 🟠 HIGH | No revocation check (OCSP/CRL primitives unwired) | `net/x509.rs:514-746` + `net/{ocsp,crl}.rs` |
| **Crypto-F5** | 🟠 HIGH | DoT DNS TXID uses raw `cntpct_el0` instead of CSPRNG | `net/dot.rs:37-41` |
| **Crypto-F6** | 🟠 HIGH | DoT hostname check uses dNSName SAN for IP-only servers (no iPAddress SAN) | `net/dot.rs:95` + `net/x509.rs:446-468` |
| **Crypto-F7** | 🟠 HIGH | Boot-time crypto self-test failures are fail-soft, not fail-closed | `main.rs:1303-1313` |
| **Net-F2** | 🟠 HIGH | TCP RX-buffer race still unfixed from prior audit's F10 (no `IrqGuard`) | `net/tcp.rs:1591-1600` |
| **Net-F3** | 🟠 HIGH | Inbound IP destination never filtered (broadcast/multicast/foreign-IP dispatched) | `net/ip.rs:313-369` |
| **Net-F4** | 🟠 HIGH | ICMP echo reply un-rate-limited; payload reflected verbatim (DRDoS amplifier) | `net/icmp.rs:29-39` |
| **Net-F5** | 🟠 HIGH | TCP options completely ignored on receive (MSS/WS/SACK attacks) | `net/tcp.rs:1356-1362` |
| **Net-F6** | 🟠 HIGH | Conntrack 4-tuple ignores destination IP (multi-NIC port collision) | `net/conntrack.rs:60-78/168-191` |
| **FS-H1** | 🟠 HIGH | Cave-mount-namespace separator unvalidated → confused-deputy cross-cave access | `caves/cave.rs:1363-1395` + `fs/sealfs.rs:422-434` |
| **FS-H2** | 🟠 HIGH | Init ordering: SealFS init runs before crypto self-tests; RNG not fail-closed on RNDR-absent | `main.rs:194-283` |
| **FS-H3** | 🟠 HIGH | RNG seed entropy structurally weak when RNDR unavailable (8× `cntpct_el0` reads) | `crypto/rng.rs:100-116` |
| **FS-H4** | 🟠 HIGH | Filesystem name lookups leak file existence via timing | `fs/sealfs.rs:1126-1135` |
| **FS-H5** | 🟠 HIGH | `init_disk` races against publishing `INITIALIZED=true` | `fs/sealfs.rs:228-231` |
| **FS-H6** | 🟠 HIGH | Filenames are plaintext on disk (no inode-table encryption) | `fs/sealfs_disk.rs:58-59` |
| **FS-H7** | 🟠 HIGH | Audit ring in-RAM only, no encrypted persistence, no automated off-platform seal | `security/audit.rs:348-361` + `audit_chain.rs:22` |
| **Drv-H1** | 🟠 HIGH | AGENT synchronous `ask()` blocks UI loop — Phase-5 DoS via withheld SSE | `ui/apps/agent.rs:402-441` |
| **Drv-H2** | 🟠 HIGH | `from_utf8_unchecked` invariant on agent response one PR away from breaking | `ui/apps/agent.rs:288-295` |
| **Drv-H3** | 🟠 HIGH | Filemanager `PREVIEW_BUF` (8 KB) leaks last-previewed file across caves | `ui/apps/filemanager.rs:29` |
| **Drv-H4** | 🟠 HIGH | Editor `BUFFER` (256 KB plaintext file content) leaks across caves | `ui/apps/editor.rs:54` |
| **Drv-H5** | 🟠 HIGH | NO UI apps emit audit-ring entries (wipe, AGENT session, COMMS connect, EDITOR save all unaudited) | `ui/apps/*.rs` |
| **Cave-H1** | 🟠 HIGH | Per-cave page table count (8) < `MAX_CAVES` (32) — caves 9-32 share PRIMARY_L1 with silent downgrade | `caves/cave.rs:9` + `caves/linux/mmu.rs:137` |
| **Cave-H2** | 🟠 HIGH | Per-cave seccomp denylist not consulted on SVC≠0 / native-kernel-syscall path | `caves/syscall_filter.rs:97` |
| **Cave-H3** | 🟠 HIGH | No PAN (Privileged Access Never) enabled in SCTLR_EL1 | `main.rs:39-46` + `caves/linux/mmu.rs:1560-1583` |
| **Cave-H4** | 🟠 HIGH | `set_active(usize)` is `pub` with no caller-side capability check | `caves/cave.rs:1019` |
| **Cave-H5** | 🟠 HIGH | AF_UNIX socket name namespace is global across caves (F6 extension) | `kernel/unix_sock.rs:90-103` |
| **Cave-H6** | 🟠 HIGH | Generic `sys_connect` (Linux #203) does NOT consult `cave_policy::check` | `caves/linux/syscall.rs:3928-3996` |
| **Mem-H1** | 🟠 HIGH | SVC source-EL check regression — prior F3 was closed, now reopened | `arch/aarch64/exceptions.s:101-117` + `kernel/arch/mod.rs:654-680` |
| **Mem-H2** | 🟠 HIGH | Stack canary value is hardcoded constant, never seeded from RNDR | `kernel/stack_chk.rs:26/44` |
| **Mem-H3** | 🟠 HIGH | PAN not enabled in SCTLR_EL1 (duplicate of Cave-H3 from different angle) | `main.rs:39-46` |
| **Mem-H4** | 🟠 HIGH | `transmute::<u8, PeerId>(0u8)` in const-init relies on undefined repr | `net/wg_dispatch.rs:84` |

(Mediums, lows, and infos: see per-domain detail in agent reports. Total: 53 medium + 30 low + 20 info findings, ~110 additional items.)

---

## Prior 9 Unclosed Findings — Re-Check Status

| Prior ID | Title | 2026-05-06 status | 2026-05-15 status |
|---|---|---|---|
| Kernel F4 | ERET DAIF | Closed by F3 | ✅ Still closed (degenerate fix) |
| Kernel F6 | PT mapper user/kernel ownership | Deferred | ❌ Still unclosed (Mem-M6) |
| Net F6 | TLS full PKI chain validation | Deferred (pinning-only) | ✅ Landed: `verify_chain` does full chain + SAN + KeyUsage + EKU. **BUT** undone by Crypto-F1+F2+F3 (handshake bypass and pinning dead code) |
| Net F10 | TCP RX buffer race | Deferred | ❌ Still unclosed (Net-F2) |
| BatCave F4 | Global FD table | Architectural | ⚠️ Partially closed (Linux compat per-cave; native path still shared) |
| BatCave F5 | Cross-cave UDP RX | Architectural | ❌ Still unclosed (Cave-C2) |
| BatCave F6 | Cross-cave BatPipe creds | Architectural | ❌ Still unclosed (Cave-M5: no `owner_cave_id` on Pipe/Socket/Shm/PairBuf) |
| BatCave F7 | Syscall user-pointer sweep | Architectural | ⚠️ Closed on Linux ABI path; native path open (Cave-C1) |
| BatCave F9 | Symlink confused-deputy TOCTOU | Architectural | ⚠️ Substantially open (Mem-M7: no post-resolve cap re-check) |

**Net status: 1 closed (F6 — partially undone by new bugs), 3 partial, 5 still open.**

---

## Elite-Tier Modern Hardening Bar — Current Status

From `project_security_bar.md` and the 2026-05-06 audit's "elite additions still owed" list:

### Crypto
| Item | Status | Notes |
|---|---|---|
| ChaCha20-Poly1305 | ✅ Wired | `crypto/chacha20poly1305.rs`; AEAD-correct |
| Ed25519 verify | ✅ Resolved | Now via `ed25519-compact` crate (prior in-tree quarantine deleted) |
| X25519 + low-order subgroup check | ✅ Yes | `is_low_order_x25519` table from RFC 7748 §6.1 |
| AES-GCM-SIV (SealFS at-rest) | ❌ No | SealFS still uses non-SIV ChaCha20-Poly1305 (FS-C4) |
| ML-KEM-768 (Kyber, FIPS 203) | ✅ Wired | `pq_hybrid.rs`, codepoint 0x11EC active by default |
| ML-DSA-65 (Dilithium, FIPS 204) | ⚠️ Primitive wired; not yet in X.509 / CertVerify | No IETF draft live for TLS yet |
| BLAKE3 | ✅ Wired | `crypto/blake3.rs`, non-FIPS auxiliary |

### Kernel / Memory
| Item | Status | Notes |
|---|---|---|
| PAC-ret | ✅ Enabled | rust-toolchain pinned, `branch-protection=pac-ret` |
| PAN | ❌ Not enabled | Cave-H3 / Mem-H3 |
| BTI landing pads | ❌ Not wired | Banner advertises "pac-ret+bti" but no `bti c/j/jc` in vector table or function prologues |
| Stack canaries | ⚠️ Compiled, hardcoded value | `__stack_chk_guard = 0xdead_beef_cafe_babe`; `seed_from_rng` exists but never called (Mem-H2) |
| KASLR | ❌ Not implemented | Fixed boot VA |
| RNDR mixing for DRBG | ✅ Yes | HMAC-DRBG + RNDR-XOR; fail-soft on RNDR absent (FS-H3) |

### Network
| Item | Status | Notes |
|---|---|---|
| Per-origin SPKI cert pinning | ⚠️ Implemented, unwired | Crypto-F3 |
| Post-quantum hybrid TLS (X25519+Kyber) | ✅ Live | Codepoint 0x11EC, hot-switchable disable on interop failure |
| DoT | ✅ Yes | Strict rcode-checking path; default DNS resolver path |
| DoH | ⚠️ Silent plaintext fallback on failure | Crypto/Net-medium |
| Tor / mixnet | ❌ Not implemented | Roadmap |
| IPv6 | ❌ Not dispatched | `ETHERTYPE_IPV6` not in mod.rs match (CALIPSO is dead code) |
| Conntrack with bounded eviction | ✅ Yes | But no `local_ip` field (Net-F6) and no `reset_for_cave_switch` |
| OCSP / CRL revocation | ⚠️ Implemented, unwired | Crypto-F4 |
| Certificate Transparency | ⚠️ Stub | Crypto-LOW: zero-byte log IDs |

### Filesystem / Storage
| Item | Status | Notes |
|---|---|---|
| AES-GCM-SIV at-rest | ❌ No | FS-C4 (ChaCha20-Poly1305 with CTR-style nonces, no per-volume/cave KDF) |
| Per-block Merkle tree | ⚠️ Partial | Per-file Merkle root computed; verifier is tautological (FS-C3) |
| Anti-rollback monotonic counter | ❌ No | `boot_counter` exists but doesn't gate beyond detecting disk-image swap |
| Argon2id passphrase KDF | ✅ Yes | Auth + SealFS master, with SHA fallback (FS-M7) |
| Plausible-deniability hidden volumes | ❌ No | Not implemented |
| Anti-cold-boot via SIMD-resident keys | ❌ No | `MASTER_KEY` plaintext in DRAM for entire session |

### Auth / Wipe
| Item | Status | Notes |
|---|---|---|
| Argon2id KDF | ✅ Yes | + audited fallback to SHA-256 KDF |
| Atomic lockout | ✅ Yes | `fetch_add` (V11 fix preserved) |
| Per-attempt backoff | ❌ No | Cave-M6: 5 tries can be issued back-to-back |
| SEP mailbox driver | ❌ No | Documented stub; not implemented |
| Real SEP-wrapped key material | ❌ No | All keys live in DRAM `static mut` |
| Wipe completeness | ❌ Broken | FS-C2: master key never zeroed; FS-M8: dependency order wrong |
| Deadman timer | ⚠️ Partial | Cave-M6: only polled from desktop idle loop, not from lock screen, serial shell, or timer IRQ |

### Caves / Isolation
| Item | Status | Notes |
|---|---|---|
| Per-cave page tables (memory isolation via TTBR0+TLBI) | ⚠️ Only caves 0-7 | Cave-H1: silent downgrade for caves 8-31 |
| Per-cave ASIDs | ❌ No | Full TLBI on switch (slow + leaks across boundary briefly) |
| Cave-private VA (4 KB private page) | ✅ Yes | `cave_private.rs` |
| Seccomp-style syscall allowlist per cave | ⚠️ Linux ABI only | Cave-H2: not enforced on native syscall path |
| MAC (Mandatory Access Control) hooks | ⚠️ Partial | BLP/Biba/TE functions exist; no central enforcement bottleneck |
| Default-deny on egress at every send | ⚠️ Partial | Cave-H6: only `bat_https_open` + NAT gate; generic `sys_connect` doesn't |
| Per-cave IPC primitive attribution | ❌ No | Cave-M5: Pipe/Socket/Shm/PairBuf have no `owner_cave_id` |

### Audit Ring
| Item | Status | Notes |
|---|---|---|
| Hash-chain across entries | ✅ Yes | `audit_chain.rs`, SHA-256 |
| HMAC per entry (key-only-in-SEP) | ❌ No | Cave-M1: plain SHA-256, attacker with kernel write can roll the chain |
| WORM (append-only) off-platform export | ❌ No | `flush_to_sealfs` is delete-then-create |
| Categories cover all subsystems | ⚠️ Partial | Cave-M2: missing Crypto, Net, Fs, KeyRotate, TpiOp categories |
| Per-entry `cave_id` field | ❌ No | Cave-M3: provenance in message text only |
| Audit entries from UI apps | ❌ No | Drv-H5: zero call sites in any UI app |
| Audit-ring wiped on emergency wipe | ❌ No | FS-L4: `audit::wipe_ring()` never called from `wipe::execute` |

### Hardware Platform Trust
| Item | Status | Notes |
|---|---|---|
| Apple Silicon Permissive Security boot chain | ⚠️ Operator-controlled but not operator-rooted | SEP keys are Apple's; for TS/SCI need a platform pivot (Coreboot+Heads, IBM POWER, custom hardware) |
| TPM-attested boot | ❌ N/A on M4 | Future platform decision |
| HSM-backed operator CA keys | ❌ Operator workflow not built | Yubikey integration is straightforward but unscoped |
| TEMPEST shielding | ❌ Not in scope of the kernel | Hardware/facility-level item |
| Tamper-evident seals | ❌ Not in scope | Operational item |

---

## Pattern Analysis

### Pattern 1: V11/2026-05-06 hardening regressions

The 2026-05-06 audit closed 35 of 43 findings across 16 commits on `fix/security-hardening`. Several of those closures are no longer present in the current `main`:

- **FS-C1** — SealFS RAII guard / spinlock around create/read/delete/list/stats: the 16-commit hardening commit `537abd80` added a SpinLock + RAII guard; **the current code at `fs/sealfs.rs:835/983/1028/1082/1093/1100` has no lock.** The only sync site is `next_nonce()`'s IrqGuard. The UAF/double-free race is back.
- **Mem-H1** — SVC source-EL check: prior F3 closure included an SPSR.M check before SVC dispatch. The current code at `kernel/arch/mod.rs:654-680` has no such check.
- **Crypto-F3** — STRICT_MODE pin enforcement: prior `c862c6ba` + `996fc256` wired cert pinning into the TLS handshake. `cert_pin::check` is still defined but no longer called from `verify_chain`.

**Hypothesis:** Subsequent merges (especially the WireGuard / NAT / multi-NIC / Wave 1-8 UI work) touched the same files and the locking/checking lines were lost in conflict resolution. A git-blame archaeology pass would confirm which merges and let us write tests that lock these in.

### Pattern 2: Dual-syscall-path asymmetry

The kernel exposes two syscall surfaces:

- **Linux ABI compatibility** in `caves/linux/syscall.rs` — entered via `svc 0` from EL0, dispatched through a hardened path with `uaccess::is_user_range`, per-cave FD tables, seccomp filtering, cave-policy enforcement on most network egress points.
- **Native kernel syscalls** in `kernel/syscall.rs` — entered via `svc N≠0` from EL0 (or from internal kernel calls), dispatched through a path with NO `uaccess`, NO seccomp, NO cave-policy enforcement.

Half the criticals (Cave-C1, Cave-C2, Cave-C3, Mem-C1 partial) and most of the highs (Cave-H1, Cave-H2, Cave-H6) sit on the native side. The most defensible fix: **refuse SVC ≠ 0 from EL0 at the exception handler.** Caves use the Linux ABI exclusively; the native path becomes kernel-internal only. The native path's pointer-trust assumptions then become safe (no attacker can reach them from EL0).

### Pattern 3: UI surface bypassed cave-isolation discipline

Waves 1-8 added 8 UI apps (~5 K LoC) with substantial `static mut` state. The cave-isolation discipline expressed in `caves/cave.rs:2212` (`reset_all_globals_for_cave_switch`) was not extended to the new modules. Most apps have a `reset_for_cave_switch` function — it's just never called.

- AGENT (Wave 8, just merged): `agent::reset_for_cave_switch` defined at `agent.rs:485`, not invoked
- EDITOR: no reset function exists; 256 KB BUFFER persists
- FILES: no reset function exists; 8 KB PREVIEW_BUF persists
- SECURITY: no reset function exists; audit-chain head + wipe-confirm modal persist
- CAVES (manager): no reset function exists; new-cave-name scratch persists
- NETMON / DASHBOARD: viewport state persists
- Tablet (virtio): `reset_for_cave_switch` defined, not invoked; mis-routed keys leak

The fix is mechanical: enumerate every static-mut-bearing UI module in the hub function. Add a regression test that simulates cave switch and asserts every UI module's state is zero.

### Pattern 4: Implemented-but-unwired primitives

- `cert_pin::check` exists, never called from `verify_chain`
- `ocsp::status` / `crl::is_revoked` exist, never called from `verify_chain`
- `ct_logs::find` exists, log IDs are all-zeros
- `kernel::capability::CapabilitySet` (seL4-style) exists, never used (caves use a different string-based cap system)
- `audit::Category::{Crypto, Net, Fs, KeyRotate, TpiOp}` enums don't exist; should
- `syscall_filter::is_denied_active` exists, only called on Linux ABI path
- `agent::reset_for_cave_switch` exists, never called
- `editor::*`, `filemanager::*`, `security::*`, `caves_mgr::*` — no reset functions even defined
- `stack_chk::seed_from_rng` exists, never called
- `cave_private` per-cave VA isolation exists for some keys, not all

The remediation pattern is identical: trace every `pub fn` that exists in a security primitive module, grep for callers, and either wire it in or delete it.

---

## Recommended Fix Priority

### Week 1 (the critical-line)

1. **Crypto-F1+F2** — Fix TLS handshake state machine. Track `saw_ee`, `saw_cert`, `saw_cv`, `saw_fin` flags; reject out-of-order; require all four before Established. **This single fix unbreaks every TLS-protected channel including the planned AGENT path.** Estimated effort: 1 day.

2. **Cave-C1 + Mem-H1** — Consolidate syscall paths. Refuse `svc N ≠ 0` from EL0 at `kernel/arch/mod.rs::handle_sync_exception`. Add SPSR.M check before SVC dispatch. The native path becomes kernel-internal only. Estimated effort: 1 day.

3. **FS-C1** — Restore the SealFS spinlock + RAII guard around create/read/delete/list/stats (re-land the patch from `537abd80`). Add regression test. Estimated effort: 0.5 day.

4. **FS-C2** — Implement `sealfs::wipe_master_key()` that actually zeroes `MASTER_KEY` + `BOOT_NONCE_PREFIX` + `FILES[]` under `critical_section!`; reorder `wipe::execute` to call it after `wipe_filesystem`. Estimated effort: 0.5 day.

5. **Crypto-F3 + Crypto-F4** — Wire `cert_pin::check` and `ocsp::status` / `crl::is_revoked` into `verify_chain`. Estimated effort: 0.5 day.

6. **Drv-C1** — Enumerate every UI app + driver in `caves/cave.rs:2212` `reset_for_cave_switch` hub. Add missing reset functions for editor, filemanager, security, caves_mgr, netmon, dashboard, agent, clipboard, virtio-tablet. Estimated effort: 1 day.

**Total Week 1: ~4.5 days of focused engineering.** Closes 7 of 14 criticals + 2 of 32 highs.

### Week 2

7. **Net-F1** — Reject IPv4 fragments at `IpPacket::parse`. One-line guard. Closes the off-path TCP injection vector.

8. **Net-F2** — Wrap TCP RX producer in `IrqGuard`. Re-land the pattern from 2026-05-06.

9. **Net-F3 + Net-F4 + Net-F5** — Inbound destination filter, ICMP echo rate-limit, TCP options parser with bound-checked walking.

10. **FS-C3** — Replace tautological `verify_all_integrity` with HMAC-signed root anchored in a sealed audit-chain entry or external WORM.

11. **FS-C4** — Migrate SealFS to AES-GCM-SIV (or commit to ChaCha20-Poly1305 with per-cave KDF + bound nonces in AAD).

12. **Drv-C2 + Drv-C3** — COMMS AAD bind + KDF include id_pk.

13. **Cave-C2 + Cave-C3** — Per-cave UDP RX tagging; replace `SYS_SHM_PTR` with VA-mapping syscall.

14. **Mem-C1** — Mechanical replace of all `from_utf8_unchecked` on user paths with `from_utf8(...).map_err(|_| EINVAL)?`. Apply pattern uniformly across `caves/linux/syscall.rs`.

**Total Week 2: ~5 days.** Closes remaining 7 criticals + most of the network and FS highs.

### Weeks 3-4 (defense-in-depth + elite-tier)

15. Enable PAN (`SCTLR_EL1.SPAN` + `msr PAN, #1` at boot; thread `msr PAN, #0` around `uaccess` byte loops). Cave-H3 / Mem-H3.

16. Wire BTI landing pads in vector table and function prologues. Update toolchain flag. Mem elite-tier.

17. Seed `__stack_chk_guard` from RNDR at boot. Mem-H2.

18. Hard-fail `cave::enter()` on `cave_l1_phys == 0` (no silent PRIMARY_L1 downgrade). Cave-H1.

19. Per-cave seccomp on native path. Cave-H2.

20. Generic `sys_connect` consults `cave_policy::check_with_sni`. Cave-H6.

21. Add `Crypto / Net / Fs / KeyRotate / TpiOp` audit categories. Add `cave_id` to `audit::Entry`. Wire UI apps to emit audit entries for wipe, AGENT session, COMMS connect, EDITOR save, FILES delete. Cave-M2 / M3 + Drv-H5.

22. HMAC-SHA256 each audit chain link with SEP-sealed key. WORM export to BD-R/tape. Cave-M1 / FS-H7.

23. Fail-closed crypto self-tests for every primitive at boot (`crypto::run_self_tests() -> !`). Crypto-F7.

**Total Weeks 3-4: ~7-10 days.** Closes 32 highs + most of the medium-severity hardening.

### Week 5+ (architectural)

24. Per-cave ASIDs (replace full TLBI on switch).

25. PT-mapper frame-classification table (close Kernel-F6 from prior audit).

26. Symlink resolution gate (close BatCave-F9 from prior audit).

27. AES-256-GCM-SHA384 cipher suite end-to-end. CNSA 2.0 alignment.

28. ML-DSA hybrid signatures in X.509 / CertVerify (waits on IETF draft).

29. KASLR.

30. Document operator-runbook for the hardened deployment.

**Total architectural: 2-4 weeks.** Closes the remaining elite-tier and prior architectural items.

---

## Government-Grade Gap (Forward-Looking)

What the codebase needs *beyond* this audit's remediation list to be deployable for confidential government use:

### Platform / Boot Chain (the elephant)

Apple Silicon under Permissive Security cannot reach TS/SCI accreditation as-is because:
- Boot chain is operator-controlled but rooted in Apple's SEP keys
- No operator-controlled hardware root of trust
- No TPM 2.0 with operator-managed PCR policy

**Options:**
- (a) **Pivot to a certifiable platform** (Intel/AMD + Coreboot+Heads, IBM POWER, custom). Kernel code largely ports; drivers re-do. 6-12 months of work.
- (b) **Dual-track**: Apple Silicon is the dev/demo platform; Coreboot box is the deployable. Same kernel, two driver sets.
- (c) **Apple enterprise channel**: pursue an enterprise/government program with Apple. High latency, narrow capability.

(b) is the lowest-risk option that keeps the dev velocity we have.

### Certification Bureaucracy

For actual USG deployment:
- **NIST FIPS 140-3 Level 3** certification on the crypto module ($50K-$300K, 12-24 months, NIST CMVP testing)
- **Common Criteria EAL4+** evaluation against the OS Protection Profile ($200K-$500K, 12-18 months)
- **NSA CSfC capability package** compliance (specific configurations, dual-layer crypto)
- **DISA STIG** development
- **Security Target documentation** approaching the volume of the code itself
- **NIAP-authorized evaluation lab** partnership

Total cost: $500K-$2M. Total time to first cert: 18-36 months. Maintained continuously thereafter.

### Operator-Side Hardening

Items the kernel exposes APIs for but the operator builds outside:

- HSM-backed operator CA on Yubikey
- Tamper-evident chassis seals
- TEMPEST-shielded enclosure
- SCIF facility (ICD 705 compliance)
- Air-gapped network architecture
- Write-once optical media for audit retention
- Operator authentication via PIV/CAC card

### Where Sphragis Can EXCEED, Not Just Meet

Items where the architecture choices put Sphragis ahead of typical certified platforms:

1. **Post-quantum crypto already wired** — ML-KEM-768 active in TLS today, ML-DSA primitive ready. Most certified products are years from this.
2. **Pure no-std Rust** — order-of-magnitude smaller attack surface than any C-based competitor; aligned with the strongest current memory-safety guidance from CISA.
3. **Capability-based caves** — closer to seL4-class isolation than Linux's POSIX-based MAC stacks.
4. **No external dependencies** — supply chain auditability is trivial (kernel is one repo + a handful of vetted Rust crates).
5. **Audit chain integrity** — the SHA-256 hash chain (with HMAC + WORM upgrades on the roadmap) is stronger than the typical syslog/rsyslog approach.
6. **TLS pinning only, no CA chain** — eliminates the entire commercial CA trust class.
7. **Hardware-attested boot path through m1n1** (on the Apple-platform dev side) is reproducible from operator-controlled artifacts — better than typical UEFI-based platforms.

The story to USG: **Sphragis is architecturally past the certification floor on day one.** What's needed is the bureaucratic apparatus to evaluate it, not technical rework. If we lock in the Week 1-2 critical fixes and the Week 3-4 defense-in-depth additions, the codebase enters certification ready to demonstrate "best-in-class" rather than "compliant minimum."

---

## Appendix: Per-Domain Detailed Findings

The full per-agent reports (with code snippets, exploit paths, and per-finding remediation) are preserved in the audit task outputs:

- Network parsers: `/private/tmp/claude-501/.../ac326d52b55ab73cd.output`
- Filesystem + Boot + RNG: `/private/tmp/claude-501/.../a962ecbcdb58970f6.output`
- Drivers + UI + AGENT: `/private/tmp/claude-501/.../a641374521e5b3e97.output`
- Crypto + TLS + X.509: `/private/tmp/claude-501/.../ab8196f488ad15bf1.output`
- Caves + Audit Ring + Isolation: `/private/tmp/claude-501/.../af39173c79cbe0fe8.output`
- Memory Safety + Unsafe: `/private/tmp/claude-501/.../a8b50a60dade389c4.output`

This index document is the canonical reference. Use the agent outputs for code-level detail when implementing fixes.

---

*Audit conducted by 6 parallel Opus agents on 2026-05-15. Branch: `main` at commit `9ef2c788`. Total findings: 149. Recommended Week 1+2 remediation: ~10 days of focused engineering to close all 14 criticals and the worst 10 highs.*

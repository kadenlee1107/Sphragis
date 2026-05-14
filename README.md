# Sphragis

**First non-Apple OS booted on Apple M4. Bare-metal Rust microkernel. Government-grade security primitives.**

> ⚠️ Research-grade. Not production-ready. APIs and on-disk formats change without notice.

Sphragis is a security-first microkernel for Apple Silicon, written in Rust and built from scratch — no Linux base, no Asahi fork, no off-the-shelf VFS or networking stack. As of April 2026 it is the first known non-Apple operating system to boot on Apple M4 hardware (Mac16,1 / J604 / T8132 "Donan"), reaching an interactive shell with a status bar over m1n1 chainload. ([boot evidence](docs/photos/2026-04-17_first_m4_boot/))

## What's it for

Security-critical deployments where the cost of a kernel compromise outweighs the cost of running a custom kernel: defense, intelligence, compliance-regulated infrastructure, anything where "we run Linux because that's what everyone runs" is a liability rather than an asset.

Sphragis is opinionated about one thing in particular: **no ambient authority**. There is no root user. There is no `sudo`. Every destructive privileged operation — wiping the audit log, downgrading a file's security classification, rotating the master key, flushing an off-platform audit seal — requires a fresh M-of-2 Ed25519 quorum from two pre-registered officers (an audit officer and a crypto officer), with one-shot consumption and a TTL. A single compromised key does not get you privileged operations.

## What works today

- **Boots on real Apple M4 hardware** via [m1n1 chainload](https://github.com/AsahiLinux/m1n1). ADT discovery, DWC3 USB-3 bring-up, ATC PHY tunables, dockchannel UART — all confirmed on real silicon. As of May 2026 this is the only published non-Apple OS boot on M4 we are aware of; the Asahi Linux installer declines M4 at the time of writing.
- **Boots in QEMU virt** for development. `cargo build --release --target aarch64-unknown-none --features gicv3` produces a kernel that runs end-to-end on `qemu-system-aarch64 -machine virt -cpu max`. Every selftest below runs headlessly under QEMU.
- **Per-cave MMU isolation.** Workloads run inside "caves" — each with its own L1 page table, mount namespace, IPC mailbox, memory quota, and security labels. Cave boundaries enforce TLB invalidation; cave-private state never crosses.
- **Encrypted filesystem (BatFS).** Per-file AEAD (ChaCha20-Poly1305), per-cave keys derived from Argon2id over a passphrase, mount-namespace prefix per cave so two caves can't see each other's filenames.
- **Multi-level security.** Bell-LaPadula sensitivity lattice (no read-up) and Biba dual lattice integrity (no read-down). MLS labels are bound into AEAD AAD — tampering with a file's classification invalidates decryption.
- **SELinux-style Type Enforcement.** Subject domains (one per cave), object types (per BatFS file), per-(domain, type, op) deny matrix, plus exec-time domain auto-transition (`domain_auto_trans`) gated by an explicit allow-list.
- **Two-person integrity (TPI).** Ed25519 M-of-2 quorum on every destructive privileged op (`audit-wipe`, `audit-seal`, `mls-declassify`, master-key rotation). Replay-resistant, TTL-bounded, role-separated, one-shot consumed.
- **Tamper-evident audit log.** Hash chain over the audit ring detects modification of any entry — the offset of the first mismatch tells the operator how far back the tampering reaches. Off-platform seal protocol extends detection past one ring rollover.
- **Hardened heap.** Per-allocation front + back canary frames keyed by a boot-random secret (SHA-256(KEY || addr || size)). Detects buffer overflow, underflow, and double-free at deallocation. POISON pattern stamped on freed blocks for double-free detection across reallocation.
- **Spectre barriers.** ARMv8.5 FEAT_SB (`sb`) speculation barriers at every cross-domain transition: EL1→EL0 `eret`, scheduler task switch, TTBR0 cave swap. NOP on cores without FEAT_SB, real mitigation where the hardware has it.
- **Information-flow taint propagation.** 32-bit operator-defined taint bitmap per cave and per file; monotonic OR-propagation through filesystem reads (file → cave) and writes (cave → file). The substrate for future egress enforcement on PII / compliance-restricted / trade-secret data.
- **WireGuard responder.** Cave-private state (no peer leaks between caves), Noise IK handshake, sliding-window replay protection. Verified end-to-end against a Python initiator over a closed wire.
- **CALIPSO + CIPSO labelling.** RFC 5570 IPv6 SECMARK encode/parse with DOI gate; RFC 2401 IPv4 IP options.
- **Supply-chain hygiene.** `Cargo.lock` audited against [OSV.dev](https://osv.dev/) with documented suppressions for non-applicable findings. Every transitive dependency is permissive-licensed (MIT / Apache-2.0 / BSD / CC0 / Unlicense); no copyleft contamination in the tree.
- **Reproducible builds.** `scripts/repro_build.sh` produces deterministic kernel images.
- **Sigstore-style release signing.** Build-step signatures + Rekor-compatible Merkle log + in-toto v0.9 attestations on release artifacts.

Each item has a headless QEMU selftest under [`scripts/`](scripts/) and a corresponding feature-branch merge in the [git history](https://github.com/kadenlee1107/Sphragis/commits/main).

## How to build & try

You need a Mac or Linux box with [`rustup`](https://rustup.rs/) (nightly toolchain) and `qemu-system-aarch64`. Then:

```sh
git clone <REPO_URL>
cd Sphragis
cargo build --release --target aarch64-unknown-none --features gicv3

# Boot smoke — verifies the kernel reaches the shell prompt.
python3 scripts/qemu_boot_smoke.py

# Security selftests — each exercises a specific primitive headlessly.
python3 scripts/qemu_tpi_wired_ops_selftest.py
python3 scripts/qemu_heap_guard_selftest.py
python3 scripts/qemu_taint_selftest.py
python3 scripts/qemu_mls_selftest.py
python3 scripts/qemu_biba_selftest.py
python3 scripts/qemu_audit_chain_selftest.py
python3 scripts/qemu_exec_trans_selftest.py
# ...etc — see scripts/qemu_*_selftest.py
```

On boot the kernel asks for a passphrase (used to derive the BatFS master key). Press return to use the dev passphrase `batman` (hardcoded for QEMU smoke tests via the `SPHRAGIS_DEV_PASSPHRASE` build-time env). Production builds read the passphrase from UART.

In the interactive shell:

```
help                      list every command
cave-usage                list active caves and their resource use
audit                     tail the audit log
audit-chain               verify the tamper-evident chain over the ring
mls-selftest              exercise the BLP + Biba lattice end-to-end
te-list                   show type-enforcement transition rules
heap-stats                heap-guard counters (alloc / free / corruption)
heap-guard-selftest       exercise the canary detection paths
taint-selftest            exercise the information-flow primitive
tpi-wired-ops-selftest    full M-of-2 quorum drill on destructive ops
```

## Architecture pointers

- [`DESIGN.md`](DESIGN.md) — top-level vision and architectural choices.
- [`DESIGN_BATCAVES.md`](DESIGN_BATCAVES.md) — the cave isolation model.
- [`DESIGN_CAVE_ISOLATION.md`](DESIGN_CAVE_ISOLATION.md) — per-cave MMU + state isolation.
- [`DESIGN_CRYPTO.md`](DESIGN_CRYPTO.md) — cryptographic primitive choices.
- [`DESIGN_NO_BROWSER.md`](DESIGN_NO_BROWSER.md) — why we ripped out the in-tree browser.
- [`DESIGN_TLS_HARDENING.md`](DESIGN_TLS_HARDENING.md) — TLS posture.
- [`DESIGN_HTTPS_SYSCALL.md`](DESIGN_HTTPS_SYSCALL.md) — kernel-mediated HTTPS for caves.
- [`docs/SESSION_JOURNAL.md`](docs/SESSION_JOURNAL.md) — chronological development log.

## Codebase layout

```
src/
  arch/         AArch64 exception vectors + low-level boot
  caves/      The "cave" isolation primitive — unit of policy enforcement
  crypto/       ChaCha20-Poly1305, Ed25519, SHA-256, Argon2id, X25519
  drivers/      virtio (QEMU) and apple/ (M4-specific HW bring-up)
  fs/           BatFS encrypted filesystem
  kernel/       Process, scheduler, IPC, pipes, shm, time, mm, heap-guard
  net/          TCP, UDP, TLS, WireGuard, NAT, conntrack
  security/     Audit ring + chain + seal, TPI quorum, OTP, auth, CVE audit
  ui/           Shell, console, font, GPU
```

## License

The repository is currently **private** under default copyright ("all rights reserved"). No license has been granted.

When the repository is opened to the public (planned), Sphragis will be **dual-licensed**:

- **AGPL-3.0-or-later** for research, academic citation, non-commercial use, and any project willing to comply with AGPL's source-availability clause.
- **Commercial license** sold separately for closed-source integration. The MongoDB / Sentry / GitLab playbook.

Dependencies are MIT / Apache-2.0 / BSD / CC0 / Unlicense throughout — verified clean as of 2026-05-13. No copyleft contamination.

## Contact

- **GitHub:** [@kadenlee1107](https://github.com/kadenlee1107)
- **General + commercial license inquiries:** [project email — pending Phase A]

## Development practice

Sphragis is built and maintained by Kaden Lee with extensive paired-programming assistance from Anthropic's Claude (Sonnet 4.6 / Opus 4.7). Every architectural decision, every threat-model judgement, every security trade-off, and the responsibility for what ships are mine. The AI handles boilerplate, accelerates implementation, and catches things a tired solo maintainer would miss; the strategy, the calls, and the accountability are human.

This is disclosed for two reasons: `git log` makes it obvious anyway, and the kind of solo + AI development that produced Sphragis in months — not years — is increasingly how serious security research gets done. Honest about it.

---

Sphragis — Kaden Lee · 2026

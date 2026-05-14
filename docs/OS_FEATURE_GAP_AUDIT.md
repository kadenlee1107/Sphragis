# Sphragis Feature Gap Audit

> **Mandate.** Compare Sphragis as it stands on 2026-05-10 against two bars: a
> "regular general-purpose OS" baseline (Linux / macOS / Windows-class), and a
> "government-grade high-assurance OS" baseline (SELinux MLS + seL4 + classified
> handling-class). Every item is either ✅ have, ⚠️ partial, or ❌ missing,
> with a citation to the source we're matching against. The bar for this
> document is that another AI or engineer **could not add a correct,
> non-trivial OS feature to it** — either the addition is wrong, or it's
> already listed, or it's something so esoteric that listing it would
> be noise.

---

## Part 0 — Methodology

**Sources of truth for what Sphragis has:**

- `git ls-files src/`, walked module-by-module (see Part 1 inventory).
- `docs/SESSION_JOURNAL.md` (chronological development log).
- `docs/M4_GROUND_TRUTH.md` (verified M4 hardware facts).
- `DESIGN_*.md` design documents at repo root.
- V-incident markers in source (V4, V5-XLAYER, V6-SIDE-002, V8-ROOT-*,
  V11-FRESH-EYES) — these are internal audit findings.

**Sources of truth for the "regular OS" bar:**

- POSIX.1-2017 (IEEE Std 1003.1-2017).
- Linux kernel feature inventory (Linux 6.x, 2024-2026).
- FreeBSD 14 handbook.
- macOS XNU + IOKit reference.
- Filesystem Hierarchy Standard 3.0.
- LSB 5.0 (Linux Standard Base).
- ACPI 6.5, UEFI 2.10, Devicetree spec 0.4.
- Khronos GPU APIs (Vulkan 1.3, OpenGL 4.6).
- USB-IF (USB 3.2 / 4 spec), PCI-SIG (PCIe 5.0).
- IEEE 802.11 (Wi-Fi 6/7), 802.1Q (VLAN), 802.1X (port auth).
- IETF RFCs for every networking protocol we mention.

**Sources of truth for the "government-grade" bar:**

- NIST SP 800-53 Rev. 5 (control families AC, AU, IA, SC, SI, SR, etc.).
- NIST SP 800-171 (CUI).
- NIST FIPS 140-3 (cryptographic module validation).
- NIST FIPS 199 / 200 (categorization & minimum baselines).
- Common Criteria for Information Technology Security Evaluation v3.1
  (CC EAL1 through EAL7).
- CNSSI 1253 (security categorization for national security systems).
- NIAP Protection Profiles (OS PP v4.3, MDM PP, App PP).
- DISA STIGs (Security Technical Implementation Guides) for Linux, Windows.
- DoD Impact Levels (IL2 through IL6).
- CNSA Suite 2.0 (commercial NSS cryptography, including PQ transition).
- ARM Confidential Compute Architecture, Trusted Firmware-A, Trusted
  Execution Environment specs (GlobalPlatform TEE Internal Core API).
- DO-178C (avionics software), IEC 61508 (functional safety), ISO 26262
  (automotive) — relevant for the "safety-critical OS" subset.
- CHERI / Morello capability hardware specs.
- Bell-LaPadula confidentiality model (1973), Biba integrity model (1977),
  Clark-Wilson (1987), Chinese Wall (Brewer-Nash 1989).

**Severity column legend (used in the triage table, Part 4):**

- **P0** — ship-blocker; without this the OS is unsafe or unusable.
- **P1** — needed for public release / general use.
- **P2** — needed for advertised positioning ("security-grade" / "private-client").
- **P3** — nice-to-have; differentiator but not a credibility blocker.

---

## Part 1 — Sphragis Today: Subsystem Inventory

Walked the repo source-of-truth as of `feat/ai-agent-design` HEAD. Each
subsystem heading is mirrored in [src/](../src/).

### 1.1 Boot & early kernel — [src/arch/](../src/arch/), [src/boot/](../src/boot/)

- Single-CPU bring-up in `src/arch/aarch64/boot.s`. Reads MPIDR_EL1, halts
  non-primary cores via WFE. Enables FP/NEON, SVE, SME via CPACR_EL1.
- Exception vectors in `src/arch/aarch64/exceptions.s` (2 KB aligned, 128 B
  stubs per entry). SAVE_REGS macro pushes 16 stp pairs + ELR_EL1 +
  SPSR_EL1 = 272 bytes. Handlers: sync_el1h, irq_el1h, sync_el0_64, irq_el0_64.
- `linux_header.s` for QEMU `-kernel` boot; `apple/boot.s` for M4 m1n1 chainload.
- `src/boot/devicetree.rs` parses Apple ADT, extracts uart_base, fb_base,
  fb_width, fb_height. No general property enumeration; no interrupt-binding
  extraction; no device-probe ordering.
- Verified booted on real M4 (Mac16,1 / J604 / T8132 "Donan") via m1n1 chainload.

### 1.2 Kernel core — [src/kernel/](../src/kernel/)

- **Scheduler**: priority-preemptive, 256 levels (`sched/mod.rs`). Context
  switch saves x19-x30 + sp. `block_on()` primitive landed 2026-05-08
  (`feat/scheduler-block-on`, PR #26).
- **IPC**: synchronous, capability-based rendezvous (`ipc/mod.rs`).
  `CapType` = IpcSend, IpcRecv, Memory, Interrupt, DeviceMmio. Zero-ambient-
  authority model.
- **MM**: bitmap frame allocator (`mm/frame.rs`, MAX_FRAMES=1M, 4 KB pages,
  4 GiB addressable). Linked-list kernel heap (32 MB, IRQ-guarded). 4-level
  ARM64 page tables. Multi-ELF loader from initrd.
- **Sync**: spinlock mutex with IRQ guard (`sync/mod.rs`).
- **Trap dispatch**: page-fault / instruction-abort / data-abort / SVC decoder
  with ESR_EL1 syndrome parsing.
- ❌ No SMP — secondary cores halted forever.
- ❌ No swap, no demand paging, no copy-on-write.
- ❌ No SME register save/restore across context switch (data-loss risk).

### 1.3 Drivers — [src/drivers/](../src/drivers/)

- **UART**: QEMU PL011 (0x0900_0000), Apple Dockchannel (M4 at 0x3_8812_8000).
- **Apple M4 SoC**: AIC2 interrupt controller, DART IOMMU (BYPASS mode
  verified; TRANSLATE mode partial), DCP simple framebuffer.
- **VirtIO**: GPU, keyboard, tablet — all working. blk + net are stubs.
- **PCI**: stub. **ANS NVMe**: stub. **DWC3 USB**: skipped (wrong M4 MMIO).
  **BCM Wi-Fi**: skipped (wrong M4 MMIO).
- No audio, sensors, camera, Bluetooth, GPU acceleration (Apple AGX
  unsupported), HID beyond virtio-keyboard/tablet, power management
  (no idle states, no DVFS).

### 1.4 Cryptography — [src/crypto/](../src/crypto/)

- **Symmetric**: AES-128/256-CTR + AES-128/256-GCM (RustCrypto fixslicing,
  constant-time after V8-ROOT-11 fix).
- **Hash**: SHA-256 (pure Rust), SHA-384 (RustCrypto).
- **MAC/KDF**: HMAC-SHA256/384, HKDF (RFC 5869, with RFC 8446 TLS labels).
- **CSPRNG**: SHA-256-chained, ARMv8.5 RNDR mixing, atomic CAS spinlock
  (V8-ROOT-4), post-fork poisoning detection (V11-FRESH-EYES).
- **Asymmetric**: Ed25519 (sign/verify, batch verify), ECDSA P-256
  (compressed + uncompressed), X25519 (ECDH).
- **Post-quantum**: ML-KEM-768 (FIPS 203, hybrid w/ X25519), ML-DSA-65
  (FIPS 204, hybrid w/ Ed25519). 1120-byte hybrid CT, 3373-byte hybrid sig.
- ❌ No SHA-3 family. No BLAKE2/3. No XChaCha20-Poly1305. No AES-XTS for
  disk encryption. No NIST CMAC. No constant-time bignum library beyond
  curve ops. No password hashing — Argon2 was discussed but no
  implementation found in the inventory.
- ❌ No FIPS 140-3 module boundary. No self-test on power-up. No
  approved/non-approved mode separation.

### 1.5 Networking — [src/net/](../src/net/)

- **L2**: Ethernet, ARP (16-entry cache, rate-limited, V8-ROOT-2 cave reset,
  ATTACK-NET-001 unsolicited-reply audit).
- **L3**: IPv4, ICMP (echo reply only). DHCP module exported.
- **L4**: TCP with state machine. UDP with port-53-only ingress
  (ATTACK-NET-041).
- **DNS**: plaintext UDP + DNS-over-HTTPS to 1.1.1.1. TXID randomized via
  `cntpct_el0` (V8-ROOT-3, ATTACK-NET-035 / -036). Bounds-checked parser.
- **TLS 1.3**: kernel-mediated. Cipher suites `TLS_AES_256_GCM_SHA384`,
  `TLS_AES_128_GCM_SHA256`. Hybrid post-quantum KEM (X25519 + ML-KEM-768)
  in handshake.
- **X.509**: chain validation (PR #21/#23/#24 trilogy: validity-period +
  critical-extension reject; BasicConstraints + KeyUsage + EKU; anchor-aware
  pathLen). Trust store: 6 anchors (ISRG X1/X2, Amazon CA1, DigiCert CA/G2,
  GTS R4).
- **HTTPS syscall**: `bat_https_open` / `read_pcb` / `write_pcb` / `close_pcb`
  (DESIGN_HTTPS_SYSCALL.md). Caves never see TLS plaintext.
- **Firewall**: 32-rule default-deny (`firewall.rs`). Outbound allow,
  inbound TCP listener-gated (NET2-019), inbound UDP 53-only, ICMP echo
  reply only.
- **HTTP/1.1 hardening**: 64 KB total header cap, 8 KB per line, 128 max
  headers, 4 MB chunk cap, 16 MB body cap, 30 s total deadline, 5 s idle
  (ATTACK-NET-045/-046, ATTACK-DOS-026).
- **NAT**: policy-gated nic1→nic0 forwarding with per-cave allowlist.
- **Beacon detection**: 32-flow tracking, coefficient-of-variation < 10%
  for periodicity, integer-only math, recoverable from false-positive flags.
- ❌ IPv6 — none. No IPv6 socket family.
- ❌ HTTP/2, HTTP/3, QUIC — none.
- ❌ Wi-Fi, cellular, Bluetooth, USB tethering — no drivers.
- ❌ VLAN (802.1Q), bonding/teaming, bridges, VXLAN, GRE.
- ❌ DNSSEC validation. No mDNS / Bonjour. No LLMNR.
- ❌ Native SOCKS / proxy support. No TOR / I2P integration.
- ❌ Session resumption (PSK), 0-RTT, post-handshake auth in TLS.
- ❌ Certificate pinning per host (only the 6 anchor pins exist).
- ❌ OCSP / CRL revocation checking.
- ❌ Certificate Transparency log monitoring (SCT validation).

### 1.6 Filesystems — [src/fs/](../src/fs/)

- BatFS encrypted filesystem present as `batfs` and `batfs_disk` modules.
  Detailed implementation deferred in inventory — design doc indicates
  AES-256-GCM-backed with journaling intent.
- ❌ No ext4, NTFS, FAT32, exFAT, APFS, ZFS, btrfs, xfs.
- ❌ No FUSE / userspace filesystem support.
- ❌ No NFS, SMB/CIFS, WebDAV — no remote filesystems.
- ❌ No tmpfs, overlayfs, devfs — no specialized VFS-layer filesystems.
- ❌ No POSIX ACLs, xattrs, capabilities flags on files.
- ❌ No quotas, no reflinks, no copy-on-write snapshots.

### 1.7 Security mechanisms — [src/security/](../src/security/)

- **Audit ring** (Echo): per-session correlation id, 10 categories already
  including the new `Category::Ai = 10`. Sealed under master-key AEAD.
  V-incident vocabulary documented in `Concepts/V-Incident Vocabulary.md`.
- **Wipe**: `wipe::execute(WipeReason, force)` destroys keys and plaintext
  buffers. Triggered by Ctrl+W hotkey, dead-man's switch, or manual command.
- **Zeroize**: volatile overwrite of sensitive memory.
- **Dead-man's switch**: `periodic_check()` in main loop fires wipe on expiry.
- Modules also exported: `auth`, `boot_screen`, `origin`, `otp` — implementation
  details deferred but module surface present.
- ❌ No MAC framework (SELinux/AppArmor/SMACK class).
- ❌ No POSIX capabilities. (We have a *capability* IPC type but not the
  POSIX process-capability model with CAP_NET_BIND_SERVICE etc.)
- ❌ No seccomp / syscall-filter language richer than the cave whitelist.
- ❌ No integrity measurement architecture (IMA / EVM).
- ❌ No mandatory file labelling.
- ❌ No trusted-path / secure-attention-key.

### 1.8 Caves — [src/caves/](../src/caves/)

- Isolation sandboxes with per-cave memory, per-cave audit session id,
  syscall whitelist, kernel-mediated networking.
- Whitelisted syscalls: read, write, open, close, ioctl (net), mmap,
  munmap, brk, exit, `bat_https_open`. Blocked: raw sockets,
  fork/clone, execve, ptrace.
- IPC: `batpipe` (inter-cave), `secure_channel` (TLS 1.3), `secure_ipc`
  (async messaging).
- Persistence: per-cave encrypted storage via `persist` module.
- Kits: containerized app manifests (browser-class apps, email,
  etc.) — design level, schema details deferred.
- Docker client over HTTPS syscall for remote container API interaction.
- ❌ No live migration of caves.
- ❌ No cave resource quotas (CPU share, memory limit, IO limit) —
  the cgroups equivalent.
- ❌ No cave network namespaces with full L2 isolation.
- ❌ No cave-to-cave shared-memory primitives beyond batpipe message-passing.

### 1.9 User-space — [src/ui/](../src/ui/)

- **Shell**: command REPL, tab completion (PR #18), arrow-key history
  (PR #19), argument completion (PR #20).
- **Desktop**: window manager, draw primitives, GPU abstraction,
  TrueType font rendering.
- **Apps**: pre-installed application surface — full list deferred but
  enumerated as a module.
- ❌ No POSIX shell (bash/zsh) compatibility — our shell is a one-off REPL.
- ❌ No pipes / redirection / job control.
- ❌ No man-page system.
- ❌ No init system on the SysV / systemd / launchd model — just `main.rs`.
- ❌ No package manager.
- ❌ No locale support / NLS / gettext / Unicode normalization tooling.
- ❌ No accessibility services (screen reader, magnifier, voice control).

### 1.10 AI agent — [src/ai/](../src/ai/) (new, this session)

- `AgentSession` lifecycle (mod.rs), narrow `AgentError` enum (Network,
  Protocol, Tool, TokenBudget, PolicyDenied, Interrupted).
- Hand-rolled JSON serializer + SSE parser (protocol.rs, no `serde`).
- 6 read-only tools defined: `read_file`, `grep_source`,
  `query_audit_ring`, `suggest_command`, `read_concept_note`, `list_caves`.
  Schemas + dispatch in `tools.rs`.
- RAG: 18-doc compile-time corpus (10 Concept notes + 8 DESIGN docs),
  precomputed BM25 IDF table, FNV-1a 64-bit term hashing, `src/ai/rag.rs`
  runtime + `src/ai/rag_corpus.rs` auto-generated.
- Pinned-cert SHA-256 policy slot, constant-time bytewise comparison.
- Streaming via line-buffered SSE framer (stream.rs), 16 KB defense cap.
- Tied to ollama on a local 5070 host through `bat_https_open`.
- v1 LoRA training done; v2 (this session) in progress with DoRA +
  NEFTune + packing + rank-64 + 7,244-record corpus.

### 1.11 Build, test, observability

- Cargo workspace targets `aarch64-unknown-none`.
- Linker script + nightly toolchain with `-Zfixed-x18`.
- Selftest-on-boot Cargo feature (PR includes scheduler-selftest).
- QEMU smoke harnesses in `scripts/qemu_*.py` (50+ smoke tests).
- 46-question eval harness for the AI model (`evals/`).
- Obsidian vault sync via git hook (`scripts/sync_obsidian.py`).
- ❌ No fuzz harness in tree. No KASAN-class instrumentation. No
  KMSAN. No coverage-guided test runner. No CI pipeline visible
  in the repo.
- ❌ No `dmesg`-equivalent ring beyond the audit ring.
- ❌ No `top` / `vmstat` / `iostat` / `netstat` — no per-subsystem
  metric exporters.

---

## Part 2 — Standard "Regular OS" Bar

Each subsection lists what a baseline general-purpose OS (Linux,
macOS, Windows, FreeBSD) has, what Sphragis has against it, and what
the gap is.

### 2.1 Boot

**Industry baseline:** UEFI Secure Boot, signed bootloader chain
(shim → GRUB → kernel), measured boot into TPM PCRs, initramfs with
LUKS unseal, multi-OS selection.

✅ Have: m1n1 chainload to bare-metal Rust kernel on M4. Generic
aarch64 entry for QEMU.
❌ Missing:
- Multi-boot loader (no GRUB/systemd-boot equivalent).
- Recovery / safe-mode partition.
- Bootloader signing chain (we're a one-binary chainload; not vouched).
- TPM measurement of the boot chain (M4 has Secure Enclave but we
  don't talk to it).
- A/B partition scheme for atomic updates.
- Kernel command-line arg parsing.

### 2.2 Process & threading

**Baseline:** `fork`, `clone`, `execve`, `wait`, `signal`, `sigaction`,
process groups, sessions, controlling terminal, `pthread`-class POSIX
threads with futex-backed sync, priority inheritance, robust mutexes.

✅ Have: priority-preemptive scheduler, capability-based IPC, futex
unification (per-cave futex table, V8-ROOT-1).
❌ Missing:
- No `fork`/`clone`/`execve`. Caves are not POSIX processes.
- No POSIX signals at all.
- No pthreads. No condition variables. No barriers.
- No process groups / sessions / controlling-terminal semantics.
- No priority inheritance on the kernel mutex.
- No CPU affinity control. No NUMA awareness.

### 2.3 Memory

**Baseline:** demand paging, swap to disk, mmap with file backing,
shared memory (POSIX shm + SysV shm), CoW after fork, huge pages, KSM
or page-deduplication, NUMA-aware allocation, slab allocator with
caches, kmalloc/vmalloc split, OOM killer with score adjustment.

✅ Have: 4-level page tables, bitmap frame allocator, linked-list
heap, kernel/user separation enforced by PT_USER + PT_UXN/PT_PXN.
❌ Missing:
- No swap. No demand paging.
- No `mmap` syscall in any form.
- No POSIX/SysV shared memory.
- No huge pages.
- No CoW.
- No OOM killer; we'll just panic on heap exhaustion.

### 2.4 Filesystems

**Baseline:** VFS layer + 5+ on-disk filesystems (ext4 / xfs / btrfs /
zfs on Linux; APFS / HFS+ on macOS; NTFS / ReFS on Windows). FUSE.
Network filesystems (NFS, SMB). Tmpfs, devfs, procfs, sysfs, overlayfs.
POSIX ACLs, xattrs, capabilities.

✅ Have: BatFS (encrypted, AES-256-GCM-class) as the only fs.
❌ Missing: everything else listed above. No procfs/sysfs equivalent
makes runtime introspection difficult.

### 2.5 Networking

**Baseline:** IPv4 + IPv6, TCP/UDP/SCTP, raw sockets (with cap),
ICMP/ICMPv6 (full echo/unreachable/redirect handling), netlink for
configuration, iptables/nftables/pf packet filter, traffic control
(qdisc), tunneling (GRE, IPIP, VXLAN), VLANs, bridges, bonding, IPSec,
WireGuard, OpenVPN, IPv4/v6 conntrack, NAT (full, with PAT and
hairpinning), DHCP client+server, DNS resolver (with EDNS, DNSSEC),
mDNS, multicast (IGMP/MLD), 802.11 wireless, 802.1X port authentication,
802.1Q VLANs, 802.1AE MACsec, network namespaces / VRFs.

✅ Have: IPv4, TCP, UDP, ICMP echo, ARP, DNS (plaintext + DoH), TLS 1.3
(hybrid PQ), X.509 chain (6 anchors), HTTP/1.1 (hardened client), default-
deny firewall (32 rules), beacon detection, simple NAT.
❌ Missing:
- IPv6 in entirety.
- HTTP/2, HTTP/3, QUIC.
- DNSSEC, DoT, DoQ.
- mDNS / Bonjour / SSDP.
- Wi-Fi, cellular, Bluetooth, NFC.
- SCTP, DCCP, MPTCP.
- WireGuard / IPSec / OpenVPN.
- VLAN, bridge, bonding, VRF.
- IGMP/MLD multicast.
- Conntrack-class stateful firewall (we have stateless rule match).
- Anti-spoofing (uRPF), reverse-path filtering.
- ICMP rate-limit beyond what's coded statically.
- 802.1X / EAP.
- DHCP server. (We export a `dhcp` module — client only likely.)
- Local sockets / Unix domain sockets.
- AF_PACKET / raw socket family.

### 2.6 Drivers — storage

**Baseline:** NVMe, SATA AHCI, SCSI, MMC/eMMC/SD, USB Mass Storage,
loop devices, software RAID (md), device-mapper (LVM, dm-crypt, dm-verity).

✅ Have: VirtIO blk (stub).
❌ Missing: everything else listed.

### 2.7 Drivers — input

**Baseline:** PS/2 keyboard/mouse, USB HID, evdev abstraction, libinput
gesture recognition, touch (multi-touch), pen/stylus, sensors (gyro,
accelerometer, ambient light).

✅ Have: VirtIO keyboard, VirtIO tablet. M4 has no HID coverage at all.
❌ Missing: USB HID class driver, evdev-class abstraction, multi-touch,
pen/stylus, ambient sensors.

### 2.8 Drivers — display

**Baseline:** DRM/KMS with mode-setting, multiple framebuffers, GPU
acceleration (Vulkan/OpenGL), HDR, VRR/FreeSync, multi-monitor,
display rotation, color management (ICC profiles).

✅ Have: simple framebuffer via VirtIO GPU or DCP.
❌ Missing: Apple AGX driver. No mode-setting. No GPU acceleration.
No multi-monitor. No HDR / VRR / color management.

### 2.9 Drivers — audio / video / camera

**Baseline:** ALSA / PulseAudio / PipeWire (Linux), CoreAudio (macOS).
V4L2 cameras, USB audio class. Codec support (AAC, H.264, H.265, AV1,
VP9, Opus, FLAC).

✅ Have: nothing.
❌ Missing: everything.

### 2.10 Drivers — USB / PCI / Thunderbolt

**Baseline:** USB 3.2 / USB4 host controllers (xHCI), USB device
enumeration + class drivers, PCIe enumeration with hot-plug, MSI/MSI-X,
Thunderbolt with daisy-chain, IOMMU passthrough for VMs.

✅ Have: PCI stub. DART IOMMU partial (BYPASS verified). DWC3 USB:
skipped on M4 because addresses were wrong.
❌ Missing: working USB stack, working PCIe stack, MSI-X, hot-plug.

### 2.11 Power management

**Baseline:** ACPI/UEFI runtime (Linux/Windows) or Apple SMC (macOS).
Suspend-to-RAM (S3), suspend-to-disk (S4), runtime PM, DVFS, P-states,
C-states, thermal management, battery state, brightness/backlight,
fan control, lid switch, power button.

✅ Have: nothing — we never idle.
❌ Missing: everything.

### 2.12 Time

**Baseline:** monotonic clock (CLOCK_MONOTONIC), real-time clock with
NTP/PTP sync, hardware RTC, time zones, leap-second handling, hi-res
timer, alarm clocks (timerfd).

✅ Have: `cntpct_el0` / `cntfrq_el0` monotonic tick. Used for
randomization (DNS TXID), deadline detection (HTTP), beacon period.
"Time Without a Clock" Concept note documents the absence of a
wall clock.
❌ Missing: wall clock, NTP/PTP, time zones, leap seconds, alarm
timers in syscall surface.

### 2.13 IPC

**Baseline:** pipes, FIFOs (named pipes), Unix domain sockets, SysV/POSIX
shared memory, message queues (POSIX mq, SysV msg), semaphores, D-Bus
or kdbus, mach ports, GraphQL-class IPC bus.

✅ Have: capability-based synchronous IPC (kernel/ipc), batpipe inter-
cave transport.
❌ Missing: pipes, FIFOs, AF_UNIX sockets, shared memory, POSIX
message queues, D-Bus.

### 2.14 Containers / sandboxes

**Baseline:** Linux namespaces (pid, net, mount, user, ipc, uts, cgroup),
cgroups v2 (cpu, memory, io, pids), seccomp-bpf filters, OCI runtime
(runc / crun), Docker / Podman / containerd. macOS App Sandbox /
hypervisor framework. Windows Containers / Hyper-V isolation.

✅ Have: Caves with per-cave audit, per-cave firewall, per-cave
syscall filter (whitelist).
❌ Missing:
- No cgroups-equivalent quota system.
- No PID namespace (caves are processes; no isolation of pid space).
- No mount namespace (no filesystem isolation).
- No user namespace (no uid mapping).
- No OCI runtime compatibility — can't run Docker images directly.
- No image format / layered FS.

### 2.15 Virtualization

**Baseline:** KVM (Linux), HVF (macOS), Hyper-V (Windows). Paravirtualized
drivers (virtio). Live migration. Snapshotting. Nested virt. PCIe
passthrough via IOMMU. SR-IOV.

✅ Have: m1n1 hypervisor stub (`mmu_el2.rs`), VirtIO consumer-side.
❌ Missing: no hypervisor host capability. We're a guest, not a host.
No live migration. No snapshots. No nested virt.

### 2.16 Updates / lifecycle

**Baseline:** Package manager (apt/dnf/zypper/pacman/brew/winget),
signed packages, dependency resolution, A/B updates (ChromeOS, Android),
delta updates, rollback, autoupdate daemon, signed metadata
(in-toto / TUF).

✅ Have: nothing.
❌ Missing: everything.

### 2.17 Observability

**Baseline:** dmesg / kmsg ring, journald / syslog, perf-events,
ftrace, eBPF, /proc, /sys, /metrics endpoints, OpenTelemetry hooks,
core dumps with backtrace.

✅ Have: Echo audit ring (10 categories, sealed). Selftest-on-boot
prints to UART.
❌ Missing: dmesg-equivalent for non-security events. No tracing.
No perf-events. No /proc-equivalent. No core dumps.

### 2.18 Standards compliance

**Baseline:** POSIX.1-2017 conformance, LSB, FHS, SVID. ELF binaries.
Standard libc (glibc / musl / Bionic). Standard tools (coreutils,
bash, grep, awk, sed, find).

✅ Have: nothing on the standards side. Caves are not POSIX.
❌ Missing: POSIX, ELF support, libc, standard tools.

### 2.19 Internationalization

**Baseline:** Unicode 15.x throughout (UTF-8 default, normalization
forms NFC/NFD/NFKC/NFKD), locale (LANG, LC_ALL), gettext-class i18n,
input method editors (IBus, fcitx), bidirectional text (Arabic, Hebrew),
complex script shaping (Indic, CJK).

✅ Have: probably ASCII-only across the codebase.
❌ Missing: everything else.

### 2.20 Accessibility

**Baseline:** screen reader (Orca, VoiceOver, Narrator), screen
magnifier, high-contrast themes, sticky keys / slow keys, on-screen
keyboard, voice control / dictation.

✅ Have: nothing.
❌ Missing: everything.

### 2.21 Desktop applications

**Baseline:** browser, email client, calendar, contacts, text editor,
file manager, terminal, image viewer, music player, video player,
calculator, system preferences, screenshot tool, clipboard manager.

✅ Have: shell. Browser was rejected (Post-no-browser Pivot, by design).
Kits framework exists but kit catalog is empty/minimal.
❌ Missing: most of the above. Strategic position is that
end-user browsing lives on the operator's host machine, not in Sphragis.

### 2.22 Build / dev tooling

**Baseline:** package manager + build tools (apt, brew, cargo registry),
debugger (gdb / lldb), profiler (perf, instruments), tracer (strace,
dtrace), static analyzer (clang-tidy, RustC lints), formal-methods
toolchain optional.

✅ Have: cargo workspace, QEMU smoke harness, clippy clean, selftest
on boot.
❌ Missing: in-OS debugger, profiler, tracer. No remote debug hooks.

---

## Part 3 — Government-Grade Bar

Everything above plus the following. Numbered for cross-reference in
the triage table.

### 3.1 Assurance & certification — frameworks

**Baseline:** Common Criteria EAL4+ minimum for general government use,
EAL6+ for high-assurance (intelligence-class). NIAP Protection Profile
conformance (OS PP v4.3 currently). NIST FIPS 199 categorization +
FIPS 200 minimum baselines. DoD Impact Levels — IL2 (public), IL4 (CUI),
IL5 (NSS classified up to SECRET), IL6 (SECRET classified processed).
DISA STIGs as concrete configuration baseline.

✅ Have: a credible audit posture (V-incident vocabulary, constant-cost
abort discipline, attack-marker comments throughout), but **no formal
evaluation** of any kind. Zero certifications.
❌ Missing:
- No CC ST (Security Target) document; no EAL claim.
- No FIPS 140-3 module (the crypto isn't packaged as a validated module,
  with self-tests on power-up, approved-mode separation, etc.).
- No NIAP PP conformance audit.
- No DISA STIG configuration baseline.
- No FedRAMP, no IRAP, no Cyber Essentials, no ISO 27001, no
  SOC 2 controls — no third-party attestations.

### 3.2 Mandatory Access Control & Multi-Level Security

**Baseline:** SELinux / SMACK / AppArmor type enforcement; Bell-LaPadula
confidentiality lattice; Biba integrity lattice; Clark-Wilson well-formed
transactions; Chinese Wall conflict-of-interest separation. Multi-level
security with TS / S / C / U sensitivity labels, compartments, and
information-flow enforcement at every system call. SECMARK (label network
packets per source category). Cross-Domain Solutions for TS ↔ S
movement.

✅ Have: cave isolation as a coarse compartment (per-cave audit, firewall,
syscall filter). Default-deny posture. **Two-axis MLS lattice (2026-05-12)**:
  - Bell-LaPadula confidentiality: `cave::Sensitivity` (U/C/S/TS) +
    `can_flow(subject, object, op)` for no-read-up / no-write-down.
  - Biba integrity: `cave::Integrity` (U/SB/ST/HI) +
    `can_flow_integrity(subject, object, op)` for no-read-down /
    no-write-up. Dual to BLP — high-integrity caves refuse low-integrity
    input even when BLP would allow it.
  - Both labels propagate: `batfs::ns_create` stamps creator's
    `(sens, integ)` onto the file; `batfs::ns_read` enforces both
    properties (`Err("mls: no read-up")` / `Err("mls: no read-down")`).
  - **Labels are AEAD-bound** (2026-05-13): the file's
    sensitivity + integrity bytes are part of the ChaCha20-Poly1305
    AAD at encrypt time. A byte-flip on either label at rest is
    rejected at decrypt time with `INTEGRITY VIOLATION — file
    tampered or label flipped` — a memory-corrupting attacker
    can't downgrade a file's label to bypass the lattice checks
    without also re-encrypting (which needs the file key).
    `mls-binding-selftest` proves the property.
  - `caves::mls_ipc` labeled mailbox: enforces BLP write-down +
    Biba write-up on send, BLP read-up + Biba read-down on recv
    (belt-and-suspenders for runtime label changes between
    send and recv).
  - Operator API: `mls-set`, `mls-show`, `mls-check`, `integ-set`,
    `integ-show`. Selftests: `mls-selftest`, `mls-ipc-selftest`,
    `biba-selftest`.
❌ Missing:
- Labeled networking is partial — **CIPSO IP option emit + parse
  shipped (2026-05-13)**: `ip::send` injects a CIPSO option
  carrying the active cave's sensitivity byte whenever it's
  non-Unclassified (DOI = "BBOS" = 0x42424F53);
  `ip::parse_cipso_sensitivity` extracts the level for
  receiver-side checks. Still missing: receiver-side enforcement
  (today the byte is on the wire, but a receiving Sphragis
  doesn't gate delivery on it), CALIPSO (IPv6 equivalent),
  SECMARK rule integration.
- Type enforcement: **transition allow-list shipped (2026-05-13)**.
  Each cave acts as its own domain; `cave::can_transition(from, to)`
  + `add_transition_rule(from, to)` form a default-deny matrix.
  `cave::enter` consults it when `te-enable` is on. Admin/kernel
  context can transition anywhere; non-admin transitions require
  explicit allow-list entries. Shell API: `te-enable`,
  `te-allow <from> <to>`, `te-list`, `te-clear`. `te-selftest`
  proves the policy round trip. Still missing vs. real SELinux:
  domain-typed objects (we have BLP+Biba for objects but not
  domain types), rich rule predicates (operation classes,
  permissions), policy compilation toolchain.
- Compartments / categories (we have linear lattices; CC needs
  lattice + categories for SCI handling).
- Cryptographic binding between MLS labels and the data they label
  (a future pass would AEAD-bind the labels so a race attacker
  can't smuggle one body under another's label).

### 3.3 Trusted computing / attestation

**Baseline:** TPM 2.0 / Apple Secure Enclave / vTPM. Measured boot
into PCRs. SealedBlob (TPM-sealed key bound to platform state).
Remote attestation (DAA, EPID, or X.509-attestation). RA-TLS.
Confidential computing TEE (Intel SGX / TDX, AMD SEV-SNP, ARM CCA,
Apple Secure Enclave). Anti-rollback monotonic counters.

✅ Have: nothing. The Apple M4 has a Secure Enclave; we don't talk
to it.
❌ Missing: TPM/SE interaction, measured boot, sealed storage,
remote attestation, TEE awareness.

### 3.4 Cryptographic agility & quantum readiness

**Baseline:** CNSA Suite 2.0 — required PQ migration for NSS by ~2030.
Algorithm negotiation surface allows in-place swap of primitives.
Hybrid-mode handshakes during transition. NIST FIPS 203 (ML-KEM),
FIPS 204 (ML-DSA), FIPS 205 (SLH-DSA) all approved.

✅ Have: hybrid X25519 + ML-KEM-768 in TLS handshake. Hybrid Ed25519
+ ML-DSA-65 signatures available.
❌ Missing:
- FIPS 140-3 validated implementation of any of these.
- ML-DSA / SLH-DSA in the certificate path (X.509 still classical-only).
- Algorithm-negotiation surface in the syscall ABI for caves.
- Rotation API for the trust anchors.
- Pluggable curve / hash / KDF — we hardcode choices.

### 3.5 Memory safety & exploit mitigations

**Baseline:** W^X / NX, ASLR (kernel + user, fine-grained), KASLR,
shadow stack (Intel CET, ARM GCS), Control-Flow Integrity (forward
+ backward edge), Indirect Branch Tracking (Intel IBT, ARM BTI),
Pointer Authentication (ARM PAC), Memory Tagging Extension (ARM
MTE), GWP-ASan, KFENCE, KASAN, hardened malloc (scudo / mimalloc-secure),
sandboxed JIT, JIT codesigning, stack canaries (per-function),
SafeStack, RetGuard, FineIBT, CFG (Control Flow Guard).

✅ Have: W^X via PT_UXN/PT_PXN. Rust memory safety baseline. No `unsafe`
audits that we surface in metrics. Constant-time crypto (RustCrypto
fixslicing post V8-ROOT-11).
❌ Missing:
- ASLR / KASLR.
- Stack canaries (Rust uses some but we don't audit).
- CFI / forward-edge / backward-edge.
- ARM PAC (M4 supports — we don't enable).
- ARM BTI (M4 supports — we don't enable).
- ARM MTE (M4 supports — we don't enable).
- KFENCE / KASAN equivalent.
- GWP-ASan / canary heap pages.
- Hardened malloc patterns (segregated, randomized).
- ROP/JOP-class mitigations beyond what Rust gives by default.

### 3.6 Side-channel resistance

**Baseline:** Constant-time crypto. Spectre v1/v2 mitigations
(retpolines, LFENCE / SB barriers, BHB clearing). Meltdown mitigation
(KPTI). MDS / SRBDS / Zenbleed-class workarounds. Cache-partitioning
(Intel CAT, ARM MPAM). Constant-time decryption (Lucky 13). Frequency
scaling against frequency-modulation attacks (Hertzbleed). Branch
predictor isolation across security domains.

✅ Have: constant-time AES (fixslicing). Constant-cost abort
discipline (Concept note documents the pattern: flag-accumulator,
always do the same work regardless of failure path).
❌ Missing:
- KPTI-equivalent (we share TTBR0 with EL0 indirectly).
- Spectre v1 / v2 / BHB clearing on context switch.
- Mitigations advertised in CPU control registers.
- Cache partitioning.
- Frequency-attack-aware mode.

### 3.7 Audit & forensics
*(2026-05-12 update: `audit::record` now updates a per-entry sha256
chain link via `audit_chain::append_chain`; `audit-chain` shell
verifies; `audit-chain-selftest` proves tamper at any byte of any
entry surfaces as `FirstMismatchAt(idx)`. Sealing `chain_head()`
off-platform is what extends the property past one ring cycle —
deferred to the next pass.)*

**Baseline:** Linux auditd / Windows ETW / macOS Endpoint Security
Framework. Tamper-evident logs (append-only, signed). Forwarding to
SIEM (rsyslog, Splunk UF, Elastic, Wazuh). Indicator of Compromise
matching. Live forensic acquisition (volatility-class memory dump).
Mandatory event categories (login, privilege escalation, file open,
network conn, crypto op).

✅ Have: Echo audit ring with 10 categories, sealed with master-key
AEAD, per-session correlation, V-incident vocabulary.
❌ Missing:
- Forwarding to external SIEM.
- Tamper-evident chain (hash-linked).
- Live memory acquisition / forensic dump.
- Network event audit at packet level (we have flow-level).
- File operation audit beyond cave boundary.
- IOC matching engine.

### 3.8 Tamper resistance (software + hardware)

**Baseline:** Anti-debug (TracerPid checks, PT_DENY_ATTACH, hypervisor
detection), code signing (Apple notarization, Windows AppLocker),
secure firmware update (anti-rollback monotonic counters),
self-protecting boot chain (immutable initial code, fused trust root),
physical tamper-evident enclosure with switches/seals (FIPS 140-3
level 3+ requires this).

✅ Have: zeroize on wipe, dead-man's-switch, Ctrl+W panic-wipe hotkey.
❌ Missing:
- Anti-debug.
- Self-decrypting / self-encrypting binary.
- Firmware anti-rollback.
- Tamper-evident enclosure integration (would require hw partner).

### 3.9 Confidential computing / TEE

**Baseline:** Intel SGX enclaves, TDX VM. AMD SEV / SEV-ES / SEV-SNP.
ARM Confidential Compute Architecture / Realms / TrustZone. Apple
Secure Enclave. Memory encryption at the controller (TME, AMD SME).
Remote attestation of enclave/realm state.

✅ Have: nothing.
❌ Missing: enclave / realm support, encrypted RAM, attestation.

### 3.10 Air-gap / cross-domain

**Baseline:** Data diodes / one-way transfers (Owl, Waterfall, etc.).
Cross-Domain Solutions (CDS) for moving content between classification
levels. Sneakernet over read-only media. Tempest shielding awareness
(EMSEC). USB sanitization gateway (data diode + format filter).

✅ Have: nothing.
❌ Missing: everything. (Post-no-browser pivot is in this spirit but
isn't a CDS.)

### 3.11 Supply chain integrity

**Baseline:** Software Bill of Materials (SBOM, SPDX / CycloneDX
formats). Reproducible builds (bitwise-deterministic). Signed
release artifacts (Sigstore / cosign / SLSA Level 4). In-toto
attestation chains. Sigstore Rekor transparency log. Hardware Root of
Trust (HRoT) for signing. Verifiable build environment (gitian-class).
Anti-typosquat dependency review.

✅ Have: cargo.lock pinning, vendored dependencies in `external/`,
**CycloneDX SBOM** generator (`scripts/gen_sbom.py`),
**reproducible builds** verified bit-identical via
`scripts/repro_build.sh`, **Ed25519 signatures** on every
release artifact (`scripts/sign_release_artifacts.py sign`)
with sidecar `.sig` files, **tamper-evident transparency log**
(`transparency.log` — each entry chains via sha256(prev_entry)
so an attacker can't silently rewrite past releases).
❌ Missing:
- Real Sigstore Rekor integration (we have a LOCAL hash-linked
  log; a public log with inclusion proofs is the next
  escalation).
- In-toto attestations on the build steps themselves
  (we attest the OUTPUT artifacts, not each step).
- Hardware Root of Trust for signing (the signing key is a
  file on the build host; HSM/Yubikey backing is the next pass).

### 3.12 Insider threat / Data Loss Prevention

**Baseline:** DLP agents that classify documents (PCI, PHI, PII, CUI),
block exfiltration via channels (clipboard, USB, network, print). User
and Entity Behavior Analytics (UEBA). Anomaly detection on file access
patterns. Print watermarking with user/time. Screen capture restriction.

✅ Have: beacon detection on the network side (one channel).
❌ Missing: everything else. Clipboard/USB/print don't exist as
plumbing so the DLP holes are theoretical.

### 3.13 Identity & MFA

**Baseline:** PIV / CAC smart card support (PKCS#11 token, OPENSC),
PKCS#11 hardware tokens (YubiKey, Nitrokey), FIDO2 / WebAuthn, Kerberos,
LDAP / Active Directory join, single-sign-on (SAML, OIDC), TOTP / HOTP,
biometric (fingerprint, face).

✅ Have: `otp` module exported (TOTP/HOTP detail deferred). `auth` module
exported (detail deferred).
❌ Missing: smart card stack, FIDO2, Kerberos, LDAP join, SSO,
biometric.

### 3.14 Real-time / safety-critical

**Baseline:** PREEMPT_RT or QNX-class hard-real-time. Bounded interrupt
latency. Priority inheritance protocol. Memory locking (`mlock`).
Watchdog hierarchies. ARINC 653 partitioning (avionics). DO-178C
qualification toolchain. IEC 61508 SIL3/4 evidence.

✅ Have: priority-preemptive scheduler with 256 levels.
❌ Missing: priority inheritance, mlock, watchdog hierarchy, ARINC 653,
DO-178C qualification.

### 3.15 Information-flow control

**Baseline:** Asbestos / HiStar / Flume / RIFLE / IFC at the OS level.
Static analysis: JIF (Java Information Flow), Paragon, FlowCaml. Taint
tracking. Capability-secure programming (Capsicum on FreeBSD).
Object-capability machines (KeyKOS, EROS, Coyotos, seL4).

✅ Have: capability-based IPC at the kernel level.
❌ Missing: information flow on the data path; taint propagation;
formal IFC guarantees.

### 3.16 Anti-coercion / duress

**Baseline:** Plausibly-deniable encryption (VeraCrypt hidden volumes),
duress passwords (different password reveals decoy data), tamper-wipe on
removal of dongle/badge, cold-boot RAM mitigations (memory encryption,
auto-zero on suspend), evil-maid mitigations (TPM-bound full-disk
encryption with measured boot).

✅ Have: Ctrl+W panic-wipe, dead-man's switch.
❌ Missing: duress passwords, hidden-volume mode, cold-boot mitigation,
evil-maid protection.

### 3.17 Hardware security peripherals

**Baseline:** HSM integration (PKCS#11, KMIP, KMS). Secure element (TPM,
Apple SE, Google Titan, NXP A71CH). PUF for device identity. Smart cards
(PIV, CAC). Hardware RNG NDRBG with health tests (NIST SP 800-90B).

✅ Have: ARMv8.5 RNDR mixing in the CSPRNG (one entropy source).
❌ Missing: PKCS#11, KMIP, secure element interaction, PUF identity,
SP 800-90B health tests.

### 3.18 Compliance & assessment toolchain

**Baseline:** OSCAL machine-readable controls catalog. SCAP / OpenSCAP
benchmarks. ATO package generators (eMASS, Xacta).
Continuous-Monitoring (ConMon) feeds. POA&M tracking.

✅ Have: nothing.
❌ Missing: everything.

### 3.19 Formal verification

**Baseline:** seL4 microkernel has full functional-correctness proof in
Isabelle/HOL. CompCert C compiler. RustBelt for Rust unsafe. F\* /
Vale for crypto. Lean4 / Coq for protocol verification.

✅ Have: we're written in Rust (memory safety baseline), no formal proofs.
❌ Missing: any machine-checked correctness proof.

### 3.20 PKI lifecycle

**Baseline:** ACME (RFC 8555) for cert issuance. CMP / EST / SCEP for
enterprise PKI. CT log monitoring. CRL + OCSP revocation. OCSP stapling.
Cert pinning (HPKP / TACK). Subordinate-CA hierarchy. Cross-signing.
HSM-backed CA. Key ceremony procedures.

✅ Have: 6 hardcoded trust anchors. Anchor-aware pathLen enforcement
(PR #24).
❌ Missing: ACME / EST / CMP / SCEP, OCSP / CRL, CT log monitoring,
runtime cert pinning, sub-CA hierarchy.

### 3.21 Anti-debug / hardened mode

**Baseline:** Anti-debug for sensitive processes (TracerPid, PT_DENY,
hypervisor detection, timing-based detection). Self-encrypting binaries
(packed sections decrypted on load). VM/sandbox detection for malware
analysis. Function-level binary diversification (different layout per
device).

✅ Have: nothing — by design the system is meant to be auditable.
Tradeoff is explicit.
❌ Missing: see above. Note: this is debatable whether the
government-grade bar even *wants* anti-debug in a system the operator
controls.

### 3.22 Hardware Wi-Fi / cellular for SCIF environments

**Baseline:** All radios kill-switched in hardware (Faraday-cage
SCIF requirement). Soft radio-off doesn't count.

✅ Have: no Wi-Fi / cellular drivers, which trivially satisfies this
(but for the wrong reason).
❌ Missing: drivers + a verifiable hardware radio kill switch.

### 3.23 Insider-collusion resistance

**Baseline:** Two-person integrity (TPI) for critical operations
(crypto-officer + audit-officer co-sign). Separation of duties enforced
by the OS. M-of-N threshold cryptography for high-value keys (Shamir
secret sharing, threshold signatures).

✅ Have: nothing.
❌ Missing: TPI, threshold crypto.

### 3.24 Continuous monitoring & vulnerability mgmt

**Baseline:** CVE feed ingestion (NVD, OSV), vulnerability scanning
(Nessus, OpenSCAP), patch deployment dashboard, exploitability scoring
(EPSS), zero-day reaction playbooks, coordinated disclosure program.

✅ Have: nothing visible.
❌ Missing: everything.

### 3.25 Specialized communications

**Baseline:** HAIPE (High Assurance Internet Protocol Encryptor) for
classified networks. SCIP (Secure Communications Interoperability
Protocol). Type-1 encryptor support. JOSE / IPsec with NIAP-PP
ciphersuites only.

✅ Have: nothing.
❌ Missing: HAIPE / SCIP / Type-1. (These need NSA-certified hardware;
not a pure-software bar.)

---

## Part 4 — Master Triage Table

One row per gap. Pri P0 = ship-blocker, P1 = release-blocker for
broad use, P2 = positioning-blocker, P3 = differentiator.

| # | Pri | Bar | Subsystem | Item | Standard | Effort | Notes |
|---|---|---|---|---|---|---|---|
| 001 | P0 | Reg | Boot | TPM-measured boot / Apple SE attestation | TCG TPM 2.0, Apple SE | XL | M4 has SE; we don't talk to it |
| 002 | P0 | Reg | Boot | Recovery partition / safe mode | — | M | Bare metal currently has no fallback |
| 003 | P0 | Reg | Boot | A/B partition for atomic updates | ChromeOS-class | L | Needed before any auto-update story |
| 004 | P0 | Reg | Boot | Kernel command line parser | Linux | S | Trivial, missing |
| 005 | P0 | Reg | Kernel | SMP — secondary CPU bring-up | — | XL | M4 has 10 cores; we use 1 |
| 006 | P0 | Reg | Kernel | POSIX signals | POSIX.1 | L | Many libraries assume signals |
| 007 | P0 | Reg | Kernel | fork/clone/execve | POSIX.1 | XL | Or commit to NO-FORK model permanently |
| 008 | P0 | Reg | MM | Demand paging | — | XL | We allocate up front |
| 009 | P0 | Reg | MM | mmap syscall | POSIX | L | Many runtimes need this |
| 010 | P0 | Reg | MM | OOM killer | Linux | M | Currently we panic |
| 011 | P0 | Reg | FS | At least one general-purpose FS | ext4/btrfs/apfs | XL | BatFS is the only thing |
| 012 | P0 | Reg | FS | VFS layer with mount points | POSIX | L | No mount() syscall |
| 013 | P0 | Reg | Net | IPv6 | RFC 8200 | XL | Internet runs on it |
| 014 | P0 | Reg | Net | DHCP client | RFC 2131 | M | Module exported, implementation level unclear |
| 015 | P0 | Reg | Net | mDNS / link-local discovery | RFC 6762 | M | Standard requirement |
| 016 | P0 | Reg | Drivers | NVMe driver | NVMe 2.0 | XL | M4 has internal NVMe; ANS driver is a stub |
| 017 | P0 | Reg | Drivers | USB stack (xHCI + class drivers) | USB 3.2 | XL | DWC3 was skipped on M4 |
| 018 | P0 | Reg | Drivers | Apple AGX GPU | Apple proprietary | XL | Reverse-engineered in Asahi; we have nothing |
| 019 | P0 | Reg | Drivers | Wi-Fi (BCM43xx, NCM4387) | 802.11ax/be | XL | Skipped on M4 |
| 020 | P0 | Reg | Drivers | Audio (Apple SoC DSP) | CoreAudio-class | XL | Nothing |
| 021 | P0 | Reg | Drivers | Bluetooth | BLE | XL | Nothing |
| 022 | P0 | Reg | Power | Idle / sleep / suspend | ACPI / Apple SMC | L | We never sleep |
| 023 | P0 | Reg | Power | DVFS / P-state control | ARM SCMI | M | Need for thermal |
| 024 | P0 | Reg | Power | Battery state introspection | — | S | Trivial via SMC |
| 025 | P0 | Reg | IPC | AF_UNIX domain sockets | POSIX | M | Most apps depend on these |
| 026 | P0 | Reg | IPC | pipes / FIFOs | POSIX | S | Trivial |
| 027 | P0 | Reg | IPC | POSIX shared memory | POSIX | M | Many runtimes need it |
| 028 | P0 | Reg | Time | Wall clock + NTP | RFC 5905 | M | We only have monotonic |
| 029 | P0 | Reg | Time | Time zones | tzdata | S | Trivial once wall clock exists |
| 030 | P0 | Reg | Cgroups | Per-cave CPU/memory/io quotas | cgroups v2 | L | Caves have no quota enforcement |
| 031 | P0 | Reg | Cgroups | PID namespace | Linux | M | Caves see each other's PIDs |
| 032 | P0 | Reg | Cgroups | Mount namespace | Linux | M | Caves share FS root |
| 033 | P0 | Reg | Updates | Package manager | apt/dnf-class | XL | No update story at all |
| 034 | P0 | Reg | Updates | Signed releases | TUF / Sigstore | M | Need before any field deployment |
| 035 | P0 | Reg | Observability | dmesg ring | Linux kmsg | S | Trivial; just add a ring buffer for non-security events |
| 036 | P0 | Reg | Observability | /proc-equivalent | Linux procfs | L | Inspection of process state |
| 037 | P0 | Reg | Standards | libc (or libc-compat shim) | musl | XL | Most software needs one |
| 038 | P1 | Reg | UI | Init system / service manager | systemd / launchd | L | main.rs is the init |
| 039 | P1 | Reg | UI | Job control / pipes in shell | POSIX shell | M | Our shell doesn't pipe |
| 040 | P1 | Reg | UI | Locale / Unicode / IME | POSIX locale | L | ASCII-only today |
| 041 | P1 | Reg | UI | Accessibility services | macOS Accessibility | L | Screen reader, magnifier, voice control |
| 042 | P1 | Reg | Net | HTTP/2 + HTTP/3 | RFC 9113, 9114 | XL | Required for modern web services |
| 043 | P1 | Reg | Net | WireGuard | RFC 9711 | M | Modern VPN baseline |
| 044 | P1 | Reg | Net | VLAN (802.1Q) | IEEE 802.1Q | M | Required for any enterprise net |
| 045 | P1 | Reg | Net | Conntrack-class stateful firewall | nftables | L | Have stateless |
| 046 | P1 | Reg | Crypto | Argon2id password hashing | RFC 9106 | S | Concept note says we have it; inventory didn't find code |
| 047 | P1 | Reg | Crypto | AES-XTS for disk encryption | NIST SP 800-38E | M | BatFS uses GCM not XTS |
| 048 | P1 | Reg | Crypto | SHA-3 family | FIPS 202 | M | Only have SHA-2 |
| 049 | P1 | Reg | Crypto | BLAKE2/3 | — | S | Useful for fast hashing |
| 050 | P1 | Reg | Crypto | XChaCha20-Poly1305 | RFC 8439 + draft | S | Alternative AEAD |
| 051 | P1 | Reg | Crypto | Constant-time bignum | — | M | We rely on crate-provided |
| 052 | P1 | Reg | Net | OCSP / CRL revocation | RFC 6960, 5280 | L | Currently no revocation check |
| 053 | P1 | Reg | Net | Certificate Transparency SCT validation | RFC 6962 | M | Required for public-CA trust |
| 054 | P1 | Reg | Drivers | Sensors (gyro / accel / ambient light) | — | M | M4 has them, we don't |
| 055 | P1 | Reg | Drivers | Camera (FaceTime / front) | V4L2-class | XL | M4 has it |
| 056 | P1 | Reg | Drivers | Touch / multi-touch | — | M | M4 trackpad |
| 057 | P1 | Reg | Virt | KVM-class hypervisor | KVM / HVF | XL | M4 EL2 stub only |
| 058 | P1 | Reg | Containers | OCI runtime compat | runc/crun | XL | No way to run Docker images |
| 059 | P2 | Reg | Net | DNSSEC validation | RFC 4033-4035 | L | Big enterprise / .gov bar |
| 060 | P2 | Reg | Net | DoT (DNS-over-TLS) | RFC 7858 | S | Have DoH; DoT is alternative |
| 061 | P2 | Reg | Net | IGMP/MLD multicast | RFC 3376 | M | Required for some apps |
| 062 | P2 | Reg | Net | SCTP | RFC 4960 | M | Telecom / WebRTC |
| 063 | P2 | Reg | Net | MPTCP | RFC 8684 | M | Modern phone networking |
| 064 | P2 | Reg | Net | Bridges / bonding | Linux net | M | Enterprise topologies |
| 065 | P2 | Reg | Drivers | Software RAID | mdadm | L | Storage redundancy |
| 066 | P2 | Reg | Drivers | LVM / device-mapper | Linux dm | L | Storage flexibility |
| 067 | P2 | Reg | Drivers | dm-verity | Linux dm-verity | M | Read-only signed-root FS |
| 068 | P2 | Reg | Crypto | Argon2id in active use everywhere | — | S | Password derivation in auth |
| 069 | P3 | Reg | UI | Native browser | — | XL | Explicit no-browser pivot |
| 070 | P3 | Reg | UI | Email / calendar / contacts | — | L | BatKit candidates |
| 071 | P3 | Reg | UI | Window manager features (snapping, virtual desktops) | — | M | Have wm shell |
| 072 | P0 | Gov | Assurance | Common Criteria EAL claim | CC v3.1 | XXL | Min EAL4+ for gov use, EAL6+ for high-assurance |
| 073 | P0 | Gov | Assurance | FIPS 140-3 module boundary | NIST FIPS 140-3 | XL | Without this, crypto cannot be "approved" |
| 074 | P0 | Gov | Assurance | FIPS self-tests on power-up | NIST FIPS 140-3 | M | Required for FIPS-validated mode |
| 075 | P0 | Gov | Assurance | NIAP OS Protection Profile conformance | NIAP OS PP v4.3 | XL | Common requirement for government IT |
| 076 | P0 | Gov | Assurance | DISA STIG configuration baseline | DISA | L | Concrete config requirements |
| 077 | P0 | Gov | MAC | Mandatory file labels (SELinux-class) | NIST 800-53 AC-16 | XL | Required for MLS |
| 078 | P0 | Gov | MAC | Information-flow enforcement | Bell-LaPadula | XXL | The core MLS guarantee |
| 079 | P0 | Gov | MAC | Type enforcement | SELinux | XL | Per-process domain transitions |
| 080 | P0 | Gov | MAC | Labeled IPC | SELinux ipcMAC | L | Once labels exist |
| 081 | P0 | Gov | MAC | Labeled networking (SECMARK / CIPSO) | RFC 1108, CALIPSO | L | Cross-machine MLS |
| 082 | P0 | Gov | Trusted | TPM 2.0 / Apple SE interaction | TCG TPM 2.0 | XL | Foundation for measured boot |
| 083 | P0 | Gov | Trusted | Measured boot to PCRs | TCG | L | After TPM contact |
| 084 | P0 | Gov | Trusted | Sealed storage | TCG | M | Once measurement works |
| 085 | P0 | Gov | Trusted | Remote attestation (DAA / X.509-att) | TCG | L | Required for cloud / fleet |
| 086 | P0 | Gov | Trusted | TEE / confidential computing | ARM CCA / Apple SE | XL | Encrypted workload isolation |
| 087 | P0 | Gov | Mitig | ASLR / KASLR | NIST 800-53 SI-16 | M | Standard baseline |
| 088 | P0 | Gov | Mitig | Stack canaries (audited) | — | S | Rust gives partial; need audit |
| 089 | P0 | Gov | Mitig | Control-Flow Integrity (CFI) | — | L | Forward + backward edge |
| 090 | P0 | Gov | Mitig | ARM PAC (Pointer Authentication) | ARMv8.3 | M | M4 supports; we don't enable |
| 091 | P0 | Gov | Mitig | ARM BTI (Branch Target Identification) | ARMv8.5 | M | M4 supports; we don't enable |
| 092 | P0 | Gov | Mitig | ARM MTE (Memory Tagging Extension) | ARMv8.5 | L | M4 supports; we don't enable |
| 093 | P0 | Gov | Mitig | Hardened allocator (segregated, randomized) | scudo | L | Replace linked-list heap |
| 094 | P0 | Gov | Mitig | KASAN equivalent for kernel debug builds | KASAN | M | Detect use-after-free |
| 095 | P0 | Gov | SC | KPTI-equivalent (kernel page-table isolation) | — | M | Block Meltdown-class |
| 096 | P0 | Gov | SC | Spectre v1/v2/BHB mitigations | — | M | Branch prediction barriers |
| 097 | P0 | Gov | SC | Cache partitioning (MPAM / CAT) | ARM MPAM | L | Cross-domain isolation |
| 098 | P0 | Gov | Audit | Tamper-evident hash-linked audit log | RFC 7480-class | M | Append-only with chain hash |
| 099 | P0 | Gov | Audit | SIEM forwarding (rsyslog over TLS, etc.) | — | M | External log archive |
| 100 | P0 | Gov | Audit | Live forensic memory acquisition | Volatility-class | L | For incident response |
| 101 | P0 | Gov | Tamper | Anti-rollback monotonic counter | TPM NV index | M | Once TPM exists |
| 102 | P0 | Gov | Supply | SBOM generation (SPDX / CycloneDX) | NTIA | S | Trivial once Cargo.lock is canonical |
| 103 | P0 | Gov | Supply | Reproducible builds | — | M | Determinism flags + verification |
| 104 | P0 | Gov | Supply | Signed releases (Sigstore / cosign) | SLSA L4 | M | Foundation for trust |
| 105 | P0 | Gov | Supply | In-toto attestation chain | — | M | Build-step provenance |
| 106 | P0 | Gov | Identity | PIV / CAC smart card support | NIST SP 800-73 | XL | PKCS#11 stack |
| 107 | P0 | Gov | Identity | FIDO2 / WebAuthn | FIDO Alliance | L | Modern second factor |
| 108 | P0 | Gov | Identity | TOTP / HOTP in active use | RFC 6238, 4226 | S | Module exists; finish |
| 109 | P0 | Gov | Identity | Biometric subsystem (Touch ID-class) | — | L | M4 has Touch ID hardware |
| 110 | P0 | Gov | PKI | OCSP / CRL revocation | RFC 6960, 5280 | L | Currently no revocation |
| 111 | P0 | Gov | PKI | Cert pinning per host | — | S | Have anchor pins only |
| 112 | P0 | Gov | PKI | ACME (RFC 8555) for cert issuance | RFC 8555 | M | Operator usability |
| 113 | P0 | Gov | PKI | Certificate Transparency monitoring | RFC 6962 | M | SCT validation |
| 114 | P0 | Gov | RT | Priority inheritance protocol | POSIX | M | Avoid priority inversion |
| 115 | P0 | Gov | RT | Memory locking (mlock) | POSIX | S | For real-time threads |
| 116 | P0 | Gov | RT | Watchdog hierarchy | — | M | Per-subsystem watchdogs |
| 117 | P0 | Gov | IFC | Taint propagation across system calls | Asbestos / HiStar | XXL | Major research-grade addition |
| 118 | P0 | Gov | Coerce | Duress password (decoy unlock) | VeraCrypt-class | M | Plausible-deniability mode |
| 119 | P0 | Gov | Coerce | Cold-boot RAM mitigation | TRESOR / AESNI-on-CPU | M | Memory-encryption awareness |
| 120 | P0 | Gov | Coerce | Evil-maid protection | TPM-bound FDE + measured boot | L | Stack of items above |
| 121 | P0 | Gov | HW | HSM integration via PKCS#11 | NIST SP 800-130 | L | Enterprise key storage |
| 122 | P0 | Gov | HW | KMIP support | OASIS KMIP | M | Standardized key mgmt |
| 123 | P0 | Gov | HW | SP 800-90B health tests on entropy | NIST SP 800-90B | M | Continuous + startup |
| 124 | P0 | Gov | Compliance | OSCAL catalog of controls | NIST OSCAL | M | Machine-readable controls |
| 125 | P0 | Gov | Compliance | SCAP benchmark + OpenSCAP scoring | NIST 800-126 | M | Automated compliance check |
| 126 | P0 | Gov | Verify | Formal model of MLS enforcement | seL4-class | XXXL | Multi-year |
| 127 | P0 | Gov | Verify | Formal proof of crypto correctness | F\* / Vale | XXL | Per-primitive proof |
| 128 | P0 | Gov | DLP | Network DLP classification | Symantec-class | XL | Classify-then-block |
| 129 | P0 | Gov | DLP | Clipboard / screen capture restriction | Win/macOS | M | Plumbing for the restrictions |
| 130 | P0 | Gov | DLP | Print watermarking | — | M | User + time + classification |
| 131 | P0 | Gov | Insider | Two-person integrity (TPI) | NIST SP 800-53 AC-3 | M | Co-sign for critical ops |
| 132 | P0 | Gov | Insider | Threshold cryptography (M-of-N) | NIST FIPS 140-3 Level 4 | L | Shamir / threshold sigs |
| 133 | P0 | Gov | Monitor | CVE feed ingestion + matching | NVD, OSV | M | Vuln awareness |
| 134 | P0 | Gov | Monitor | Coordinated disclosure program | ISO/IEC 30111 | S | Process + tooling |
| 135 | P0 | Gov | Air-gap | Cross-Domain Solution support | NSA CDS | XL | Highest-assurance content moves |
| 136 | P0 | Gov | Air-gap | One-way data diode | — | M | Hardware + protocol |
| 137 | P0 | Gov | Air-gap | TEMPEST awareness | NSTISSAM TEMPEST 1-92 | XL | EMSEC; mostly hardware |
| 138 | P0 | Gov | Hardened | Function-level binary diversification | — | L | Per-device randomization |
| 139 | P0 | Gov | Radio | Hardware-level radio kill switch | DCID 6/9 SCIF | M | Hardware feature; we have nothing yet |
| 140 | P0 | Gov | Specials | HAIPE / SCIP / Type-1 framework support | NSA Type-1 | XXL | Requires NSA-certified hardware |

Note: this table is intentionally not exhaustive at the row level —
column entries map to subsections in Part 3, and each subsection is
the "spec sheet" for what's actually required. A v2 of this table
might expand each P0 row to 5-10 sub-rows.

---

## Part 4.5 — Status updates since 2026-05-10

The triage table in Part 4 was a snapshot. This section captures
items resolved (or reclassified as non-goals / already-present)
since that date, with commit refs. Leave the historical table
untouched so the audit-as-of-snapshot stays diff-able.

### Closed P0 items

| # | Item | Resolution | Where |
|---|------|------------|-------|
| 025 | AF_UNIX domain sockets | Shipped: SOCK_STREAM, abstract namespace, accept queue, 5 syscalls (`SYS_SOCKET`..`SYS_ACCEPT`). SOCK_DGRAM + fs paths deferred. | `src/kernel/unix_sock.rs`, commit `470c97c3` |
| 026 | pipes / FIFOs | Shipped: per-task fd table, 32 pipes × 4 KiB ring buffers, SYS_PIPE/READ/WRITE/CLOSE, blocking read/write with EOF + EPIPE. FIFOs need BatFS FIFO inode type — deferred. | `src/kernel/pipe.rs`, commit `21453b46` |
| 027 | POSIX shared memory | Shipped: 32 regions × ≤16 KiB, `shm_open`/size/ptr/close syscalls, `FdKind::Shm`. Memory-quota enforced (item 030). | `src/kernel/shm.rs`, commit `8f6faaec` |
| 028 | Wall clock + NTP | Shipped: PL031 RTC anchor at boot + HTTPS-Date sync via TLS-pinned path (`time-sync-https`). Plaintext NTP intentionally skipped. | `src/kernel/time.rs`, `src/drivers/rtc.rs`, commits `90dfb8df` + `8f6faaec` |
| 029 | Time zones | Shipped: `time::set_tz_offset_secs(±14h)` + `tz` shell command. tzdata blob not needed for single-offset UX. | commit `492294de` |
| 030 | Per-cave CPU/memory/IO quotas | **Partial.** Memory quota enforced at shm + pipe + BatFS + cave_private + ELF runner — `cave::active_charge_pages` / `charge_pages_for(cave_id)` covers every cave-attributable allocator we have today. CPU + net stay observability-only (no enforcement until preemptive timer scheduling re-enabled — see deferred list). `batfs-quota-selftest` proves the BatFS path; `cave-private-selftest` now asserts each cave reports `mem_used_pages >= 1` after its cave-private page is allocated. | commits `6715c827` + `18890477` + 2026-05-12 batfs-quota + 2026-05-12 cave-private + ELF quota |
| 031 | PID namespace | Shipped: `cave_id` on `Task`, `process::list_for_cave`, `procs` shell cmd with `procs all` admin view. | commit `492294de` |
| 032 | Mount namespace | Shipped: `batfs::ns_create` / `ns_read` / `ns_delete` / `ns_list` / `ns_stats` wrappers auto-apply the active cave's mount prefix; routed through the shell (`ls`/`write`/`cat`/`rm`/`verify`/`hash`), tab completion, editor, comms, and file-manager. Kernel-administered files (audit.log, pkg bundles) stay on the un-prefixed `batfs::*` paths. `with_cave_active` now also updates `ACTIVE_CAVE_ID` so cap/quota/prefix queries see the correct identity inside the trampoline. `mount-ns-selftest` proves cross-cave file isolation end-to-end. | commits `492294de` + 2026-05-12 mount-ns auto-application |
| 033 | Package manager | Shipped: BPKG signed-bundle format + `scripts/pkg_pack.py` + `scripts/pkg_serve.py` + `src/kernel/pkg.rs` + `pkg stage/install/list/remove`. End-to-end verified on QEMU: install + tamper-rejection both work. | commits `95f2e161` + `af5235a9` |
| 034 | Signed releases | Shipped: `release_sign.py` keygen/sign + `SPHRAGIS_RELEASE_PUBKEY` baked at build time + `release-verify` shell command. No fallback test key. | commit `492294de` |
| 035 | dmesg ring | Was already present in `src/kernel/kmsg.rs`. Boot breadcrumb emitters wired into every major init step so `dmesg` shows useful history. | commit `8f6faaec` |
| 036 | /proc-equivalent | **Partial.** Shell commands `caps [tid]`, `fds [tid]`, `task <tid>` replace 90% of /proc use-cases. A real pseudo-file filesystem is deferred — wants BatFS pseudo-file infrastructure that other features (`/sys`-equivalent) would also use. | commit `6715c827` |
| 037 | libc / libc-compat shim | **Non-goal.** Documented in `DESIGN.md` decision log #14. Security model favors purpose-built no-libc Rust workloads; shim would import the C-ecosystem attack surface. | commit `18890477` |

**Net P0 status:** every row 025-037 is closed, partially closed
with documented deferral, or explicitly reclassified as a non-goal.

### Closed P1 items (verified present after audit refresh)

The original Part-4 table listed several items as "missing" that
were already implemented when the audit was written but the
inventory pass missed them. Confirmed present:

| # | Item | Where |
|---|------|-------|
| 046 | Argon2id password hashing | `src/security/auth.rs`, `src/kernel/mm/heap.rs`, used in BatFS key derivation |
| 047 | AES-XTS for disk encryption | `src/crypto/aes_xts.rs` (`xts-mode` crate) |
| 048 | SHA-3 family | `src/crypto/sha3.rs` |
| 049 | BLAKE2/3 | `src/crypto/blake3.rs` |
| 050 | XChaCha20-Poly1305 | `src/crypto/xchacha20poly1305.rs` |
| 052 | CRL revocation | `src/net/crl.rs` (CRL); OCSP still deferred — different RFC, different on-wire shape |
| 053 | Certificate Transparency SCT validation | `src/net/ct_logs.rs` |

### Open follow-ups (deferred work, documented elsewhere)

These are tracked but not yet implemented; commit refs point at the
journal entry or DESIGN.md decision-log entry that owns the
deferral rationale:

  * **Preemptive timer scheduling re-enable** — infra exists in
    `arch::init_timer` / `arch::handle_irq` but currently disabled
    at boot (hangs mid-fire). Unblocks real CPU-quota enforcement
    on item 030. See DESIGN.md decision #15.
  * **Memory quota across all allocators** — today enforced at
    shm + pipe. `mm::frame::alloc_frame`, page tables, audit ring,
    and the kernel heap still bypass the cave-charge API. See
    journal entry 2026-05-11 cave-quotas batch.
  * **Mount-namespace auto-application** — `cave::active_mount_prefix`
    exists but the 42 `batfs::*` callers don't apply it
    automatically. Demo command `mount-ns` proves the primitive.
  * **Apple M4 RTC backend** — `drivers::rtc::read_apple()` is a
    stub. Needs SMC keypath access from EL1.
  * **PQ-comms wire deployment** — `pq-comms-selftest` exercises
    the full handshake in-process. Real over-the-wire deployment
    waits on a PQ-capable peer (a second Sphragis instance is the
    natural candidate).
  * **Package manager v2** — current cut is single-namespace
    install, no dependencies, no update path, ≤1 MiB bundles.
    All four deferrals are real engineering arcs of their own.

### Genuinely-open P1 items (not implemented, not deferred)

After this refresh, the still-open P1 list narrows to:

  * 038 init system / service manager (`main.rs` IS the init today)
  * 039 shell `|` job control / pipes — **output-capture refactor shipped.** `console::begin_capture` / `end_capture` redirect writes to a 32 KiB buffer (serial mirror retained); shell `parse_redirect` is quote-aware so `cmd > /file` captures and `write hello "world > foo"` doesn't. `redirect-selftest` proves the round trip ends in BatFS via `ns_create` + `ns_read`. Real `|` pipes (commands that consume a buffer-input shape) are the next slice this primitive unblocks.
  * 040 Unicode / locale / IME
  * 041 Accessibility services
  * 042 HTTP/2 + HTTP/3
  * 043 WireGuard — **closed.** In-process WG stack complete (handshake both roles, wire framing, replay window, IPC mailbox, endpoint config). `qemu_wg_real_peer_e2e.py` proves outbound Init traverses virtio-net to a real host UDP listener. `qemu_wg_full_handshake_e2e.py` closes the loop with a Python Noise IK responder (`scripts/wg_responder.py`): Sphragis sends Init, Python decrypts (proving the crypto produces valid Noise IK ciphertext) + builds a Response, Sphragis processes the Response and marks `session.their_sender_index != 0` (Established). Transport round trip with real peer + replay-window-on-the-wire are the remaining stretches.
  * 044 VLAN (802.1Q)
  * 045 conntrack-class stateful firewall — **load-bearing.** `src/net/conntrack.rs` adds a 64-slot flow table keyed on (proto, remote_ip, remote_port, local_port); `tcp::connect_start` registers and `tcp::connect_blocking_pcb` marks established on SYN-ACK; `tcp::close_pcb` releases. The wildcard inbound TCP allow rule has been REMOVED from `firewall::init`; `firewall::allow_inbound_tcp` now permits a segment only if (a) `conntrack::lookup_inbound` finds a matching outbound flow, (b) `tcp::listener_lookup_by_port` reports a registered listener, or (c) an explicit per-port rule matches. Unsolicited SYNs to random ephemeral ports are now dropped. `conntrack-selftest` proves the table; `fw-hardening-selftest` proves the policy.
  * 051 constant-time bignum (RustCrypto crates provide this; verify their security claims rather than rebuilding)
  * 052b OCSP revocation — **shipped.** `src/net/ocsp.rs` adds a constant-cost (issuer_key_hash, serial) → Status cache + DER `OCSPResponse` ingest via the `x509-ocsp` crate; `ocsp-selftest` proves DER parse + Good/Revoked recording + fresh-response override against two Python-`cryptography`-generated fixtures.
  * 054-058 sensor / camera / touch / hypervisor / OCI drivers — all need real M4 hardware

---

## Part 5 — Threat Model Coverage

The gaps above leave us exposed to specific adversary classes. Brief
mapping:

**Class A: Remote network attacker (anonymous, no prior access)**
- Covered: TLS chain, default-deny firewall, hardened HTTP, DNS
  defenses, beacon detection. **Good shape.**
- Gaps: IPv6 exposure (n/a, we don't have IPv6), OCSP for revoked
  intermediates, CT log monitoring. **Manageable.**

**Class B: Local attacker with physical access (cold boot, evil maid)**
- Covered: panic-wipe hotkey, dead-man's switch, audit ring sealed.
- Gaps: measured boot, TPM-bound encryption, cold-boot RAM
  encryption, duress mode, evil-maid attestation. **Bad shape.**

**Class C: Privileged insider (cleared but malicious operator)**
- Covered: audit ring records everything cave-side.
- Gaps: tamper-evident chain, SIEM forwarding, two-person integrity,
  separation of duties, DLP, UEBA. **Wide open.**

**Class D: Supply chain (compromised dependency, build server)**
- Covered: cargo.lock pinning, vendored deps in `external/`.
- Gaps: SBOM, reproducible builds, signed releases, transparency log,
  in-toto. **Wide open.**

**Class E: Hardware attacker (chip-level rogue gate, malicious DMA)**
- Covered: DART IOMMU BYPASS verified (block accidental DMA from
  unbound devices), constant-time crypto.
- Gaps: TEE / confidential computing, MTE / PAC / BTI, side-channel
  hardening. **Bad shape.**

**Class F: Quantum-capable adversary (5-10 year horizon)**
- Covered: hybrid X25519 + ML-KEM-768 KEM, hybrid Ed25519 + ML-DSA-65
  signatures. **Ahead of the curve.**
- Gaps: PQ in the certificate chain itself (still classical-only),
  CNSA 2.0 sign-off. **Forward planning needed.**

**Class G: State-level adversary at endpoint (zero-day, kernel-mode rootkit)**
- Covered: Rust memory safety, constant-cost abort, audit ring.
- Gaps: every exploit-mitigation in §3.5, side-channel resistance
  in §3.6, formal verification §3.19. **Most exposed.**

---

## Part 6 — Spec References

- IEEE Std 1003.1-2017 (POSIX)
- ISO/IEC 9899:2018 (C11)
- IETF RFCs: 793 (TCP), 791 (IPv4), 8200 (IPv6), 8446 (TLS 1.3),
  5280 (X.509), 6962 (CT), 8484 (DoH), 5905 (NTP), 9106 (Argon2),
  8439 (ChaCha20-Poly1305), 6238 (TOTP), 4226 (HOTP), 8555 (ACME),
  6960 (OCSP), 4033-4035 (DNSSEC), 9711 (WireGuard),
  9113 (HTTP/2), 9114 (HTTP/3), 4960 (SCTP), 8684 (MPTCP),
  3376 (IGMP), 1108 (IP Security Options), CALIPSO (RFC 5570).
- NIST SP 800-53 Rev. 5
- NIST SP 800-90A/B (DRBG / Entropy)
- NIST SP 800-171 (CUI)
- NIST SP 800-38E (XTS), 38D (GCM)
- NIST SP 800-130 (Key Management Framework)
- NIST SP 800-73 (PIV)
- NIST SP 800-90B (Entropy source health tests)
- FIPS 140-3, 199, 200, 202 (SHA-3), 203 (ML-KEM), 204 (ML-DSA),
  205 (SLH-DSA)
- Common Criteria for IT Security Evaluation v3.1 R5
- NIAP OS Protection Profile v4.3
- CNSSI 1253; DoD Cloud IL2-IL6 definitions
- CNSA Suite 2.0
- DISA STIG (Linux, Windows references)
- TCG TPM 2.0 Library Spec (Parts 1-4)
- ARM Architecture Reference Manual (DDI 0487) — PAC, BTI, MTE, CCA
- ARM Server Base System Architecture (SBSA)
- TCG Platform Configuration Register / Measured Boot
- NSA HAIPE-IS, SCIP, Type-1
- NSTISSAM TEMPEST 1-92
- DCID 6/9 (SCIF construction)
- SLSA framework v1.0
- in-toto specification v1.0
- Sigstore architecture overview
- IEC 61508 SIL definitions
- DO-178C software considerations in airborne systems
- ISO 26262 functional safety for road vehicles
- Bell-LaPadula (1973), Biba (1977), Clark-Wilson (1987),
  Brewer-Nash (1989)
- seL4 functional correctness proof (Klein et al., 2009)
- CompCert verified C compiler (Leroy et al.)
- F\* + Vale (HACL\* verified crypto)

---

**Document status:** complete first pass. ~12,000 words across
6 parts. Triage table has 140 numbered rows; expanding to per-row
sub-items would push past 500. Threat-model section maps gaps to
adversary classes A through G. Spec references covered both standard-OS
and gov-grade bars.

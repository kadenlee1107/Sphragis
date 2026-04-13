# Bat_OS — Design Document

## Understanding Summary

- **What:** Bat_OS — a fully custom, bare-metal operating system for Apple Silicon (M4 MacBook)
- **Why:** Eliminate supply chain attack vectors. Zero external dependencies. Every line of code is controlled and auditable. Government/private-client grade security for personal use.
- **Who:** Single user. No distribution, no downloads, no one else ever touches it.
- **Stack:** Rust microkernel + minimal ARM64 assembly. Framebuffer UI first, GPU-accelerated later.
- **Security model:** Full disk + per-file encryption backed by Secure Enclave. Passphrase + hardware token auth. Duress wipe code, dead man's switch, failed attempt destruction.
- **Networking:** All traffic encrypted + routed through VPN → Tor. Strict allowlist firewall. Default deny all.
- **Aesthetic:** Black and white Batman aesthetic. High contrast, bat iconography, sharp angular design.
- **Apps:** 8 individually isolated, hard-focused applications.

## Assumptions

- Dual-booting alongside macOS initially for development purposes
- Asahi Linux reverse-engineering docs are the primary hardware reference
- Touch ID hardware will be ignored (no biometrics by design)
- Wi-Fi and Bluetooth drivers will need reverse-engineering or Asahi-based reference
- No multi-user, no permissions model beyond "is it Kaden or not"
- Development and testing done in QEMU before bare metal deployment

---

## Architecture: Microkernel

Everything that can run in userspace does. The kernel handles only:

1. **Scheduler** — priority preemptive, security services always preempt
2. **Memory Manager** — isolated virtual address space per service, hardware MMU enforced
3. **IPC** — typed message channels, sync/async, zero-copy via capability-granted shared memory
4. **Interrupt Dispatcher** — catches hardware interrupts, routes to userspace drivers, priority-aware
5. **Capability System (seL4-inspired)** — grant/delegate/revoke model, no ambient authority
6. **Secure Enclave Interface** — only kernel talks to SE, services request crypto ops via IPC

Target kernel size: ~8,000-12,000 lines of Rust.

---

## Boot Chain & Authentication

```
Power On → Apple iBoot → Bat_OS Boot Stub (ASM) → Authentication Gate (Rust) → Kernel Init
```

### Authentication Gate
- Passphrase prompt (framebuffer text, raw pixels)
- YubiKey challenge-response via USB
- Both factors must pass

### Failure Modes
- **Wrong attempts:** counter incremented, max exceeded → Secure Enclave nukes all keys permanently
- **Duress passphrase:** fake boot animation displayed, silent full wipe in background, eventually "crashes"
- **Success:** Secure Enclave releases master decryption key, dead man's switch timer starts

### Dead Man's Switch
- Timer stored in Secure Enclave (tamper-resistant)
- Configurable interval (e.g., 48 hours)
- Must be refreshed by passphrase periodically
- Expiry → SE nukes all keys on next boot
- Cannot be disabled without auth

---

## Encryption Model

### Three Layers
1. **Secure Enclave** — master keys generated, stored, and used inside SE hardware. Keys never touch main RAM.
2. **Full Disk Encryption** — AES-256-XTS, entire SSD encrypted, unreadable without boot auth
3. **Per-File Encryption** — unique derived key per file, compartmentalized

### Integrity
- Merkle tree verification on filesystem blocks
- Detects any single byte modification on disk at read time

---

## Network Security Stack

### Outbound Traffic Flow
```
Application → Firewall (per-service allowlist) → DNS (DoH only, DNSSEC validated)
→ TLS 1.3 (cert pinning, no downgrade) → VPN (WireGuard, Rust impl)
→ Tor (3-hop onion routing) → WiFi (randomized MAC)
```

### What Each Attacker Sees
- **WiFi sniffer:** encrypted VPN blob, random MAC
- **ISP:** VPN traffic only, can't see Tor or content
- **VPN provider:** Tor traffic, can't see destination or content
- **Tor entry node:** VPN IP, not real IP
- **Tor exit node:** TLS encrypted blob, can't read content
- **Destination:** Tor exit IP, TLS content only

Nobody in the chain sees the full picture.

### Inbound Traffic
- DEFAULT: DENY ALL
- Only responses to outbound requests accepted
- No open ports, no listeners
- Machine is invisible to port scans
- Exception: explicit temporary inbound caps for security toolkit (auto-expire)

### DNS
- DNS over HTTPS only
- Hardcoded resolver (no DHCP override)
- DNSSEC validated
- Cache encrypted at rest
- No plaintext DNS ever leaves the machine

---

## Userspace Services

### Tier 1: Hardware Drivers (launch first)
- NVMe Driver — disk access
- USB Driver — YubiKey, peripherals
- Display Driver — framebuffer control
- WiFi Driver — network hardware (Asahi reference)
- Keyboard/Trackpad Driver — SPI/HID input
- Thunderbolt Driver — strict DMA attack prevention

### Tier 2: Core System Services (launch second)
- **Filesystem Service** — custom encrypted FS, Merkle tree integrity
- **Network Service** — firewall, Tor, VPN, TLS, DoH
- **Input Service** — HID event routing, panic hotkey, keylogger-proof isolation
- **UI Compositor** — owns framebuffer, composites app buffers, bat theme
- **Dead Man's Switch Service** — SE-backed timer, independent of other services

### Tier 3: Applications (launch last)

| App | Allowed IPC Channels |
|-----|---------------------|
| Terminal | FS, Input, Compositor |
| File Manager | FS, Input, Compositor |
| Code Editor | FS, Input, Compositor |
| Browser | Net, Input, Compositor, FS (scoped to downloads only) |
| Comms Client | Net, Input, Compositor, SE (sign only) |
| Net Monitor | Net (read-only), Input, Compositor |
| Security Toolkit | Net, FS, Input, Compositor |
| System Dashboard | All services (read-only) |

---

## UI & Aesthetic

### Color Palette
| Token | Hex | Usage |
|-------|-----|-------|
| bg-deep | #000000 | Desktop, panel background |
| bg-mid | #0A0A0A | Window backgrounds |
| bg-surf | #141414 | Cards, input fields |
| border | #1E1E1E | Subtle dividers |
| text-dim | #5A5A5A | Secondary text |
| text-mid | #A0A0A0 | Body text |
| text-hi | #FFFFFF | Primary text, icons |
| accent | #FFFFFF | Focused elements, active states |
| danger | #FF0000 | Threats, wipe alerts |
| secure | #00FF00 | Verified, encrypted |

### Design Principles
- Pure black desktop with subtle white bat emblem (watermark, centered)
- Tiling window manager — keyboard-driven, no floating windows
- Sharp, angular — no rounded corners
- Monospace font throughout
- 1px white window borders on black
- Bat icon in every window title bar
- Status bar always visible: encryption state, network route, dead man's switch countdown
- No animations in Phase 1

### Boot Screen
- Minimal black screen, white text
- Bat emblem, passphrase field, YubiKey status
- No version number, no hints, no information leakage

### Duress Fake Boot
- Indistinguishable from real boot
- Progress bar crawls slowly
- Full wipe executes behind the scenes
- Eventually "crashes" — attacker assumes broken, data is gone

---

## Development Phases

### Phase 0: Toolchain & Environment
- Rust aarch64-unknown-none target
- QEMU ARM64 virt machine
- GDB kernel debugging
- Project structure + cargo build system + custom linker script
- **Deliverable:** `cargo build` produces a binary QEMU can boot

### Phase 1: Boot & Bare Metal
- ARM64 boot stub (assembly)
- Exception levels (EL1 kernel mode)
- MMU + page tables
- UART console output
- Basic heap allocator
- **Deliverable:** boot in QEMU, print "BAT_OS ALIVE" to serial console

### Phase 2: Microkernel Core
- Memory manager (page allocator, virtual address spaces)
- Process/task abstraction
- Scheduler (priority preemptive)
- IPC (synchronous first)
- Interrupt dispatcher
- Capability system (grant + revoke)
- **Deliverable:** kernel launches two userspace processes that exchange IPC messages

### Phase 3: Essential Drivers (QEMU virtio)
- virtio-blk (virtual disk)
- virtio-net (virtual network)
- virtio-input (virtual keyboard/mouse)
- virtio-gpu (virtual framebuffer)
- All in userspace with capabilities
- **Deliverable:** read/write disk, send/receive packets, accept input, draw pixels

### Phase 4: Filesystem & Encryption
- Custom filesystem format
- AES-256-XTS full disk encryption (software impl)
- Per-file encryption with derived keys
- Merkle tree integrity checking
- **Deliverable:** encrypted file CRUD from shell, integrity tampering detected

### Phase 5: Networking & Security Stack
- TCP/IP stack (custom, minimal, Rust)
- TLS 1.3
- DNS-over-HTTPS
- Allowlist firewall
- WireGuard client (Rust, from spec)
- Tor client (minimal circuit builder)
- **Deliverable:** fetch a web page through Tor + VPN from QEMU

### Phase 6: UI & Applications
- 6a: Compositor, tiling WM, bat theme, font rendering
- 6b: Apps built one at a time:
  1. Terminal/Shell
  2. System Dashboard
  3. File Manager
  4. Code Editor
  5. Network Monitor
  6. Security Toolkit
  7. Comms Client
  8. Web Browser (hardest, last)
- **Deliverable:** full desktop environment with all 8 apps in QEMU

### Phase 7: Apple Silicon Hardware
- Replace virtio with real M4 drivers:
  - NVMe (Apple ANS)
  - WiFi (Broadcom, Asahi reference)
  - Display (Apple DCP)
  - Keyboard/Trackpad (SPI/HID)
  - USB / Thunderbolt (XHCI + Apple TB)
  - Secure Enclave (SEP) — replace software crypto
  - GPU (AGX) — replace framebuffer rendering
- **Deliverable:** Bat_OS boots on real M4 MacBook

### Phase 8: Auth & Hardening
- Boot authentication (passphrase + YubiKey)
- Duress wipe system
- Dead man's switch (SE-backed)
- Failed attempt counter + auto-wipe
- Panic hotkey
- Full security audit
- Penetration test the OS
- **Deliverable:** fully hardened Bat_OS on bare metal

---

## Decision Log

| # | Decision | Alternatives Considered | Why Chosen |
|---|----------|------------------------|------------|
| 1 | Bare-metal custom OS | Linux distro, hypervisor/VM OS | Maximum security requires zero external dependencies and full code control. Supply chain attacks (npm/npx) motivated eliminating all third-party code. |
| 2 | Single user, no distribution | Multi-user, open source | OS is a personal security tool. No one else should ever access or possess it. Simplifies design (no user accounts, no permissions model). |
| 3 | Rust + minimal ARM64 assembly | C, Zig, mixed | Rust eliminates ~70% of memory safety vulnerabilities. No runtime/GC overhead. unsafe blocks make dangerous code explicit and auditable. Assembly only where hardware demands it. |
| 4 | Microkernel architecture | Monolithic, hybrid | Security is structural, not bolted on. Compromised service can't escape its sandbox. Tiny kernel (~10K LOC) is auditable. Each service isolated by hardware MMU. |
| 5 | seL4-inspired capability system | POSIX permissions, ACLs | Capabilities are unforgeable, support delegation/revocation, and enforce no ambient authority. Gold standard for secure microkernels. |
| 6 | Full disk + per-file + Secure Enclave encryption | Full disk only, encrypted volumes | Three-layer defense. Physical theft → disk encryption. Disk encryption broken → per-file encryption. Both broken → keys are in SE hardware, never in RAM. |
| 7 | Passphrase + YubiKey, no biometrics | Fingerprint, voice, multi-factor with biometrics | Biometrics can be coerced from unconscious/unwilling person. Voice can be AI-replicated from video. Passphrase is knowledge only. YubiKey can be physically destroyed in emergency. |
| 8 | Duress wipe + dead man's switch + attempt limit | Standard lockout | Under coercion, duress code silently wipes while showing fake boot. Dead man's switch ensures data dies if user is incapacitated. Attempt limit prevents brute force. |
| 9 | VPN wrapping Tor (VPN → Tor → destination) | Tor only, VPN only, Tor → VPN | ISP can't see Tor usage (sees VPN). VPN provider can't see destination (sees Tor). Tor exit can't read content (TLS). No single point reveals both identity and activity. |
| 10 | Framebuffer first, GPU later | GPU from start | Framebuffer gets UI working immediately in QEMU. GPU (Apple AGX) requires complex reverse-engineered drivers. Build the experience first, accelerate later. |
| 11 | Tiling window manager | Floating/stacking WM | Keyboard-driven is faster. No click-jacking attack surface. Fits tactical aesthetic. No wasted screen space. |
| 12 | QEMU development first, bare metal later | Develop on real hardware | Can't risk bricking only MacBook during development. QEMU provides debugger attachment, snapshots, and safe iteration. virtio drivers stand in for real hardware. |
| 13 | Black and white color scheme | Dark + yellow (Batman traditional) | User preference. High contrast, clean, no color ambiguity. Red (danger) and green (verified) are the only color accents. |

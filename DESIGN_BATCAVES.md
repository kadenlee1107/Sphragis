# BatCaves — Secure Container Runtime for Sphragis

## Understanding Summary

- **What:** BatCaves — isolated container environments that run any Kali Linux tool inside Sphragis
- **Why:** Full pentesting/security toolkit without compromising the host OS. Supply chain attacks (npm, etc.) are contained — a compromised tool can't escape its BatCave
- **Who:** Kaden — single user
- **How:** One shared minimal Linux kernel runs as a Sphragis microkernel service. BatCaves are isolated environments on top. Tools installed from Kali repos on demand. GUI + CLI both supported.
- **Isolation:** Every BatCave starts with ZERO access. Capabilities granted explicitly per-cave (network, filesystem, display, USB, raw sockets, inter-cave IPC). Enforced by seL4-inspired capability system at the microkernel level.
- **Lifecycle:** Persistent or ephemeral. Persistent can downgrade to ephemeral (one-way ratchet — anti-coercion). Create with `--tools` or install incrementally.
- **Destruction:** Dead man's switch, duress wipe, panic hotkey — ALL BatCaves die with the OS. No exceptions.
- **Networking:** All BatCave traffic goes through the secure network pipeline (allowlist firewall → TLS → VPN → Tor).

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  SPHRAGIS DESKTOP                      │
│  Terminal │ Dashboard │ Files │ NetMon │ Editor      │
│                    │                                 │
│            ┌───────┴────────┐                        │
│            │ BatCave Manager │  (6th app)             │
│            └───────┬────────┘                        │
├────────────────────┼────────────────────────────────┤
│              BATCAVE RUNTIME                         │
│                    │                                 │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐               │
│  │BatCave A│ │BatCave B│ │BatCave C│               │
│  │pentest  │ │forensics│ │recon    │               │
│  │         │ │         │ │         │               │
│  │Burp     │ │Autopsy  │ │nmap     │               │
│  │Wireshark│ │Volatil. │ │sqlmap   │               │
│  │Nethunter│ │         │ │         │               │
│  ├─────────┤ ├─────────┤ ├─────────┤               │
│  │Cap:     │ │Cap:     │ │Cap:     │               │
│  │net,raw, │ │fs(ro),  │ │net,     │               │
│  │display  │ │display  │ │(none)   │               │
│  └────┬────┘ └────┬────┘ └────┬────┘               │
│       │           │           │                     │
│  ┌────┴───────────┴───────────┴────┐                │
│  │  SHARED LINUX KERNEL SERVICE     │                │
│  │  (minimal — syscall translation) │                │
│  │  Runs as Sphragis userspace task   │                │
│  └──────────────┬──────────────────┘                │
├─────────────────┼───────────────────────────────────┤
│           MICROKERNEL                                │
│  Scheduler │ Memory │ IPC │ Capabilities             │
│  Every BatCave syscall → capability check            │
└─────────────────────────────────────────────────────┘
```

## Security Model (6 Layers)

1. **Memory isolation** — each BatCave gets its own L1 page table. Hardware MMU
   enforced via `TTBR0_EL1` swap on `cave::enter` (STUMP #145). Every cave's L1
   maps the kernel identity range `[0x40000000, 0x140000000)` (so kernel itself
   stays reachable while a cave is active) plus MMIO; native caves have NO EL0
   user window, so any drop-to-EL0 also faults. Cross-cave switches flush the
   whole TLB via `tlbi vmalle1`. **Limitation:** today there's no ASID, so the
   TLB flush is a sledgehammer (fine for single-CPU; revisit when SMP lands).
   And since native caves have no EL0 code today, the practical security
   benefit is TLB isolation, not preventing user-mode reads — those become
   meaningful when WASM/scripted workloads land inside native caves.
   Docker-backed caves keep relying on the Mac kernel's container isolation;
   their `cave_l1_phys` stays 0 because adding our own L1 would be redundant.
2. **Capability gate** — every syscall passes through microkernel capability check. No cap = no access.
   **Limitation:** as of the most recent audit, capability checks are wired at
   one syscall site (net write/sendto). Most syscalls don't gate. Closing this
   is queued as a future STUMP — the capability infrastructure (`cave::has_cap`,
   `grant_cap`/`revoke_cap`) is solid; the call-site coverage isn't.
3. **Filesystem encryption** — per-BatCave derived encryption key (HMAC-SHA256
   of the BatFS master keyed by cave name, STUMP #111 audit C011). Destroy key
   = data unrecoverable. Master is Argon2id-derived from the boot passphrase
   (STUMP #138).
4. **Network firewall** — all traffic through allowlist firewall + secure
   pipeline. No backdoors. Per-cave egress allowlist (`net::cave_policy`) +
   per-cave token-bucket shaper + SNI peek/reject all enforced on the cave
   NAT path.
5. **Display sandbox** — GUI tools render to dedicated framebuffer region.
   Can't read other windows. **Limitation:** bounding-box clipping is real
   (`cave.rs::alloc_display`) but cross-cave readback is not actively
   prevented; future work for genuine output isolation.
6. **Destruction guarantee** — all wipe events (deadman, duress, panic)
   destroy all BatCave keys AND drop persisted manifests (STUMP #135) AND
   free per-cave L1 frames (STUMP #145). No survivors.

## Shell Commands

```
batcave create <name> [--tools tool1,tool2] [--ephemeral]
batcave install <name> <tool>
batcave grant <name> <capability>
batcave revoke <name> <capability>
batcave enter <name>
batcave gui <name> <tool>
batcave list
batcave destroy <name>
batcave seal <name>          # persistent → ephemeral (one-way, irreversible)
```

## Grantable Capabilities

- `net` — outbound network access (through secure pipeline)
- `raw` — raw sockets (packet crafting, sniffing)
- `display` — GUI window on Sphragis desktop
- `fs:<path>` — read/write to specific encrypted vault directory
- `usb` — USB device access (WiFi adapters, HID devices)
- `ipc:<other_cave>` — inter-BatCave communication
- Any resource the microkernel manages can be granted

## Lifecycle Rules

- **Persistent** (default): survives reboots, tools stay installed
- **Ephemeral** (`--ephemeral`): destroyed on shutdown, zero trace
- **Seal** (`batcave seal`): downgrades persistent → ephemeral. ONE-WAY. Cannot be reversed. Anti-coercion design.
- **Concurrent limit:** 32 caves (`MAX_CAVES` in `src/batcave/cave.rs`).
  This is a static-array cap, not a fundamental design choice — easy
  to bump when there's pressure (the constant is one number to
  change). Earlier doc revisions said "no limit"; STUMP #144 corrected
  that to match what the code actually enforces.

### Persistence implementation (STUMP #135)

The "survives reboots" guarantee is implemented in `src/batcave/persist.rs`.
Each Persistent cave is mirrored to BatFS as a `__cave__<name>` manifest
file (encrypted by BatFS itself with the boot master key). The manifest
captures the exact subset of `BatCave` state that needs to survive:

- name, cave_type, backing (Native/Docker), image (for Docker)
- granted capabilities, installed tools

Fields that DO NOT persist (re-derived/re-allocated each boot):

- `state` — caves wake up `Stopped`; running state is process-level
- `fs_key` — deterministic from `(boot master_key, name)`, so the
  cave's existing BatFS data still decrypts after restore
- `display_x/y/w/h` — re-allocated by `batcave enter`

Mutation hooks: `create`, `create_docker`, `grant_cap`, `revoke_cap`,
`install_tool` all call `persist::save()` after they touch RAM. `seal`
and `destroy` call `persist::delete()`. `destroy_all` (deadman / duress
/ panic / wipe) deletes every manifest before zeroing the in-RAM table
— a wipe must take out the registry too, otherwise the next boot
resurrects everything.

`cave::init` calls `persist::restore_all()` after `batfs::init`
unlocks the filesystem with the operator passphrase. Each manifest
that decrypts cleanly (Poly1305 tag matches) is reinstalled into a
free `CAVES[]` slot. Tampered manifests are silently skipped — BatFS
returns `INTEGRITY VIOLATION` and the entry stays out.

### Anti-coercion property of seal

Seal must hold across reboots, not just within a single boot. If a
sealed cave's manifest survived to disk, an attacker who panicked the
operator into sealing → rebooted the box would see the original
Persistent cave reincarnate. `persist::delete(name)` inside `seal()`
makes the ratchet permanent: once sealed, the cave is gone from disk
forever.

## BatCave Manager (6th Desktop App)

Shows all BatCaves, status, type, installed tools, granted capabilities, active sessions. Full visual management alongside terminal commands.

## The npm Attack Scenario

A compromised npm package runs inside a Docker-backed BatCave:
1. Tries to read host filesystem → **BLOCKED** (no host FS capability + Mac
   kernel container isolation).
2. Tries to connect to C2 server → **BLOCKED** (not in `cave_policy`
   allowlist; SNI peek + token-bucket shaper enforce this on the egress
   NAT path).
3. Tries to read another BatCave → **BLOCKED** for Docker caves (separate
   container kernel namespace) and for native caves (separate `TTBR0_EL1`
   per cave, STUMP #145; cross-cave TLB flush on every switch).
4. Tries to keylog → bounding-box display clip is real (rendering can't
   leak to neighbour caves' framebuffer regions); active readback
   prevention is a future hardening pass.
5. It's trapped. A brain in a jar — modulo the limitations enumerated in
   the Security Model layers above.

## Decision Log

| # | Decision | Alternatives | Why Chosen |
|---|----------|-------------|------------|
| 1 | Both GUI + CLI | CLI only, GUI only | Max flexibility |
| 2 | Manual per-container caps | Profiles, zero+request | Clean, specific |
| 3 | Shared Linux kernel service | Per-container kernel, syscall layer | Docker model, best balance |
| 4 | Kali repos on demand | Pre-built images, standalone bins | Just works |
| 5 | Create + install incrementally | Upfront only, recipes | Flexible |
| 6 | Persistent/ephemeral per cave | All one type | Choice + one-way ratchet |
| 7 | All wipes kill all caves | Independent policies | No survivors |
| 8 | No concurrent limit | Fixed limits | M4 has the power |
| 9 | Traffic through secure pipeline | Direct access | No backdoors |
| 10 | Called "BatCaves" | Containers, sandboxes | On brand |
| 11 | BatCave Manager app | Terminal only | Visual management |

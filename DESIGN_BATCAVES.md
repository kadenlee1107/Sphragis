# BatCaves — Secure Container Runtime for Bat_OS

## Understanding Summary

- **What:** BatCaves — isolated container environments that run any Kali Linux tool inside Bat_OS
- **Why:** Full pentesting/security toolkit without compromising the host OS. Supply chain attacks (npm, etc.) are contained — a compromised tool can't escape its BatCave
- **Who:** Kaden — single user
- **How:** One shared minimal Linux kernel runs as a Bat_OS microkernel service. BatCaves are isolated environments on top. Tools installed from Kali repos on demand. GUI + CLI both supported.
- **Isolation:** Every BatCave starts with ZERO access. Capabilities granted explicitly per-cave (network, filesystem, display, USB, raw sockets, inter-cave IPC). Enforced by seL4-inspired capability system at the microkernel level.
- **Lifecycle:** Persistent or ephemeral. Persistent can downgrade to ephemeral (one-way ratchet — anti-coercion). Create with `--tools` or install incrementally.
- **Destruction:** Dead man's switch, duress wipe, panic hotkey — ALL BatCaves die with the OS. No exceptions.
- **Networking:** All BatCave traffic goes through the secure network pipeline (allowlist firewall → TLS → VPN → Tor).

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  BAT_OS DESKTOP                      │
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
│  │  Runs as Bat_OS userspace task   │                │
│  └──────────────┬──────────────────┘                │
├─────────────────┼───────────────────────────────────┤
│           MICROKERNEL                                │
│  Scheduler │ Memory │ IPC │ Capabilities             │
│  Every BatCave syscall → capability check            │
└─────────────────────────────────────────────────────┘
```

## Security Model (6 Layers)

1. **Memory isolation** — each BatCave gets own page table. Hardware MMU enforced.
2. **Capability gate** — every syscall passes through microkernel capability check. No cap = no access.
3. **Filesystem encryption** — per-BatCave derived encryption key. Destroy key = data unrecoverable.
4. **Network firewall** — all traffic through allowlist firewall + secure pipeline. No backdoors.
5. **Display sandbox** — GUI tools render to dedicated framebuffer region. Can't read other windows.
6. **Destruction guarantee** — all wipe events (deadman, duress, panic) destroy all BatCave keys.

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
- `display` — GUI window on Bat_OS desktop
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

A compromised package runs inside a BatCave:
1. Tries to read host filesystem → **BLOCKED** (no host FS capability)
2. Tries to connect to C2 server → **BLOCKED** (not in firewall allowlist)
3. Tries to read another BatCave → **BLOCKED** (separate page table)
4. Tries to keylog → **BLOCKED** (display sandbox)
5. It's trapped. A brain in a jar.

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

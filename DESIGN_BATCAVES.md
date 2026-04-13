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
- **No limit** on concurrent BatCaves — hardware decides

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

# Bat_OS dev architecture — the machines and how they fit

## The three pieces

```
┌──────────────────────────────────────────────┐
│  GitHub (kadenlee1107/Bat_OS, private)       │
│  • Source of truth for everything            │
│  • Both Claudes pull/push                    │
│  • Survives any single-machine failure       │
└────┬──────────────────────────────────┬──────┘
     │ git                              │ git
     ▼                                  ▼
┌─────────────────────┐        ┌─────────────────────┐
│  Mac M4 (Mac16,1)   │        │  Ubuntu host        │
│  = the TARGET       │        │  = the PROXY HOST    │
│                     │        │                      │
│  In macOS:          │        │  Always reachable    │
│  • Mac Claude       │        │  via Tailscale       │
│    runs here        │        │  • Ubuntu Claude     │
│  • Rust toolchain   │        │    runs here         │
│  • `cargo build`    │        │  • Drives m1n1       │
│    produces         │        │    chainload via     │
│    target/          │        │    /dev/ttyACM0      │
│    bat_os_apple.bin │        │  • Captures serial   │
│                     │        │    to logfile        │
│  In m1n1 mode:      │        │  • Captures screen   │
│  • iBoot → m1n1 →   │        │    via Elgato (USB)  │
│    chainloaded      │        │  • Can run cargo     │
│    Bat_OS           │        │    too (rebuilds    │
│  • macOS NOT active │        │    without round-    │
│  • Mac Claude       │ USB-C  │    trip to Mac)      │
│    CANNOT run here  │───────▶│                      │
└─────────────────────┘        └─────────────────────┘
        ▲                                 ▲
        │ HDMI                            │ USB-in
        │ (USB-C→HDMI                     │
        │  adapter on MBA)                │
        └──── Elgato capture card ────────┘
```

## Who does what

| Operation | Where it happens |
|---|---|
| Edit Bat_OS Rust source | Mac or Ubuntu (either can) |
| Build `bat_os_apple.bin` | Mac (primary), Ubuntu (possible after `rustup` install) |
| Build/patch m1n1 | Mac (already done) |
| Run `chainload.py` | Ubuntu only (Windows can't; Mac isn't the host, it's the target) |
| Receive serial output | Ubuntu (via USB-CDC-ACM on /dev/ttyACM0) |
| Capture screen | Ubuntu (via Elgato as /dev/video0), optional |
| Write to SESSION_JOURNAL | Whichever Claude is active |
| `git commit` + `git push` | Whichever Claude is active |
| Install Asahi/kmutil stuff | Mac (in Recovery) — already done once |

## The mutual-exclusion constraint

**Mac can be in macOS XOR m1n1, never both.** This is the fundamental
constraint that shaped the whole workflow:

- When the Mac is in **macOS**: Bat_OS is NOT running. Mac Claude
  is available. Build, code, commit here.
- When the Mac is in **m1n1**: Bat_OS might be running. Mac Claude
  is unavailable. Ubuntu Claude drives the proxy, captures output.

The Claudes don't overlap in time but they share state via GitHub.
Commits are the baton.

## The session journal flow

```
Mac Claude finishes work
  → commits + pushes to GitHub
  → appends to SESSION_JOURNAL.md
  → user reboots Mac into m1n1

Ubuntu Claude wakes up
  → git pull
  → reads SESSION_JOURNAL.md top entry
  → does the work described
  → appends NEW entry with results
  → commits + pushes

User reboots Mac back to macOS
  → Mac Claude wakes up
  → git pull
  → reads Ubuntu Claude's new entry
  → continues the work
```

## Why GitHub, not just a shared local folder

Three reasons:
1. **Power-loss survival.** Ephemeral Ubuntu live-USB used to lose
   everything. Now insight flows through GitHub → durable.
2. **Version history.** Every diff, every debug attempt, every revert
   is preserved. If a change broke boot, we can bisect to the exact
   commit.
3. **No direct connection needed.** Mac and Ubuntu never have to be
   on at the same time, connected at the same time, or even reachable.
   GitHub is the dead drop.

## Tailscale for live driving (after basic setup works)

Once persistent Ubuntu is set up + Tailscale is running on both
machines, Mac Claude can drive Ubuntu remotely via SSH over the
Tailscale VPN. Example session:

```
Mac Claude: "let's rebuild and reboot"
  → cargo build --release (on Mac)
  → scp target/bat_os_apple.bin ubuntu:~/batos/latest.bin
  → ssh ubuntu 'cd ~/batos && ./scripts/chainload.sh latest.bin'
  → ssh ubuntu 'tail -100 ~/batos/serial.log'
  → reads the serial log in the chat, diagnoses
```

This way Mac Claude can exercise the full build → deploy → test loop
*while* the Mac itself is in macOS. You only lose this capability
during the brief window when the Mac is in m1n1 mode.

## File layout at a glance

```
/Users/kadenlee/Bat_OS/                    (Mac side)
~/bat_os/                                  (Ubuntu side, after clone)
├── CLAUDE.md                 ← Claude onboarding (both sides)
├── UBUNTU_QUICKSTART.md      ← Paste-and-go for Ubuntu
├── docs/
│   ├── SESSION_JOURNAL.md    ← Async handoff between Claudes
│   ├── M4_GROUND_TRUTH.md    ← RE findings, authoritative
│   ├── ARCHITECTURE.md       ← this file
│   ├── DEBUGGING_RUNBOOK.md  ← known failure modes
│   ├── UBUNTU_SETUP.md       ← persistent Ubuntu setup
│   └── photos/               ← screen photos (historical)
├── src/                      ← Bat_OS kernel source
├── external/m1n1/            ← pre-patched m1n1 (with -S flag)
├── scripts/                  ← Ubuntu-side automation
│   ├── setup.sh              ← one-time Ubuntu env setup
│   ├── chainload.sh          ← chainload with serial capture
│   ├── rebuild.sh            ← cargo build (works on either side)
│   ├── capture_serial.sh     ← tail+save serial to log
│   ├── capture_screen.sh     ← Elgato frame grab
│   └── sync.sh               ← pull/push helper
└── target/                   ← cargo artifacts (gitignored)
```

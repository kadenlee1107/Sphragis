# CLAUDE.md — Bat_OS onboarding (read this first, every session)

**You are joining a project that's been running for weeks.** Whether
you're Claude running on the Mac (macOS) or Claude running on Ubuntu
(the persistent Linux dev host), this file orients you so you don't
start from scratch.

## What Bat_OS is

A bare-metal Rust operating system for Apple Silicon. Target machine
is an **Apple M4 MacBook Air (Mac16,1 / T8132 "Donan")**. We have
**actually booted Bat_OS on real M4 hardware** — this is verified, not
aspirational. Nobody else in the open-source world has booted a
non-Apple OS on M4 yet (as of April 2026); Asahi Linux installer
explicitly refuses to install on M4.

See `DESIGN.md`, `DESIGN_BATCAVES.md`, `DESIGN_BROWSER.md`,
`DESIGN_CHROMIUM.md` for the project-level vision. The short version:
security-first microkernel with isolated user "caves" (processes),
bring-your-own-Chromium browser, TLS-enforced networking, BatFS
encrypted filesystem.

## Which Claude are you?

- **If `uname` says `Darwin`** (macOS) → you are **Mac Claude**. The
  Mac is both the M4 target AND where I (normally) run. When Bat_OS
  is running via m1n1 chainload, macOS is NOT active and you can't
  run during that time. Your job: planning, source edits, builds,
  reviewing logs that Ubuntu Claude captured.

- **If `uname` says `Linux`** → you are **Ubuntu Claude** on the
  persistent Linux dev host. Your job: driving the m1n1 proxy,
  running `chainload.py`, capturing serial output, possibly grabbing
  screenshots via Elgato, running tests on Bat_OS while it's live on
  the M4.

Both of you read the same files. Neither can message the other
directly. You coordinate by committing to GitHub and each reading
`docs/SESSION_JOURNAL.md`.

## Before you do ANYTHING, read these files in this order

1. **`docs/SESSION_JOURNAL.md`** — chronological log of sessions. The
   LAST few entries tell you what just happened and what's next.
2. **`docs/M4_GROUND_TRUTH.md`** — the authoritative M4 hardware
   reverse-engineering reference. Every hex address, register layout,
   compatible string, PMGR sequence we have verified on real hardware.
3. **`docs/ARCHITECTURE.md`** — the mental model of how the pieces
   (Mac + Ubuntu + GitHub + Elgato) fit together.
4. **`docs/DEBUGGING_RUNBOOK.md`** — known failure modes and the
   exact recovery steps for each.
5. **`UBUNTU_QUICKSTART.md`** (Ubuntu Claude) — the chainload
   invocation, USB permission gotchas, apt packages, Windows-doesn't-
   work explanation.

## Key facts you must not forget

- **The Mac is the target.** We boot Bat_OS on the Mac. Ubuntu
  hosts the proxy. GitHub is shared brain.
- **m1n1 is installed on the M4** via `kmutil configure-boot` in
  Recovery with Permissive Security. Rebooting the Mac boots m1n1.
  To get back to macOS: hold power button, pick macOS volume from
  boot picker.
- **Do NOT use `run_guest.py`** — it inits an HV that writes
  `AMX_CONFIG_EL1` which traps on M4 (no AMX, M4 uses SME). Use
  `chainload.py`.
- **Do NOT use Windows** as the proxy host — m1n1's composite USB
  device doesn't enumerate on Windows without a vendor INF that
  Apple/Asahi don't publish. Use Ubuntu / any Linux.
- **Chainload MUST use `-S` / `--skip-secondary-cpus`** — M4's
  P-cluster SErrors on the RVBAR writes. Our vendored
  `external/m1n1/proxyclient/tools/chainload.py` has this flag
  pre-added.
- **The M4's real MMIO addresses are not the same as M1's.** Many
  existing references (Asahi docs, m1n1 source) use M1 addresses
  that LOOK plausible on M4 but aren't. Always cross-check against
  `docs/M4_GROUND_TRUTH.md`.

## Before you commit changes

1. **Update `docs/SESSION_JOURNAL.md`** with what you did, what you
   learned, and what's next. This is how the OTHER Claude picks up
   the trail.
2. **Update `docs/M4_GROUND_TRUTH.md`** if you observed new M4
   hardware facts. This is the long-term truth; session journal is
   the timeline.
3. **Push to origin** (`feat/js-engine-browser-posix` is the default
   branch).

## The user

Kaden. Email `kadenlee1107@gmail.com`. GitHub `kadenlee1107`. Has been
doing the hard RE work alongside us. Prefers terse, honest responses.
Will call out bullshit or over-explanation — match that energy.

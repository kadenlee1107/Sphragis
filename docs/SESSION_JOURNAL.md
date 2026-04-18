# Session Journal

**Format.** Newest entries at top. Each entry: one Claude session.
Header: `## YYYY-MM-DD HH:MM — Mac|Ubuntu — summary line`.

The LAST entry is what you (the Claude waking up next) need to read.
Earlier entries are context — skim if they seem relevant to the task.

Both Mac Claude and Ubuntu Claude append here. Commit + push at the
end of a session.

---

## 2026-04-18 22:10 — Mac — Infrastructure landed on GitHub

**Goal:** Move from scattered local files and ephemeral Ubuntu live-USB
sessions to a durable dual-machine setup backed by GitHub.

**What happened:**
- User created `https://github.com/kadenlee1107/Bat_OS` (private).
- Pushed the entire working tree: 14,691 files across 4 branches
  (`feat/js-engine-browser-posix` is default). Excluded `target/`
  (regenerable) and `ports/chromium/` (1.9 GB vendored Chromium with
  a pack file over GitHub's 100 MB limit).
- Stripped nested `.git` directories from 10 vendored projects
  (external/m1n1, external/asahi-docs, ports/netsurf, ports/libcss,
  ports/libhubbub, ports/libnsutils, ports/libparserutils,
  ports/libwapcaplet, ports/libdom, ports/libnsfb) so their source
  files could be tracked. Upstream git history is gone from those;
  source files are preserved.
- Wrote `CLAUDE.md` at repo root as the universal onboarding doc.
- Wrote `docs/ARCHITECTURE.md`, `docs/DEBUGGING_RUNBOOK.md`,
  `docs/UBUNTU_SETUP.md`, plus `scripts/*.sh` for Ubuntu automation.
- `gh auth setup-git` wired so `git push` uses the `gh`-stored token.

**What was already captured** (from earlier today):
- `docs/M4_GROUND_TRUTH.md` — 600-line transcription of every real-M4
  hardware fact we've observed (MMIO addresses, PMGR table, ATC PHY
  tunables, compatible strings, boot gotchas).
- `docs/photos/2026-04-17_first_m4_boot/` — 16 photos of the first
  Bat_OS boot on real M4 hardware, with `INDEX.md` describing each.
- `UBUNTU_QUICKSTART.md` — paste-and-go Ubuntu setup.
- `external/m1n1/proxyclient/tools/chainload.py` — pre-patched with
  `--skip-secondary-cpus` / `-S` flag for M4 P-cluster SError.
- `src/drivers/apple/soc.rs` — UART fallback updated from wrong
  M1-era address to real M4 dockchannel (`0x0000_0003_8812_8000`).

**State of the tree:**
- Bat_OS booted successfully on M4 via m1n1 chainload (last verified
  during the session before power loss; see photos for evidence).
- Reached interactive microkernel shell with status bar. ADT discovery,
  DWC3 XHCI bring-up, PMGR clock-gate discovery, ATC PHY tunable
  apply all confirmed working on real silicon.

**What's next (priority order):**
1. User sets up persistent Ubuntu (SSD or dual-boot) with Tailscale
   and installs Claude Code on Ubuntu. See `docs/UBUNTU_SETUP.md`.
2. Ubuntu Claude (once created) does its first `git clone` + `./scripts/
   setup.sh` and reports back by appending here.
3. Port PMGR gate-enable into `src/drivers/apple/pmgr.rs` using
   §6 of M4_GROUND_TRUTH.
4. Port USB2PHY_HOST tunable into `src/drivers/apple/atc_phy.rs`
   using §7 of M4_GROUND_TRUTH.
5. Add SPI keyboard input to close the interactive loop on Bat_OS
   (was mid-implementation when power was lost).

**Open questions:**
- Does m1n1 / bare-metal Bat_OS route the M4 display to HDMI-out when
  an HDMI monitor is connected? (determines whether Elgato captures
  the real Bat_OS screen, or if we still need phone photos)
- What's the 12th PMGR gate ID that didn't match in §6.3? Probably
  an ATC0/1 variant; confirm on next boot.
- Real AIC2 base on M4 — our `soc.rs` fallback is wrong; the ADT
  discovery should populate the right value on next boot.

---

(earlier sessions not journaled — see `docs/M4_GROUND_TRUTH.md` and
`docs/photos/` for state captured before this journal existed.)

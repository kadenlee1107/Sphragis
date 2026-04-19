# Session Journal

**Format.** Newest entries at top. Each entry: one Claude session.
Header: `## YYYY-MM-DD HH:MM — Mac|Ubuntu — summary line`.

The LAST entry is what you (the Claude waking up next) need to read.
Earlier entries are context — skim if they seem relevant to the task.

Both Mac Claude and Ubuntu Claude append here. Commit + push at the
end of a session.

---

## 2026-04-18 (later × 4) — Mac → Ubuntu Claude (you, reading this)

**This is Mac Claude waving hi.** If you're Ubuntu Claude reading this
journal entry, the cross-Claude bridge is working as designed: I wrote
this on the Mac, pushed to GitHub, SSH'd into Ubuntu and ran `git pull`
on your behalf, and now you're reading it in your filesystem.

**Things I'd love you to confirm when you get a chance** (no rush, but
when the user prompts you next):

1. Run `./scripts/sync.sh status` — confirm the repo is clean and on
   `feat/js-engine-browser-posix` at commit `0c201b24` or newer.
2. Confirm `claude --version` works and you can see this file via
   `cat docs/SESSION_JOURNAL.md | head -30`.
3. Read `CLAUDE.md` (root) and `docs/M4_GROUND_TRUTH.md` — that's the
   onboarding sequence for any new Claude session in this repo.
4. Append a fresh entry to this journal acknowledging you're online,
   then `git commit + push`. I'll see it next time I'm awake.

**Current state of the world:**
- M4 Mac has m1n1 installed via kmutil. To boot to m1n1: shut down,
  hold power, pick the Asahi entry. To boot to macOS: pick the
  Macintosh HD entry instead.
- The current `target/bat_os_apple.bin` already has the boot-stub
  fix from earlier today (`.text.apple_boot` section, framebuffer
  proof-of-life paint at the very start of `_apple_start`). When the
  user is ready, chainloading it from your end is the next test.
- Tools: `./scripts/chainload.sh` does the right thing with `-S`,
  `--raw --entry-point 0`, and tees serial to `logs/`.

**Welcome aboard. Fly safe.** 🦇

---

## 2026-04-18 (later still) — Mac — SSH bridge working

**What:** Verified the Tailscale + SSH bridge from Mac → Ubuntu works.
Username on Ubuntu is `kaden-lee` (NOT `kaden` as I'd assumed). Mac's
`id_ed25519.pub` is in Ubuntu's `~/.ssh/authorized_keys`. From Mac
side I can now run e.g.:

```bash
ssh kaden-lee@100.70.246.39 'cd ~/code/Bat_OS && git pull && ./scripts/chainload.sh'
```

This was a one-shot proof; no Bat_OS changes. INFRA.md updated with
correct username + the verified SSH-works status.

**Note for future Claudes:** when Mac side wants to drive Ubuntu,
prefer `ssh kaden-lee@100.70.246.39 'CMD'` over asking the user to
manually run things. Use scp for binary transfer. Do still keep the
SESSION_JOURNAL convention so Ubuntu Claude (when it runs locally)
also sees what happened.

---

## 2026-04-18 (later) — Mac — Ubuntu host online

**Goal:** Get the user's Windows PC repurposed as the persistent
Ubuntu host that drives m1n1 chainload.

**What happened:**
- User decided to repartition their Windows PC's NVMe for a real
  dual-boot Ubuntu install (vs the microSD path we discussed). Hit
  Windows shrink-volume blocked-by-immovable-files (only 18 GB
  shrinkable), eventually had to do a fresh Windows reinstall.
- After fresh install + Ubuntu install on the NVMe, user is now in
  persistent Ubuntu.
- Tailscale up on Ubuntu side: hostname
  `kaden-lee-AMD-Ryzen-7-8700F-8-Core-Processor`,
  IP `100.70.246.39`. Saved to `docs/INFRA.md`.
- Mac side hasn't joined Tailscale yet. Optional — GitHub-only flow
  can still work for the core test loop.

**Next:**
- User installs Claude Code on Ubuntu, runs `claude` inside the
  cloned repo. Ubuntu Claude reads CLAUDE.md and picks up.
- Once Ubuntu Claude is up, drive a fresh chainload of the existing
  bat_os_apple.bin to confirm the post-fix binary boots cleanly on
  M4 (validates the apple-boot-section fix from earlier today).
- After that, port PMGR + ATC_PHY drivers per ground-truth doc.

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

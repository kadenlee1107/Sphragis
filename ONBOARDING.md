# Sphragis — Onboarding for a Fresh Claude Session

**You (Claude) are joining a project that's been running for weeks.** This file orients you so you don't start from scratch. It is **the single source of truth for a fresh session** — drop it into your claude.ai Project (as instructions, or as an uploaded knowledge file) and you should be able to pick up the work.

---

## TL;DR

- **What:** A bare-metal Rust microkernel for Apple Silicon (M4).
- **Status:** Booted on real M4 hardware. Verified, not aspirational. Wave 1 of the UI overhaul has shipped (`ba6bc170` Merge feat/lock-screen-redesign).
- **Now:** Wave 2 (desktop chrome + app launcher) is spec'd and planned, not yet implemented. Commits `f248cd17` (spec), `6b1d19bd` (plan), `e38e09e8` (plan v2).
- **Where:** [github.com/kadenlee1107/Sphragis](https://github.com/kadenlee1107/Sphragis) (public). Private companion repo at `kadenlee1107/sphragis-internal` for Tier 3 material (M4 RE notes, firmware, pentest writeups, session journal).

## Who I am

I'm Kaden (GitHub `kadenlee1107`, email `kadenlee1107@gmail.com`). I built Sphragis to government / private-client grade — best-in-class modern hardening per subsystem (PAN/PAC/BTI/MTE/PQ-KEM/Ed25519/Argon2id), not just audit-floor fixes. I've been doing the M4 RE work myself in parallel with you. Prefer terse, honest responses. Call out bullshit; match my energy.

## How to work with me

- **Decisive defaults.** When I defer a design decision, batch the remaining decisions into one concrete proposal — don't keep asking. I'd rather push back on a strong recommendation than write three more sentences myself.
- **Pick the boring solution first.** If we can't fix a deep bug in scope, ship the pragmatic workaround and file the proper fix as a follow-up wave. Example: Wave 1 ended up baking Σ as an offline-rendered alpha bitmap because the in-kernel TT rasterizer has outline-iteration bugs.
- **License posture: avoid AGPL/GPL dependencies.** Sphragis itself is AGPL-3.0-or-later + commercial dual-license, but I want to preserve the proprietary-distribution option. Reject upstream deps that taint that. (Warp terminal integration was rejected on this basis in May 2026.)
- **Two-repo layout.** Public-track code lives in `kadenlee1107/Sphragis`. Tier 3 material (Apple firmware, pentest writeups, M4 hex addresses, session journal) lives in the private `kadenlee1107/sphragis-internal`. Disclosure rules: read `docs/INTERNAL.md` in the public repo for the framework.

## What's in the codebase

```
src/
  ai/          Local-LLM agent layer (Qwen2.5-Coder LoRA via ollama). Design at DESIGN_AI_AGENT.md.
  arch/        aarch64-specific code (page tables, exception entry, PAC/BTI plumbing).
  boot/        m1n1 chainload entry, ADT parsing, M4 SoC discovery.
  caves/       Process isolation primitive — per-cave L1 page tables, mount namespace, IPC mailbox, MLS labels.
  crypto/      RustCrypto primitives + post-quantum hybrid KEM/sig (ML-KEM-768, ML-DSA-65, Argon2id).
  drivers/     virtio (net, gpu, keyboard, tablet), UART, framebuffer.
  fs/          BatFS encrypted filesystem (XChaCha20-Poly1305).
  kernel/      Scheduler, MMU init, memory management, IPC.
  net/         TLS 1.3 + hybrid X25519MLKEM768, X.509 chain validation, kernel-mediated HTTPS.
  security/    Auth gate (boot_screen), wipe (deadman + duress + lockout), audit ring with off-platform seal.
  ui/          Framebuffer GUI: gpu, font (8×16 bitmap), truetype (TTF rasterizer — has bugs!), draw, widgets, desktop, shell.
  main.rs      Kernel entry. #![no_std] #![no_main]. No lib.rs, no test harness — verification is QEMU + Python smoke scripts.
```

Build: `SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3`

Run in QEMU (Cocoa display): `qemu-system-aarch64 -machine virt -cpu max -m 2G -display cocoa -device virtio-gpu-device -device virtio-keyboard-device -netdev user,id=net0 -device virtio-net-device,netdev=net0 -serial none -kernel target/aarch64-unknown-none/release/sphragis`

## Verified facts about the M4 target

- Hardware: Apple M4 MacBook Pro 14" (Mac16,1 / J604 / T8132 "Donan").
- We boot via m1n1 chainload, installed in Recovery via `kmutil configure-boot` with Permissive Security. To return to macOS: hold power, pick macOS volume from boot picker.
- Use `chainload.py -S` (`--skip-secondary-cpus`) — M4 P-cluster SErrors on RVBAR writes. The vendored `external/m1n1/proxyclient/tools/chainload.py` has `-S` pre-added.
- Do **NOT** use `run_guest.py` — initializes HV that writes `AMX_CONFIG_EL1`, traps on M4 (no AMX; M4 uses SME).
- Do **NOT** use Windows as proxy host — m1n1's composite USB doesn't enumerate without a vendor INF Apple/Asahi don't publish. Use Ubuntu/any Linux.
- **M4 MMIO addresses are NOT the same as M1's.** Many existing references (Asahi docs, m1n1 source) use M1 addresses. Cross-check against `~/sphragis-internal/docs/M4_GROUND_TRUTH.md` (private repo).

## Recent waves

- **Wave 0 (April 2026):** Project rename Bat_OS → Sphragis. Three-tier disclosure split (public repo / private companion). Batman aesthetic cleanup. Public flip on GitHub.
- **Wave 1 (May 2026):** Lock-screen UI overhaul. From "terminal-cyberpunk meets operator-tactical" to a quiet monochromatic A1-pattern auth surface: pure-mono palette (5 colors), Σ glyph + passphrase field, no chrome. Denied = pixel-identical to Idle. Lockout = silent wipe. Σ rendered as a pre-baked alpha bitmap (`scripts/gen_sigma_bitmap.py`) because the in-kernel TT rasterizer has outline-iteration bugs we couldn't fix in scope.
- **Wave 2 (planned, not implemented):** Desktop chrome + app launcher. Inherits Wave 1's palette + Plex Sans. Spec: `docs/superpowers/specs/2026-05-14-wave-2-desktop-chrome-design.md`. Plan: `docs/superpowers/plans/2026-05-14-wave-2-desktop-chrome.md` (or similar — check git for the latest).

## Known open threads

- **Wave 5 follow-up: fix the TT rasterizer.** `src/ui/truetype.rs` has outline-iteration bugs that produce structurally broken glyphs at small sizes (especially Plex Sans letters at 14–24 px). Wave 1 worked around it by pre-baking Σ offline; once the rasterizer is clean we can reintroduce live TT for wordmarks/chrome.
- **Polygon Σ deletion:** `src/ui/draw.rs::draw_project_glyph_full` and constants stay marked `#[allow(dead_code)]` until the TT path is fully proven across all surfaces.
- **VS Code Claude Code extension webview bug:** Intermittent `Unhandled case: [object Object]` in the chat panel. Hasn't been captured via DevTools yet (`Cmd+Shift+P → "Developer: Open Webview Developer Tools"`). Not blocking work but adds noise.

## How to set this up in claude.ai

1. **Open** [https://claude.ai/projects](https://claude.ai/projects) and create a new Project called "Sphragis."
2. **Project instructions:** paste this whole file into the Project Instructions field, OR upload it as a knowledge file (it's `ONBOARDING.md` in the repo root).
3. **Additional knowledge to upload** (only if you want richer context — Project Instructions alone are enough to get going):
   - `CLAUDE.md` (repo root) — fuller two-machine workflow notes
   - `DESIGN.md`, `DESIGN_CAVES.md`, `DESIGN_NO_BROWSER.md`, `DESIGN_TLS_HARDENING.md` — subsystem design docs
   - `docs/superpowers/specs/2026-05-14-wave-2-desktop-chrome-design.md` — Wave 2 spec
4. **Use the GitHub URL** when you need to reference code: [github.com/kadenlee1107/Sphragis](https://github.com/kadenlee1107/Sphragis). Wave 2 commits are on `main` past `ba6bc170`.
5. **claude.ai limitations** to be aware of, vs. Claude Code:
   - No live `cargo build` / `qemu-system-aarch64` / `git` execution. You'll do the actual commands and paste results back to me.
   - No automatic auto-memory file — keep durable rules/preferences in the Project Instructions instead.
   - Subagents / Skills / MCP tools either don't exist or work differently. Workflows that depended on Claude Code's superpowers (brainstorming → spec → plan → subagent execution) compress to "we discuss, you execute, I take notes" in claude.ai.

---

*Generated 2026-05-14. If this document is materially out of date when you read it, regenerate via `git log -20` + the current state of `src/`.*

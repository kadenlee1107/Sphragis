# Lock Screen Redesign — Design

**Date:** 2026-05-14
**Status:** Approved (brainstormed); pending implementation plan
**Wave:** 1 of N (UI overhaul; see *Scope boundary*)

## Goal

Replace the "terminal-cyberpunk meets operator-tactical" boot/lock screen with a quiet, monochromatic, Apple-grade auth surface that reads as a serious institutional tool rather than a developer demo. Strip every UI element that exists for the developer's benefit rather than the operator's. Bake a threat-model principle directly into the visual design: a denied attempt is visually indistinguishable from idle.

## Motivation

Two complaints converged:

1. **Clutter.** The current lock screen exposes kernel-internal facts (SHA-256 KDF · 16 ROUNDS, fake boot log, fake clock, AES-256-CTR pill, hostname, build hash, "SIGNED") that an end user neither needs nor benefits from. Some of it (caps indicator wired through `keyboard::caps_active` / `tablet::caps_active`) historically didn't even work.
2. **Register.** Bright cyan (`#22D3EE`) on near-black with crosshair corner marks reads as a hacking-movie prop, not as a system a government or private-client buyer would deploy. The visual identity is working against the product positioning.

A third motivation surfaced during brainstorming: **silent denial is also a security property.** If a failed attempt looks visually identical to idle, an attacker brute-forcing the field has no signal whether their last 7 characters were close, wrong, or whether the system registered the input at all. A legitimate user retries — the dots reappear when they do. The two audiences are correctly served by different cues.

## Visual system

### Palette (pure mono, no accent)

Five named colors. No state-color overlay (no red on denial, no green on grant, no cyan on focus). Color is removed entirely from the lock screen.

| Name | Hex | Use |
|------|-----|-----|
| `BG` | `#0d0d10` | screen background |
| `PANEL` | `#18181c` | field panel fill |
| `HAIRLINE` | `#2a2a30` | field border |
| `INK` | `#e5e7eb` | primary text + glyph |
| `MID` | `#6b7280` | available for placeholder / inert states (unused in v1) |

Hex literals are encoded as `(A<<24)|(R<<16)|(G<<8)|B` in source per `boot_screen.rs` line 29–37 (framebuffer is `B8G8R8A8`; little-endian store lands as B,G,R,A).

### Typography

Two faces, one type family. Both shipped permissively (SIL OFL — compatible with AGPL-3.0).

| Element | Face | Weight / Style | Size |
|---------|------|----------------|------|
| Σ glyph | IBM Plex Serif | Italic 400 | ~96 px cap height |
| Wordmark "SPHRAGIS" | IBM Plex Sans | Medium 500, `letter-spacing: 0.4em` | ~14 px |

Both faces subsetted to a fixed codepoint set: `A`–`Z`, `0`–`9`, `Σ`, space, `.`, `-`. That's 40 codepoints — covers the wordmark plus modest forward-compat for any future small text on this surface. Embedded as `include_bytes!` blobs into the kernel binary. Estimated combined footprint after subsetting: ~280 KB; if it ends up materially larger after running the subsetter, the codepoint list is the dial to turn first.

### Glyph rendering

The current `draw::draw_project_glyph_full` polygon-rasterized Σ (top bar + bottom bar + two parallelogram diagonals) is replaced by a TrueType render via the existing `src/ui/truetype.rs` rasterizer, which becomes wired into the boot-screen paint path for the first time. The polygon Σ path stays in the codebase under `#[allow(dead_code)]` for one wave so we can revert if TT proves unstable; it gets deleted in Wave 2.

## Layout

A centered three-element vertical stack. Nothing else on screen.

```
                                                    
                                                    
                          Σ                         
                                                    
                       SPHRAGIS                     
                                                    
                ┌──────────────────────┐            
                │                      │            
                └──────────────────────┘            
                                                    
                                                    
```

### Element specs

| Element | Position | Notes |
|---------|----------|-------|
| Σ glyph | horizontally centered, ~80 px above screen vertical center | INK on BG, no fill effects |
| Wordmark | horizontally centered, 16 px below glyph baseline | INK on BG |
| Field panel | horizontally centered, 40 px below wordmark | 480 × 56 px, PANEL fill, 1 px HAIRLINE border |
| Typed dots | inside the field, centered vertically | 8 × 8 px squares of INK, 8 px gap between dots |

### What is *not* on screen

This is a complete enumeration. The implementation plan should treat anything not listed above as something to remove.

- No status pills (`ENCRYPTED`, `BATFS`, `M1N1`, `NET`).
- No system identity strip (`HOST … KERNEL … ARCH …`).
- No hairline rules.
- No version line (`V0.5.0-DEV · BUILD … · SIGNED`).
- No `[AUTH] PASSPHRASE` label.
- No `SHA-256 KDF · 16 ROUNDS` label.
- No chevron `>` prompt inside the field.
- No inline attempts indicator inside the field.
- No cursor (caret, blinking or otherwise). The dot count *is* the cursor position.
- No helper hint row (`RETURN TO SUBMIT · BACKSPACE TO EDIT`).
- No caps-lock indicator.
- No crosshair corner marks.
- No boot log block.
- No fake system clock block.
- No attempts-remaining pill.
- No denial overlay.
- No "SYSTEM LOCKED" banner.
- No "ACCESS GRANTED" text.

## State machine

Four states. **Idle and Denied are pixel-identical.**

```
            ┌──────────┐
            │   IDLE   │  empty field
            └────┬─────┘
                 │ keypress
                 ▼
            ┌──────────┐
            │  TYPING  │  one dot per char
            └────┬─────┘
       ┌─────────┼─────────┐
       │ enter   │ enter   │ enter
       ▼ wrong   ▼ right   ▼ duress
  ┌─────────┐ ┌──────────┐ ┌──────────────┐
  │  DENIED │ │ GRANTED  │ │ FAKE-PANIC    │ (existing path, unchanged)
  └────┬────┘ └────┬─────┘ └──────────────┘
       │           │
       │ 2.5s      │ ~200 ms
       │ silent    │ then desktop
       ▼ cooldown
   back to IDLE
   (identical pixels)
```

### State definitions

**Idle.** Glyph, wordmark, empty field. No microcopy.

**Typing.** Glyph, wordmark, field with one INK dot per typed character. Backspace removes the trailing dot; once dots reach zero, the screen is pixel-identical to Idle.

**Denied.** On `AuthResult::Failed`, the screen returns to Idle (empty field) immediately. Input is ignored for 2.5 s of silent cooldown (matches existing `HOLD_DENIED_MS`). No microcopy, no color change, no shake, no border tint. After cooldown, input is accepted again. **An attacker watching the screen has no visible signal that their attempt was rejected.** A legitimate user knows because their typed dots vanished; if they begin typing again, dots reappear and they recover.

**Granted.** On `AuthResult::Success`, the screen holds for ~200 ms with the typed dots still visible (no "ACCESS GRANTED" text, no green tint), then transitions to the desktop. The hold is short enough to feel responsive, long enough to confirm "yes, it took." This is a deliberate change from the current `HOLD_GRANTED_MS = 900` — the longer hold existed to give the user time to read the green "ACCESS GRANTED" text; without that text, 200 ms is plenty. The implementation plan updates `HOLD_GRANTED_MS` accordingly.

**Lockout (attempts exhausted).** On `AuthResult::LockedOut`, the screen stays in Idle while `wipe::execute(WipeReason::Lockout, …)` runs in the background. After wipe completes, the system halts in a `wfe` loop. No "SYSTEM LOCKED" overlay. An attacker has no visible signal that they triggered the wipe; they see the same idle screen they've seen all along, then eventually the screen never wakes again.

**Duress.** `AuthResult::Duress` continues to invoke the existing `fake_boot_and_wipe` decoy path unchanged. Out of scope for this spec.

## What this spec removes

Concrete kill list. The implementation plan deletes each of these by name.

### From `src/security/boot_screen.rs`

- Functions: `draw_status_pill`, `draw_corner`, `draw_boot_log`, `draw_clock_block`.
- Tables: `BOOT_LOG`.
- Constants: `HAIRLINE_Y`, the cyan / green / amber / red / faint / hair-hi / dim-txt palette names.
- The four pill calls in `paint_lock_screen` and their associated identity strip.
- The version line draw call.
- The `[AUTH] …` label draw call and the KDF label draw call.
- The chevron draw, the cursor draw, the inline attempts-counter draw.
- The helper hint row draw and the caps-lock draw.
- The `ACCESS DENIED` overlay in `paint_lock_screen` (the entire `if state == LockState::Denied { … }` block).
- The `SYSTEM LOCKED` banner in `run()`'s `LockedOut` arm.
- The `ACCESS GRANTED` text inside the field (the entire `if let LockState::Granted(_) = state { … }` granted-text block).

### From `src/ui/draw.rs`

Nothing deleted in this wave. `draw_project_glyph_full` and its constants stay marked `#[allow(dead_code)]` so we can revert if TT rasterization disappoints. Wave 2 removes them.

### Palette shrinks

From 16 named colors (`BG`, `PANEL`, `HAIR`, `HAIR_HI`, `INK`, `MID`, `DIM_TXT`, `FAINT`, `CYAN`, `CYAN_DIM`, `GREEN`, `GREEN_DIM`, `AMBER`, `AMBER_DIM`, `RED`, `RED_DIM`) down to 5 (`BG`, `PANEL`, `HAIRLINE`, `INK`, `MID`).

### Driver-level state kept intact

Caps-lock plumbing (`crate::drivers::virtio::keyboard::caps_active`, `crate::drivers::virtio::tablet::caps_active`) is **not** removed. Other consumers (shell, future apps) may want it. The lock screen simply stops reading it.

## Implementation outline

Sketch only. The implementation plan (separate document, written next) decomposes this into bite-sized tasks.

1. **Font infrastructure.** Verify `src/ui/truetype.rs` rasterizes glyphs into the framebuffer correctly via a small offline harness. Subset IBM Plex Serif Italic and IBM Plex Sans Medium to the fixed codepoint set above. Embed as `include_bytes!`. Extend `src/ui/truetype.rs` (not a new file) with the API surface the boot-screen paint needs: `draw_glyph_at(codepoint, x, y, size_px, color)` and `glyph_width(codepoint, size_px)`.
2. **New boot-screen paint.** Rewrite `paint_lock_screen` end-to-end against the new layout and state machine. The four-state enum stays in shape (`Idle / Typing / Denied / Granted`); `Denied` calls the same painter as `Idle`. Delete every helper listed above.
3. **State-machine wiring.** Change `run()` so `AuthResult::Failed` does the 2.5 s silent cooldown and returns to the idle paint, with no denial overlay. Change `AuthResult::LockedOut` to skip the banner and go straight to `wipe::execute`.
4. **Palette cleanup.** Trim color constants to the 5 we keep; delete the rest. Verify nothing else in the kernel imported them (`rg 'boot_screen::(CYAN|GREEN|AMBER|RED)'`).
5. **Verification.** Boot under QEMU, walk every state — idle → typing → wrong → silent return → typing → correct → desktop. Then exhaust attempts and confirm silent wipe. No screenshot diffs (we have no baseline); manual visual confirmation against this spec's layout.

## Scope boundary

This is **Wave 1**. The user's brief was "we need to redo everything but we will just focus on the lock screen for now." Subsequent waves are named here so they don't accidentally drift into this spec's plan.

| Wave | Surface | Notes |
|------|---------|-------|
| **2** | Desktop chrome | Title bar, taskbar, window decorations, app launcher. Inherits Wave 1's palette + Plex Sans. Own layout brainstorm. |
| **3** | Shell + console palette | `src/ui/shell.rs` (11k lines) and `src/ui/console.rs` use ad-hoc colors. Same palette/font system applied. |
| **4** | Widget pack | `src/ui/widgets.rs` buttons, inputs, menus. |
| **5** | Cursor blink + optional scanline overlay | Cosmetic. Requires the 1 Hz timer hook the original Claude-Design spec called for. Not blocking on this wave. |
| **Follow-up** | Polygon Σ deletion | Once TT is proven across all surfaces, `draw_project_glyph_full` and its constants go. |
| **Follow-up** | Static status surfacing | Where do "network isolated", "deadman armed", "encrypted-fs OK" appear if not on the lock screen? Answer probably lives in Wave 2's system tray. |

## Non-goals

- No animation (no cursor blink, no fade transitions, no scanline overlay) in this wave.
- No accessibility audit, screen-reader support, or alternate-input modality. The lock screen is keyboard-only by design.
- No internationalization. "SPHRAGIS" is the only word on screen; it doesn't translate.
- No theme system. The palette is hardcoded. Wave 2 may introduce one.
- No telemetry, no metrics. The lock screen reports nothing externally.

# Desktop Chrome + App Launcher — Design

**Date:** 2026-05-14
**Status:** Approved (brainstormed); pending implementation plan
**Wave:** 2 of N (UI overhaul; Wave 1 was the lock screen)

## Goal

Replace the current cyberpunk desktop with a quiet canvas + floating-window multi-app experience that inherits Wave 1's mono palette and silent register. The desktop is where the user lands after unlock and where they manage caves, files, network, security, comms, the AI agent, the shell, and the editor. It must feel like a "real operating system" — multiple apps open at once, drag-to-move windows, fast switching — without breaking the calm Wave 1 just established.

## Motivation

After Wave 1 landed the lock screen in a deliberately quiet register, the desktop sitting behind it still rendered in the old terminal-cyberpunk style. A user who unlocks an Apple-grade lock screen and then sees bright cyan status pills, fake boot logs, and chrome corner marks loses the credibility Wave 1 just built.

The desktop also has a structural problem: there's no clear paradigm. The current `src/ui/desktop.rs` (637 lines), `src/ui/wm.rs` (639 lines), and 8 separate apps grew organically without a unifying mental model. This wave decides what the post-auth experience IS, so the per-app redesigns (Wave 3 forward) have something to inherit.

## Mental model

The desktop is a **quiet canvas** with a **soft watermark Σ** centered behind everything. On the canvas, the user sees a **floating-window workspace**: tap an app icon to open it as a movable, resizable window; multiple windows can be visible at once; windows running inside a cave label the cave name in their chrome so contexts never collapse.

There is **no dock and no taskbar in v1**. Launching apps and switching windows happens through keyboard shortcuts (`⌘K`, `⌘TAB`) and a single click-target — the "SPHRAGIS" wordmark in the top-left — which summons the app grid as a full-screen overlay.

The top bar is a thin status strip. The user customizes which status badges appear; the only fixed elements are the brand wordmark on the left and a lock-glyph at the far right.

## Visual system

### Palette

Same five colors as Wave 1: `BG #0d0d10`, `PANEL #18181c`, `HAIRLINE #2a2a30`, `INK #e5e7eb`, `MID #6b7280`. No new colors. Two new derived tones used as background-only:

| Name | Hex | Use |
|------|-----|-----|
| `WATERMARK` | `#1c1d22` | the soft Σ watermark behind everything (~3-5% lighter than BG) |
| `SHADOW`    | rgba(0,0,0,0.6) | drop-shadow under floating windows |

### Typography

The kernel-side wordmark "SPHRAGIS" in the top-left renders at the 8×16 bitmap font's native size (so it stays crisp without scaling artifacts — the same constraint that drove the lock-screen wordmark drop in Wave 1). Status badges, window titles, and window-body labels also use the 8×16 bitmap font.

The watermark Σ is the same baked alpha bitmap shipped in Wave 1 (`src/ui/sigma_bitmap.rs`), drawn larger (e.g. 360 px) and with reduced alpha so it reads as background texture rather than a foreground glyph.

The Wave 5 follow-up (TrueType rasterizer fix) will eventually let us swap top-bar text to Plex Sans without rendering bugs.

### Layout — top bar

A 22-px-tall strip across the top of the screen. Background is the panel color with a 60 % opacity over the canvas so the watermark Σ shows through faintly.

**Left:** the wordmark "SPHRAGIS" (uppercase, letterspaced). Clicking it summons the app launcher overlay.

**Right (customizable status strip):** A user-configurable list of status badges, ending in two fixed glyphs:
- `⋯` — opens the config sheet to add / remove / reorder badges. Preference persists per-user in SealFS.
- `⏻` — locks the system (drops back to the Wave 1 lock screen). Same effect as `⌘L`.

**Default badge set (ships out of the box, in this order):**

1. `NET ISOLATED` / `NET ROUTED` — network mode. Always visible. Turns INK-bright when the mode flips.
2. `DEADMAN HH:MM` — countdown to the next deadman ratchet. Turns INK-bright at < 5 min.
3. `HH:MM` — 24-hour clock.

**Available but off by default:**
`CAVES n` (running count), `ATTEMPTS n` (only meaningful as a warning when below the threshold), `MEM x%`, `CPU x%`, `AUDIT <last entry>`, `CAVE <name of focused window's cave>`.

### Layout — desktop body

When no windows are open, the 8-app launcher grid is the canvas content. When windows are open, the grid dims to ~22% opacity and floats behind the windows. The grid is visible-but-non-interactive in this state — clicking through to a tile from behind windows does not launch (clicks fall through to the window or are absorbed by the canvas). The user re-enters launcher mode via the brand wordmark or `⌘K`.

**Launcher grid:** 4 columns × 2 rows, 8 apps. Each app is a 22 × 22 px solid silhouette icon placeholder above a 7-px label (e.g. `CAVES`). No background, no border on the tile itself — the icon + label sit directly on the canvas.

**The 8 apps:**

| Slot | App | Existing module |
|------|-----|-----------------|
| 1 | CAVES | `src/ui/apps/caves_mgr.rs` |
| 2 | FILES | `src/ui/apps/filemanager.rs` |
| 3 | NET | `src/ui/apps/netmon.rs` |
| 4 | SECURITY | `src/ui/apps/security.rs` |
| 5 | SHELL | `src/ui/shell.rs` (11 k lines — wrapped here as a launchable app) |
| 6 | EDITOR | `src/ui/apps/editor.rs` |
| 7 | COMMS | `src/ui/apps/comms.rs` |
| 8 | AGENT | (new — corresponds to the design at `DESIGN_AI_AGENT.md`; this wave only stubs the icon + opens an empty window) |

Per-app icon design is Wave 3 work. The Wave 2 placeholder is a solid 22 × 22 rounded square in `MID`.

### Layout — windows

Floating, movable, resizable. Default open position is roughly centered on the canvas; subsequent windows offset down-and-right so they don't stack identically.

**Chrome:** 22 px tall, `PANEL` background, 1 px `HAIRLINE` bottom border. Contains:
- A 8 × 8 px close glyph (open circle) at the far left.
- The window title text, INK-colored when focused, MID-colored when not.
- If the window is hosting an app inside a cave, the cave name is appended after a `·` separator: `SHELL · kali-recon`, `EDITOR · scratch-1`. Two open shells on different caves therefore never read as the same window.

**Body:** the app's own content. The app fills the entire space below the chrome. Apps are not aware of being floating-windowed; the WM clips and offsets their draws.

**Window border:** 1 px `HAIRLINE` all around.
**Drop shadow:** soft `SHADOW` color, 10 px y-offset, 30 px blur. Distinguishes the window from the canvas.

**Z-order:** click any window body or chrome to focus. The focused window draws on top of all others; its title is INK-bright.

**Resize:** drag any corner. No visible handle — corner is a 12 × 12 hit zone. Cursor changes on hover (host-OS-dependent in QEMU; for the real M4 path, the WM dispatches a resize cursor).

**Move:** drag the chrome. Anywhere in the chrome (except the close circle) is a drag handle.

**Close:** click the open circle, or press `⌘W`.

**Minimum window size:** 280 × 160 px. Below that, the chrome runs out of room for the close circle + title.

**No minimize, no maximize in v1.** Adding them is a Wave 3+ decision per app.

## State machine

The desktop has four states. The window manager handles transitions; apps don't see the state.

```
                ┌────────────────┐
                │   LAUNCHER     │  full-screen 8-app grid
                │   (canvas)     │
                └────────┬───────┘
            click app    │
                ▼
                ┌────────────────┐
                │   ACTIVE       │  ≥ 1 window open
                │   (windows)    │  launcher dimmed behind
                └────┬───────┬───┘
       ⌘K or         │       │ close last window
       click brand   │       ▼
                ▼     ┌──────────────┐
       ┌────────────┐ │  LAUNCHER    │
       │  OVERLAY   │ │  (canvas)    │
       │  (grid     │ └──────────────┘
       │   over win)│
       └────────────┘
            click app or outside → back to ACTIVE
```

**LAUNCHER (canvas):** no windows. 8-app grid is visible at full opacity. Top bar present.

**ACTIVE:** ≥ 1 window. Grid dims to 22 %; windows float in front. Top bar present.

**OVERLAY:** triggered by `⌘K` or clicking the brand. The grid renders at full opacity over the existing windows (windows stay in their positions underneath). Clicking an app launches it (returns to ACTIVE with a new window). Clicking outside the grid dismisses the overlay (returns to ACTIVE). `Esc` also dismisses.

**LOCK:** `⌘L` or clicking `⏻` exits `desktop::run()` and returns control to the boot-flow caller, which re-enters `boot_screen::run()` (the Wave 1 lock screen). On successful re-unlock, `desktop::run()` resumes — open windows + their positions persist in WM state, so the same workspace appears.

## What gets removed

Concrete kill list from the current `desktop.rs` / `wm.rs`:

- The cyberpunk-style desktop background (whatever the current 637-line `desktop.rs` paints).
- Existing cyan / amber / green / red status indicators on the desktop, if any.
- Any current "boot log" or "status grid" desktop widgets.
- The current top-bar / menu-bar / taskbar code, if it exists.

The 8 existing apps (`src/ui/apps/*`) are **NOT touched** in this wave. They get the new window chrome wrapper because the WM owns the chrome, but their internal cyberpunk aesthetic stays for now and gets fixed in Wave 3+.

## Implementation outline

Sketch only. The plan decomposes this into bite-sized tasks.

1. **WM rewrite (`src/ui/wm.rs`).** New `Window` struct (position, size, title, cave-name option, body-paint callback). New `WindowManager` struct (Vec of windows + focus index, modulo no-std-friendly fixed buffer — probably 8 max). Methods: `open(app)`, `close(window_id)`, `focus(window_id)`, `cycle()`, `move_drag(window_id, dx, dy)`, `resize_drag(window_id, corner, dx, dy)`, `paint_all(fb, w, h)`.

2. **Desktop rewrite (`src/ui/desktop.rs`).** New `paint_desktop` that paints in z-order: canvas BG → watermark Σ → launcher grid (opacity per state) → all windows (back to front) → top bar (on top of everything). `desktop::run()` event loop dispatches keyboard / pointer events to the WM or the top-bar / launcher per the state machine.

3. **Top bar (`src/ui/topbar.rs` — new file).** A small focused module: `paint_topbar(fb, w, badges)` and `handle_click(x, y)`. The badge list lives in WM state and is loaded from SealFS on `desktop::init()`.

4. **App-icon plumbing.** Each app gets registered with a name, a 22×22 icon (currently the placeholder rounded square — a TODO comment marks where the real icon goes during Wave 3), and an `open()` callback returning a `Window`.

5. **Keyboard wiring.** `⌘K` toggles overlay. `⌘TAB` cycles focus. `⌘L` locks. `⌘W` closes focused. `Esc` dismisses overlay.

6. **Cave-name plumbing.** The WM's `Window` struct gets an `Option<&'static str> cave_name`. When an app is launched from inside a cave context (caves manager → open shell in cave), the launcher fills this in; the chrome renders `TITLE · cave_name`.

7. **Customization.** A small config sheet (a modal overlay rendered by the top bar) lists every available badge with a toggle and a drag handle. Saves to SealFS at `/system/desktop/topbar.cfg`.

8. **Verification.** Boot under QEMU, walk all four states, open ≥ 3 windows, drag / resize / close, lock + unlock + confirm the workspace persists, toggle badges in the config sheet.

## Scope boundary

| Wave | Surface | Notes |
|------|---------|-------|
| **3** | First-app redesign (CAVES, since caves are the killer feature) | Sets the per-app visual + interaction pattern. App internals get the Wave 1 palette + new widgets. |
| **4** | Other 6 apps refreshed against the Wave-3 pattern | Mechanical once Wave 3's pattern is set. Each app gets its own real icon (replaces the Wave 2 placeholder). |
| **5** | Shell + console palette refresh + TT rasterizer fix | Carries Wave 1's deferred rasterizer work. Once fixed, the top-bar wordmark + window titles can move to Plex Sans. |
| **Follow-up** | Dock / taskbar / virtual desktops | Only added if users actually demand them. The Wave-2 model gets you to working multi-window without them. |
| **Follow-up** | Per-app icon design | Each app's wave covers its own icon; until then, the placeholder square. |

## Non-goals

- No virtual desktops / workspaces in v1.
- No window snap-to-edge or tiling in v1.
- No dock or taskbar.
- No theme switch — always dark per the Wave 1 palette.
- No multi-monitor.
- No accessibility / screen-reader support (keyboard-only design).
- No internationalization. ASCII labels only.

## Open questions

- **Cave-aware launcher.** Right now the launcher is a flat 8-app grid. A near-future question is whether opening, e.g., a shell from inside the caves manager (so the shell launches *in* that cave's context) should go through a different launcher affordance than a fresh shell from the desktop. Decision deferred to Wave 3 (caves redesign) — the WM-level `cave_name` plumbing supports either choice.

- **Agent app.** The AGENT slot's design is at `DESIGN_AI_AGENT.md` and not landed. Wave 2 only stubs the icon + opens an empty placeholder window; the agent itself ships in its own wave (probably Wave 6+, since it depends on the rasterizer fix to render long-form text well).

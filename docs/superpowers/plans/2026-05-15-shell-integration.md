# Wave 6 — SHELL Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the existing dead-code `shell::run()` loop body into the WM event pipeline as `pub fn handle_key` + a real `pub fn paint`, replacing the Wave-2 stub at `src/ui/shell.rs:11185`. No technical changes to commands, parsing, history, or autocomplete.

**Architecture:** All work concentrated in `src/ui/shell.rs` and two helper edits — a small `cell_pixel_pos` helper added to `src/ui/console.rs`, and the `AppId::Shell` slot in `src/ui/apps_registry.rs` rewired to the new handlers. The per-byte dispatch logic at `shell.rs` lines 96–214 of `run()` becomes the body of `handle_key`; the stack-locals `cmd_buf`, `cmd_len`, `esc` move to module-level statics. `paint()` delegates to the existing `console::redraw_in_rect(rect)` and overlays a 1-cell INK cursor block.

**Tech Stack:** Rust 2024 edition, `aarch64-unknown-none` target (no-std, alloc available via linked-list allocator). Build: `cargo build --release --target aarch64-unknown-none --features gicv3`. Verification: `cargo clippy -- -D warnings` clean + QEMU walk-through.

**Verification reality check.** Same as Waves 1–5: this crate is `#![no_std] #![no_main]`, no `lib.rs`, no test harness. `cargo test` doesn't run kernel code. Every task's verification is "build is clean (no clippy warnings under `-D warnings`)" plus a QEMU walk-through at the end. There is no unit-test step.

---

## Pre-flight

- [ ] **Step 0a: Confirm clean working tree and create the feature branch.**

```bash
cd /Users/kadenlee/Sphragis
git status --short
git checkout -b feat/shell-integration
```
Expected: clean tree, on `feat/shell-integration` after.

- [ ] **Step 0b: Confirm baseline build is clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: both `Finished release profile`, zero warnings.

- [ ] **Step 0c: Verify kernel API surface (investigation only).**

The spec flagged 5 API gaps. Pre-flight confirms each:

```bash
# 1. EscState constructibility for static init
grep -nE 'pub enum EscState|pub struct EscState|#\[default\]' src/ui/shell_history.rs | head -5

# 2. console private constants (cursor cell-pixel mapping)
grep -nE 'MARGIN_X|MARGIN_Y|^const|font::CHAR_W' src/ui/console.rs | head -8

# 3. console::cursor() signature
grep -nE 'pub fn cursor\b' src/ui/console.rs

# 4. shell_history public API
grep -nE 'pub fn (prev|next|record|reset_cursor)' src/ui/shell_history.rs

# 5. shell_completion public API
grep -nE 'pub fn (complete_command|complete_argument|split_for_completion_parts|arg_kind_for_parts)' src/ui/shell_completion.rs
```

Expected resolutions:
- **EscState**: enum with `#[default]` on `Idle` variant. Static init via `EscState::Idle` works directly (no `const fn` needed — unit variants are const-constructable).
- **console private constants**: `MARGIN_X = 16`, `MARGIN_Y = 32` are private; `STATUS_BAR_H = 28` private. `font::CHAR_W = 8`, `font::CHAR_H = 16` are `pub`. **Task 1 adds `pub fn cell_pixel_pos(col: u32, row: u32) -> (u32, u32)` to `console.rs`** to encapsulate the calculation without exposing the private margins.
- **console::cursor**: `pub fn cursor() -> (u32, u32)` — already pub per Wave-5 pre-flight reading.
- **shell_history**: `pub fn prev() -> Option<&[u8]>`, `pub fn next() -> Option<&[u8]>`, `pub fn record(bytes: &[u8])`, `pub fn reset_cursor()` — all pub (used by `run()` already).
- **shell_completion**: `pub fn complete_command`, `pub fn complete_argument`, `pub fn split_for_completion_parts`, `pub fn arg_kind_for_parts`, `pub enum ArgKind` — all pub.

If any are private, make them `pub` in Task 1 as a one-line change.

---

## File structure

| File | Status | Responsibility |
|------|--------|----------------|
| `src/ui/console.rs` | **MODIFY** | Add `pub fn cell_pixel_pos(col, row) -> (u32, u32)` helper for cursor overlay |
| `src/ui/shell.rs` | **MODIFY** | Add `pub fn handle_key`, real `pub fn paint`, `pub fn handle_click`, module statics, first-paint banner |
| `src/ui/apps_registry.rs` | **MODIFY** | Wire `AppId::Shell` to `shell::handle_key` / `shell::handle_click` |

---

## Task 1: Add cell_pixel_pos helper to console.rs

Add a small public helper that maps a console cell (`col`, `row`) to pixel coordinates within the framebuffer. The cursor overlay in Task 2 needs this to draw a 1-cell INK block at the prompt position.

**Files:**
- Modify: `src/ui/console.rs`

- [ ] **Step 1: Find a good insertion point near the other public helpers.**

```bash
grep -nE 'pub fn cursor\b|pub fn set_cursor' src/ui/console.rs | head -2
```

The new function goes right after `pub fn cursor()` (around line 262).

- [ ] **Step 2: Add the helper.**

Use the Edit tool to insert just below the closing brace of `pub fn cursor()`:

```rust

/// Map a cell coordinate (col, row) to a pixel position in framebuffer
/// coordinates. Used by external overlays (e.g. SHELL's cursor block)
/// that need to draw on top of the console without leaking the
/// private margin / status-bar constants.
pub fn cell_pixel_pos(col: u32, row: u32) -> (u32, u32) {
    let x = MARGIN_X + col * font::CHAR_W;
    let y = MARGIN_Y + row * font::CHAR_H;
    (x, y)
}

/// Cell width / height in pixels. Re-exports the font module's
/// constants under the console module's name for external callers
/// that don't want to import `font` directly.
pub fn cell_size() -> (u32, u32) {
    (font::CHAR_W, font::CHAR_H)
}
```

- [ ] **Step 3: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

Expected: clean. If clippy flags `cell_size` as dead-code, add `#[allow(dead_code)]` (Task 2 makes it live). The file already has `#![allow(dead_code)]` or per-item allows — match the pattern.

- [ ] **Step 4: Commit.**

```bash
git add src/ui/console.rs
git commit -m "$(cat <<'EOF'
console: add cell_pixel_pos + cell_size helpers for external overlays

Wave 6 SHELL needs to draw a cursor block at the prompt position
without importing console's private MARGIN_X / MARGIN_Y constants.
The new helpers encapsulate the cell-to-pixel mapping so the
console module stays the single source of truth for its layout.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 2: SHELL handle_key + paint + handle_click

Replace the Wave-2 `paint()` stub at `src/ui/shell.rs:11185` with real WM-facing handlers. Add module-level statics for the input state previously owned by `run()`'s stack frame.

**Files:**
- Modify: `src/ui/shell.rs` — append new public functions; the existing 11k lines stay untouched.

- [ ] **Step 1: Read the existing `run()` function body to confirm the input-byte dispatch shape.**

```bash
sed -n '19,215p' src/ui/shell.rs
```

The new `handle_key` mirrors the inner `match c { ... }` block at lines 96–214, with two changes:
- All `platform::serial_putc` / `platform::serial_puts` calls are dropped (no UART mirror in WM path)
- `cmd_buf`, `cmd_len`, `esc`, `redraw` closure all move from stack-locals to module statics

- [ ] **Step 2: Read existing imports + the Wave-2 `paint()` stub.**

```bash
sed -n '1,18p' src/ui/shell.rs
sed -n '11180,11197p' src/ui/shell.rs
```

The new code adds these imports if missing:
- `use crate::ui::apps_registry::AppEvent;`
- `use crate::ui::wm::WindowRect;`
- `use core::sync::atomic::{AtomicBool, Ordering};`
- `use super::shell_history::{ArrowKey, EscState, FeedResult};`

Note: `super::shell_history::*` is already imported INSIDE `run()` at line 37. The new top-of-file `use` makes it available at module scope so the new code can reference it.

- [ ] **Step 3: Replace the Wave-2 `paint()` stub and append new handlers.**

Use the Edit tool to replace the existing Wave-2 stub block (lines 11182–11196 — find the exact bounds first with `grep -nE 'pub fn paint|integrating in Wave 5' src/ui/shell.rs`):

Old text to replace (verbatim — verify exact whitespace before editing):

```rust
/// Wave 2 stub — placeholder paint for the SHELL slot. The 11k-line
/// console is integrated properly in Wave 5.
// Wave 2 shim — refresh in Wave 3+
pub fn paint(rect: crate::ui::wm::WindowRect) {
    use crate::ui::font;
    let msg = "SHELL - integrating in Wave 5";
    let tx = rect.x + 12;
    let ty = rect.y + 12;
    font::draw_str(
        crate::ui::gpu::framebuffer(),
        crate::ui::gpu::width(),
        tx, ty, msg,
        0xFFE5E7EB, 0xFF0D0D10,
    );
}
```

New text:

```rust
// ─── Wave 6 WM-facing handlers ───────────────────────────────────

use crate::ui::apps_registry::AppEvent;
use crate::ui::wm::WindowRect;
use core::sync::atomic::{AtomicBool, Ordering};

static mut SHELL_CMD_BUF: [u8; MAX_CMD_LEN] = [0u8; MAX_CMD_LEN];
static mut SHELL_CMD_LEN: usize = 0;
static mut SHELL_ESC: super::shell_history::EscState =
    super::shell_history::EscState::Idle;
static SHELL_INITED: AtomicBool = AtomicBool::new(false);

const BANNER_LINE_1: &str = "      ___       _      ___  ___\n";
const BANNER_LINE_2: &str = "     | _ ) __ _| |_   / _ \\/ __|\n";
const BANNER_LINE_3: &str = "     | _ \\/ _` |  _| | (_) \\__ \\\n";
const BANNER_LINE_4: &str = "     |___/\\__,_|\\__|  \\___/|___/\n";

/// WM-app paint entry. First call paints the welcome banner + prompt;
/// subsequent calls just repaint the scrollback into the rect.
pub fn paint(rect: WindowRect) {
    use crate::ui::console;

    if !SHELL_INITED.load(Ordering::Relaxed) {
        console::init_in_window();
        console::puts_hi(BANNER_LINE_1);
        console::puts_hi(BANNER_LINE_2);
        console::puts_hi(BANNER_LINE_3);
        console::puts_hi(BANNER_LINE_4);
        console::puts("\n");
        console::puts("  Microkernel Shell v0.3 — Type 'help' for commands\n");
        console::puts("  Zero dependencies. Zero trust.\n");
        console::puts("\n");
        console::prompt();
        SHELL_INITED.store(true, Ordering::Relaxed);
    }
    console::redraw_in_rect(rect);
    paint_cursor_block(rect);
}

/// Draw a 1-cell INK block at the console cursor position.
/// Repaints the char under it (if any) in BG-on-INK so it stays
/// readable.
fn paint_cursor_block(rect: WindowRect) {
    use crate::ui::{console, gpu};

    let (col, row) = console::cursor();
    let (cw, ch) = console::cell_size();
    let (cell_x, cell_y) = console::cell_pixel_pos(col, row);

    // The cell-pixel-pos is in screen coordinates; the rect is also
    // in screen coordinates. Skip the overlay if the cursor would
    // land outside the window.
    if cell_x < rect.x || cell_y < rect.y
        || cell_x + cw > rect.x + rect.w
        || cell_y + ch > rect.y + rect.h {
        return;
    }
    gpu::fill_rect(cell_x, cell_y, cw, ch, 0xFFE5E7EB); // INK
    // (The char under the cursor is left invisible — same as a
    //  standard block cursor in xterm. Rewriting it in BG-on-INK
    //  would require reading the SB_BUF cell which the console
    //  module doesn't expose; the simpler approach is fine.)
}

/// WM-app key entry. Drives the same per-byte dispatch as the
/// dead-code `run()` loop body, with the input state held in
/// module statics instead of stack locals.
pub fn handle_key(c: u8) -> AppEvent {
    use crate::ui::console;
    use super::shell_history::{ArrowKey, FeedResult};

    // Ensure first-paint state is consistent — if a key arrives
    // before paint() ever ran (shouldn't happen but defensive),
    // bail. The first paint will arrive on the next tick.
    if !SHELL_INITED.load(Ordering::Relaxed) {
        return AppEvent::Repaint;
    }

    // Run through the ANSI ESC parser. WM keys arrive pre-parsed
    // as 0x90–0x93 for arrows, so this is mostly a passthrough,
    // but kept for any future UART-style ESC sequences.
    let raw = c;
    let parsed = unsafe { (*core::ptr::addr_of_mut!(SHELL_ESC)).feed(raw) };
    let c = match parsed {
        FeedResult::Consumed => return AppEvent::Repaint,
        FeedResult::Arrow(ArrowKey::Up) => {
            handle_history(super::shell_history::prev());
            return AppEvent::Repaint;
        }
        FeedResult::Arrow(ArrowKey::Down) => {
            match super::shell_history::next() {
                Some(line) => handle_history(Some(line)),
                None       => handle_history(None),
            }
            return AppEvent::Repaint;
        }
        FeedResult::Arrow(_) => return AppEvent::Repaint, // L/R ignored
        FeedResult::Pass(b)  => b,
    };

    match c {
        // Kernel-keyboard arrow codes (Wave 2+). The ESC parser
        // above handles UART-style ESC sequences; these handle
        // the direct kernel codes.
        0x90 => { // Up
            handle_history(super::shell_history::prev());
            AppEvent::Repaint
        }
        0x91 => { // Down
            match super::shell_history::next() {
                Some(line) => handle_history(Some(line)),
                None       => handle_history(None),
            }
            AppEvent::Repaint
        }
        0x92 | 0x93 => AppEvent::Repaint, // L/R ignored

        b'\r' | b'\n' => {
            console::putc(b'\n');
            let cmd_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SHELL_CMD_LEN)) };
            if cmd_len > 0 {
                let cmd = unsafe {
                    core::str::from_utf8_unchecked(
                        &(*core::ptr::addr_of!(SHELL_CMD_BUF))[..cmd_len]
                    )
                };
                execute(cmd);
                let bytes = unsafe { &(*core::ptr::addr_of!(SHELL_CMD_BUF))[..cmd_len] };
                super::shell_history::record(bytes);
                unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(SHELL_CMD_LEN), 0); }
            }
            console::prompt();
            AppEvent::Repaint
        }

        0x08 | 0x7F => {
            // Backspace
            let cmd_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SHELL_CMD_LEN)) };
            if cmd_len > 0 {
                unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(SHELL_CMD_LEN), cmd_len - 1); }
                console::putc(0x08);
                super::shell_history::reset_cursor();
            }
            AppEvent::Repaint
        }

        0x03 => {
            // Ctrl+C
            console::puts("^C\n");
            unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(SHELL_CMD_LEN), 0); }
            super::shell_history::reset_cursor();
            console::prompt();
            AppEvent::Repaint
        }

        0x1B => {
            // Esc — same as Ctrl+C.
            console::puts("^C\n");
            unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(SHELL_CMD_LEN), 0); }
            super::shell_history::reset_cursor();
            console::prompt();
            AppEvent::Repaint
        }

        0x09 => {
            // Tab — autocomplete.
            handle_tab();
            AppEvent::Repaint
        }

        0x20..=0x7E => {
            // Printable ASCII.
            let cmd_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SHELL_CMD_LEN)) };
            if cmd_len < MAX_CMD_LEN - 1 {
                unsafe {
                    (*core::ptr::addr_of_mut!(SHELL_CMD_BUF))[cmd_len] = c;
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(SHELL_CMD_LEN), cmd_len + 1);
                }
                console::putc(c);
                super::shell_history::reset_cursor();
            }
            AppEvent::Repaint
        }

        _ => AppEvent::Unhandled,
    }
}

/// Wave 6 click handler — no click-driven behavior. Returns Consumed
/// so the desktop doesn't reinterpret the click as a focus-other
/// gesture (though the window IS already focused at click time).
pub fn handle_click(_mx: i32, _my: i32, _body: WindowRect) -> AppEvent {
    AppEvent::Consumed
}

/// Replace the current input line on screen + cmd_buf with `new_bytes`.
/// `None` means "clear to empty." Used by Up/Down history nav.
fn handle_history(line: Option<&[u8]>) {
    use crate::ui::console;
    let old_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SHELL_CMD_LEN)) };
    // Erase old chars on screen.
    for _ in 0..old_len {
        console::putc(0x08);
    }
    match line {
        Some(bytes) => {
            let n = bytes.len().min(MAX_CMD_LEN);
            unsafe {
                let dst = core::ptr::addr_of_mut!(SHELL_CMD_BUF) as *mut u8;
                for i in 0..n {
                    core::ptr::write(dst.add(i), bytes[i]);
                }
                core::ptr::write_volatile(core::ptr::addr_of_mut!(SHELL_CMD_LEN), n);
            }
            for &b in &bytes[..n] {
                console::putc(b);
            }
        }
        None => {
            unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(SHELL_CMD_LEN), 0); }
        }
    }
}

/// Tab autocomplete — adapted from `run()` lines 132–204 with the
/// UART mirror dropped.
fn handle_tab() {
    use crate::ui::console;
    let cmd_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SHELL_CMD_LEN)) };
    let line = unsafe {
        core::str::from_utf8_unchecked(
            &(*core::ptr::addr_of!(SHELL_CMD_BUF))[..cmd_len]
        )
    };
    let split = super::shell_completion::split_for_completion_parts(line);
    if let Some(info) = split {
        let kind = super::shell_completion::arg_kind_for_parts(
            &info.parts[..info.parts_len], info.arg_index,
        );
        let current = info.current;
        if kind != super::shell_completion::ArgKind::None {
            let r = super::shell_completion::complete_argument(kind, current);
            let ext = r.extension_bytes();
            let take = ext.len().min(MAX_CMD_LEN.saturating_sub(cmd_len + 1));
            let mut new_len = cmd_len;
            for &b in &ext[..take] {
                unsafe { (*core::ptr::addr_of_mut!(SHELL_CMD_BUF))[new_len] = b; }
                new_len += 1;
                console::putc(b);
            }
            unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(SHELL_CMD_LEN), new_len); }
            if r.match_count > 1 {
                console::putc(b'\n');
                for i in 0..r.names_len as usize {
                    let name = r.name_at(i);
                    for &b in name {
                        console::putc(b);
                    }
                    console::puts("  ");
                }
                console::putc(b'\n');
                console::prompt();
                let cur_buf = unsafe { &(*core::ptr::addr_of!(SHELL_CMD_BUF))[..new_len] };
                for &b in cur_buf {
                    console::putc(b);
                }
            }
        }
    } else {
        let r = super::shell_completion::complete_command(line);
        let ext = r.extension_bytes();
        let take = ext.len().min(MAX_CMD_LEN.saturating_sub(cmd_len + 1));
        let mut new_len = cmd_len;
        for &b in &ext[..take] {
            unsafe { (*core::ptr::addr_of_mut!(SHELL_CMD_BUF))[new_len] = b; }
            new_len += 1;
            console::putc(b);
        }
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(SHELL_CMD_LEN), new_len); }
        if r.match_count > 1 {
            console::putc(b'\n');
            for &name in r.candidate_slice() {
                console::puts(name);
                console::puts("  ");
            }
            console::putc(b'\n');
            console::prompt();
            let cur_buf = unsafe { &(*core::ptr::addr_of!(SHELL_CMD_BUF))[..new_len] };
            for &b in cur_buf {
                console::putc(b);
            }
        }
    }
}
```

- [ ] **Step 4: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -8
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -8
```

Likely issues + fixes:
- **`super::shell_history::EscState::Idle` not const-constructable**: if the compiler rejects the static init, change `static mut SHELL_ESC: ... = EscState::Idle;` to `static mut SHELL_ESC: Option<super::shell_history::EscState> = None;` and lazily construct in `handle_key`.
- **Rust 2024 raw-pointer autoref lint** (caught in Wave 5): `&(*core::ptr::addr_of!(X))[..n]` patterns may need binding through a local — adapt per the Wave 5 fix pattern.
- **`MAX_CMD_LEN` not in scope**: it's defined at line 11 of shell.rs (`const MAX_CMD_LEN: usize = 256`). Same scope as the new code.
- **`execute` private vs needed-here**: the new code calls `execute(cmd)` which is the existing private function at line 229. Same module, so no visibility change needed.
- **`console::cell_size` / `cell_pixel_pos` dead-code on first build**: covered by Task 1's `#[allow(dead_code)]` notes; Task 2's `paint` references them so the warning resolves.
- **`shell_completion::ArgKind` enum**: may need `pub use` to make it visible — confirm with `grep -nE 'pub enum ArgKind' src/ui/shell_completion.rs`.

- [ ] **Step 5: Commit.**

```bash
git add src/ui/shell.rs
git commit -m "$(cat <<'EOF'
shell: Wave 6 — WM handle_key + paint + first-paint banner

Replaces the Wave-2 stub paint() that just rendered "SHELL —
integrating in Wave 5". New behavior:
* paint() — first call paints the welcome banner + initial prompt
  via the existing console module; subsequent calls call
  console::redraw_in_rect + paint_cursor_block (1-cell INK overlay
  at console::cursor()).
* handle_key(c) — derived from the per-byte dispatch in the dead-
  code run() loop body (lines 96-214). All input state held in
  module-level statics (SHELL_CMD_BUF, SHELL_CMD_LEN, SHELL_ESC).
  No UART mirror in this path — the headless main::serial_shell
  loop still drives UART independently. Esc maps to Ctrl+C
  (cancel input).
* handle_click — Consumed (no click-driven behavior in Wave 6).

The 75 existing command implementations + parser + history +
autocomplete are untouched. Wave 6 is pure WM plumbing.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 3: Wire AppId::Shell to the new handlers

The `apps_registry::APPS` array still points the SHELL slot at `default_handle_key` / `default_handle_click`. Wave 6 wires it to the new functions.

**Files:**
- Modify: `src/ui/apps_registry.rs`

- [ ] **Step 1: Find the current SHELL entry.**

```bash
grep -nE 'AppId::Shell' src/ui/apps_registry.rs
```

Expected: one match around line 64 with `handle_key: default_handle_key, handle_click: default_handle_click`.

- [ ] **Step 2: Replace the SHELL entry to point at the new handlers.**

Use the Edit tool. Find:

```rust
    AppDescriptor { id: AppId::Shell,    label: "SHELL",    title: "SHELL",    paint: paint_shell,    handle_key: default_handle_key, handle_click: default_handle_click },
```

Replace with:

```rust
    AppDescriptor { id: AppId::Shell,    label: "SHELL",    title: "SHELL",    paint: paint_shell,    handle_key: crate::ui::shell::handle_key, handle_click: crate::ui::shell::handle_click },
```

This is a single-line change matching the pattern used for Caves/Files/Net/Security/Editor in Waves 3/4/5.

- [ ] **Step 3: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

Expected: clean.

- [ ] **Step 4: Commit.**

```bash
git add src/ui/apps_registry.rs
git commit -m "$(cat <<'EOF'
apps_registry: wire AppId::Shell to shell::handle_key / handle_click

Wave 2 stubbed the SHELL slot's input handlers as default no-ops.
Wave 6 lights them up — the new paint() at src/ui/shell.rs now has
real handle_key/handle_click siblings, so the SHELL window receives
keystrokes and renders properly.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 4: QEMU walk-through

Manual visual confirmation. **No commit.**

- [ ] **Step 1: Rebuild + relaunch QEMU.**

```bash
cd /Users/kadenlee/Sphragis
pkill -9 -f 'qemu-system-aarch64' 2>/dev/null
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
qemu-system-aarch64 \
  -machine virt -cpu max -m 2G \
  -display cocoa \
  -device virtio-gpu-device \
  -device virtio-keyboard-device \
  -device virtio-mouse-device \
  -netdev user,id=net0 \
  -device virtio-net-device,netdev=net0 \
  -serial none \
  -kernel target/aarch64-unknown-none/release/sphragis &
```

Unlock with `sphragis-dev`.

- [ ] **Step 2: Verify SHELL opens.**

Press `5`. Confirm:
- SHELL window opens with the welcome banner (`Bat OS` ASCII art) and "Microkernel Shell v0.3 — Type 'help' for commands"
- `sphragis >` prompt visible
- Solid INK cursor block immediately after the prompt

- [ ] **Step 3: Verify basic input.**

Type `help` and press Enter. Confirm:
- Each key echoes to the prompt as you type
- Cursor advances after each char
- Enter executes — command output scrolls in
- New prompt appears below the output with cursor blinking at the new position

- [ ] **Step 4: Verify Backspace + Ctrl+C + Esc.**

- Type `garbage`, press Backspace 7 times — confirm chars erase one by one
- Type `more garbage`, press Ctrl+C — confirm `^C` prints and a fresh prompt appears
- Type `even more garbage`, press Esc — confirm same `^C` cancel behavior

- [ ] **Step 5: Verify Tab autocomplete.**

- Type `hel` + Tab — should complete to `help` (single match)
- Type `c` + Tab — should show multiple candidates (caves, cat, clear, etc.) on a new line + reprompt with `c` still in the input

- [ ] **Step 6: Verify history navigation.**

- Execute 3 different commands (`help`, `ls`, `caves list`)
- Press ↑ — should recall the most recent (`caves list`)
- Press ↑ again — should recall `ls`
- Press ↑ again — should recall `help`
- Press ↓ — should walk forward through history
- Press ↓ from the most recent — should clear the input line

- [ ] **Step 7: Verify the Wave-5 demo loop (the whole point of Wave 6).**

- In SHELL, type `write notes.txt "## My notes\nhello world\n// a comment\n"` + Enter
- Press `2` to switch to FILES — confirm `notes.txt` appears in the sidebar with a state dot
- Use J/K to highlight `notes.txt`, press Enter — confirm EDITOR opens with the content loaded, including the `// a comment` line rendered in MID
- Type some characters → dirty marker turns INK
- Press `S` → dirty clears
- Press Esc → back to FILES
- Press `5` back to SHELL — confirm the SHELL prompt is preserved (banner not repainted)

- [ ] **Step 8: Verify cross-app keyboard parity.**

- `1` opens CAVES, `2` FILES, `3` NET, `4` SECURITY, `6` EDITOR — all still launch from inside SHELL?  
  Wait — they won't. SHELL's `handle_key` consumes digits as text input. Same constraint as EDITOR-with-file-loaded. To switch from SHELL the operator uses Tab (cycle), Ctrl+D (close), or clicks another window.
- Press Tab — confirms focus cycles to the next window
- Press Ctrl+D — confirms SHELL window closes

This is the documented Wave-6 behavior. If it's a UX problem, Wave 7 can add an explicit "leave SHELL" affordance.

- [ ] **Step 9: Kill QEMU.**

```bash
pkill -9 -f 'qemu-system-aarch64'
```

- [ ] **Step 10: No commit.**

If any step surfaced a defect, return to the relevant earlier task.

---

## Task 5: Push + finishing-a-development-branch

- [ ] **Step 1: Push to origin.**

```bash
cd /Users/kadenlee/Sphragis
git push -u origin feat/shell-integration
```

- [ ] **Step 2: Invoke `superpowers:finishing-a-development-branch`.**

Recommended choice: "Merge back to main locally" — full pattern: checkout main → `--no-ff` merge → verify build/clippy → delete local branch → push origin main → delete origin's feature branch → journal entry.

---

## Spec coverage map (self-review)

| Spec section | Task |
|--------------|------|
| §Scope — In: new `pub fn handle_key` | Task 2 |
| §Scope — In: real `pub fn paint` | Task 2 |
| §Scope — In: `pub fn handle_click` | Task 2 |
| §Scope — In: module-level statics | Task 2 |
| §Scope — In: `SHELL_INITED` first-paint guard | Task 2 |
| §Scope — In: wire `AppId::Shell` | Task 3 |
| §Scope — Out: selection mode | Not implemented (correct) |
| §Scope — Out: mouse selection / clipboard | Not implemented (correct) |
| §Scope — Out: status / action strip | Not implemented (correct per spec) |
| §Scope — Out: cave-context indicator | Not implemented (existing prompt covers it) |
| §Scope — Out: kill `run()` | Stays as dead-code (correct) |
| §Scope — Out: command-implementation changes | Untouched (correct) |
| §Scope — Out: console.rs internal changes | Only the additive `cell_pixel_pos` + `cell_size` helpers (Task 1) — additive, not internal |
| §Scope — Out: UART mirroring | Dropped in Task 2's handler |
| §Visual system — INK cursor block | Task 2 (`paint_cursor_block`) |
| §Layout — full-window console + banner | Task 2 (first-paint banner + `redraw_in_rect`) |
| §Input handling table — all bytes mapped | Task 2 (`handle_key` match arms cover all 8 byte classes) |
| §Module statics — cmd_buf, cmd_len, esc, inited | Task 2 |
| §Paint — first-paint init + redraw_in_rect + cursor | Task 2 |
| §Handle click — Consumed | Task 2 |
| §Open API gaps — all 5 | Pre-flight 0c (Task 1 adds `cell_pixel_pos` + `cell_size`) |
| §Failure modes — overflow / empty Enter / concurrent UART | Existing `run()` behaviors preserved (correct) |
| §Reuse from prior waves | Task 2 uses existing `console::*`, `shell_history::*`, `shell_completion::*`, `execute()` |
| §Demo flow — write notes.txt → open in EDITOR | Task 4 Step 7 |
| §Testing — build + clippy + QEMU walk | Task 4 |

No gaps.

---

## Out-of-scope reminders (do not implement in Wave 6)

- Selection mode wiring (keystrokes → `console::enter_select_mode` etc.)
- Mouse selection / right-click paste / clipboard
- Status strip or action strip
- Per-cave prompt chrome
- Blinking cursor
- Killing the dead-code `run()` function
- Left/Right cursor movement within the command line
- Multi-line / heredoc input
- Touching any of the 75 command implementations
- Refactoring console.rs internals (only the additive helpers)

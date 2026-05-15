# Wave 7 — COMMS Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Recolor `src/ui/apps/comms.rs` from the Wave-1 cyberpunk palette to the calm Wave-4 register, fix the Wave-2-temp full-screen call site so `paint()` uses the rect it's passed, and wire `handle_key`/`handle_click` to return `AppEvent`. The wire protocol, crypto, and connection state machine stay untouched.

**Architecture:** Single-file rewrite of `render()` + `handle_key()` + new `handle_click()` in `src/ui/apps/comms.rs`. One line in `src/ui/apps_registry.rs` to wire the `AppId::Comms` slot. No new modules, no new widgets, no new statics. All existing crypto / send_message / recv_message paths stay verbatim.

**Tech Stack:** Rust 2024 edition, `aarch64-unknown-none` target (no-std). Build: `cargo build --release --target aarch64-unknown-none --features gicv3`. Verification: `cargo clippy -- -D warnings` clean + QEMU walk-through.

**Verification reality check.** Same as Waves 1–6: no `lib.rs`, no test harness. Every task's verification is "build clean + clippy clean" plus a QEMU walk-through at the end.

---

## Pre-flight

- [ ] **Step 0a: Confirm clean working tree and create the feature branch.**

```bash
cd /Users/kadenlee/Sphragis
git status --short
git checkout -b feat/comms-redesign
```
Expected: clean tree, on `feat/comms-redesign` after.

- [ ] **Step 0b: Confirm baseline build is clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: both `Finished release profile`, zero warnings.

- [ ] **Step 0c: Verify the small API surface (no code change — investigation only).**

```bash
# 1. palette constants exist + are pub
grep -nE 'pub const (BG|PANEL|HAIRLINE|INK|MID|FAINT)' src/ui/palette.rs

# 2. AppId::Comms slot wiring + paint_comms shim
grep -nE 'AppId::Comms|paint_comms' src/ui/apps_registry.rs

# 3. PEER_IP / PEER_PORT statics exist + are set by connect()
grep -nE 'PEER_IP|PEER_PORT' src/ui/apps/comms.rs | head -6

# 4. CommState enum
grep -nE 'pub enum CommState' src/ui/apps/comms.rs

# 5. existing draw_strip signature (header background)
grep -nE 'pub fn draw_strip' src/ui/widgets.rs
```

Expected resolutions:
- `p::BG`, `p::PANEL`, `p::HAIRLINE`, `p::INK`, `p::MID`, `p::FAINT` all pub at `src/ui/palette.rs:10-18`.
- `AppId::Comms` slot at `src/ui/apps_registry.rs:66` (currently `default_handle_key/default_handle_click`). `paint_comms` shim at line ~82.
- `static mut PEER_IP: u32 = 0` at comms.rs:86, `static mut PEER_PORT: u16 = 0` at comms.rs:87. Both set in `connect()` at lines 293–294; both zeroed in `disconnect()` at lines 592–593. Use them for the dynamic peer label.
- `pub enum CommState { Disconnected, Connecting, Connected, Error }` at comms.rs:43.
- `pub fn draw_strip(x, y, w, h, dark: bool, bottom: bool)` in widgets.rs — render the header background. Works regardless of palette choice (uses its own internal colors).

---

## File structure

| File | Status | Responsibility |
|------|--------|----------------|
| `src/ui/apps/comms.rs` | **MODIFY** | Rewrite `render()` + `handle_key()` + add `handle_click()`. Drop the file-local cyberpunk palette constants. Existing protocol / crypto / state-machine code untouched. |
| `src/ui/apps_registry.rs` | **MODIFY** | Wire `AppId::Comms` to `comms::handle_key` / `comms::handle_click` |

---

## Task 1: Recolor render() + new handle_key + handle_click

The body of `render()` is currently 137 lines (613–749) using the cyberpunk palette and a Wave-2-temp full-screen rect. Rewrite to use the Wave-4 calm palette + the rect passed in. Replace `draw_conn_pill` calls with plain `font::draw_str` in INK. Rewire `handle_key` to return `AppEvent`. Add `handle_click`. Drop the dead `BG/FG/FG_HI/DIM/GREEN/RED/CYAN/BORDER/INPUT_BG` constants.

**Files:**
- Modify: `src/ui/apps/comms.rs`

- [ ] **Step 1: Drop the file-local cyberpunk palette constants.**

Delete lines 31–39 (the 9 `const` declarations starting with `BG` and ending with `INPUT_BG`). These constants are no longer used after Task 1 lands.

Verify with `grep -nE '^const (BG|FG|FG_HI|DIM|GREEN|RED|CYAN|BORDER|INPUT_BG)' src/ui/apps/comms.rs` — should return zero lines after the edit.

- [ ] **Step 2: Replace the existing `render()` function and add a new `paint()` shim that forwards the rect.**

The Wave-2 `paint()` shim at line 861 currently ignores the rect and calls `render()` (which then synthesizes a full-screen rect at line 622). Replace BOTH `paint()` and `render()` with the new versions below.

Find both functions and use the Edit tool to replace them. Use `grep -nE 'pub fn render\(\)|pub fn paint\(' src/ui/apps/comms.rs` to locate.

**Replace `pub fn render() { ... }`** (currently lines 613–749, ending at the `}` that closes the function — verify with grep) **with this new body that takes a `rect` parameter and uses the Wave-4 palette:**

```rust
fn render(rect: crate::ui::wm::WindowRect) {
    use crate::ui::widgets::draw_strip;
    use crate::ui::palette as p;

    let fb = gpu::framebuffer();
    let sw = gpu::width();
    gpu::fill_rect(rect.x, rect.y, rect.w, rect.h, p::BG);

    let header_h: u32 = 32;
    let composer_h: u32 = 28;
    let body_y = rect.y + header_h;
    let composer_y = rect.y + rect.h - composer_h;
    let body_total_h = composer_y.saturating_sub(body_y);
    let shell_h = (body_total_h * 7 / 20).max(96);
    let shell_y = composer_y - shell_h - 1;
    let body_h = shell_y.saturating_sub(body_y);

    // ── HEADER STRIP ──────────────────────────────────────────────
    draw_strip(rect.x, rect.y, rect.w, header_h, false, true);
    let h_text_y = rect.y + (header_h - 16) / 2;
    font::draw_str(fb, sw, rect.x + 16, h_text_y, "COMMS", p::INK, p::BG);

    let st = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(STATE)) };
    let state_x = rect.x + 16 + 6 * 8;  // after "COMMS" + 8px gap
    let state_label = match st {
        CommState::Disconnected => alloc::string::String::from("DISCONNECTED"),
        CommState::Connecting   => format_state("CONNECTING"),
        CommState::Connected    => format_state("CONNECTED · peer"),
        CommState::Error        => alloc::string::String::from("ERROR"),
    };
    font::draw_str(fb, sw, state_x, h_text_y, state_label.as_str(), p::INK, p::BG);

    // Right side: cipher + key prefix as MID text (only when connected).
    if st == CommState::Connected {
        let cipher = "ChaCha20-Poly1305";
        let c2s = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(C2S_KEY)) };
        let hex = b"0123456789abcdef";
        let mut buf = [0u8; 48];
        let mut n = 0;
        for &b in cipher.as_bytes() { if n < buf.len() { buf[n] = b; n += 1; } }
        for &b in b" \xc2\xb7 K " { if n < buf.len() { buf[n] = b; n += 1; } }
        for i in 0..4 {
            if n < buf.len() { buf[n] = hex[(c2s[i] >> 4) as usize]; n += 1; }
            if n < buf.len() { buf[n] = hex[(c2s[i] & 0x0f) as usize]; n += 1; }
        }
        for &b in b"..." { if n < buf.len() { buf[n] = b; n += 1; } }
        let right = unsafe { core::str::from_utf8_unchecked(&buf[..n]) };
        let right_w = (n as u32) * 8;
        if rect.w > right_w + 16 {
            font::draw_str(fb, sw,
                rect.x + rect.w.saturating_sub(right_w + 16),
                h_text_y, right, p::MID, p::BG);
        }
    }

    // ── TIMELINE BODY ─────────────────────────────────────────────
    if st == CommState::Disconnected {
        draw_disconnected_empty(rect.x, body_y, rect.w, body_h, sw, fb);
    } else {
        draw_timeline(rect.x, body_y, rect.w, body_h, sw, fb);
    }

    // ── EMBEDDED SHELL STRIP ──────────────────────────────────────
    gpu::fill_rect(rect.x, shell_y, rect.w, 1, p::HAIRLINE);
    crate::ui::console::redraw_in_rect(crate::ui::wm::WindowRect {
        x: rect.x + 8, y: shell_y + 4,
        w: rect.w.saturating_sub(16), h: shell_h.saturating_sub(8),
    });

    // ── COMPOSER ──────────────────────────────────────────────────
    gpu::fill_rect(rect.x, composer_y, rect.w, composer_h, p::PANEL);
    gpu::fill_rect(rect.x, composer_y, rect.w, 1, p::HAIRLINE);
    let c_text_y = composer_y + (composer_h - 16) / 2;
    let prompt_color = if st == CommState::Disconnected { p::FAINT } else { p::INK };
    font::draw_str(fb, sw, rect.x + 16, c_text_y, ">", prompt_color, p::PANEL);
    let typed_x = rect.x + 16 + 2 * 8;

    let (compose_text, compose_len): (&str, usize) = unsafe {
        let len = core::ptr::read_volatile(core::ptr::addr_of!(COMPOSE_LEN));
        let bytes = &(*core::ptr::addr_of!(COMPOSE_BUF))[..len];
        (core::str::from_utf8_unchecked(bytes), len)
    };
    if st == CommState::Disconnected {
        font::draw_str(fb, sw, typed_x, c_text_y,
            "(composer disabled - not connected)", p::FAINT, p::PANEL);
    } else {
        font::draw_str(fb, sw, typed_x, c_text_y, compose_text, p::INK, p::PANEL);
        // 1-cell INK block cursor at the next-char position. Repaint
        // the char under it (if any) in BG-on-INK so it stays readable.
        let cur_x = typed_x + (compose_len as u32) * 8;
        let cell_top = composer_y + (composer_h - 16) / 2;
        gpu::fill_rect(cur_x, cell_top, 8, 16, p::INK);
        if compose_len < MAX_MSG_LEN {
            let bytes = unsafe { &(*core::ptr::addr_of!(COMPOSE_BUF))[..MAX_MSG_LEN] };
            if compose_len < bytes.len() && (0x20..=0x7E).contains(&bytes[compose_len]) {
                let s = unsafe { core::str::from_utf8_unchecked(
                    core::slice::from_ref(&bytes[compose_len])) };
                font::draw_str(fb, sw, cur_x, cell_top, s, p::BG, p::INK);
            }
        }
    }

    // Char counter on the right, always MID.
    let mut buf = [0u8; 16];
    let n = format_dec_local(compose_len, &mut buf);
    let n_str = unsafe { core::str::from_utf8_unchecked(&buf[..n]) };
    let suffix = " / 80";
    let total_w = (n as u32 + suffix.len() as u32) * 8;
    if rect.w > total_w + 16 {
        let cx = rect.x + rect.w - 16 - total_w;
        font::draw_str(fb, sw, cx, c_text_y, n_str, p::MID, p::PANEL);
        font::draw_str(fb, sw, cx + n as u32 * 8, c_text_y, suffix, p::FAINT, p::PANEL);
    }
}

/// Format `STATE_LABEL · peer <ip>:<port>` using the PEER_IP /
/// PEER_PORT statics. Used by both CONNECTING and CONNECTED to
/// build the dynamic header text.
fn format_state(prefix: &str) -> alloc::string::String {
    use alloc::format;
    let ip = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(PEER_IP)) };
    let port = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(PEER_PORT)) };
    if ip == 0 && port == 0 {
        alloc::string::String::from(prefix)
    } else {
        format!("{} {}.{}.{}.{}:{}",
            prefix,
            (ip >> 24) & 0xff,
            (ip >> 16) & 0xff,
            (ip >> 8) & 0xff,
            ip & 0xff,
            port,
        )
    }
}
```

**Replace `pub fn paint(rect: crate::ui::wm::WindowRect) { let _ = rect; render(); }`** at line 861–863 **with:**

```rust
pub fn paint(rect: crate::ui::wm::WindowRect) {
    render(rect);
}
```

(The `render` function is now private and rect-aware; only `paint()` is `pub`.)

- [ ] **Step 3: Replace `draw_disconnected_empty` to use the Wave-4 palette.**

The existing function at line 759 has 7 parameters; the new one needs fewer:

```rust
fn draw_disconnected_empty(
    x: u32, y: u32, w: u32, h: u32,
    sw: u32, fb: *mut u32,
) {
    use crate::ui::palette as p;
    let text = "(no peer connected - use ";
    let cmd  = "comms connect <ip>:<port>";
    let tail = " in shell)";
    let total = (text.len() + cmd.len() + tail.len()) as u32 * 8;
    let cx = x + (w.saturating_sub(total)) / 2;
    let cy = y + h / 2 - 8;
    font::draw_str(fb, sw, cx, cy, text, p::MID, p::BG);
    font::draw_str(fb, sw, cx + text.len() as u32 * 8, cy, cmd, p::INK, p::BG);
    font::draw_str(fb, sw, cx + (text.len() + cmd.len()) as u32 * 8, cy, tail, p::MID, p::BG);
}
```

- [ ] **Step 4: Replace `draw_timeline` to use the Wave-4 palette (drop green/cyan distinction).**

The existing function at line 775 has 10 parameters; the new one needs fewer:

```rust
fn draw_timeline(
    x: u32, y: u32, _w: u32, h: u32,
    sw: u32, fb: *mut u32,
) {
    use crate::ui::palette as p;
    unsafe {
        let row_h: u32 = 18;
        let pad_l: u32 = 16;
        let max_rows = (h.saturating_sub(24) / row_h) as usize;
        let total = MSG_COUNT;
        let start = if total > max_rows { total - max_rows } else { 0 };
        let count = total - start;
        let baseline_y = y + h - 12 - (count as u32) * row_h;
        let mut row_y = baseline_y;
        for i in start..total {
            let idx = i % MAX_MESSAGES;
            let messages = &*core::ptr::addr_of!(MESSAGES);
            let msg = &messages[idx];
            if !msg.active { continue; }
            let mins = msg.timestamp / 60;
            let secs = msg.timestamp % 60;
            let mut ts_buf = [0u8; 7];
            ts_buf[0] = b'[';
            ts_buf[1] = b'0' + ((mins / 10) % 10) as u8;
            ts_buf[2] = b'0' + (mins % 10) as u8;
            ts_buf[3] = b':';
            ts_buf[4] = b'0' + ((secs / 10) % 10) as u8;
            ts_buf[5] = b'0' + (secs % 10) as u8;
            ts_buf[6] = b']';
            let ts_str = core::str::from_utf8_unchecked(&ts_buf);
            font::draw_str(fb, sw, x + pad_l, row_y, ts_str, p::MID, p::BG);
            let (arrow, sender) = if msg.outgoing {
                (">>", "you ")
            } else {
                ("<<", "peer")
            };
            font::draw_str(fb, sw, x + pad_l + 8 * 8, row_y, arrow, p::INK, p::BG);
            font::draw_str(fb, sw, x + pad_l + (8 + 4) * 8, row_y, sender, p::INK, p::BG);
            let text_x = x + pad_l + (8 + 4 + 7) * 8;
            let text = core::str::from_utf8_unchecked(&msg.text[..msg.text_len]);
            font::draw_str(fb, sw, text_x, row_y, text, p::INK, p::BG);
            row_y += row_h;
        }
    }
}
```

- [ ] **Step 5: Rewrite `handle_key` to return `AppEvent` and add `handle_click`.**

Replace the existing `pub fn handle_key(ch: u8) { ... }` at line 836 with:

```rust
pub fn handle_key(c: u8) -> crate::ui::apps_registry::AppEvent {
    use crate::ui::apps_registry::AppEvent;
    unsafe {
        match c {
            b'\r' | b'\n' => {
                if COMPOSE_LEN > 0 {
                    let bytes = &(*core::ptr::addr_of!(COMPOSE_BUF))[..COMPOSE_LEN];
                    let _ = send_message(bytes);
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(COMPOSE_LEN), 0);
                }
                AppEvent::Repaint
            }
            0x08 | 0x7F => {
                if COMPOSE_LEN > 0 {
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(COMPOSE_LEN), COMPOSE_LEN - 1);
                }
                AppEvent::Repaint
            }
            0x20..=0x7E => {
                if COMPOSE_LEN < MAX_MSG_LEN - 1 {
                    let buf_ptr = core::ptr::addr_of_mut!(COMPOSE_BUF) as *mut u8;
                    core::ptr::write(buf_ptr.add(COMPOSE_LEN), c);
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(COMPOSE_LEN), COMPOSE_LEN + 1);
                }
                AppEvent::Repaint
            }
            _ => AppEvent::Unhandled,
        }
    }
}

pub fn handle_click(_mx: i32, _my: i32, _body: crate::ui::wm::WindowRect)
    -> crate::ui::apps_registry::AppEvent
{
    crate::ui::apps_registry::AppEvent::Consumed
}
```

- [ ] **Step 6: Add `extern crate alloc;` if not already present.**

Check the top of `comms.rs`:

```bash
grep -nE '^extern crate alloc' src/ui/apps/comms.rs
```

If absent, add `extern crate alloc;` near the top (just below `#![allow(dead_code)]` at line 1).

- [ ] **Step 7: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -8
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -8
```

Likely issues:
- **Rust 2024 autoref lint**: `&(*core::ptr::addr_of!(X))[..n]` patterns may need a local binding (Waves 5 + 6 hit this). Fix per:
  ```rust
  let buf = unsafe { &*core::ptr::addr_of!(COMPOSE_BUF) };
  let slice = &buf[..n];
  ```
- **Unused imports from the old `widgets::{...}` block**: the original `render()` had `use crate::ui::widgets::{self as W, draw_strip, draw_conn_pill, State, BG as W_BG, INK as W_INK, ...}`. After the rewrite only `draw_strip` is used. Remove the rest from the import.
- **Unused `format_dec_local`**: should still be used by the char counter — verify it's still called.
- **Unused `compute_pill_w`** (line 751): used to be called by the right-side pills. Now dead. Delete the function.
- **`#[allow(dead_code)]`** at the top of the file already exists; should cover any residual.

- [ ] **Step 8: Commit.**

```bash
git add src/ui/apps/comms.rs
git commit -m "$(cat <<'EOF'
comms: Wave 7 — recolor to Wave-4 calm register

Replaces the Wave-1 cyberpunk palette with the calm
INK/MID/FAINT register used by every other Wave-4+ app:

* Drop the file-local BG/FG/FG_HI/DIM/GREEN/RED/CYAN/BORDER/
  INPUT_BG constants. Use crate::ui::palette as p throughout.
* Connection state: plain font::draw_str text in INK instead of
  the colored draw_conn_pill widget. Dynamic peer label built from
  PEER_IP / PEER_PORT (was hardcoded to "10.0.2.42:9100").
* Cipher + key prefix on the right: MID text "ChaCha20-Poly1305 ·
  K <hex8>...", visible only when CONNECTED.
* Timeline: arrows (>> / <<) and sender labels (you / peer) both
  in INK. Direction text carries the distinction; color was
  redundant.
* Composer cursor: 1-cell INK block (matches EDITOR's pattern)
  instead of the 7-wide CYAN underscore. Char under cursor
  repainted in BG-on-INK.
* Char counter: MID always. Drops the AMBER-at-70 / RED-at-80
  flags.
* Disconnected empty-state: MID surround with INK on the
  highlighted command (was DIM + CYAN + DIM).

Also fixes the Wave-2-temp full-screen call site — render() now
takes a WindowRect from paint() instead of synthesizing one from
gpu::width() / gpu::height(). handle_key returns AppEvent; new
handle_click returns Consumed.

Wave 7 keeps the wire protocol, crypto, send_message / recv_message,
connect / disconnect, the CommState machine, and all 32-message
timeline buffer logic untouched.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 2: Wire AppId::Comms to the new handlers

The `apps_registry::APPS` array still points the COMMS slot at `default_handle_key` / `default_handle_click`. Wave 7 wires it to the new functions.

**Files:**
- Modify: `src/ui/apps_registry.rs`

- [ ] **Step 1: Find the current COMMS entry.**

```bash
grep -nE 'AppId::Comms' src/ui/apps_registry.rs
```

Expected: one match around line 66.

- [ ] **Step 2: Replace the COMMS entry to point at the new handlers.**

Use the Edit tool. Find:

```rust
    AppDescriptor { id: AppId::Comms,    label: "COMMS",    title: "COMMS",    paint: paint_comms,    handle_key: default_handle_key, handle_click: default_handle_click },
```

Replace with:

```rust
    AppDescriptor { id: AppId::Comms,    label: "COMMS",    title: "COMMS",    paint: paint_comms,    handle_key: crate::ui::apps::comms::handle_key, handle_click: crate::ui::apps::comms::handle_click },
```

Same single-line pattern Waves 3/4/5/6 used for their respective apps.

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
apps_registry: wire AppId::Comms to comms::handle_key / handle_click

Wave 2 stubbed the COMMS slot's input handlers as default no-ops.
Wave 7 lights them up — comms::handle_key now returns AppEvent and
comms::handle_click consumes clicks without action. The COMMS app
window now receives keystrokes (composer text input + Enter to
send) properly.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 3: QEMU walk-through

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

- [ ] **Step 2: Verify COMMS empty state.**

Press `7` to open COMMS. Confirm:
- Header: "COMMS  DISCONNECTED" — both in INK on a HAIRLINE-bottomed strip.
- Body: centered MID text "(no peer connected — use 'comms connect <ip>:<port>' in shell)" with the command portion in INK.
- Embedded shell strip visible in the lower-middle band — `sphragis >` prompt visible (if SHELL has been opened previously).
- Composer at the bottom: PANEL background, FAINT `>` prompt, FAINT "(composer disabled - not connected)" placeholder, MID "0 / 80" counter.
- **Confirm zero green / red / amber / cyan pixels anywhere in the COMMS window.**

- [ ] **Step 3: Verify composer input gating.**

Type a few keys — confirm nothing happens in the composer (composer is disabled when disconnected). The keys still echo into wherever has focus globally.

Actually correction: per the existing code, the composer DOES accept input even when disconnected (the "disabled" label only paints during render; handle_key still appends bytes to COMPOSE_BUF). The Enter key is the one that gates on COMPOSE_LEN > 0 plus state. **Test reality**: type chars and watch — does the FAINT placeholder text disappear and your typing appear? If yes, that's the current behavior (Wave 7 doesn't change it).

- [ ] **Step 4: Verify a connect flow (best-effort — needs an active server).**

If you have a `comms_test_server.py` running on the host or you can fake one:
- Press Tab (or close + reopen with `5`) to reach SHELL.
- In SHELL: `comms identify <host>` then `comms pin <hex>` then `comms connect <ip>:<port>`.
- Cycle back to COMMS: header should update through CONNECTING → CONNECTED with the real peer ip:port appended.

If you don't have a test server: skip this step. The render path for CONNECTED is the same code path as the others (just a different `match` arm), so the Disconnected verification covers the visual register.

- [ ] **Step 5: Verify cross-app keyboard parity.**

- Open COMMS, type some chars in the composer.
- Press Tab — focus should cycle to the next window.
- Press Ctrl+D — should close the focused window.
- Press `7` from the desktop launcher — should re-open COMMS (or refocus if open). COMMS still has its COMPOSE_BUF state.

Note: digits 1..8 typed inside COMMS get inserted into the composer (matches SHELL / EDITOR-with-file behavior). Switch out via Tab / Ctrl+D / mouse-click another window.

- [ ] **Step 6: Kill QEMU.**

```bash
pkill -9 -f 'qemu-system-aarch64'
```

- [ ] **Step 7: No commit.**

If any step surfaced a defect, return to the relevant earlier task.

---

## Task 4: Push + finishing-a-development-branch

- [ ] **Step 1: Push to origin.**

```bash
cd /Users/kadenlee/Sphragis
git push -u origin feat/comms-redesign
```

- [ ] **Step 2: Invoke `superpowers:finishing-a-development-branch`.**

Recommended choice: "Merge back to main locally" — full pattern: checkout main → `--no-ff` merge → verify build/clippy → delete local branch → push origin main → delete origin's feature branch → journal entry.

---

## Spec coverage map (self-review)

| Spec section | Task |
|--------------|------|
| §Scope — drop file-local cyberpunk palette constants | Task 1 Step 1 |
| §Scope — replace draw_conn_pill with INK text | Task 1 Step 2 |
| §Scope — right-side cipher + key as MID text | Task 1 Step 2 |
| §Scope — timeline arrows + senders both INK | Task 1 Step 4 |
| §Scope — composer INK block cursor | Task 1 Step 2 (composer section) |
| §Scope — char counter always MID | Task 1 Step 2 (counter section) |
| §Scope — disconnected empty-state MID + INK | Task 1 Step 3 |
| §Scope — fix wm::content_rect Wave-2-temp | Task 1 Step 2 (render takes rect) |
| §Scope — handle_key returns AppEvent | Task 1 Step 5 |
| §Scope — new handle_click returns Consumed | Task 1 Step 5 |
| §Scope — wire AppId::Comms | Task 2 |
| §Out — no action-strip hotkeys | Not implemented (correct) |
| §Out — no layout restructure | Not implemented (correct) |
| §Out — no protocol / crypto changes | Untouched (correct) |
| §Out — keep embedded shell strip | Kept (correct) |
| §Visual system — palette constants | Task 1 (uses `palette as p`) |
| §Layout — header/timeline/embedded-shell/composer | Task 1 Step 2 (same math as before) |
| §Input handling table | Task 1 Step 5 (same arms as before) |
| §Open API gaps — palette / AppId / PEER_IP / CommState / draw_strip | Pre-flight 0c (all confirmed exist) |
| §Open API gaps — stored peer ip:port (gap #5) | Task 1 Step 2 uses PEER_IP / PEER_PORT via `format_state()` |
| §Failure modes | Existing behaviors preserved (correct) |
| §Demo flow — verify in QEMU | Task 3 |

No gaps.

---

## Out-of-scope reminders (do not implement in Wave 7)

- Action-strip hotkeys (e.g. `[D]isconnect`, `[C]lear`).
- Modifier-keybinds in COMMS.
- Restructuring the header / body / composer / embedded shell strip layout.
- Multi-peer / peer list / channel switching.
- File attachments / large message support.
- Touching the wire protocol or crypto.
- Modifying `console.rs`, `widgets.rs`, `palette.rs`, or any shared file beyond the import changes in `comms.rs`.
- Mouse selection / clipboard.
- Disconnected-state input gating (current behavior: composer accepts text but Enter is no-op since `send_message` requires connection — unchanged).

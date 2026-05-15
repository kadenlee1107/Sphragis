# Wave 7 — COMMS Redesign — Design

Recolor `src/ui/apps/comms.rs` from the Wave-1 cyberpunk palette
(GREEN / RED / CYAN / AMBER) to the calm Wave-4 register
(INK / MID / FAINT). Fix the Wave-2-temp `wm::content_rect()`
substitute so `paint()` uses the rect it's passed. Wire
`handle_key` / `handle_click` to return `AppEvent` and register
them in `apps_registry`. **No structural changes** — the
header / timeline / embedded shell strip / composer layout stays
exactly as designed in Wave 1.

## Scope

In:

- Drop the file-local cyberpunk palette constants (`BG`, `FG`,
  `FG_HI`, `DIM`, `GREEN`, `RED`, `CYAN`, `BORDER`, `INPUT_BG` at
  `comms.rs:31-39`). Use `crate::ui::palette as p` everywhere.
- Replace `draw_conn_pill` calls with plain text in INK for the
  connection state indicator.
- Replace the right-side cipher + key pills with MID text.
- Timeline messages: arrows (`>>` / `<<`) and sender labels
  (`you` / `peer`) both in INK — direction text carries the
  distinction.
- Composer cursor: 1-cell INK block (matches EDITOR) instead of
  the 7-wide × 2-tall CYAN underscore.
- Char counter: always MID, drop the AMBER-at-70 / RED-at-80
  flags. The count itself is the signal.
- Disconnected-state empty message: switch the highlighted-command
  middle segment from CYAN to INK on a MID surround.
- Fix `comms.rs:622` Wave-2-temp call site — `render()` accepts the
  `WindowRect` from `paint()` instead of synthesizing a full-screen
  rect.
- New `pub fn handle_key(c: u8) -> AppEvent` — same body as the
  existing `handle_key`, but returns `Repaint` for handled bytes
  and `Unhandled` for the rest.
- New `pub fn handle_click(_mx, _my, _body) -> AppEvent` —
  returns `Consumed` (no click-driven actions in Wave 7).
- Wire `AppId::Comms` in `apps_registry.rs` from
  `default_handle_key` / `default_handle_click` to the new functions.

Out:

- New action-strip hotkeys (would collide with text input the way
  EDITOR's S/R did; not adding).
- Restructuring the layout (header / body / composer / embedded
  shell strip stays).
- Modifying the wire protocol, crypto, or connection state machine.
- Dropping the embedded shell strip (user picked the pure-recolor
  scope).
- Selection / clipboard / multi-conversation / file attachments.
- Changing the message log size (32 messages × 80 chars stays).

## Visual system

Inherits Wave-3/4/5/6. No new constants.

- `p::BG = #0D0D10`, `p::PANEL = #18181C`, `p::HAIRLINE = #2A2A30`
- `p::INK = #E5E7EB`, `p::MID = #6B7280`, `p::FAINT = #4A4D55`
- 8×16 bitmap font (`src/ui/font.rs`)
- Pure-mono discipline preserved (no green/red/amber, no orange)

## Layout

Identical to Wave 1. Drawing it out for reference:

```
┌──────────────────────────────────────────────────────────────────┐
│ COMMS  CONNECTED · peer 192.168.1.1:9100   ChaCha20-Poly1305 · K a1b2c3d4... │  header (32 px)
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  [14:32:01] >> you   hello world                                 │  timeline body
│  [14:32:05] << peer  hi there                                    │  (rows 18 px)
│  [14:33:00] >> you   what's your status                          │
│  [14:33:12] << peer  green                                       │
│                                                                  │
├──────────────────────────────────────────────────────────────────┤
│ sphragis > comms connect 192.168.1.1:9100                        │  embedded shell strip
│ Connected to peer.                                               │  (35% of body, min 96 px)
│ sphragis >                                                       │
├──────────────────────────────────────────────────────────────────┤
│ > greetings from sphragis█                                  21 / 80 │  composer (28 px)
└──────────────────────────────────────────────────────────────────┘
```

### Header

- 32 px tall, `draw_strip` background
- Left: "COMMS" wordmark in INK
- After wordmark: connection state text in INK
  - `DISCONNECTED` (no annotation)
  - `CONNECTING · <ip>:<port>` (the ip:port comes from the connect
    call's stored target; if not stored, just `CONNECTING`)
  - `CONNECTED · peer <ip>:<port>`
  - `ERROR` (no annotation; if an error reason is captured the
    existing code path may attach it — leave that logic intact)
- Right (only when `CONNECTED`): cipher + key prefix as MID text
  separated by `·`, e.g. `ChaCha20-Poly1305 · K a1b2c3d4...`

### Timeline body

Same row math as Wave 1 (18 px pitch, anchor newest-to-bottom).
Color changes:

- Timestamp `[HH:MM]` — MID
- Arrow `>>` / `<<` — INK (was CYAN / GREEN)
- Sender `you` / `peer` — INK (was CYAN / GREEN)
- Message text — INK
- Disconnected empty-state — `(no peer connected — use 'comms
  connect <ip>:<port>' in shell)` rendered in MID with the
  command in INK (was DIM + CYAN + DIM)

### Embedded shell strip

No changes — `console::redraw_in_rect` is already wave-4-palette
agnostic (uses the console module's own internal colors, which
are themselves INK / BG / MID equivalents). Keeps the HAIRLINE
above it.

### Composer

- 28 px tall, PANEL background, HAIRLINE above
- `> ` prompt — INK when connected, FAINT when disconnected (was
  CYAN / FAINT)
- Typed text — INK (when connected) or FAINT placeholder (when
  disconnected)
- Cursor — 1-cell INK block (matches EDITOR's `paint_cursor_block`
  pattern) at the next-char position. The char under the cursor
  (if any) is repainted in BG-on-INK so it stays readable. Drops
  the CYAN underscore.
- Char counter `N / 80` on the right — MID always. Drops the
  AMBER-at-70 / RED-at-80 flags.

## Input handling

The existing `handle_key(c: u8)` body stays, but the return type
changes from `()` to `AppEvent`:

| Byte | Effect | Return |
|------|--------|--------|
| `0x0D` Enter (`\r`) / `0x0A` Enter (`\n`) | If `COMPOSE_LEN > 0`, call `send_message(&COMPOSE_BUF[..COMPOSE_LEN])`, reset COMPOSE_LEN. | `Repaint` |
| `0x08` / `0x7F` Backspace | Decrement COMPOSE_LEN if > 0. | `Repaint` |
| `0x20`–`0x7E` printable | Append to COMPOSE_BUF if `COMPOSE_LEN < MAX_MSG_LEN - 1`. | `Repaint` |
| `0x1B` Esc | No-op in Wave 7 (no modal-ish state to cancel; could clear composer in Wave 8+). | `Unhandled` |
| Other | Ignored. | `Unhandled` |

`Unhandled` for non-handled bytes lets the desktop's global keys
work — e.g. Ctrl+L lock (`0x0C`), Ctrl+D close (`0x04`), Tab cycle
(`0x09`). Note: digits 1..8 are printable ASCII so they get
inserted into the composer — same as SHELL and EDITOR-with-file.
To switch from COMMS to another app the operator uses Ctrl+D
(close), Tab (cycle), or clicks another window.

`pub fn handle_click(_mx, _my, _body) -> AppEvent` — returns
`Consumed`. No click-driven actions in Wave 7.

## Open API gaps

Pre-flight investigation (Task 0c) resolves:

1. **`crate::ui::palette` constants** — all Wave-4+ apps use these.
   Confirm `p::BG`, `p::PANEL`, `p::HAIRLINE`, `p::INK`, `p::MID`,
   `p::FAINT` are all `pub` (verified across Waves 4-6).

2. **`apps_registry::AppId::Comms` and `paint_comms`** — exist
   (verified in `src/ui/apps_registry.rs` lines 66 and ~82
   respectively).

3. **`draw_strip`, `draw_conn_pill`, `State`** — currently used by
   `comms.rs`. Wave 7 drops `draw_conn_pill` and `State` usages
   (replaces them with `font::draw_str` text). `draw_strip` stays
   for the header background — confirm it's compatible with the
   Wave-4 palette (renders a 1-px HAIRLINE bottom border over BG;
   the existing function should work as-is regardless of palette
   choice).

4. **`send_message`, `recv_message`, `connect`, `disconnect`, the
   `STATE` static and `CommState` enum, `MESSAGES` / `MSG_COUNT` /
   `MAX_MESSAGES` constants** — all exist; Wave 7 doesn't touch
   them.

5. **Stored peer ip:port for the CONNECTING/CONNECTED label** —
   `comms.rs` line 657 already hardcodes `"peer 10.0.2.42:9100"`
   for the connected state. Investigation: does the module store
   the real peer address from `connect()` for use in the label?
   If yes, use it; if no, either store it (one-line static) or
   keep the hardcoded placeholder string with a comment flagging
   it as Wave 7 holdover. Pre-flight decides which.

## Reuse from prior waves

- `crate::ui::palette as p` — Wave 3+ palette module.
- `crate::ui::console::redraw_in_rect` — Wave 1 widget, embedded
  shell strip uses it.
- `crate::ui::widgets::draw_strip` — Wave 1 widget, header
  background.
- `crate::ui::font::draw_str` — Wave 1 bitmap-font drawing.
- `crate::ui::gpu::{fill_rect, framebuffer, width, height}` —
  framebuffer primitives.
- `crate::ui::apps_registry::AppEvent` and the `AppId::Comms`
  variant.

## Failure modes

- **Long compose line** (`COMPOSE_LEN >= MAX_MSG_LEN - 1`):
  silently ignore further input chars (existing behavior at line
  849).
- **Empty Enter** (`COMPOSE_LEN == 0`): no-op (existing line 840).
- **`send_message` fails**: existing logic logs the error inline.
  Wave 7 doesn't change the failure surface.
- **Disconnected + Enter**: existing logic at line 840 only sends
  if COMPOSE_LEN > 0; the composer hint says "(composer disabled —
  not connected)" but a determined operator could still type and
  Enter — `send_message` would then fail. Existing behavior;
  unchanged.

## Testing

- Build clean: `cargo build --release --target aarch64-unknown-none --features gicv3`
- Clippy clean: `cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings`
- QEMU walk-through:
  - Boot, unlock with `sphragis-dev`
  - Press `7` → COMMS opens with the calm-register banner:
    "COMMS  DISCONNECTED" header in INK, MID empty-state
    message in the timeline body, FAINT `> ` prompt + char
    counter `0 / 80` in MID.
  - Type a few chars in the composer — should appear as INK with
    the new block cursor advancing.
  - Press `5` (well, won't work because COMMS eats the digit as
    text — fine, use Tab to cycle to SHELL).
  - In SHELL: `comms connect 127.0.0.1:9100` (or any address).
    Comms switches to CONNECTING then either CONNECTED or ERROR.
    Header text updates with the new state.
  - In COMMS, send a message via Enter — should append a timeline
    row with `[HH:MM] >> you  <text>` all in INK.
  - Confirm no green/red/amber/cyan anywhere in the COMMS window.
- No automated tests (kernel has no `lib.rs` / test harness).

## Out-of-scope reminders (do not implement in Wave 7)

- Action-strip hotkeys (e.g. `[D]isconnect`, `[C]lear`).
- Modifier-keybinds in COMMS (no actions need them).
- Restructuring the header / body / composer layout.
- Dropping or rewiring the embedded shell strip.
- Multi-peer / peer list / channel switching.
- File attachments / large message support / scrollback in the
  timeline (beyond the existing 32-message buffer).
- Touching the wire protocol or any crypto code.
- Modifying `console.rs`, `widgets.rs`, or any other shared file
  beyond the comms.rs import changes.

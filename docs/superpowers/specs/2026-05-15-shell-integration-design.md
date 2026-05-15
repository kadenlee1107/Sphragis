# Wave 6 — SHELL Integration — Design

Replace the Wave-2 `paint()` stub at `src/ui/shell.rs:11185` with a
real WM-app SHELL. Decompose the existing dead-code `run()` loop body
into a stateful `handle_key(c: u8) -> AppEvent` that the desktop
event pipeline can drive. **No technical changes** to commands,
parsing, history, autocomplete, or the 75 existing command
implementations — Wave 6 is the looks + WM plumbing only.

## Scope

In:

- New `pub fn handle_key(c: u8) -> AppEvent` in `src/ui/shell.rs`,
  derived from the per-byte dispatch logic at lines 96–214 of
  `run()`.
- Real `pub fn paint(rect: WindowRect)` replacing the Wave-2 stub at
  line 11185 — delegates to existing `console::redraw_in_rect(rect)`
  + overlays a block cursor at `(CURSOR_X, CURSOR_Y)`.
- New `pub fn handle_click(mx, my, body) -> AppEvent` — Wave 6 ships
  click consumed (no click-driven behavior; selection mode is
  out-of-scope).
- Module-level statics for the input state previously stack-local in
  `run()`: `cmd_buf`, `cmd_len`, `esc` (the ANSI ESC parser, kept
  for compatibility with the ESC-sequence path even though WM keys
  arrive pre-parsed as `0x90`–`0x93`).
- `static SHELL_INITED: AtomicBool` first-paint guard: clears the
  rect, paints the welcome banner, calls `console::prompt()` once,
  flips the flag.
- Wire `AppId::Shell` in `src/ui/apps_registry.rs` to the new
  `handle_key` / `handle_click` instead of the default no-ops.

Out:

- Selection mode (`console::enter_select_mode` etc. — already coded;
  Wave 7+ wires shift+arrow if useful).
- Mouse selection / right-click paste / clipboard.
- Status strip or action strip — user chose pure-terminal.
- Cave-context indicator in chrome (the existing `console::prompt`
  already encodes cave info; no extra chrome needed).
- Killing or rewriting the dead-code `run()` — stays with its
  existing `#[allow(dead_code)]`.
- Modifying any of the 75 command implementations.
- Modifying `src/ui/console.rs` (it already exposes everything we
  need: `redraw_in_rect`, `putc`, `puts`, `puts_hi`, `prompt`,
  `cursor()`).
- UART mirroring inside `handle_key` — the headless `main::
  serial_shell` loop still drives UART independently. Mirroring
  both would double-print when both inputs are active.

## Visual system

Inherits Wave-3/4. No new constants. Console module already uses
its own palette (BG / FG / a few accent colors used by `puts_hi`);
the cursor block uses Wave-4 INK (`#E5E7EB`).

## Layout

Full-window console. No status strip, no action strip. The console
output area fills the window minus its existing internal margins.

```
┌──────────────────────────────────────────────────────────────────┐
│       ___       _      ___  ___                                  │  welcome banner
│      | _ ) __ _| |_   / _ \/ __|                                 │  (puts_hi rows)
│      | _ \/ _` |  _| | (_) \__ \                                 │
│      |___/\__,_|\__|  \___/|___/                                 │
│                                                                  │
│   Microkernel Shell v0.3 — Type 'help' for commands              │
│   Zero dependencies. Zero trust.                                 │
│                                                                  │
│ sphragis > write notes.txt ""█                                   │  prompt + cursor
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

The cursor (`█` in the mockup) is a 1-cell solid INK rectangle. When
the cursor position has a char (e.g., the operator hasn't reached
end-of-line), the char is repainted in BG-on-INK so it stays
readable.

## Input handling

Mirrors `run()` lines 96–214 with state pulled into module statics.

| Byte | Effect |
|------|--------|
| `0x0D` Enter | Append `\n` to console, execute via `execute(cmd)`, record in history (`shell_history::record`), clear `cmd_buf`, reprompt |
| `0x08` / `0x7F` Backspace | Decrement `cmd_len`, emit `0x08 b' ' 0x08` to erase the last char on screen, reset history cursor |
| `0x03` Ctrl+C | Print `^C\n`, clear `cmd_buf`, reset history cursor, reprompt |
| `0x1B` Esc | Same as Ctrl+C (cancel current input). Matches Wave-4 register's "Esc = cancel current mode-ish state." |
| `0x09` Tab | Autocomplete via existing `shell_completion::complete_command` / `complete_argument` |
| `0x90` (Up) | History prev via `shell_history::prev` — redraws input line |
| `0x91` (Down) | History next via `shell_history::next` — redraws input line |
| `0x92` (Left) / `0x93` (Right) | Ignored. (Current shell ignores too — no in-line editing.) |
| `0x20`–`0x7E` printable | Append to `cmd_buf` if `cmd_len < MAX_CMD_LEN - 1`; emit char to console; reset history cursor |
| Other | `AppEvent::Unhandled` so the desktop's launcher shortcuts (1..8) and global keys (Ctrl+L lock, Tab cycle, Ctrl+D close) still work |

The "Other → Unhandled" arm is what makes the desktop's 1..8
launcher reachable from inside SHELL: a digit IS a printable ASCII
char, so it gets inserted as text. But Ctrl-modified digits and
non-ASCII keys fall through. Practical impact: to switch from SHELL
to another app the operator uses Tab (cycle), Ctrl+D (close), or
clicks another window. Same as EDITOR's behavior once a file is
loaded.

## Module statics

```rust
static mut SHELL_CMD_BUF: [u8; MAX_CMD_LEN] = [0; MAX_CMD_LEN];
static mut SHELL_CMD_LEN: usize = 0;
static mut SHELL_ESC: EscState = EscState::default_const();  // see note
static SHELL_INITED:  AtomicBool = AtomicBool::new(false);
```

**Note on `EscState`:** the existing `shell_history::EscState`
struct (used in `run()` line 38) needs a `const fn default_const()`
or similar that can initialize a static. If `EscState` already
implements `Default` non-const, the simplest fix is to wrap it in
`Option<EscState>` initialized to `None` and lazily construct on
first use. Pre-flight resolves which path.

## Paint

```rust
pub fn paint(rect: WindowRect) {
    if !SHELL_INITED.load(Ordering::Relaxed) {
        console::init_in_window();
        console::puts_hi(BANNER_LINE_1);
        // ... all banner lines ...
        console::puts("  Microkernel Shell v0.3 — Type 'help' for commands\n");
        console::puts("  Zero dependencies. Zero trust.\n");
        console::puts("\n");
        console::prompt();
        SHELL_INITED.store(true, Ordering::Relaxed);
    }
    console::redraw_in_rect(rect);
    paint_cursor(rect);
}
```

`paint_cursor` reads `console::cursor() -> (col, row)`, computes the
pixel position relative to `rect`, draws a 1-cell INK rect, and
re-renders the char (if any) at that position in BG-on-INK.

The cursor's pixel position is in screen coordinates derived from
console's internal `CHAR_W` × `CHAR_H` cell grid plus its
`MARGIN_X` / `MARGIN_Y` constants. Pre-flight confirms these are
either pub or have accessors; if not, Wave 6 adds them.

## Handle click

```rust
pub fn handle_click(_mx: i32, _my: i32, _body: WindowRect) -> AppEvent {
    AppEvent::Consumed
}
```

No click-driven behavior in Wave 6. Returning `Consumed` (not
`Unhandled`) prevents the desktop from interpreting a click in the
shell window as a focus-other-window gesture — though since the
window IS focused at click time, this is mostly cosmetic.

## First paint init

The console module has `console::init()` (full-screen mode, used by
the headless boot path) and `console::init_in_window()` (WM mode,
just resets the cursor). Wave 6 uses `init_in_window` on first paint
to avoid stomping the desktop chrome.

The banner is a copy of `run()` lines 22–30. Pre-flight may extract
these into a `BANNER` constant for re-use, or inline them in the
init block.

## Open API gaps

Pre-flight investigation (Task 0c) resolves:

1. **`EscState::default()` const-ness** — can it init a static? If
   not, use `Option<EscState>` pattern.
2. **`console::CHAR_W` / `CHAR_H` / `MARGIN_X` / `MARGIN_Y`** — are
   they `pub`? If not, add small `pub fn cell_pixel_pos(col, row)
   -> (x, y)` helper inside console.rs.
3. **`console::cursor()`** — already verified `pub`, returns `(col,
   row)` as `(u32, u32)`.
4. **`shell_history` module's public API** — `prev()`, `next()`,
   `record(&[u8])`, `reset_cursor()` are used in `run()`; verify
   each is `pub`.
5. **`shell_completion` module's public API** — `complete_command`,
   `complete_argument`, `split_for_completion_parts`,
   `arg_kind_for_parts`, `ArgKind` — verify each is `pub`.

If any are private, Wave 6 makes them `pub` (single-line changes).

## Reuse from prior waves

- `console::redraw_in_rect` — existing widget, already used by other
  apps embedding shell views (FS, CM, BC per the function's doc
  comment).
- `console::putc`, `console::puts`, `console::puts_hi`,
  `console::prompt`, `console::cursor`, `console::init_in_window` —
  all existing pub functions.
- `shell_history::{prev, next, record, reset_cursor}` — existing.
- `shell_completion::{complete_command, complete_argument, ...}` —
  existing.
- `execute(cmd: &str)` — existing private function at line 229.
  Stays private; called only from `run()` (dead) and the new
  `handle_key`.
- `apps_registry::AppEvent` and the `AppId::Shell` variant — exist.

## Failure modes

- **`cmd_len` overflow**: silently ignore further input chars (current
  behavior at line 206 — `cmd_len < MAX_CMD_LEN - 1` guard).
- **Empty Enter**: `cmd_len == 0` skips the `execute` call; just emits
  newline + reprompt. (Existing behavior at line 102.)
- **Concurrent UART + GUI input**: each path has its own cmd buffer.
  The headless `main::serial_shell` doesn't share `SHELL_CMD_BUF`
  with the WM path. Both paths invoke the same `execute()` which
  operates on the passed `&str`, no shared mutable state at execute
  time. Race risk is minimal in cooperative single-CPU.
- **First paint before banner finishes**: `SHELL_INITED` flips
  AFTER the banner is written, so a concurrent paint (impossible
  in single-CPU cooperative model) would re-banner. Fine.

## Testing

- Build clean: `cargo build --release --target aarch64-unknown-none --features gicv3`
- Clippy clean: `cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings`
- QEMU walk-through:
  - Boot, unlock with `sphragis-dev`
  - Press `5` → SHELL window opens, banner paints, prompt appears
  - Type `help` + Enter → command list scrolls in
  - Type `caves list` + Enter → cave list
  - Type a partial command + Tab → autocomplete suggests
  - Press ↑ → previous command recalls; ↓ → forward in history
  - Press Backspace → last char deletes
  - Press Ctrl+C → input cancels, fresh prompt
  - Type `write notes.txt "hello\n"` + Enter → command runs;
    `ls` confirms file present
  - Press `2` to switch to FILES → confirm `notes.txt` visible
  - Press Enter on `notes.txt` → confirm EDITOR opens with content
- No automated tests (kernel has no `lib.rs` / test harness).

## Demo flow this unblocks (Wave 5 follow-up)

The Wave 5 walk-through stopped at "no way to create new files in
UI". Wave 6 closes that loop:

1. Boot + unlock
2. Press `5` → SHELL
3. `write notes.txt "## My notes\n"` + Enter
4. Press `2` → FILES (the new file shows up)
5. Press Enter on `notes.txt` → EDITOR opens with the content
6. Edit, press `S` to save, press Esc to return to FILES

End-to-end, all four Wave-4/5 apps tie together.

## Out-of-scope reminders (do not implement in Wave 6)

- Mouse selection / clipboard.
- Status strip / action strip / cave indicator chrome.
- Blinking cursor (calm-mono register; solid block only).
- Touching the 75 command implementations.
- Killing the dead-code `run()` function.
- Modifying `console.rs` beyond optional pub-ifying of constants.
- Selection mode wiring (already coded; no UI trigger added).
- Horizontal scroll of long commands (current shell wraps via
  console's existing scrollback).
- In-line editing (Left/Right cursor within command).
- Multi-line / heredoc input.

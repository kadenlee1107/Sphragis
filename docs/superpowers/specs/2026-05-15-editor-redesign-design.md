# Wave 5 — EDITOR redesign + TT cleanup — Design

Replace the legacy 826-line cyberpunk EDITOR (`src/ui/apps/editor.rs`)
with a calm Wave-4-register single-buffer editor that opens files
from FILES, edits them in place, and saves back to SealFS. Bundle a
trivial commit deleting the orphaned 1327-line `src/ui/truetype.rs`
(`#![allow(dead_code)]`, zero call sites, dead since the no-browser
pivot).

## Scope

In:

- Rewrite `src/ui/apps/editor.rs` to the Wave-4 register
- Single-buffer text editor (1024 lines × 256 cols)
- Cursor navigation: ↑/↓/←/→ for char-by-char, PgUp/PgDn for
  viewport jumps, Home/End for line start/end
- Insertion / deletion / Enter (split line) / Backspace
- Scrolling viewport; cursor stays visible
- Light comment-line tokenization: lines starting with `//`, `#`,
  `;`, or `--` (after whitespace) render in MID; everything else
  in INK
- Open flow: pressing Enter or [E] (or clicking the [E]dit action
  strip button) on a selected file in FILES loads its bytes into
  EDITOR and switches the active app
- Save flow: [S]ave overwrites the file via
  `sealfs::ns_delete + sealfs::ns_create`
- Revert flow: [R]evert reloads from SealFS; ConfirmModal protects
  this if the buffer is dirty
- Esc flow: if dirty, ConfirmModal "Discard unsaved changes?";
  commit (or no-dirty case) switches active app back to FILES
- Empty state: "No file open — press 2 to open from FILES"
- Wire the FILES app's [E] action (currently FAINT/Wave-5 stub from
  Wave 4) to the actual handoff
- Delete `src/ui/truetype.rs` and the `pub mod truetype;` declaration
  in `src/ui/mod.rs`

Out:

- Selection / copy / paste (Wave 6+; needs clipboard infrastructure)
- Multi-buffer / tabs (the existing visual-only 3 tabs go away;
  Wave 6+ can revisit if useful)
- Per-language syntax highlighting beyond comment-line awareness
- New-file creation in EDITOR (use SHELL `write foo.txt ""` for now)
- "Save as" with name prompt — Wave 5 always saves back to the
  loaded file's name
- Undo / redo (Wave 6+)

## Visual system

Inherits Wave-3/4. No new constants.

- BG `#0D0D10`, PANEL `#18181C`, HAIRLINE `#2A2A30`
- INK `#E5E7EB`, MID `#6B7280`, FAINT `#4A4D55`
- 8×16 bitmap font (`src/ui/font.rs`)
- Pure-mono discipline preserved (no green/red/amber)

## Layout

Custom 3-row layout — full-width status strip top, gutter+text in the
middle, action strip at bottom. Not Inspector (no sidebar) and not
Cockpit (no big metric panels) — a third layout pattern specific to
text editors.

```
┌─────────────────────────────────────────────────────────────────┐
│ ● notes.txt · modified              14.2 KB · enc · L 9 : C 24 │  status strip, 28 px
├──────┬──────────────────────────────────────────────────────────┤
│   1  │ ## Reconnaissance notes                                  │  MID (comment line)
│   2  │                                                          │
│   3  │ Target: corp.example                                     │  INK
│   4  │ Subdomains:                                              │  INK
│   5  │   - app.corp.example                                     │  INK
│   6  │   - mail.corp.example                                    │  INK
│   7  │   - vpn.corp.example                                     │  INK
│   8  │                                                          │
│   9  │ Last sweep: 2026-05-12█                                  │  current row, PANEL bg
│  10  │ // TODO: refresh after merge                             │  MID
│  11  │ # pull in subdomains-tier-2.txt                          │  MID
├──────┴──────────────────────────────────────────────────────────┤
│ Save · Revert · Esc back to FILES                                │  action strip, 28 px
└─────────────────────────────────────────────────────────────────┘
```

### Status strip (top)

- Left: dirty marker dot (filled INK = modified, hollow MID = saved),
  filename in INK, " · modified" caption in MID when dirty
- Right: size · encrypted/plain · MLS classification · cursor position
  `L <line> : C <col>`
- Single 28-px row, HAIRLINE below

The dirty marker reuses `paint_state_dot(x, y, filled)` from Wave 3.

### Gutter + text region (middle)

- 36-px gutter (5 chars × 8 px = 40 px nominal; tightened to 36 with
  small horizontal padding)
- Gutter renders right-aligned line numbers in FAINT; the
  current-cursor line renders in INK
- 1-px HAIRLINE vertical divider between gutter and text
- Text region starts at gutter-right-edge + 12 px padding
- Current-cursor line gets a PANEL background tint across the
  entire visible width (gutter + text)
- Cursor rendered as a 1-px-wide INK block at the next character
  position (last column = INK-on-INK block, indicating "type to
  append")
- Text rendered per line: scan from the first non-whitespace byte;
  if it matches `//`, `#`, `;`, or `--`, the whole line is MID;
  otherwise INK
- Viewport: only lines `[viewport_start .. viewport_start + visible_rows]`
  are painted; visible_rows derived from body height / CHAR_H

### Action strip (bottom)

- HAIRLINE above, 28-px row, padding 14 px
- `paint_action_strip` (Wave-3 widget) renders the actions
- Actions: `[S]ave`, `[R]evert`, `[Esc] back to FILES`
- Save + Revert enabled iff dirty (FAINT otherwise via
  `Action::enabled = false`)
- The "Esc back" entry is not a hotkey for the action strip per se
  — it's a hint that pressing Esc returns to FILES. Renders as a
  FAINT label item

## Cross-app handoff

Wave 4's FILES action strip showed `[E]dit (W5)` as a FAINT stub.
Wave 5 wires it up.

### Kernel-side hint slot

A module-private static in `src/ui/apps/editor.rs`:

```rust
static mut PENDING_FILE: [u8; NAME_MAX] = [0; NAME_MAX];
static mut PENDING_LEN:  usize = 0;

pub fn set_pending_file(name: &str) { /* writes name into PENDING_*  */ }
fn take_pending_file() -> Option<&'static [u8]> { /* clears + returns */ }
```

`set_pending_file` is `pub` so the FILES app can call it across
modules. `take_pending_file` is private to EDITOR.

### FILES app changes

In `src/ui/apps/filemanager.rs`:

- `handle_key_viewing` adds `b'\n' | 0x0A` (Enter) → open in EDITOR
- `handle_key_viewing`'s existing `b'e' | b'E'` arm changes from
  `AppEvent::Consumed` (Wave-4 stub) to the open-in-EDITOR path
- The action strip's `[E]dit (W5)` entry becomes `[E]dit` with
  `enabled: true`
- `action_strip_hit_test` returning `'E'` triggers the same open path

The open path is one helper:

```rust
fn open_selected_in_editor() -> AppEvent {
    let (count, _) = sealfs::ns_stats();
    if count == 0 { return AppEvent::Consumed; }

    let mut name_buf = [0u8; NAME_MAX];
    let mut name_len = 0;
    let sel = selected_file();
    let mut row_index: usize = 0;
    sealfs::ns_list(|n, _, _| {
        if row_index == sel {
            let l = n.len().min(NAME_MAX);
            name_buf[..l].copy_from_slice(&n.as_bytes()[..l]);
            name_len = l;
        }
        row_index += 1;
    });
    if name_len == 0 { return AppEvent::Consumed; }
    let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };

    crate::ui::apps::editor::set_pending_file(name);
    crate::ui::desktop::set_active_app(crate::ui::apps_registry::AppId::Editor);
    AppEvent::Repaint
}
```

`desktop::set_active_app` is a new entry point — see "Open API gaps"
below.

### EDITOR consumes the hint

`paint()` checks for a pending file on each call:

```rust
if let Some(name_bytes) = take_pending_file() {
    let name = unsafe { core::str::from_utf8_unchecked(name_bytes) };
    load_from_sealfs(name);
}
```

Idempotent — once consumed, the slot clears and subsequent paints
no-op the load.

### Esc back to FILES

`handle_key` for Esc (`0x1B`) — if dirty, opens ConfirmModal
"Discard unsaved changes?"; commit (or non-dirty case) calls
`desktop::set_active_app(AppId::Files)`. FILES preserves its prior
selection / viewport state.

## Buffer + edit semantics

```rust
const MAX_LINES:    usize = 1024;
const MAX_LINE_LEN: usize = 256;

struct Buffer {
    lines:      [[u8; MAX_LINE_LEN]; MAX_LINES],
    line_lens:  [u16; MAX_LINES],
    line_count: usize,
}

static mut BUFFER: Buffer = Buffer { ... };
static mut CURSOR_ROW: usize = 0;
static mut CURSOR_COL: usize = 0;
static mut VIEWPORT_START: usize = 0;
static mut DIRTY: bool = false;
static mut CURRENT_FILE_NAME: [u8; NAME_MAX] = [0; NAME_MAX];
static mut CURRENT_FILE_LEN:  usize = 0;
```

Files >1024 lines truncate at load time with a one-row banner in the
status strip: "truncated to 1024 lines" — flagged to the operator;
edit + save would overwrite with the truncated content (their problem
if they save, but at least it's visible).

### Key bindings

| Key | Effect |
|-----|--------|
| `←` `→` `↑` `↓` | Char / line cursor move; viewport scrolls with cursor at edges |
| `PgUp` `PgDn` | Move viewport by `visible_rows`; cursor follows |
| `Home` `End` | Start / end of current line |
| Printable ASCII | Insert at cursor; advance cursor; set DIRTY |
| `Backspace` | Delete char before cursor (or join with prev line if at col 0); set DIRTY |
| `Enter` | Split line at cursor; cursor moves to start of new line; set DIRTY |
| `Tab` | Insert 4 spaces (matches Wave 1 EDITOR behavior) |
| `S` (no modifier) | Save — overwrite via `sealfs::ns_delete` + `sealfs::ns_create` |
| `R` (no modifier) | Revert — ConfirmModal if dirty; reload from SealFS |
| `Esc` | If dirty, ConfirmModal; else switch active app to FILES |

The unmodified `S` / `R` / `Esc` capture is the action-strip hotkey
pattern from Waves 3/4. The user-mode editing typing has no modal
"insert/normal mode" distinction — this is a calm text editor, not
vim.

## Open API gaps

Pre-flight investigation required (same as Wave 4 §"Kernel API
gaps"). The implementation plan's pre-flight resolves these.

1. **`desktop::set_active_app(AppId)`** — does the WM expose a way
   to programmatically switch the active app? If yes, use as-is.
   If no, Wave 5 adds it. Likely lives in `src/ui/desktop.rs` next
   to whatever state tracks "currently focused window."

2. **`apps_registry::AppId::Files` / `AppId::Editor`** — these
   enum variants exist (Wave 4 confirmed). Just confirm import path.

3. **`sealfs::ns_delete + sealfs::ns_create` overwrite pattern** —
   shell.rs `write` command uses this. Confirm the exact API
   (signatures verified in Wave 4: `ns_delete(&str) -> Result<(),
   &'static str>`, `ns_create(&str, &[u8]) -> Result<(), &'static
   str>`).

4. **Keyboard scan codes for PgUp/PgDn/Home/End** — Wave-4 NET/
   SECURITY apps used `0x90` (Up) and `0x91` (Down) from
   `virtio::keyboard`. Confirm PgUp/PgDn/Home/End codes (likely
   `0x9C`/`0x9D`/`0x9E`/`0x9F` per the existing keyboard module).

5. **Truetype delete safety** — confirm `src/ui/truetype.rs` has
   zero call sites (`grep -rE 'truetype::' src/` returns nothing
   per Wave 5 brainstorm). The `pub mod truetype;` declaration in
   `src/ui/mod.rs` is the only reference.

## Failure modes

- **Load fails** (file doesn't exist or read error): empty buffer
  + status strip shows "load failed: <reason>" in MID. User can
  press Esc back to FILES and try a different file.
- **Save fails** (SealFS full, locked, key missing): status strip
  shows "save failed: <reason>" in MID; dirty flag stays set so
  the user knows the buffer hasn't reached disk.
- **Buffer overflow** (typing past 256 cols on a line): silently
  ignored (no-op). The 256-col limit is generous for typical
  prose / config / log content.
- **Truncated load** (file >1024 lines): status strip shows
  "truncated to 1024 lines" warning. Operator is responsible for
  not saving over the original if they care.

## Testing

- Build clean: `cargo build --release --target aarch64-unknown-none --features gicv3`
- Clippy clean: `cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings`
- QEMU walk-through (manual, same pattern as Waves 2/3/4):
  - Boot, unlock with `sphragis-dev`
  - Use SHELL `write notes.txt "## Note\nhello\n"` to create a test
    file (SHELL is still the no-WM headless shell; that's Wave 6
    work)
  - Open FILES (`2`), select `notes.txt`, press Enter
  - Confirm EDITOR opens with the file loaded
  - Type a few chars, confirm dirty marker turns INK
  - Press `S`, confirm dirty marker clears
  - Press `Esc`, confirm return to FILES with selection preserved
  - Open another file, edit, press `Esc` without saving →
    ConfirmModal appears → Cancel returns to EDITOR; Discard
    returns to FILES
  - Try opening a file >1024 lines (or simulate) → confirm
    "truncated" warning paints

No automated tests — the kernel has no `cargo test` harness (no
`lib.rs`).

## Reuse from Wave 4

| Widget | Use |
|--------|-----|
| `paint_state_dot` | Dirty marker in status strip |
| `paint_action_strip` | Bottom action strip |
| `action_strip_hit_test` | Click → key routing |
| `Action` struct | Save / Revert / Esc-back entries |
| `ConfirmModal` + `paint_confirm_modal` | Revert / Esc-with-dirty confirmation |
| `confirm_modal_key` + `ModalAction` | Modal routing |

`InspectorLayout` is not used — EDITOR's full-width text region
doesn't fit the sidebar+detail shape.

## Out-of-scope reminders (do not implement in Wave 5)

- Selection / clipboard
- Search / replace
- Multi-buffer tabs
- Per-language syntax highlighting beyond comment-line awareness
- Undo / redo
- New-file creation in EDITOR
- "Save as" rename
- Indentation-aware Tab (just inserts 4 spaces)
- Line wrapping (long lines clip; user scrolls horizontally — wait,
  we don't have horizontal scroll; long lines just clip at the
  right edge of the visible area)

Wave 6+ revisits whichever of these is most-needed in practice.

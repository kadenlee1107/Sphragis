# Caves Manager — Design

**Date:** 2026-05-14
**Status:** Brainstormed and approved
**Wave:** 3 of N (UI overhaul; Waves 1–2 shipped 2026-05-14)
**Prior wave:** [Desktop chrome + app launcher](2026-05-14-desktop-chrome-design.md)

## Goal

Redesign the CAVES app — the first per-app redesign of the Wave 1/2 UI sweep — to the same calm institutional register as the lock screen and desktop chrome. Capture the recurring app patterns (inspector layout, action strip, status-field list, inline edit form, confirm modal, state dot) as **reusable widgets in `src/ui/widgets.rs`** so Wave 4 apps (NET, FILES, SECURITY, EDITOR, COMMS) can inherit them directly.

## Motivation

The existing `src/ui/apps/caves_mgr.rs` (508 lines) has a real layout — 60/40 table + detail panel with an embedded shell strip and a footer — but it ships in the pre-Wave-1 cyberpunk palette (CYAN / GREEN / AMBER / RED, all imported from `src/ui/widgets.rs`). It also paints full-screen, ignoring its `WindowRect`. After Wave 2 (windowed chrome + calm palette), a user who unlocks the lock screen → lands on a calm desktop → clicks CAVES → sees aggressive cyberpunk colors snaps the credibility line.

Wave 3 also needs to set the pattern other apps inherit. The Wave 2 spec calls Wave 4 "mechanical once the pattern is set" — that statement is only true if Wave 3 surfaces real reusable widgets, not just visually refreshes one app.

## Visual system

Inherits the [Wave 2 palette](2026-05-14-desktop-chrome-design.md#palette) verbatim:

| Name | Hex | Use this wave |
|------|-----|----------------|
| `BG` | `#0d0d10` | window body, form-field background |
| `PANEL` | `#18181c` | sidebar fill, selected-row highlight, header strips |
| `HAIRLINE` | `#2a2a30` | dividers, borders, separators |
| `INK` | `#e5e7eb` | focused content, selected name, active hotkey letter, filled state dot |
| `MID` | `#6b7280` | secondary text (field keys, action labels), hollow state dot |

No new colors. The whole app is monochromatic — no green / amber / red. **State signals use brightness shift, not hue shift**, matching Wave 2's alert-badge convention.

### Typography

Same constraints as Waves 1–2: bitmap font only (TT rasterizer fix lands Wave 5). All text in `src/ui/widgets.rs` and the new CAVES app uses the kernel's 8×16 bitmap font.

## Layout — Inspector

```
┌──────────────────────────────────────────────────────────────────┐
│ CAVES                              (Wave-2 window chrome)        │ ← 22 px chrome (Wave 2)
├──────────────────────┬───────────────────────────────────────────┤
│ CAVES (4)     2 ●    │ kali-recon                                │ ← detail header
│                      │ RUNNING · ENTERED 2H 14M AGO              │ ← state line
│ › ● kali-recon       │ ───────────────────────────────────────── │
│   ● tor-browser      │ PID    1842                               │
│   ○ workshop-old     │ NET    isolated                           │
│   ○ tmp-scratch      │ MLS    sens=Secret integ=SystemTrusted    │ ← status field list
│                      │ MOUNT  / /home/kali-recon                 │
│                      │ TAINT  0x00000001 · PII                   │
│ + new cave           │ AUDIT  142 lines · ok                     │
│                      │ ───────────────────────────────────────── │
│                      │ [E]nter  [S]top  [C]onfigure  [D]estroy   │ ← action strip
└──────────────────────┴───────────────────────────────────────────┘
   sidebar (38%)        detail panel (62%)
```

### Window dimensions

Default size 720 × 480 (matches Wave 2's `wm::WindowRect` defaults for new windows). The app must respect its `WindowRect` and degrade gracefully:
- Below 600 px wide: collapse the sidebar to a 32-px-wide rail showing only state dots, no names.
- Below 320 px tall: drop the action strip; user must use hotkeys.

### Sidebar (38% of window body width)

| Region | Contents |
|--------|----------|
| Header strip | `CAVES (n)` on the left, `k ●` running-count on the right, with a 1-px HAIRLINE bottom border. |
| Cave list | One row per cave: state dot + name. Selected row gets PANEL fill, with a `›` selection cursor in INK. Unselected names are MID; the selected name is INK. |
| Pinned bottom | `+ new cave` row. Always last. When create mode is active, this row becomes the selected row (gets the same PANEL highlight). |

Rows are clickable: clicking selects + focuses the detail panel on that cave. Keyboard up/down moves selection. Enter (or click) on `+ new cave` enters create mode.

### Detail panel (62% of window body width)

Three modes:

| Mode | Trigger | Contents |
|------|---------|----------|
| **View** (default) | A cave is selected in the sidebar. | Name header → state line → field list (PID / NET / MLS / MOUNT / TAINT / AUDIT) → action strip. |
| **Create** | `^N` pressed, `+ new cave` clicked, or `n` pressed while sidebar focused. | "New cave" header → editable form (NAME / NET MODE / MLS SENS / MLS INTEG / MOUNT / TAINT) → footer hint `Enter to Create · Esc to cancel`. |
| **Configure** | `C` pressed (or `[C]onfigure` clicked) on a selected cave. | "Configure <name>" header → same form, pre-filled with the cave's current values → footer hint `Enter to Apply · Esc to cancel`. |

The state-dot column reserves a fixed 2-character width in the sidebar so the `›` selection cursor doesn't shift dot position when selection moves.

### Action strip (bottom of detail panel, View mode only)

Four actions:

| Hotkey | Label | Kernel call |
|--------|-------|-------------|
| `E` | `[E]nter` | `cave::enter(name)` |
| `S` | `[S]top` | `cave::stop(name)` — hard kill; stop-with-grace is deferred. After stop, the UI stays in View mode on the same cave (which now shows `○` Stopped). |
| `C` | `[C]onfigure` | Switches detail panel to Configure mode |
| `D` | `[D]estroy` | Opens destroy-confirm modal |

Action labels render as `[E]nter` etc. with the bracketed letter in INK and the rest in MID. The whole `[E]nter` token is a click target (whole word + brackets). Hotkey + click are equivalent.

### Action availability

| Action | Enabled when |
|--------|--------------|
| `E`nter | cave state is Running OR Idle (not Stopped — stopped caves must Configure or Destroy first) |
| `S`top | cave state is Running |
| `C`onfigure | always (view-only mode for read-only fields is acceptable; Wave 3 makes all fields editable) |
| `D`estroy | always |

Disabled actions render in `FAINT` (a derived color = MID at ~50% — use `0xFF4A4D55`) and don't accept clicks or hotkeys. Hovering / focusing them shows no cursor change.

## Cave creation form

Triggered by `^N` (any state), `n` (sidebar focused), or click on `+ new cave`.

### Fields

| Field | Type | Default | Edit |
|-------|------|---------|------|
| `NAME` | text, 1–16 ASCII chars | empty (focused on entry) | type to edit, Backspace. Allowed chars: `a-z 0-9 - _`. No uppercase, no spaces, no other punctuation. Collision check is case-sensitive (but since uppercase is rejected, this is effectively case-insensitive). |
| `NET MODE` | enum | `isolated` | Space or → cycles `isolated → routed → custom`; Shift+Space or ← cycles backward. **Note: kernel API TBD — see [§Kernel API gaps](#kernel-api-gaps).** |
| `MLS SENS` | enum (`Sensitivity`) | `Confidential` | Same cycle. Values: `Unclassified · Confidential · Secret · TopSecret`. |
| `MLS INTEG` | enum (`Integrity`) | `Sandboxed` | Same cycle. Values: `Untrusted · Sandboxed · SystemTrusted · HighIntegrity`. |
| `MOUNT` | text, single line | `/ /home/<name>` (auto-derived from NAME field) | type to edit. If user hasn't edited and NAME changes, MOUNT auto-updates. Once user has edited MOUNT manually, NAME changes leave MOUNT alone. **Implementation note:** this auto-update is CAVES-specific UI state, not a feature of the shared `InlineEditForm` widget. The widget treats each field as independent text. The "dirty" tracking lives in `caves_mgr.rs`. |
| `TAINT` | hex u32 | `0x00000000` | type hex digits, 8 chars max, `0x` prefix is implicit |

### Field navigation

- `Tab` advances to next field; wraps to NAME after TAINT.
- `Shift+Tab` reverses.
- Click into any field focuses it directly.
- Currently-focused field renders with `INK` border (rest of borders are `HAIRLINE`).

### Validation

`NAME` is the only required field. The Create / Apply button is enabled only when NAME is non-empty AND doesn't collide with an existing cave name. If NAME is invalid (empty / collision / bad char), the footer shows the reason in MID: e.g. `NAME exists` or `NAME must be 1–16 chars`.

### Submit + cancel

- `Enter` (from any field) submits if NAME is valid; otherwise focuses NAME and shows the reason.
- `Esc` cancels: leaves the form without committing, returns the detail panel to View mode for whichever cave was previously selected (or the empty state if none).

### Submit sequence (kernel calls)

Cave creation is multi-step because the kernel's existing API only sets NAME + ephemeral at `create()`. The form must:

1. Call `cave::create(name, ephemeral=false)` → returns `cave_id` or error.
2. If create succeeds: call `cave::set_sensitivity_by_name(name, sens)`.
3. Call `cave::set_integrity_by_name(name, integ)`.
4. Set NET mode (API TBD — see [§Kernel API gaps](#kernel-api-gaps)).
5. Set MOUNT (API TBD).
6. Set TAINT (likely `taint::stamp(cave_id, value)` — verify during plan).

If any post-`create()` step fails, leave the partially-configured cave in place and surface the error in the footer; the user can re-`C`onfigure to fix. Don't roll back the cave — caves are cheap, and partial configuration is recoverable.

## Configure flow

`C` (or click `[C]onfigure`) on a selected cave switches to Configure mode with the same form pre-filled. The NAME field is read-only in this mode (kernel API doesn't support rename today; show as `INK` text without a border). All other fields are editable.

Submit calls the same per-field setter sequence (skip `create()`). Cancel reverts to View mode.

## Destroy confirmation

Press `D` (or click `[D]estroy`) on a selected cave → centered modal overlay. The modal is a `ConfirmModal` widget reused across apps for destructive operations.

### Modal contents

- Title: `Destroy <name>?`
- Body lists the irreversible consequences:
  - kill all processes inside the cave
  - zero the cave's encryption keys
  - wipe its SealFS subtree
  - clear MLS labels + taint records
- Footer: `IRREVERSIBLE.` in INK on its own line
- Commit hint: `D again to confirm · Esc to cancel`

### Interaction

- Second `D` press (within the modal) commits: calls `cave::destroy(name)`, closes modal, returns to View mode on the next remaining cave (or the empty state).
- `Esc` cancels: closes modal, leaves cave intact.
- Click outside the modal: cancels (treat as Esc).
- No timeout; no enter-to-confirm (Enter is reserved for the form's primary action).

If `cave::destroy` returns an error, the modal stays open and surfaces the error message in the footer in `INK`; user must Esc out and try again or accept the partial state.

## State dot semantics

Two states, pure monochrome:

| Glyph | Meaning | Color |
|-------|---------|-------|
| `●` | Running (cave has at least one live process) | `INK` |
| `○` | Stopped or Idle (no live processes) | `MID` |

Drawn as a 4-px solid circle inscribed in a 6-px grid (matches Wave-2 launcher tile silhouette dimensions for visual consistency). Hollow is the same circle with a 1-px ring drawn over BG fill.

No third state in Wave 3. If Wave 4+ needs Paused / Suspended / Crashed, extend then — for now the two-state model covers what the kernel reports.

## Empty state

When `cave::list` returns zero caves:

| Region | Contents |
|--------|----------|
| Sidebar header | `CAVES (0)` left, blank right (no running count) |
| Sidebar body | Empty |
| Sidebar pinned bottom | `+ new cave` (selected by default) |
| Detail panel | "No caves yet" header → "Press `N` or click `+ new cave` to create one." in MID. No action strip. |

`N` (no modifier) jumps to Create mode from the empty state for one-keystroke onboarding. Once any cave exists, `^N` is required (plain `N` is reserved for cave selection by first letter — Wave 4 follow-up).

## Pattern primitives — extracted to `src/ui/widgets.rs`

Six new public widgets. Each has a documented API; Wave 4 apps `use crate::ui::widgets::*` and compose them. All widgets paint into a caller-supplied `WindowRect`.

### 1. `InspectorLayout`

Sidebar + detail split. The sidebar width is a fraction of the rect width.

```rust
pub struct InspectorLayout {
    pub sidebar_pct: u32,   // default 38
    pub body_rect: WindowRect,
}

impl InspectorLayout {
    pub fn new(rect: WindowRect) -> Self;
    pub fn sidebar_rect(&self) -> WindowRect;
    pub fn detail_rect(&self) -> WindowRect;
    pub fn paint_divider(&self);  // 1px HAIRLINE vertical line between sidebar and detail
}
```

Used by CAVES (Wave 3) and FILES, NET, SECURITY (Wave 4).

### 2. `ActionStrip`

Bottom-of-panel hotkey row. Each entry is `[L]abel` with the letter in INK and rest in MID.

```rust
pub struct Action<'a> {
    pub hotkey: char,           // 'E'
    pub label: &'a str,         // "Enter" — total rendered string is "[E]nter"
    pub enabled: bool,          // false renders in FAINT, doesn't accept input
}

pub fn paint_action_strip(rect: WindowRect, actions: &[Action]);
pub fn action_strip_hit_test(rect: WindowRect, mx: i32, my: i32, actions: &[Action])
    -> Option<char>;  // returns the hotkey of the clicked action
```

Used by every Wave 4 app.

### 3. `StatusFieldList`

Key/value rows. Keys render in MID with letter-spacing; values in INK.

```rust
pub struct StatusField<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

pub fn paint_status_field_list(rect: WindowRect, fields: &[StatusField]);
```

Auto-aligns keys to a column width = longest key + 2-char padding. Caller controls field order.

### 4. `InlineEditForm`

The Tab-cycling, Space-cycling editable form. Generic over field types.

```rust
pub enum FieldKind<'a> {
    Text { buf: &'a mut [u8], len: usize, max: usize },
    Enum { values: &'a [&'a str], selected: usize },
    Hex32 { value: u32 },
}

pub struct FormField<'a> {
    pub key: &'a str,
    pub kind: FieldKind<'a>,
    pub readonly: bool,
}

pub fn paint_inline_edit_form(rect: WindowRect, fields: &[FormField], focused: usize);
pub fn handle_form_key(fields: &mut [FormField], focused: &mut usize, c: u8) -> FormAction;
pub fn handle_form_click(fields: &mut [FormField], focused: &mut usize, rect: WindowRect, mx: i32, my: i32);

pub enum FormAction { None, Submit, Cancel }
```

Wave-3-internal: pre-populating with cave values + dispatching kernel calls on submit is the app's job, not the widget's.

Used by CAVES (Wave 3); likely by FILES (file properties) and SECURITY (deadman config) in Wave 4.

### 5. `ConfirmModal`

Centered destructive-action overlay.

```rust
pub struct ConfirmModal<'a> {
    pub title: &'a str,
    pub body_lines: &'a [&'a str],
    pub commit_key: char,        // 'D' for destroy, 'W' for wipe, etc.
}

pub fn paint_confirm_modal(screen_w: u32, screen_h: u32, modal: &ConfirmModal);
pub fn confirm_modal_key(modal: &ConfirmModal, c: u8) -> ModalAction;

pub enum ModalAction { None, Commit, Cancel }
```

Modal claims the full-screen below TOPBAR_H, dims everything else (alpha-blend toward BG at ~35%), centers a PANEL-filled panel sized to its content. Used wherever destructive UX shows up.

### 6. `StateDot`

```rust
pub fn paint_state_dot(x: u32, y: u32, filled: bool);
```

6×6 px glyph. Drawn at `(x, y)`; filled is INK, hollow is a 1-px ring in MID over BG fill.

## Kernel API gaps

Items the spec assumes but that need verification (or stubbing) during the implementation plan:

1. **NET MODE per cave.** `crate::net::is_isolated()` exists as a global Wave-2 stub. Is net isolation per-cave? If so, what's the setter? If not, we either (a) drop NET MODE from the form and show a single global indicator, or (b) add a per-cave `set_net_mode` API as a Wave-3 kernel change.
2. **MOUNT.** The form lets the user edit a mount-string. Verify what cave-mount editing looks like at the kernel level. If it's complex, downgrade MOUNT to read-only in the form for Wave 3.
3. **TAINT.** `taint::stamp(cave_id, value)` is mentioned in the 2026-05-13 journal; confirm signature and verify it accepts a fresh u32.
4. **Rename.** Plan currently says NAME is read-only in Configure mode because no rename API. If rename exists, lift the restriction.
5. **`cave::list` API.** Verify `pub fn list<F: FnMut(&Cave)>(callback: F)` (line 2034) is the right enumeration entry point. Decide whether the UI calls it on every paint or maintains a cached snapshot invalidated on actions.

The plan's pre-flight should resolve all five before Task 1.

## State machine

```
                ┌──────────────┐
                │  EMPTY       │ ← no caves
                └──────┬───────┘
                       │ create cave
                       ▼
              ┌────────────────┐
              │  VIEWING       │ ← default; cave selected, detail showing
              └──┬───────────┬─┘
                 │           │  ^N / n / click "+ new cave"
                 │           ▼
                 │   ┌────────────────┐
                 │   │  CREATING      │
                 │   └──────┬─────────┘
                 │          │ Enter (valid) → cave::create + setters
                 │          ▼
                 │   back to VIEWING (new cave selected)
                 │
                 │  C / click [C]onfigure
                 ▼
         ┌────────────────┐
         │  CONFIGURING   │
         └──────┬─────────┘
                │ Enter (valid) → per-field setters
                ▼
         back to VIEWING (same cave selected)

  Anywhere in VIEWING:
    D / click [D]estroy
    ┌─────────────────────┐
    │  CONFIRM_DESTROY    │  ← modal overlay
    └──────┬──────────────┘
           │ second D / click Commit → cave::destroy
           ▼
    back to VIEWING (next cave selected, or EMPTY)
```

Esc returns to the previous state from CREATING, CONFIGURING, and CONFIRM_DESTROY.

## Implementation outline

Sketch only. The writing-plans skill produces the bite-sized plan next.

1. **New widgets in `src/ui/widgets.rs`.** Extract or write the 6 primitives. The current widgets.rs has cyberpunk-era helpers (`draw_strip`, `draw_kv_row`, `CYAN`, `AMBER`, etc.); refresh those with Wave-2 palette imports or replace where the existing API doesn't match the new design.
2. **Replace `src/ui/apps/caves_mgr.rs`.** New file structure:
   - `State` enum: `Viewing { selected: usize }`, `Creating { form: FormState }`, `Configuring { cave: usize, form: FormState }`, `ConfirmDestroy { cave: usize }`.
   - `static mut APP_STATE` with volatile-helper access (Rust-2024 convention from Wave 2).
   - `pub fn paint(rect: WindowRect)` — dispatches on APP_STATE.
   - `pub fn handle_key(c: u8) -> Event` — Wave 3 will surface this as a new entry point the desktop event loop can call when the focused window is CAVES.
   - `pub fn handle_click(mx: i32, my: i32) -> Event` — same for pointer.
3. **Wire app keyboard + pointer dispatch.** Today's `src/ui/wm.rs` only paints; the desktop event loop in `src/ui/desktop.rs` routes input to its own state machine, not to the focused window's app. Wave 3 must extend the WM/desktop boundary with two new entry points: when a key arrives and a window is focused, the app gets first dibs; when a pointer click lands inside a window's body (not chrome), the app gets the click in body-local coordinates. The plan's first task should be defining the exact dispatch contract — an `AppEvents` extension to `AppDescriptor` is the likely shape, but verify by reading the Wave-2 `apps_registry::AppDescriptor` first.
4. **Kernel-API verification.** Resolve the 5 gaps in [§Kernel API gaps](#kernel-api-gaps). Stubs go in the relevant `caves/` / `net/` / `taint/` modules with `// Wave 3 stub` markers if a real setter doesn't exist yet.
5. **QEMU walk-through.** Manual: open CAVES, see at least one cave (or empty state), press N, type a name, Enter, watch a new cave appear in the sidebar. Cycle through Configure / Destroy. No automated test (`#![no_std] #![no_main]` constraint; see Waves 1–2 plans).

## What's NOT in v1 (deferred to later waves)

- Drag-to-reorder caves in the sidebar
- Multi-select (delete-many / configure-many)
- Cave search / filter
- Rename cave (depends on kernel API)
- Process list inside a cave (deferred to Wave 4's NET or a Wave 5 SHELL integration)
- Live audit log streaming in the detail panel (AUDIT field shows count + status only)
- Cave templates / clone-from-existing
- Per-cave color theme / icon
- Animations on state transitions
- Stop-with-grace (`S` is currently a hard kill via `cave::stop`)
- Right-click context menu

## Scope boundary

| Wave | Surface | Notes |
|------|---------|-------|
| **4** | FILES + NET + SECURITY + EDITOR + COMMS | Each app picks up the 6 widgets from `src/ui/widgets.rs` and composes them. Mechanical — most apps need the same shape (sidebar + detail + actions + status fields). |
| **5** | SHELL + console palette refresh + TT rasterizer fix | SHELL is 11k lines and gets its own scoping. Once the rasterizer fix lands, revisit Wave-2/3 typography (window titles, app headers) and migrate from bitmap to Plex Sans. |
| **Follow-up** | Per-cave icons / templates / search | Once the app's daily use is real, the surface deepens. |
| **Follow-up** | Rename cave (kernel API + UI lift) | Depends on cave rename support. |

## Non-goals

- No animations or transitions. State changes are instant.
- No accessibility audit (mirrors Waves 1–2).
- No internationalization (kernel is English-only by spec).
- No theme system (palette hardcoded).
- No remote display (kernel framebuffer only).
- No drag-and-drop between apps.
- Bitmap font only; live TT rendering for app text is a Wave 5 follow-up.

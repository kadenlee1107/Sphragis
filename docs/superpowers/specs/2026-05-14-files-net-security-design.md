# Wave 4 — FILES + NET + SECURITY Design

**Date:** 2026-05-14
**Status:** Brainstormed and approved
**Wave:** 4 of N (UI overhaul; Waves 1–3 shipped 2026-05-14)
**Prior wave:** [Caves Manager redesign](2026-05-14-caves-manager-design.md)

## Goal

Redesign three of the five remaining cyberpunk-era apps — **FILES**, **NET**, **SECURITY** — to the Wave 1/2/3 calm institutional register, composed from the Wave-3 widget library plus four new shared widgets this wave extracts. EDITOR and COMMS deferred to later waves because their content shapes (text editing surface, message/crypto routing) don't fit either of the two layout patterns this wave is locking in.

## Motivation

After Wave 3 shipped, three apps still render in the pre-Wave-1 cyberpunk palette and ignore their `WindowRect`. They also have content shapes that no longer match what an operator actually needs:

* **FILES** is a flat metadata table. The current "encrypted: yes · SHA-256: abc…" rows are largely **static configuration boilerplate** that the operator already knows. What's missing is the actual file — text or hex preview — which today requires an EDITOR (Wave-5 stub). Result: FILES is "wearing shoes but not walking."
* **NET** is an interface + firewall status display with no live activity stream. Operators in a security-first OS want to see **what's happening** (DNS lookups, TCP opens, firewall drops, TLS handshakes), not just configuration. The existing flow-box diagram is decorative, not informative.
* **SECURITY** duplicates the caves list (now in CAVES app from Wave 3), and its SECURITY PIPELINE / INTEGRITY panels aren't surfacing the live data an operator's panic console should: deadman countdown, audit chain status, attempted auths, taint flow.

Wave 4 also locks in the **two layout patterns** the OS uses long-term:
* **Inspector** (sidebar + detail) — for "browse a list, drill into one." Established by Wave 3 caves_mgr; applied to FILES this wave.
* **Cockpit** (multi-panel dashboard with live activity log) — for "monitor system state at a glance." New this wave; applied to NET and SECURITY. Wave 5+ apps with cockpit-shaped content (audit, observability) inherit.

## Visual system

Inherits the [Wave 2 palette](2026-05-14-desktop-chrome-design.md#palette) verbatim, accessed through `crate::ui::palette` (`pub const BG / PANEL / HAIRLINE / INK / MID / FAINT`). No new colors. **Pure-mono discipline preserved**: destructive actions (`[W]ipe NOW`, `[D]elete`) render in INK like every other action token — the verb and the ConfirmModal carry the protective intent, not the color. This matches Wave 3's destroy-cave treatment.

Typography: bitmap font only (`font::draw_str`). Wave 5 will revisit when the TT rasterizer is fixed.

## Layout patterns

### Inspector (Wave 3)

Sidebar + detail split, default 38% / 62% via `widgets::InspectorLayout::with_sidebar_pct(38)`. Sidebar = list-of-things with state dots and selection cursor. Detail = drilldown of the selected item. Used by Wave 4's FILES.

### Cockpit (new in Wave 4)

```
┌─ Stats strip ──────────────────────────────────────── RX 4.2 KB/s · TX 1.1 KB/s ┐  (optional, NET only)
├──────────────────────────────────────────────────────────────────────────────────┤
│ ┌─ Panel ─────────────┐  ┌─ Panel ──────────────────────────────────────────┐  │
│ │ Top-row state       │  │ Top-row state                                    │  │
│ └─────────────────────┘  └──────────────────────────────────────────────────┘  │
│                                                                                  │
│ ┌─ Activity log ───────────────────────────────────────────────────────────────┐│
│ │ 14:32:01  event_kind   summary                                                ││
│ │ 14:31:58  event_kind   summary                                                ││
│ │ ↑↓ scrolls                                                                    ││
│ └───────────────────────────────────────────────────────────────────────────────┘│
│                                                                                  │
│ ┌─ Panel ─────────────┐  ┌─ Panel ──────────────────────────────────────────┐  │  (optional bottom row, SECURITY only)
│ └─────────────────────┘  └──────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────────┤
│ [A]ction · [B]action · [C]action                                                 │
└──────────────────────────────────────────────────────────────────────────────────┘
```

No `CockpitLayout` struct — the layout is straightforward rect-splitting math owned by each app, using `widgets::paint_status_panel` for each panel and `widgets::paint_activity_log` for the middle stream. Stats strip and bottom row are optional.

## App: FILES

### Purpose

Inspector + file viewer. The user can browse files, see metadata, **and view the actual contents** (text or hex preview). Editing is deferred to EDITOR (Wave 5). FILES becomes a useful audit / inspection / cleanup tool day one.

### Layout

Inspector (38% / 62%). No top stats strip; no bottom row.

**Sidebar:** file list, one row per file. State dot per row: filled INK = encrypted, hollow MID = plain. Selected row gets PANEL highlight + `›` cursor in INK. No pinned `+ new file` row — files come from EDITOR-save or kernel writes, not from this app.

**Detail panel:**
* Header row: filename in INK (left) + one-line metadata in MID (right): `<size> · encrypted|plain · MLS=<sens>` (truncated if too long).
* 1-px HAIRLINE separator.
* Preview region (~90% of remaining height): renders the file contents. See [§File preview](#file-preview).
* 1-px HAIRLINE separator.
* Action strip: `[D]elete · [E]dit` where `[E]dit` is FAINT-colored with `(Wave 5)` suffix.

### File preview

Two modes, chosen by a printable-ASCII sniff on the first 256 decrypted bytes:

* **Text mode** (mostly printable): line-numbered view. Line numbers in MID, content in INK by default (or `p::MID` if the file isn't currently encrypted — operator visual cue). Lines wrap at the visible width (not truncated). Viewport shows `≈viewport_h / 16` lines; `↑/↓` arrows scroll half-viewport (~10 lines) per press.
* **Hex mode** (binary): xxd-style: offset (MID), 16 bytes of hex (INK), ASCII column (INK with `.` for non-printable). 6 bytes per line rendered. `↑/↓` arrows scroll half-viewport (~8 rows) per press.

**Sniff heuristic**: count printable-ASCII bytes (`0x20–0x7E` plus `\n \t \r`) in first 256 bytes. If `≥ 90%` printable → text mode; else hex mode.

### Encrypted file handling

* If the file's `encrypted` flag is `false` → read directly, sniff + preview.
* If `encrypted = true`:
  * The kernel attempts to decrypt using the active cave's keys (set by `cave::enter`).
  * If decryption succeeds → preview the plaintext.
  * If decryption fails (active cave doesn't have the key, or no active cave) → preview shows:
    ```
    encrypted · 8.0 KB
    preview requires cave context
    ```
    in MID, centered vertically. Metadata strip still shows full info (size, MLS, owner cave name from BatFS metadata).
* Wave 4 does NOT implement cave-key switching UI. Operator switches via the CAVES app's `[E]nter` action first, then opens FILES.

### Actions

| Hotkey | Label | Behavior |
|--------|-------|----------|
| `D` | `[D]elete` | Opens `ConfirmModal` with the file name + irreversibility warning. Second `D` calls `batfs::ns_delete(name)`. Selection drops to next file (or empty state). |
| `E` | `[E]dit` | **Wave-5 stub.** Renders FAINT + `(Wave 5)`. Click / hotkey both no-op for now. |

No `[O]pen` action — preview IS the open. No `[R]ename`, `[N]ew` — those need BatFS API work that's out of scope.

### State machine

```
        ┌───────────────┐
        │   EMPTY       │   (no files)
        └───────┬───────┘
                │ first batfs::ns_create
                ▼
        ┌───────────────┐
        │   VIEWING     │   (default: file selected, preview rendered)
        └───────┬───────┘
                │ D
                ▼
        ┌────────────────────┐
        │  CONFIRM_DELETE    │   (modal overlay)
        └───────┬────────────┘
                │ second D → batfs::ns_delete
                ▼
        back to VIEWING (next file) or EMPTY
```

No CREATING / CONFIGURING modes. FILES is purely browse + preview + delete.

## App: NET

### Purpose

Live cockpit showing what the network is doing right now. State (mode, firewall) up top; activity stream in the middle; minimal actions at the bottom.

### Layout

Cockpit. From top to bottom:

1. **Stats strip** (single row, padded 8-px): `RX <rate>` · `TX <rate>` (left) · `PEAK <bytes>` · `UPTIME <hh:mm:ss>` (right). Background BG, 1-px HAIRLINE bottom border.
2. **Top panel row** (`grid 1fr 1.5fr`, 12-px gap):
   * **MODE panel**: label `MODE`, value `ISOLATED` (or `ROUTED` / `CUSTOM`) in INK, 1-line semantic explanation in MID (`no outbound to non-cave routes`).
   * **FIREWALL panel**: label `FIREWALL`, header-right badge `default: DENY` in INK, KV body: `rules <X> allow · <Y> deny` / `drops <N> in last 60s` / `last drop <ts> · <target>`.
3. **Activity log** (fills middle):
   * Label `ACTIVITY` letter-spaced top-left, viewport indicator top-right (`showing last 7 of 142 · ↑↓ scrolls`).
   * Entries: `<timestamp>  <kind>  <summary>`. Kinds: `dns`, `tcp_open`, `tcp_close`, `fw_drop`, `tls_hs`. Timestamp + label-of-kind in MID; summary in INK.
   * Wave 4 ships a hardcoded set of event kinds; the kernel logs to a fixed ring (≤256 entries). The kind set is extensible — Wave 5+ can add more without touching the widget.
4. **Action strip**: `[T]oggle isolation · [C]lear counters`.

### Actions

| Hotkey | Label | Behavior |
|--------|-------|----------|
| `T` | `[T]oggle isolation` | Flips global net mode between `Isolated` and `Routed`. Calls `net::set_isolation(bool)` (Wave-4 kernel stub). |
| `C` | `[C]lear counters` | Resets RX/TX/peak counters and clears the activity ring. Calls `net::clear_counters()` (Wave-4 kernel stub). Logged to AUDIT via the activity ring's own `counters_cleared` entry. |

### Counter + activity model

The kernel's `net` module gains (Wave-4 stubs in pre-flight if missing):

* `net::rx_rate() -> u32` — bytes/sec, rolling average over last second.
* `net::tx_rate() -> u32` — same.
* `net::peak_bytes() -> u64` — high-water RX or TX bytes since boot or last `clear_counters`.
* `net::uptime_secs() -> u64` — seconds since boot.
* `net::activity_iter(F)` — `FnMut(&ActivityEntry)` callback over the ring (newest-first or oldest-first; pick one and document).
* `net::activity_count() -> usize` — total entries.

`ActivityEntry`:

```rust
pub struct ActivityEntry {
    pub ts:      u64,         // monotonic seconds
    pub kind:    ActivityKind,
    pub summary: [u8; 96],    // ASCII null-padded
    pub summary_len: u8,
}

pub enum ActivityKind {
    Dns, TcpOpen, TcpClose, FwDrop, TlsHs, CountersCleared,
}
```

If the kernel doesn't have an activity ring today, pre-flight adds it as a small `[ActivityEntry; 256]` ring with `static mut` head/tail (matching the established kernel-static pattern). Net subsystem call sites that generate events get a one-line `net::activity::push(ActivityKind::Dns, &fmt_summary)`.

## App: SECURITY

### Purpose

Operator panic console. Live monitoring of every security pillar an operator might need to act on. The most loaded app in the OS — and the one with destructive actions.

### Layout

Cockpit. From top to bottom:

1. **Top panel row** (`grid 1fr 1fr`, 12-px gap):
   * **DEADMAN panel**: label `DEADMAN`, header-right state badge (`ARMED` / `DISARMED` / `TRIGGERED`) in INK, big bitmap-font countdown (mm:ss or h:mm:ss for >1h), 1-line sub-caption in MID (`wipe in 47 min · check-in by 15:19:36`).
   * **AUTH panel**: label `AUTH`, header-right state badge (`ok` / `lockout`) in INK, big bitmap-font ratio (`4 / 5`), 1-line sub-caption in MID (`attempts remaining · last fail 2h 14m ago`).
2. **AUDIT log** (fills middle):
   * Label `AUDIT` letter-spaced top-left, chain-status in MID right (`chain: 1,847 entries · root <hash>… · verified ok`).
   * Entries: `<timestamp>  <kind>  <summary>`. Kinds: `cave_create`, `set_taint`, `file_delete`, `net_isolate`, `cave_enter`, `auth_pass`, `auth_fail`, plus any operator-meaningful action.
   * Pulled from the existing audit ring (tamper-evident chain — already exists per yesterday's audit-tamper-evident-chain branch).
3. **Bottom panel row** (`grid 1fr 1fr`, 12-px gap):
   * **TAINT panel**: label `TAINT`, header-right count badge (`N caves tainted`), KV body: `system OR <hex> · <semantic labels>` then per-tainted-cave `<name> <hex>`. Labels hardcoded: bit 0=PII, bit 1=CRYPTO, bit 2=AUDIT, bit 3=NETWORK. Higher bits show as `0x<hex>` without label.
   * **INTEGRITY panel**: label `INTEGRITY`, header-right state badge (`clean` / `<N> denies`), KV body: `BLP denies <N> in 24h` / `Biba denies <N> in 24h · last <ts>` / `TE denies <N> in 24h`.
4. **Action strip**: `[R]e-arm deadman · [V]erify chain · [W]ipe NOW`.

### Actions

| Hotkey | Label | Behavior |
|--------|-------|----------|
| `R` | `[R]e-arm deadman` | Resets the deadman timer to its configured period (48h default). Instant — no confirm (the deadman doesn't fire by accident; the operator should be able to re-arm without friction). Calls `deadman::arm(hours)`. |
| `V` | `[V]erify chain` | Walks the audit chain from genesis to head, verifying each entry's hash. Blocks for the duration (paints `chain: verifying…` in the AUDIT panel header). Result writes back to the header as `verified ok` or `verified FAIL @ entry N`. Calls `audit::verify_chain()`. |
| `W` | `[W]ipe NOW` | Opens `ConfirmModal` listing what wipe destroys: all cave keys, BatFS, audit ring, MLS labels, taint records. Same modal widget as caves_mgr destroy. Second `W` calls `wipe::execute(WipeReason::Manual, false)`. **The `[W]ipe NOW` token renders in INK** (pure-mono discipline). |

### Removed from existing security.rs

* **ACTIVE BATCAVES panel** — moved to CAVES app in Wave 3.
* **SECURITY PIPELINE diagram** — replaced by AUDIT log (the pipeline was decorative; the audit log is informative).
* **Existing INTEGRITY panel** — restructured into the new INTEGRITY panel with explicit deny counts.

## Pattern primitives — extracted to `src/ui/widgets.rs`

Four new shared widgets land alongside the Wave-3 set. All use the existing palette module + bitmap font + GPU primitives.

### 1. `paint_activity_log`

Paginated time-stamped event stream. Used by NET (activity ring) and SECURITY (audit chain).

```rust
pub struct ActivityEntry<'a> {
    pub timestamp_str: &'a str,   // pre-formatted "HH:MM:SS"
    pub kind:          &'a str,   // pre-formatted, e.g. "dns" / "tcp_open"
    pub summary:       &'a str,
}

pub fn paint_activity_log(
    rect: WindowRect,
    entries: &[ActivityEntry],
    viewport_start: usize,
    total: usize,
);
```

Renders one entry per line. Timestamp + kind in MID (kind padded to 12 chars for alignment), summary in INK. Top-right of rect shows `showing last N of M · ↑↓ scrolls`. Caller owns scroll state.

### 2. `paint_status_panel`

Bordered PANEL with a labeled header strip. Used 6× across NET + SECURITY.

```rust
pub struct StatusPanel<'a> {
    pub label:        &'a str,        // letter-spaced caps top-left, e.g. "MODE"
    pub header_right: Option<&'a str>, // optional INK badge top-right
    pub body:         &'a [StatusField<'a>],  // reuses Wave-3 StatusField
}

pub fn paint_status_panel(rect: WindowRect, panel: &StatusPanel);
```

Draws PANEL fill + HAIRLINE 1-px border. Header row at the top with `label` (letter-spaced MID) + `header_right` (INK if present). Body region renders the field list via the existing `paint_status_field_list`.

### 3. `paint_big_metric`

Large value display for DEADMAN countdown and AUTH attempts.

```rust
pub fn paint_big_metric(rect: WindowRect, label: &str, value: &str, sub: &str);
```

Letter-spaced label (MID) at top, big value (INK) centered, sub-caption (MID) below. Sphragis has only one font size — `font::draw_str` — so "big" is achieved by rendering each character at 2× scale via a simple per-pixel block-fill helper. Add `font::draw_str_scale(fb, w, x, y, s, fg, bg, scale: u32)` if not present (likely needs adding; pre-flight verifies). At `scale=2` each glyph is 16×32 px instead of 8×16.

### 4. `paint_file_preview`

Text/hex viewer pane. Used by FILES.

```rust
pub fn paint_file_preview(
    rect: WindowRect,
    bytes: &[u8],
    viewport_start: usize,
);
```

Sniffs first 256 bytes for printable-ASCII ratio (≥90% → text, else hex). Text mode: line-numbered rendering, MID line numbers + INK content. Hex mode: xxd-style offset (MID) + hex (INK) + ASCII gutter (INK with `.` for non-printable).

Two helper modes (text / hex) live behind the same entry point so callers don't have to choose.

## Kernel API gaps

Items the spec assumes but that pre-flight needs to verify, stub, or escalate:

1. **`net::set_isolation(bool)`** — flips global net mode. Replaces the current Wave-2 `is_isolated()` stub that returns hardcoded `true`. Pre-flight either confirms it exists or adds it.
2. **`net::rx_rate() / tx_rate() / peak_bytes() / uptime_secs() / clear_counters()`** — pre-flight scans `src/net/` for these or equivalents. Missing pieces stubbed.
3. **`net::activity` ring + push API** — likely missing; pre-flight adds a 256-entry ring with `static mut` head/tail and the call-site hook `net::activity::push(kind, &summary)`. Existing net code sites that generate dns/tcp/fw events get a one-line addition.
4. **`audit::iter() / audit::chain_len() / audit::root_hash() / audit::verify_chain()`** — the tamper-evident chain shipped in yesterday's `feat/audit-tamper-evident-chain` branch. Pre-flight confirms the API shape; the SECURITY app calls it directly.
5. **`taint::for_each_tainted_cave(F)` or equivalent enumeration** — pre-flight scans `src/caves/cave.rs` taint section (around lines 747–822 per Wave-3 pre-flight) for an iterator.
6. **`integrity::deny_counts_24h() -> (blp, biba, te)`** — likely missing; pre-flight adds a simple counter struct in the integrity subsystem with rolling 24h windows.
7. **`deadman::arm(hours: u32)` and `deadman::remaining_secs() -> u64`** — confirm both exist. Wave 2 stubbed `deadman::seconds_remaining()`; verify it's still the canonical read API.
8. **`font::draw_str_scale(fb, w, x, y, s, fg, bg, scale)`** — pre-flight checks if a scaled-bitmap path exists. If not, adds a simple per-pixel block-fill renderer that takes a scale factor (1 or 2 only for Wave 4).

The pre-flight task should resolve all eight before any per-app work begins.

## Implementation outline

Sketch only. The writing-plans skill produces the bite-sized plan next.

1. **Pre-flight resolutions** — investigate the 8 kernel API gaps, write a pre-flight resolution doc (parallels Wave-3's pre-flight pattern). Identify which need stub-in-Task-2-equivalent work.

2. **Kernel stubs** (one task) — land the kernel-side additions identified in pre-flight (net activity ring, set_isolation, scaled font, etc.). Storage-only where appropriate; full enforcement is later-wave work.

3. **Four new widgets** in `src/ui/widgets.rs` — each its own task with build/clippy verification:
   1. `paint_activity_log`
   2. `paint_status_panel`
   3. `paint_big_metric` + `font::draw_str_scale` if needed
   4. `paint_file_preview`

4. **FILES app rewrite** — replace `src/ui/apps/filemanager.rs` with the Inspector + file viewer design. Reuses Wave-3 InspectorLayout, ActionStrip, ConfirmModal, plus the new `paint_file_preview`. Wires `apps_registry.rs` to point at new handlers.

5. **NET app rewrite** — replace `src/ui/apps/netmon.rs` with the Cockpit design. Uses `paint_status_panel`, `paint_activity_log`, ActionStrip.

6. **SECURITY app rewrite** — replace `src/ui/apps/security.rs` with the panic console. Uses `paint_status_panel`, `paint_big_metric`, `paint_activity_log`, ActionStrip, ConfirmModal.

7. **QEMU walk-through** — manual visual verification of all three apps + the actions that don't require deferred apps (Open-in-EDITOR is FAINT-only, so doesn't need EDITOR alive).

8. **Push + finishing-a-development-branch** — same pattern as Waves 1, 2, 3. Merge to main locally, push, delete branches, update session journal.

## What's NOT in Wave 4 (deferred)

* **EDITOR** redesign — separate later wave; needs text-editing surface that doesn't fit either pattern.
* **COMMS** redesign — separate later wave; needs message routing UI that doesn't fit either pattern.
* **SHELL** — Wave 5 per Wave 2 spec.
* **TT rasterizer fix** — Wave 5.
* **Backgrounded audit-chain verification** — Wave 4 blocks during verify; backgrounding adds threading complexity.
* **NET activity filter UI** — `[F]ilter` action dropped.
* **TAINT operator-defined bit dictionary** — hardcoded labels in Wave 4; dict file lives in `BatFS:/system/security/taint.toml` (or similar) in a later wave.
* **Per-cave NET enforcement** — Wave 3 stored `Cave.net_mode`; Wave 4 still uses global isolation. Per-cave routing/firewall is kernel work.
* **BatFS rename / new-file APIs** — neither FILES nor any other app needs them in Wave 4.
* **Animations or transitions.**
* **Right-click context menus.**

## Scope boundary

| Wave | Surface | Notes |
|------|---------|-------|
| **5** | EDITOR + SHELL + TT rasterizer fix + console palette refresh | EDITOR needs a text-editing surface; SHELL needs the 11k-line console folded into a Wave-2 window. TT rasterizer fix unlocks Plex Sans for window titles + form fields. Heaviest visual-impact wave. |
| **6** | COMMS | Crypto pinning, message routing — needs its own brainstorm. May fit Inspector (message list + thread detail) or be its own pattern. |
| **Wave 4 follow-up** | Per-cave NET enforcement (kernel) + activity-filter UI + taint dict | Once Wave 4 ships, NET's foundation is solid enough to add real per-cave routing. Activity filter and operator-defined taint labels are UX polish. |
| **Wave 5 follow-up** | Refactor Wave 1/2 modules to import `crate::ui::palette::*` instead of their local duplicates | Pure cleanup. |

## Non-goals

* No animations or transitions. State changes are instant.
* No accessibility audit (mirrors Waves 1–3).
* No internationalization (kernel is English-only by spec).
* No theme system (palette hardcoded).
* No remote display (kernel framebuffer only).
* No drag-and-drop between apps.
* Bitmap font only for app text; live TT rendering deferred to Wave 5.
* No color exceptions for destructive actions (`[W]ipe NOW`, `[D]elete` render in INK; ConfirmModal and verb wording carry the protection).

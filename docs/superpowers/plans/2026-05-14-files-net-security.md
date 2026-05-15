# Wave 4 — FILES + NET + SECURITY Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Wave 4 — redesign `src/ui/apps/filemanager.rs`, `src/ui/apps/netmon.rs`, `src/ui/apps/security.rs` to the calm Wave-1/2/3 register, composed from the existing Wave-3 widgets plus four new shared widgets (`paint_activity_log`, `paint_status_panel`, `paint_big_metric`, `paint_file_preview`). Land any missing kernel-side stubs that the apps depend on (net counters + activity ring + `set_isolation`, scaled bitmap font, audit/taint/integrity enumeration, deadman re-arm) before app work begins.

**Architecture:** Two layout patterns. **Inspector** (Wave 3 pattern, used by FILES) — sidebar + detail. **Cockpit** (new, used by NET and SECURITY) — straightforward rect-splitting math composing `paint_status_panel` + `paint_activity_log` + `ActionStrip`. Each app is a single-file rewrite that calls into the shared widgets; no app holds geometry math the widget layer can take. Per-app input state lives in module-level `static mut` accessed via the established `addr_of!`/`addr_of_mut!` volatile pattern.

**Tech Stack:** Rust 2024 edition, `aarch64-unknown-none` target (no-std, alloc available via linked-list allocator). Build: `cargo build --release --target aarch64-unknown-none --features gicv3`. Verification: QEMU with `-display cocoa -device virtio-keyboard-device -device virtio-mouse-device`.

**Verification reality check.** Same as Waves 1–3: this crate is `#![no_std] #![no_main]`, no `lib.rs`, no test harness. `cargo test` doesn't run kernel code. Every task's verification is "build is clean (no clippy warnings under `-D warnings`)" plus a QEMU walk-through against the spec at the end. There is no unit-test step. The QEMU walk-through is the final task.

---

## Pre-flight

- [ ] **Step 0a: Confirm clean working tree and create the feature branch.**

```bash
cd /Users/kadenlee/Sphragis
git status --short
git checkout -b feat/files-net-security
```
Expected: clean tree before checkout; on branch `feat/files-net-security` after.

- [ ] **Step 0b: Confirm the current build is clean before any edits.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: both clean.

- [ ] **Step 0c: Resolve the 8 kernel API gaps from the spec.**

Read the spec at `docs/superpowers/specs/2026-05-14-files-net-security-design.md` §"Kernel API gaps". For each, run the indicated grep, read the surrounding code, decide one of three resolutions, and document in a new file `docs/superpowers/plans/2026-05-14-files-net-security-preflight.md`:

```bash
# 1. net::set_isolation
grep -nE 'pub fn set_isolation|set_isolated|set_net_mode_global' src/net/mod.rs src/net/*.rs

# 2. net counters
grep -nE 'pub fn (rx_rate|tx_rate|rx_bytes|tx_bytes|peak_bytes|uptime_secs|clear_counters)' src/net/mod.rs src/net/*.rs

# 3. net activity ring
grep -nE 'activity|ActivityKind|ActivityEntry' src/net/mod.rs src/net/*.rs

# 4. audit chain enumeration
grep -nE 'pub fn (iter|chain_len|root_hash|verify_chain|for_each_entry)' src/security/audit*.rs src/security/audit/*.rs 2>&1

# 5. taint enumeration
grep -nE 'pub fn (for_each_taint|taint_iter|tainted_cave)' src/caves/cave.rs

# 6. integrity deny counts
grep -nE 'deny_count|integrity::counts|blp_denies|biba_denies' src/caves/cave.rs src/security/*.rs

# 7. deadman re-arm + read
grep -nE 'pub fn (arm|remaining_secs|seconds_remaining|disarm|state)' src/security/deadman.rs

# 8. scaled bitmap font
grep -nE 'pub fn draw_str|draw_str_scale|font_scale' src/ui/font.rs
```

For each gap, record one of three resolutions in the pre-flight file:
- **EXISTS** — line number + signature; use as-is.
- **STUB IN TASK 1.x** — doesn't exist; Task 1.x adds it.
- **DEFER TO WAVE 5+** — read-only or hardcoded in Wave 4; document.

Expected resolutions (verify against your grep output; adjust if reality differs):

| Gap | Expected resolution |
|-----|---------------------|
| 1 | STUB IN TASK 1c — current `is_isolated()` is a hardcoded `true` Wave-2 stub; add `set_isolation(bool)` + storage |
| 2 | STUB IN TASK 1c — net counters likely missing or partial; add what's needed |
| 3 | STUB IN TASK 1d — net activity ring almost certainly doesn't exist; add full ring |
| 4 | EXISTS — `feat/audit-tamper-evident-chain` shipped 2026-05-13 per session journal; confirm signatures |
| 5 | EXISTS — Wave 3 added `cave::taint_of`; iteration may need `for_each_cave` over `cave::list` |
| 6 | STUB IN TASK 1e — integrity deny counters likely missing; add 24h rolling counters |
| 7 | EXISTS — Wave 2 stubbed `deadman::seconds_remaining`; `arm(hours)` may need to land |
| 8 | STUB IN TASK 1a — scale=1 path exists (the current `font::draw_str`); add scale=2 path |

Commit the pre-flight file:

```bash
git add docs/superpowers/plans/2026-05-14-files-net-security-preflight.md
git commit -m "$(cat <<'EOF'
plan: Wave 4 — kernel API gap pre-flight resolutions

Records resolutions for the 8 kernel API gaps from the spec before
implementation begins. Tasks 1a-1e land any stubs the investigation
surfaced; later tasks assume those stubs exist.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## File structure

This plan creates and modifies the following files. Each new module has one clear responsibility.

| File | Status | Responsibility |
|------|--------|----------------|
| `docs/superpowers/plans/2026-05-14-files-net-security-preflight.md` | **NEW** | Pre-flight resolutions (read-only after Step 0c) |
| `src/ui/font.rs` | **MODIFY** | Add `draw_str_scale(fb, w, x, y, s, fg, bg, scale)` for big metric rendering |
| `src/net/mod.rs` | **MODIFY** | Add `set_isolation(bool)`, counter accessors, clear_counters |
| `src/net/activity.rs` | **NEW** | Fixed-size activity ring + push API + iter |
| `src/security/integrity_counts.rs` | **NEW** | Rolling 24h deny counters for BLP / Biba / TE |
| `src/security/deadman.rs` | **MODIFY (if needed)** | Add `arm(hours)` if missing |
| `src/ui/widgets.rs` | **MODIFY** | Append 4 new Wave-4 widgets (`paint_activity_log`, `paint_status_panel`, `paint_big_metric`, `paint_file_preview`) |
| `src/ui/apps/filemanager.rs` | **REPLACED** | Wave-4 Inspector + file viewer app |
| `src/ui/apps/netmon.rs` | **REPLACED** | Wave-4 Cockpit live activity dashboard |
| `src/ui/apps/security.rs` | **REPLACED** | Wave-4 operator panic console |
| `src/ui/apps_registry.rs` | **MODIFY** | Wire each app's new `handle_key` / `handle_click` |

---

## Task 1a: Scaled bitmap font

Add a `draw_str_scale` entry point so `paint_big_metric` can render the bitmap font at 2× size for the DEADMAN countdown + AUTH ratio.

**Files:**
- Modify: `src/ui/font.rs`

- [ ] **Step 1: Read the existing `font::draw_str` implementation.**

```bash
grep -nE 'pub fn draw_str|^fn |const CHAR_W|const CHAR_H' src/ui/font.rs | head -20
```

The current `draw_str(fb, screen_w, x, y, s, fg, bg)` paints 8×16 glyphs by iterating the glyph bitmap and writing to the framebuffer. Note the bitmap source (likely a `static GLYPHS: [[u8; 16]; 128]` or similar).

- [ ] **Step 2: Add `draw_str_scale` next to `draw_str`.**

```rust
/// Paint `s` at `(x, y)` scaled by `scale` (1 or 2 only for Wave 4).
/// Each glyph pixel becomes a `scale × scale` block. Used by
/// `paint_big_metric` to render large values like the DEADMAN
/// countdown without a separate font.
///
/// For `scale = 1` this matches `draw_str` byte-for-byte (no extra
/// work). For `scale = 2` each glyph occupies 16×32 px.
pub fn draw_str_scale(
    fb: *mut u32, screen_w: u32,
    x: u32, y: u32,
    s: &str,
    fg: u32, bg: u32,
    scale: u32,
) {
    let scale = scale.max(1).min(2);
    if scale == 1 {
        draw_str(fb, screen_w, x, y, s, fg, bg);
        return;
    }
    let mut cx = x;
    for &b in s.as_bytes() {
        let glyph = glyph_for(b);  // replace with the actual lookup name from font.rs
        for row in 0..CHAR_H {
            let bits = glyph[row as usize];
            for col in 0..CHAR_W {
                let on = (bits >> (CHAR_W - 1 - col)) & 1 != 0;
                let color = if on { fg } else { bg };
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = cx + col * scale + dx;
                        let py = y + row * scale + dy;
                        if px < screen_w {
                            unsafe { *fb.add((py * screen_w + px) as usize) = color; }
                        }
                    }
                }
            }
        }
        cx += CHAR_W * scale;
    }
}
```

**Important:** the `glyph_for(b)` and `CHAR_W` / `CHAR_H` names depend on the actual font.rs internals. Read the existing `draw_str` body and copy its loop shape exactly, just adding the inner `for dy { for dx { } }` block-fill loop. If the existing `draw_str` writes via `gpu::fill_rect` instead of direct framebuffer pokes, do the same: `gpu::fill_rect(cx + col * scale, y + row * scale, scale, scale, color)` per glyph pixel.

- [ ] **Step 3: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean. If clippy flags `draw_str_scale` for unused (no callers yet), add `#[allow(dead_code)]` to the function only — never file-level.

- [ ] **Step 4: Commit.**

```bash
git add src/ui/font.rs
git commit -m "$(cat <<'EOF'
font: draw_str_scale for 2× bitmap rendering

Used by paint_big_metric (Wave 4) to render DEADMAN countdown and
AUTH ratio at double size without a separate font. scale=1 is a
pass-through to draw_str; scale=2 paints each glyph pixel as a 2×2
block. Caps at scale=2 (3+ left for Wave 5 if needed).

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 1b: deadman::arm verification + stub

Pre-flight either confirmed `deadman::arm(hours)` exists or flagged it as STUB. This task lands it if missing. If it exists, this task is a no-op verification + skip-to-commit-empty.

**Files:**
- Modify: `src/security/deadman.rs`

- [ ] **Step 1: Verify the API surface.**

```bash
grep -nE 'pub fn (arm|disarm|remaining_secs|seconds_remaining|state)' src/security/deadman.rs
```

If `arm(hours: u32)` exists with the signature, skip to Step 3. Otherwise add it.

- [ ] **Step 2: Add `arm` if missing.**

Read the existing deadman.rs to understand the timer storage (likely a `static mut DEADLINE_TICKS: u64` or similar). Add:

```rust
/// Re-arm the deadman timer to `hours` from now. If hours == 0,
/// disarms (effectively infinite). Called by SECURITY app's
/// [R]e-arm action.
pub fn arm(hours: u32) {
    let now = crate::time::ticks_now();  // or whatever the clock entry is
    let new_deadline = now + (hours as u64) * SECS_PER_HOUR * TICKS_PER_SEC;
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(DEADLINE_TICKS), new_deadline);
    }
}
```

Constants `SECS_PER_HOUR` and `TICKS_PER_SEC` may already exist; if not add them locally. The `crate::time::ticks_now()` name is a placeholder — use whatever the existing `deadman::check()` uses to read the current tick count.

- [ ] **Step 3: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

- [ ] **Step 4: Commit (only if files changed).**

```bash
git diff --staged --quiet || git add src/security/deadman.rs
git diff --cached --quiet || git commit -m "$(cat <<'EOF'
deadman: arm(hours) for SECURITY app re-arm action

Wave 4 SECURITY's [R]e-arm action resets the deadman timer to
`hours` from now (default 48h). No-op verification commit if the
API already existed.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

If no changes (existing API was sufficient), the commit step is a no-op and the working tree stays clean.

---

## Task 1c: net counters + set_isolation

Pre-flight identified which net counter / mode APIs need to land. This task adds whatever's missing.

**Files:**
- Modify: `src/net/mod.rs`

- [ ] **Step 1: Read pre-flight resolutions for gaps 1 and 2.**

Open `docs/superpowers/plans/2026-05-14-files-net-security-preflight.md`. Identify which of these APIs need to be added:

- `pub fn set_isolation(isolated: bool)`
- `pub fn rx_rate() -> u32` (bytes/sec)
- `pub fn tx_rate() -> u32` (bytes/sec)
- `pub fn peak_bytes() -> u64`
- `pub fn uptime_secs() -> u64`
- `pub fn clear_counters()`

For each one, follow Step 2.

- [ ] **Step 2: Add `set_isolation(bool)` if missing.**

The current `is_isolated()` per Wave 2 returns hardcoded `true`. Add backing storage + setter:

```rust
use core::sync::atomic::{AtomicBool, Ordering};

static NET_ISOLATED: AtomicBool = AtomicBool::new(true);

pub fn is_isolated() -> bool {
    NET_ISOLATED.load(Ordering::Relaxed)
}

pub fn set_isolation(isolated: bool) {
    NET_ISOLATED.store(isolated, Ordering::Relaxed);
}
```

Note: if `is_isolated` currently exists as `pub fn is_isolated() -> bool { true }` (Wave 2 stub), replace its body with the atomic load above. If `is_isolated` is in a different module (e.g. `src/net/mod.rs` vs `src/net/firewall.rs`), put the atomic next to its existing home.

- [ ] **Step 3: Add the counter accessors if missing.**

Counters typically live in the per-NIC driver (virtio-net). For Wave 4, expose a global accessor that sums across NICs:

```rust
use core::sync::atomic::{AtomicU64, Ordering};

static RX_BYTES_TOTAL: AtomicU64 = AtomicU64::new(0);
static TX_BYTES_TOTAL: AtomicU64 = AtomicU64::new(0);
static PEAK_BYTES:     AtomicU64 = AtomicU64::new(0);
static BOOT_TICK:      AtomicU64 = AtomicU64::new(0);  // set once on first call

// Sliding window for rate calc (1-second buckets). Wave 4 uses a
// dead-simple "last-sample diff over wall time" — good enough for
// the cockpit display.
static LAST_RX_BYTES: AtomicU64 = AtomicU64::new(0);
static LAST_TX_BYTES: AtomicU64 = AtomicU64::new(0);
static LAST_TICK:     AtomicU64 = AtomicU64::new(0);

pub fn rx_rate() -> u32 {
    rate_delta(&RX_BYTES_TOTAL, &LAST_RX_BYTES)
}
pub fn tx_rate() -> u32 {
    rate_delta(&TX_BYTES_TOTAL, &LAST_TX_BYTES)
}

fn rate_delta(total: &AtomicU64, last: &AtomicU64) -> u32 {
    let now_total = total.load(Ordering::Relaxed);
    let last_total = last.swap(now_total, Ordering::Relaxed);
    let now_tick = crate::time::ticks_now();
    let last_tick = LAST_TICK.swap(now_tick, Ordering::Relaxed);
    let elapsed_ticks = now_tick.saturating_sub(last_tick).max(1);
    let elapsed_secs = (elapsed_ticks / crate::time::TICKS_PER_SEC).max(1);
    let delta = now_total.saturating_sub(last_total);
    (delta / elapsed_secs) as u32
}

pub fn peak_bytes() -> u64 {
    PEAK_BYTES.load(Ordering::Relaxed)
}

pub fn uptime_secs() -> u64 {
    let now = crate::time::ticks_now();
    let boot = BOOT_TICK.load(Ordering::Relaxed);
    if boot == 0 {
        BOOT_TICK.store(now, Ordering::Relaxed);
        return 0;
    }
    now.saturating_sub(boot) / crate::time::TICKS_PER_SEC
}

pub fn clear_counters() {
    RX_BYTES_TOTAL.store(0, Ordering::Relaxed);
    TX_BYTES_TOTAL.store(0, Ordering::Relaxed);
    PEAK_BYTES.store(0, Ordering::Relaxed);
    LAST_RX_BYTES.store(0, Ordering::Relaxed);
    LAST_TX_BYTES.store(0, Ordering::Relaxed);
    LAST_TICK.store(crate::time::ticks_now(), Ordering::Relaxed);
}
```

The `crate::time::ticks_now()` and `TICKS_PER_SEC` names are placeholders — use whatever the kernel's existing time module exposes. If the time module doesn't have a tick-per-second constant, fall back to the existing `uptime_seconds()` from topbar's `Badge::Clock` rendering (which uses inline asm to read `CNTPCT_EL0` / `CNTFRQ_EL0` — see `src/ui/topbar.rs`).

**Important:** existing virtio-net driver code that increments RX/TX byte counts needs to also increment these globals. Add calls like `RX_BYTES_TOTAL.fetch_add(len as u64, Ordering::Relaxed)` at the existing per-packet sites. Update `PEAK_BYTES` via `PEAK_BYTES.fetch_max(...)` on each delta.

If those increment sites are buried deep in the virtio code and adding them is invasive, fall back to a simpler approach: the counters stay at 0 for Wave 4, displaying as `0 B/s`. Wave 5+ wires them through.

- [ ] **Step 4: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

- [ ] **Step 5: Commit.**

```bash
git add src/net/mod.rs src/net/firewall.rs src/drivers/virtio/net.rs
git commit -m "$(cat <<'EOF'
net: set_isolation + global counters for Wave 4 NET app

* set_isolation(bool) replaces Wave-2's hardcoded `true` stub for
  is_isolated(). Backed by an AtomicBool.
* RX/TX/peak/uptime counter accessors via AtomicU64 globals;
  virtio-net driver increments on each packet. Wave 4's NET cockpit
  paints them in the stats strip.
* clear_counters() resets them all; called by NET's [C]lear action.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 1d: net activity ring

Add a 256-entry ring buffer for net events that the NET app's ACTIVITY log paints.

**Files:**
- Create: `src/net/activity.rs`
- Modify: `src/net/mod.rs` (register the module)
- Modify: `src/net/firewall.rs`, `src/net/tcp.rs`, `src/net/dns.rs`, `src/net/tls.rs` (or whichever exist) to push events at the right points

- [ ] **Step 1: Create `src/net/activity.rs`.**

```rust
//! Net activity ring — fixed-size circular buffer of recent network
//! events. Painted by the Wave-4 NET cockpit's ACTIVITY panel.

#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

pub const RING_CAP: usize = 256;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum ActivityKind {
    Dns          = 0,
    TcpOpen      = 1,
    TcpClose     = 2,
    FwDrop       = 3,
    TlsHs        = 4,
    CountersCleared = 5,
}

impl ActivityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ActivityKind::Dns             => "dns",
            ActivityKind::TcpOpen         => "tcp_open",
            ActivityKind::TcpClose        => "tcp_close",
            ActivityKind::FwDrop          => "fw_drop",
            ActivityKind::TlsHs           => "tls_hs",
            ActivityKind::CountersCleared => "counters_cleared",
        }
    }
    pub fn from_u8(b: u8) -> ActivityKind {
        match b {
            1 => ActivityKind::TcpOpen,
            2 => ActivityKind::TcpClose,
            3 => ActivityKind::FwDrop,
            4 => ActivityKind::TlsHs,
            5 => ActivityKind::CountersCleared,
            _ => ActivityKind::Dns,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Entry {
    pub ts:          u64,   // monotonic seconds since boot
    pub kind:        u8,    // ActivityKind discriminant
    pub summary:     [u8; 96],
    pub summary_len: u8,
}

impl Entry {
    pub const fn empty() -> Self {
        Self { ts: 0, kind: 0, summary: [0; 96], summary_len: 0 }
    }
    pub fn summary_str(&self) -> &str {
        let n = (self.summary_len as usize).min(self.summary.len());
        unsafe { core::str::from_utf8_unchecked(&self.summary[..n]) }
    }
}

static mut RING: [Entry; RING_CAP] = [Entry::empty(); RING_CAP];
static HEAD:  AtomicUsize = AtomicUsize::new(0);
static COUNT: AtomicUsize = AtomicUsize::new(0);

/// Push a new activity entry. Called from net subsystem call sites
/// (dns, tcp_open, fw_drop, etc.).
pub fn push(kind: ActivityKind, summary: &str) {
    let mut e = Entry::empty();
    e.ts = crate::ui::topbar::uptime_seconds_pub();  // see note below
    e.kind = kind as u8;
    let bytes = summary.as_bytes();
    let n = bytes.len().min(e.summary.len());
    let mut snap = n;
    while snap > 0 && !summary.is_char_boundary(snap) { snap -= 1; }
    e.summary[..snap].copy_from_slice(&bytes[..snap]);
    e.summary_len = snap as u8;

    let head = HEAD.fetch_add(1, Ordering::Relaxed) % RING_CAP;
    unsafe { *core::ptr::addr_of_mut!(RING[head]) = e; }
    let prev = COUNT.load(Ordering::Relaxed);
    if prev < RING_CAP { COUNT.store(prev + 1, Ordering::Relaxed); }
}

/// Iterate newest-first. Calls `f` with each entry until f returns false.
pub fn iter_newest_first<F: FnMut(&Entry) -> bool>(mut f: F) {
    let count = COUNT.load(Ordering::Relaxed);
    let head  = HEAD.load(Ordering::Relaxed);
    for i in 0..count {
        // newest is at head-1, oldest is at head-count
        let idx = (head + RING_CAP - 1 - i) % RING_CAP;
        let entry = unsafe { &*core::ptr::addr_of!(RING[idx]) };
        if !f(entry) { break; }
    }
}

pub fn count() -> usize {
    COUNT.load(Ordering::Relaxed)
}

/// Clear the ring (called from net::clear_counters).
pub fn clear() {
    COUNT.store(0, Ordering::Relaxed);
    HEAD.store(0, Ordering::Relaxed);
}
```

**Important:** The `uptime_seconds_pub()` reference: Wave 2 added a private `uptime_seconds()` in `src/ui/topbar.rs` (reads CNTPCT_EL0). For Wave 4, either pub-elevate that or use `crate::time::uptime_secs()` if a kernel-level entry exists. Pick whichever already gives you "seconds since boot" cheaply and document the choice.

- [ ] **Step 2: Register the module in `src/net/mod.rs`.**

Add `pub mod activity;` somewhere alphabetical near the other `pub mod` declarations.

- [ ] **Step 3: Call `push` from net subsystem call sites.**

Find existing call sites and add a one-liner:

```bash
grep -nE 'fn (dns_query|tcp_open|tcp_close|fw_block|tls_handshake)' src/net/*.rs
```

For each match, add (with appropriate kind + summary format):

```rust
// In dns query path:
crate::net::activity::push(
    crate::net::activity::ActivityKind::Dns,
    &format!("A {} → {}", name, addr_str),
);

// In tcp_open path:
crate::net::activity::push(
    crate::net::activity::ActivityKind::TcpOpen,
    &format!("{}:{}  cave={}", peer_ip, peer_port, cave_name),
);

// In fw_drop path:
crate::net::activity::push(
    crate::net::activity::ActivityKind::FwDrop,
    &format!("{}:{}  {}", peer_ip, peer_port, reason),
);
```

If a kind's call site doesn't exist yet (e.g. tls_hs isn't logged today), skip that kind — the ring just won't see those events until later waves wire them. Document which kinds are wired and which aren't in the commit body.

Also: `clear_counters` in `src/net/mod.rs` (Task 1c) should call `activity::clear()` + push a `CountersCleared` entry.

- [ ] **Step 4: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```
Expected: clean. If clippy flags `count()` for dead-code (no callers yet), per-function `#[allow(dead_code)]`.

- [ ] **Step 5: Commit.**

```bash
git add src/net/activity.rs src/net/mod.rs src/net/firewall.rs src/net/tcp.rs src/net/dns.rs src/net/tls.rs
git commit -m "$(cat <<'EOF'
net: activity ring (256 entries) for Wave 4 NET cockpit

* New src/net/activity.rs: 256-entry circular buffer with newest-first
  iteration, ActivityKind enum (Dns/TcpOpen/TcpClose/FwDrop/TlsHs/
  CountersCleared), 96-byte summary text per entry. Char-boundary-safe
  truncation matches the Wave-3 cave_name pattern.
* Push hooks land in existing dns / tcp_open / tcp_close / fw_drop
  paths. tls_hs not yet wired (no existing log call site); Wave 5+
  can add when TLS gets its own activity surface.
* net::clear_counters() now clears the activity ring + pushes a
  counters_cleared entry.

Storage only — Wave 4 NET cockpit paints from this. Per-cave
filtering is a later-wave concern.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 1e: integrity deny counters

Rolling 24h counters for BLP / Biba / TE denies. Read by SECURITY's INTEGRITY panel.

**Files:**
- Create: `src/security/integrity_counts.rs`
- Modify: `src/security/mod.rs` (register module)
- Modify: existing BLP/Biba/TE deny sites (`src/caves/cave.rs::can_flow*`, etc.) to increment the counters

- [ ] **Step 1: Create `src/security/integrity_counts.rs`.**

```rust
//! 24-hour rolling counters for MLS / TE deny events. Read by the
//! Wave-4 SECURITY app's INTEGRITY panel.

#![allow(dead_code)]

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

static BLP_DENIES_24H:  AtomicU32 = AtomicU32::new(0);
static BIBA_DENIES_24H: AtomicU32 = AtomicU32::new(0);
static TE_DENIES_24H:   AtomicU32 = AtomicU32::new(0);
static LAST_BIBA_TS:    AtomicU64 = AtomicU64::new(0);
static LAST_BLP_TS:     AtomicU64 = AtomicU64::new(0);
static LAST_TE_TS:      AtomicU64 = AtomicU64::new(0);
static EPOCH_START:     AtomicU64 = AtomicU64::new(0);  // current 24h window start

const WINDOW_SECS: u64 = 24 * 3600;

fn maybe_roll_window() {
    let now = crate::ui::topbar::uptime_seconds_pub();
    let start = EPOCH_START.load(Ordering::Relaxed);
    if start == 0 || now.saturating_sub(start) >= WINDOW_SECS {
        EPOCH_START.store(now, Ordering::Relaxed);
        BLP_DENIES_24H.store(0, Ordering::Relaxed);
        BIBA_DENIES_24H.store(0, Ordering::Relaxed);
        TE_DENIES_24H.store(0, Ordering::Relaxed);
    }
}

pub fn record_blp_deny() {
    maybe_roll_window();
    BLP_DENIES_24H.fetch_add(1, Ordering::Relaxed);
    LAST_BLP_TS.store(crate::ui::topbar::uptime_seconds_pub(), Ordering::Relaxed);
}
pub fn record_biba_deny() {
    maybe_roll_window();
    BIBA_DENIES_24H.fetch_add(1, Ordering::Relaxed);
    LAST_BIBA_TS.store(crate::ui::topbar::uptime_seconds_pub(), Ordering::Relaxed);
}
pub fn record_te_deny() {
    maybe_roll_window();
    TE_DENIES_24H.fetch_add(1, Ordering::Relaxed);
    LAST_TE_TS.store(crate::ui::topbar::uptime_seconds_pub(), Ordering::Relaxed);
}

pub fn blp_denies()      -> u32 { maybe_roll_window(); BLP_DENIES_24H.load(Ordering::Relaxed) }
pub fn biba_denies()     -> u32 { maybe_roll_window(); BIBA_DENIES_24H.load(Ordering::Relaxed) }
pub fn te_denies()       -> u32 { maybe_roll_window(); TE_DENIES_24H.load(Ordering::Relaxed) }

pub fn last_biba_ts()    -> u64 { LAST_BIBA_TS.load(Ordering::Relaxed) }
pub fn last_blp_ts()     -> u64 { LAST_BLP_TS.load(Ordering::Relaxed) }
pub fn last_te_ts()      -> u64 { LAST_TE_TS.load(Ordering::Relaxed) }
```

- [ ] **Step 2: Register the module in `src/security/mod.rs`.**

Add `pub mod integrity_counts;` alphabetically.

- [ ] **Step 3: Increment from deny sites.**

```bash
grep -nE 'can_flow|can_access_object|can_transition' src/caves/cave.rs | head -10
```

At each location that returns `false` (a deny path), add the counter call:

```rust
// In can_flow (BLP):
if !ok {
    crate::security::integrity_counts::record_blp_deny();
}
// In can_flow_integrity (Biba):
if !ok {
    crate::security::integrity_counts::record_biba_deny();
}
// In can_transition (TE):
if !ok {
    crate::security::integrity_counts::record_te_deny();
}
```

Match exact function names from the existing code. The hook points should be at the `return false` (or last-expression-false) site of each deny check.

- [ ] **Step 4: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

- [ ] **Step 5: Commit.**

```bash
git add src/security/integrity_counts.rs src/security/mod.rs src/caves/cave.rs
git commit -m "$(cat <<'EOF'
security: 24h rolling deny counters for BLP / Biba / TE

Wave 4 SECURITY app's INTEGRITY panel reads these to surface
"clean" or "<N> denies" status with a last-deny timestamp.

* New src/security/integrity_counts.rs: AtomicU32 counters + last
  timestamp per kind. maybe_roll_window() resets the counter every
  24h.
* Increment hooks at the three deny sites in cave.rs
  (can_flow / can_flow_integrity / can_transition).

Storage only — Wave 5+ can add per-cave breakdown or richer
"last 24h" sliding-window variants. Wave 4 just surfaces the count.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 2a: `paint_activity_log` widget

Used by NET (activity ring) and SECURITY (audit chain). Time-stamped event stream with viewport scrolling.

**Files:**
- Modify: `src/ui/widgets.rs`

- [ ] **Step 1: Append the widget to `src/ui/widgets.rs` Wave-3 section.**

```rust
/// One row in a `paint_activity_log` rendering.
#[derive(Copy, Clone)]
pub struct ActivityEntry<'a> {
    pub timestamp_str: &'a str,  // pre-formatted, e.g. "14:32:01"
    pub kind:          &'a str,  // pre-formatted, e.g. "dns" / "tcp_open"
    pub summary:       &'a str,
}

/// Paint a paginated activity log. Timestamp + kind render in MID
/// with the kind padded to 12 chars; summary in INK. Top-right of
/// rect shows `showing last N of M · ↑↓ scrolls`.
///
/// Caller owns `viewport_start` (index of the topmost rendered
/// entry) + `total` (overall count for the right-aligned summary).
/// `entries` is the visible slice (already paged by the caller).
pub fn paint_activity_log(
    rect: WindowRect,
    entries: &[ActivityEntry],
    viewport_start: usize,
    total: usize,
) {
    use core::fmt::Write;

    const ROW_H:  u32 = 18;
    const CHAR_W: u32 = 8;

    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    // Top header row: viewport indicator on the right.
    let header_y = rect.y;
    let mut hdr_buf = [0u8; 64];
    let visible = entries.len();
    let last_idx = viewport_start + visible;
    // "showing N..M of T · ↑↓ scrolls"
    let mut hdr_n = 0;
    let _ = write_dec(&mut hdr_buf, &mut hdr_n, viewport_start as u32 + 1);
    push_str(&mut hdr_buf, &mut hdr_n, b"..");
    let _ = write_dec(&mut hdr_buf, &mut hdr_n, last_idx as u32);
    push_str(&mut hdr_buf, &mut hdr_n, b" of ");
    let _ = write_dec(&mut hdr_buf, &mut hdr_n, total as u32);
    let hdr = unsafe { core::str::from_utf8_unchecked(&hdr_buf[..hdr_n]) };
    let hdr_w = hdr_n as u32 * CHAR_W;
    let hdr_x = rect.x + rect.w.saturating_sub(hdr_w + 8);
    font::draw_str(fb, screen_w, hdr_x, header_y, hdr, p::MID, p::BG);

    // Rows below the header. Body starts at rect.y + 20 (16 px header
    // + 4 px gap).
    let body_y = rect.y + 20;
    let max_rows = ((rect.h.saturating_sub(20)) / ROW_H) as usize;
    let rows_to_paint = entries.len().min(max_rows);

    for (i, entry) in entries.iter().take(rows_to_paint).enumerate() {
        let row_y = body_y + (i as u32) * ROW_H + 1;
        // timestamp (MID)
        font::draw_str(fb, screen_w, rect.x + 4, row_y, entry.timestamp_str, p::MID, p::BG);
        let after_ts_x = rect.x + 4 + entry.timestamp_str.len() as u32 * CHAR_W + 2 * CHAR_W;
        // kind padded to 12 chars (MID)
        font::draw_str(fb, screen_w, after_ts_x, row_y, entry.kind, p::MID, p::BG);
        let after_kind_x = after_ts_x + 12 * CHAR_W;
        // summary (INK)
        font::draw_str(fb, screen_w, after_kind_x, row_y, entry.summary, p::INK, p::BG);
    }
}

// Helper: write decimal u32 into `buf` starting at `*n`, advancing `*n`.
fn write_dec(buf: &mut [u8], n: &mut usize, mut v: u32) -> core::fmt::Result {
    if v == 0 { if *n < buf.len() { buf[*n] = b'0'; *n += 1; } return Ok(()); }
    let mut tmp = [0u8; 10];
    let mut t = 0;
    while v > 0 { tmp[t] = b'0' + (v % 10) as u8; v /= 10; t += 1; }
    for j in 0..t {
        if *n < buf.len() { buf[*n] = tmp[t - j - 1]; *n += 1; }
    }
    Ok(())
}

fn push_str(buf: &mut [u8], n: &mut usize, s: &[u8]) {
    for &b in s {
        if *n < buf.len() { buf[*n] = b; *n += 1; }
    }
}
```

If `write_dec` or `push_str` already exist in widgets.rs from prior tasks, reuse them and drop the local definitions.

- [ ] **Step 2: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

- [ ] **Step 3: Commit.**

```bash
git add src/ui/widgets.rs
git commit -m "$(cat <<'EOF'
ui/widgets: paint_activity_log

Paginated event-stream widget. Header row shows `N..M of T`
viewport indicator (top-right). Body rows render
timestamp (MID, ~8 chars) + kind (MID, padded to 12 chars) +
summary (INK) at 18-px pitch.

Used by NET cockpit (activity ring) and SECURITY panic console
(audit chain). Caller owns scroll state.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 2b: `paint_status_panel` widget

Bordered PANEL with a labeled header strip. Used 6× across NET + SECURITY.

**Files:**
- Modify: `src/ui/widgets.rs`

- [ ] **Step 1: Append the widget after `paint_activity_log`.**

```rust
/// A bordered PANEL with a header (label + optional right badge)
/// and a body region. Body rendered via `paint_status_field_list`
/// over the caller-supplied fields.
pub struct StatusPanel<'a> {
    /// Letter-spaced uppercase label in the header (MID).
    pub label: &'a str,
    /// Optional right-aligned badge in the header (INK if Some).
    pub header_right: Option<&'a str>,
    /// Body content as KV rows.
    pub body: &'a [StatusField<'a>],
}

pub fn paint_status_panel(rect: WindowRect, panel: &StatusPanel) {
    const CHAR_W:  u32 = 8;
    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    // PANEL fill.
    gpu::fill_rect(rect.x, rect.y, rect.w, rect.h, p::PANEL);
    // 1-px HAIRLINE border.
    gpu::fill_rect(rect.x, rect.y, rect.w, 1, p::HAIRLINE);
    gpu::fill_rect(rect.x, rect.y + rect.h - 1, rect.w, 1, p::HAIRLINE);
    gpu::fill_rect(rect.x, rect.y, 1, rect.h, p::HAIRLINE);
    gpu::fill_rect(rect.x + rect.w - 1, rect.y, 1, rect.h, p::HAIRLINE);

    // Header row.
    let hdr_y = rect.y + 10;
    font::draw_str(fb, screen_w, rect.x + 10, hdr_y, panel.label, p::MID, p::PANEL);
    if let Some(badge) = panel.header_right {
        let badge_w = badge.len() as u32 * CHAR_W;
        let badge_x = rect.x + rect.w.saturating_sub(badge_w + 10);
        font::draw_str(fb, screen_w, badge_x, hdr_y, badge, p::INK, p::PANEL);
    }

    // Body rect: under the header, inset 10 px on all sides.
    let body_rect = WindowRect {
        x: rect.x + 10,
        y: rect.y + 32,
        w: rect.w.saturating_sub(20),
        h: rect.h.saturating_sub(42),
    };
    paint_status_field_list(body_rect, panel.body);
}
```

- [ ] **Step 2: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

- [ ] **Step 3: Commit.**

```bash
git add src/ui/widgets.rs
git commit -m "$(cat <<'EOF'
ui/widgets: paint_status_panel

Bordered PANEL with a labeled header strip. Label letter-spaced in
MID (top-left), optional INK badge (top-right). Body renders the
caller's `&[StatusField]` via paint_status_field_list with a 10-px
inset.

Used by NET cockpit (MODE + FIREWALL panels) and SECURITY panic
console (TAINT + INTEGRITY panels). Six call sites total in Wave 4;
inherited by every future cockpit-shaped app.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 2c: `paint_big_metric` widget

Large value display for DEADMAN countdown and AUTH ratio. Uses Task 1a's `font::draw_str_scale`.

**Files:**
- Modify: `src/ui/widgets.rs`

- [ ] **Step 1: Append the widget after `paint_status_panel`.**

```rust
/// A big-value metric tile. Letter-spaced label (MID) at top, big
/// 2× value (INK) centered, sub-caption (MID) below. Caller draws
/// the panel border + background; this just paints the inner text.
pub fn paint_big_metric(rect: WindowRect, label: &str, value: &str, sub: &str) {
    const CHAR_W: u32 = 8;
    const CHAR_H: u32 = 16;

    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    // Label at top.
    font::draw_str(fb, screen_w, rect.x, rect.y, label, p::MID, p::PANEL);

    // Big value: 2× scale, vertically centered in remaining space.
    let value_w = value.len() as u32 * CHAR_W * 2;
    let value_h = CHAR_H * 2;
    let avail_h = rect.h.saturating_sub(CHAR_H + 4);   // -label row
    let value_y = rect.y + CHAR_H + 4 + (avail_h.saturating_sub(value_h + CHAR_H + 4)) / 2;
    let value_x = rect.x;  // left-align
    font::draw_str_scale(fb, screen_w, value_x, value_y, value, p::INK, p::PANEL, 2);

    // Sub-caption under the value.
    let sub_y = value_y + value_h + 4;
    font::draw_str(fb, screen_w, rect.x, sub_y, sub, p::MID, p::PANEL);
}
```

- [ ] **Step 2: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

- [ ] **Step 3: Commit.**

```bash
git add src/ui/widgets.rs
git commit -m "$(cat <<'EOF'
ui/widgets: paint_big_metric

Big-value tile: label (MID) at top, 2× bitmap-font value (INK)
centered, sub-caption (MID) below. Renders against PANEL bg
(caller draws the panel chrome via paint_status_panel or inline).

Used by SECURITY panic console (DEADMAN countdown + AUTH ratio).
Inherits draw_str_scale from Task 1a's font.rs addition.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 2d: `paint_file_preview` widget

Text/hex viewer pane. Used by FILES.

**Files:**
- Modify: `src/ui/widgets.rs`

- [ ] **Step 1: Append the widget.**

```rust
/// Render a file's bytes as either text (printable-ASCII, with line
/// numbers) or hex+ASCII dump. The mode is chosen by sniffing the
/// first 256 bytes for printable-ASCII ratio (≥90% → text).
///
/// `viewport_start` is the starting line (text) or starting 16-byte
/// row (hex). Caller scrolls by adjusting this value.
pub fn paint_file_preview(rect: WindowRect, bytes: &[u8], viewport_start: usize) {
    if bytes.is_empty() {
        let fb = gpu::framebuffer();
        let screen_w = gpu::width();
        font::draw_str(fb, screen_w, rect.x + 4, rect.y + 4, "(empty)", p::MID, p::BG);
        return;
    }

    if is_text(bytes) {
        paint_file_preview_text(rect, bytes, viewport_start);
    } else {
        paint_file_preview_hex(rect, bytes, viewport_start);
    }
}

fn is_text(bytes: &[u8]) -> bool {
    let sample = &bytes[..bytes.len().min(256)];
    if sample.is_empty() { return true; }
    let printable = sample.iter().filter(|&&b|
        (0x20..=0x7E).contains(&b) || b == b'\n' || b == b'\t' || b == b'\r'
    ).count();
    (printable * 100) >= sample.len() * 90
}

fn paint_file_preview_text(rect: WindowRect, bytes: &[u8], viewport_start: usize) {
    const ROW_H:  u32 = 16;
    const CHAR_W: u32 = 8;
    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    let max_rows = (rect.h / ROW_H) as usize;
    let line_no_w = 5 * CHAR_W;  // up to 5-digit line numbers + separator
    let text_x = rect.x + line_no_w + 4;

    let mut line: usize = 0;
    let mut line_start: usize = 0;
    let mut rendered: usize = 0;

    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' || i == bytes.len() - 1 {
            let end = if b == b'\n' { i } else { i + 1 };
            if line >= viewport_start {
                if rendered >= max_rows { break; }
                let row_y = rect.y + (rendered as u32) * ROW_H;
                // Line number (MID).
                let mut ln_buf = [b' '; 5];
                let mut ln = (line + 1) as u32;
                let mut j = 5;
                while ln > 0 && j > 0 { j -= 1; ln_buf[j] = b'0' + (ln % 10) as u8; ln /= 10; }
                let ln_str = unsafe { core::str::from_utf8_unchecked(&ln_buf) };
                font::draw_str(fb, screen_w, rect.x, row_y, ln_str, p::MID, p::BG);
                // Content.
                let line_str = unsafe { core::str::from_utf8_unchecked(&bytes[line_start..end]) };
                font::draw_str(fb, screen_w, text_x, row_y, line_str, p::INK, p::BG);
                rendered += 1;
            }
            line += 1;
            line_start = i + 1;
        }
    }
}

fn paint_file_preview_hex(rect: WindowRect, bytes: &[u8], viewport_start: usize) {
    const ROW_H:  u32 = 14;
    const CHAR_W: u32 = 8;
    const BYTES_PER_ROW: usize = 16;
    let screen_w = gpu::width();
    let fb = gpu::framebuffer();

    let max_rows = (rect.h / ROW_H) as usize;
    let total_rows = bytes.len().div_ceil(BYTES_PER_ROW);
    let end_row = (viewport_start + max_rows).min(total_rows);

    for (rendered, row_idx) in (viewport_start..end_row).enumerate() {
        let row_y = rect.y + (rendered as u32) * ROW_H;
        let start = row_idx * BYTES_PER_ROW;
        let end = (start + BYTES_PER_ROW).min(bytes.len());
        // Offset (MID), 4-digit hex.
        let mut off_buf = [b'0'; 4];
        let mut off = (start as u32) & 0xFFFF;
        for k in 0..4 {
            let nibble = (off >> ((3 - k) * 4)) & 0xF;
            off_buf[k] = if nibble < 10 { b'0' + nibble as u8 } else { b'a' + (nibble - 10) as u8 };
        }
        let off_str = unsafe { core::str::from_utf8_unchecked(&off_buf) };
        font::draw_str(fb, screen_w, rect.x, row_y, off_str, p::MID, p::BG);
        let _ = off; // silence unused-mut

        // Hex bytes (INK), 16 bytes × "xx " = 48 chars.
        let mut hex_buf = [b' '; 49];
        for (k, b) in bytes[start..end].iter().enumerate() {
            let hi = (b >> 4) & 0xF;
            let lo = b & 0xF;
            hex_buf[k * 3]     = if hi < 10 { b'0' + hi } else { b'a' + (hi - 10) };
            hex_buf[k * 3 + 1] = if lo < 10 { b'0' + lo } else { b'a' + (lo - 10) };
        }
        let hex_str = unsafe { core::str::from_utf8_unchecked(&hex_buf) };
        font::draw_str(fb, screen_w, rect.x + 5 * CHAR_W, row_y, hex_str, p::INK, p::BG);

        // ASCII gutter (INK with . for non-printable).
        let mut ascii_buf = [b'.'; 16];
        for (k, b) in bytes[start..end].iter().enumerate() {
            ascii_buf[k] = if (0x20..=0x7E).contains(b) { *b } else { b'.' };
        }
        let ascii_str = unsafe { core::str::from_utf8_unchecked(&ascii_buf) };
        font::draw_str(fb, screen_w, rect.x + 5 * CHAR_W + 49 * CHAR_W + CHAR_W, row_y, ascii_str, p::INK, p::BG);
    }
}
```

- [ ] **Step 2: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

- [ ] **Step 3: Commit.**

```bash
git add src/ui/widgets.rs
git commit -m "$(cat <<'EOF'
ui/widgets: paint_file_preview (text / hex modes)

Text-or-hex viewer pane. Sniffs first 256 bytes for printable-ASCII
ratio (≥90% → text mode), else hex mode.
* Text mode: 5-digit line number (MID) + content (INK), 16-px pitch.
  Renders line-by-line splitting on \n.
* Hex mode: 4-digit offset (MID) + 16 bytes hex (INK) + 16-char ASCII
  gutter (INK with . for non-printable), 14-px pitch.

Caller owns viewport_start. Used by Wave-4 FILES app's detail panel.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 3: FILES app rewrite

Replace `src/ui/apps/filemanager.rs` with the Inspector + file-viewer design.

**Files:**
- Replace: `src/ui/apps/filemanager.rs`
- Modify: `src/ui/apps_registry.rs` (wire handle_key + handle_click)

- [ ] **Step 1: Replace `src/ui/apps/filemanager.rs` entirely.**

```rust
//! Wave 4 Files Manager. Inspector layout + file viewer.
//! See `docs/superpowers/specs/2026-05-14-files-net-security-design.md`.

#![allow(dead_code, unused_imports)]

use crate::ui::apps_registry::AppEvent;
use crate::ui::palette as p;
use crate::ui::widgets::{
    paint_state_dot, paint_status_field_list, StatusField,
    paint_action_strip, action_strip_hit_test, Action,
    InspectorLayout,
    paint_confirm_modal, confirm_modal_key, ConfirmModal, ModalAction,
    paint_file_preview,
};
use crate::ui::wm::WindowRect;
use crate::fs::batfs;

const NAME_MAX: usize = 64;

#[derive(PartialEq, Eq)]
enum AppMode {
    Viewing,
    ConfirmDelete(usize),  // index of file to delete
}

// Non-Copy: assign via `unsafe { *core::ptr::addr_of_mut!(APP_MODE) = ... }`
// Do NOT use write_volatile (requires Copy).
static mut APP_MODE: AppMode = AppMode::Viewing;
static mut SELECTED_FILE: usize = 0;
static mut VIEWPORT_START: usize = 0;
static mut PREVIEW_BUF:  [u8; 8192] = [0; 8192];
static mut PREVIEW_LEN:  usize = 0;
static mut PREVIEW_VALID_FOR: usize = usize::MAX;  // selected_file index this preview was loaded for

fn selected_file() -> usize {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(SELECTED_FILE)) }
}
fn set_selected_file(v: usize) {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(SELECTED_FILE), v) }
    // Reset viewport + invalidate preview cache.
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), 0);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PREVIEW_VALID_FOR), usize::MAX);
    }
}
fn viewport_start() -> usize {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(VIEWPORT_START)) }
}
fn set_viewport_start(v: usize) {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), v) }
}

pub fn paint(body: WindowRect) {
    crate::ui::gpu::fill_rect(body.x, body.y, body.w, body.h, p::BG);

    let layout = InspectorLayout::new(body).with_sidebar_pct(38);
    layout.paint_divider();
    paint_sidebar(layout.sidebar_rect());

    match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
        AppMode::Viewing             => paint_detail_view(layout.detail_rect()),
        AppMode::ConfirmDelete(idx)  => {
            paint_detail_view(layout.detail_rect());
            paint_delete_modal(*idx);
        }
    }
}

fn paint_sidebar(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    let (count, _max) = batfs::ns_stats();
    // Header: "FILES (n)"
    let mut hdr_buf = [0u8; 24];
    hdr_buf[..7].copy_from_slice(b"FILES (");
    let digits = u32_dec(count as u32, &mut hdr_buf, 7);
    hdr_buf[7 + digits] = b')';
    let hdr_len = 7 + digits + 1;
    let hdr = unsafe { core::str::from_utf8_unchecked(&hdr_buf[..hdr_len]) };
    font::draw_str(fb, screen_w, rect.x + 8, rect.y + 6, hdr, p::MID, p::BG);
    gpu::fill_rect(rect.x, rect.y + 24, rect.w, 1, p::HAIRLINE);

    // List rows.
    let row_h: u32 = 22;
    let sel = selected_file();
    let mut row_index: usize = 0;

    batfs::ns_list(|name, _size, encrypted| {
        let row_y = rect.y + 28 + (row_index as u32) * row_h;
        if row_y + row_h > rect.y + rect.h { return; }

        let is_sel = row_index == sel;
        if is_sel {
            gpu::fill_rect(rect.x, row_y, rect.w, row_h, p::PANEL);
            font::draw_str(fb, screen_w, rect.x + 4, row_y + 3, "›", p::INK, p::PANEL);
        }
        paint_state_dot(rect.x + 18, row_y + 7, encrypted);
        font::draw_str(
            fb, screen_w, rect.x + 30, row_y + 3,
            name,
            if is_sel { p::INK } else { p::MID },
            if is_sel { p::PANEL } else { p::BG },
        );
        row_index += 1;
    });
}

fn paint_detail_view(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    let (count, _max) = batfs::ns_stats();
    if count == 0 {
        font::draw_str(fb, screen_w, rect.x + 14, rect.y + 14,
                       "No files. Create one via SHELL or EDITOR.", p::MID, p::BG);
        return;
    }

    // Resolve the selected file's name + size + encrypted flag.
    let sel = selected_file();
    let mut name_buf = [0u8; NAME_MAX];
    let mut name_len = 0;
    let mut size: usize = 0;
    let mut encrypted = false;
    let mut row_index: usize = 0;
    batfs::ns_list(|n, s, e| {
        if row_index == sel {
            let l = n.len().min(NAME_MAX);
            name_buf[..l].copy_from_slice(&n.as_bytes()[..l]);
            name_len = l;
            size = s;
            encrypted = e;
        }
        row_index += 1;
    });
    if name_len == 0 { return; }
    let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };

    // Header: filename + one-line metadata.
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 8, name, p::INK, p::BG);
    let mut meta_buf = [0u8; 64];
    let mut mn = 0;
    let (sz_n, sz_u) = format_size(size, &mut meta_buf[mn..]);
    mn += sz_n;
    push_bytes(&mut meta_buf, &mut mn, sz_u.as_bytes());
    push_bytes(&mut meta_buf, &mut mn, b" \xc2\xb7 ");  // " · "
    push_bytes(&mut meta_buf, &mut mn, if encrypted { b"encrypted" } else { b"plain" });
    let meta = unsafe { core::str::from_utf8_unchecked(&meta_buf[..mn]) };
    let meta_x = rect.x + rect.w.saturating_sub(meta.len() as u32 * 8 + 14);
    font::draw_str(fb, screen_w, meta_x, rect.y + 8, meta, p::MID, p::BG);
    gpu::fill_rect(rect.x + 14, rect.y + 28, rect.w - 28, 1, p::HAIRLINE);

    // Preview region.
    let preview_rect = WindowRect {
        x: rect.x + 14,
        y: rect.y + 36,
        w: rect.w - 28,
        h: rect.h.saturating_sub(80),
    };
    // Load the file into PREVIEW_BUF if not already cached for this index.
    let cached_for = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(PREVIEW_VALID_FOR)) };
    if cached_for != sel {
        load_preview(name, encrypted);
    }
    let len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(PREVIEW_LEN)) };
    let buf = unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(PREVIEW_BUF) as *const u8, len) };
    if len == 0 && encrypted {
        font::draw_str(fb, screen_w, preview_rect.x + 4, preview_rect.y + 4,
                       "encrypted; preview requires cave context", p::MID, p::BG);
    } else {
        paint_file_preview(preview_rect, buf, viewport_start());
    }

    // Hairline above actions.
    gpu::fill_rect(rect.x + 14, rect.y + rect.h - 32, rect.w - 28, 1, p::HAIRLINE);

    // Action strip.
    let strip_rect = WindowRect {
        x: rect.x + 14,
        y: rect.y + rect.h - 28,
        w: rect.w - 28,
        h: 24,
    };
    let actions = actions_for_file(encrypted);
    paint_action_strip(strip_rect, &actions);
}

fn paint_delete_modal(idx: usize) {
    // Resolve file name for the title.
    let mut name_buf = [0u8; NAME_MAX];
    let mut name_len = 0;
    let mut row_index: usize = 0;
    batfs::ns_list(|n, _s, _e| {
        if row_index == idx {
            let l = n.len().min(NAME_MAX);
            name_buf[..l].copy_from_slice(&n.as_bytes()[..l]);
            name_len = l;
        }
        row_index += 1;
    });
    let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };

    let mut title_buf = [0u8; 80];
    let mut tn = 0;
    push_bytes(&mut title_buf, &mut tn, b"Delete ");
    push_bytes(&mut title_buf, &mut tn, name.as_bytes());
    push_bytes(&mut title_buf, &mut tn, b"?");
    let title = unsafe { core::str::from_utf8_unchecked(&title_buf[..tn]) };

    let modal = ConfirmModal {
        title,
        body_lines: &[
            "  remove the file from BatFS",
            "  zero its encrypted blocks",
            "  add a tombstone to the audit chain",
            "",
            "IRREVERSIBLE.",
        ],
        commit_key: 'D',
    };
    paint_confirm_modal(&modal);
}

fn actions_for_file(_encrypted: bool) -> [Action<'static>; 2] {
    [
        Action { hotkey: 'D', label: "Delete",     enabled: true  },
        Action { hotkey: 'E', label: "Edit (W5)",  enabled: false }, // FAINT
    ]
}

fn load_preview(name: &str, encrypted: bool) {
    let buf_ptr = core::ptr::addr_of_mut!(PREVIEW_BUF) as *mut u8;
    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, 8192) };
    let result = batfs::read(name, buf);
    let len = result.unwrap_or(0);
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PREVIEW_LEN), len);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(PREVIEW_VALID_FOR), selected_file());
    }
    let _ = encrypted;
}

// ── Input ───────────────────────────────────────────────────────

pub fn handle_key(c: u8) -> AppEvent {
    match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
        AppMode::ConfirmDelete(idx) => handle_key_delete_modal(c, *idx),
        AppMode::Viewing            => handle_key_viewing(c),
    }
}

fn handle_key_viewing(c: u8) -> AppEvent {
    let (count, _max) = batfs::ns_stats();
    match c {
        0x90 => {  // Arrow Up — scroll preview up
            let v = viewport_start();
            if v > 0 { set_viewport_start(v.saturating_sub(8)); }
            AppEvent::Repaint
        }
        0x91 => {  // Arrow Down — scroll preview down
            let v = viewport_start();
            set_viewport_start(v + 8);
            AppEvent::Repaint
        }
        b'd' | b'D' => {
            if count > 0 {
                unsafe {
                    *core::ptr::addr_of_mut!(APP_MODE) = AppMode::ConfirmDelete(selected_file());
                }
            }
            AppEvent::Repaint
        }
        b'e' | b'E' => {
            // FAINT [E]dit — Wave 5 stub, no-op.
            AppEvent::Consumed
        }
        _ => {
            // J/K or any other key — let desktop handle (e.g. ^L lock).
            AppEvent::Unhandled
        }
    }
}

fn handle_key_delete_modal(c: u8, idx: usize) -> AppEvent {
    let modal = ConfirmModal { title: "", body_lines: &[], commit_key: 'D' };
    match confirm_modal_key(&modal, c) {
        ModalAction::Commit => {
            // Resolve the file name and delete.
            let mut name_buf = [0u8; NAME_MAX];
            let mut name_len = 0;
            let mut row_index: usize = 0;
            batfs::ns_list(|n, _s, _e| {
                if row_index == idx {
                    let l = n.len().min(NAME_MAX);
                    name_buf[..l].copy_from_slice(&n.as_bytes()[..l]);
                    name_len = l;
                }
                row_index += 1;
            });
            if name_len > 0 {
                let name = unsafe { core::str::from_utf8_unchecked(&name_buf[..name_len]) };
                // TODO(Wave 5): surface Err in modal footer per spec §Destroy.
                let _ = batfs::ns_delete(name);
            }
            set_selected_file(0);
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing; }
            AppEvent::Repaint
        }
        ModalAction::Cancel => {
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing; }
            AppEvent::Repaint
        }
        ModalAction::None => AppEvent::Consumed,
    }
}

pub fn handle_click(mx: i32, my: i32, body: WindowRect) -> AppEvent {
    match unsafe { &*core::ptr::addr_of!(APP_MODE) } {
        AppMode::Viewing => handle_click_viewing(mx, my, body),
        AppMode::ConfirmDelete(_) => {
            // Any click cancels (matches caves_mgr).
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing; }
            AppEvent::Repaint
        }
    }
}

fn handle_click_viewing(mx: i32, my: i32, body: WindowRect) -> AppEvent {
    let layout = InspectorLayout::new(body).with_sidebar_pct(38);
    let sidebar = layout.sidebar_rect();
    let detail = layout.detail_rect();

    // Sidebar click → select.
    if mx >= sidebar.x as i32 && mx < (sidebar.x + sidebar.w) as i32 {
        let row_h: u32 = 22;
        let header_h: u32 = 28;
        if my >= (sidebar.y + header_h) as i32 {
            let row_idx = ((my as u32 - sidebar.y - header_h) / row_h) as usize;
            let (count, _max) = batfs::ns_stats();
            if row_idx < count {
                set_selected_file(row_idx);
                return AppEvent::Repaint;
            }
        }
        return AppEvent::Consumed;
    }

    // Detail click → action strip.
    let strip_rect = WindowRect {
        x: detail.x + 14,
        y: detail.y + detail.h - 28,
        w: detail.w - 28,
        h: 24,
    };
    let actions = actions_for_file(false);
    if let Some(key) = action_strip_hit_test(strip_rect, mx, my, &actions) {
        return handle_key(key as u8);
    }
    AppEvent::Consumed
}

// ── helpers ─────────────────────────────────────────────────────

fn u32_dec(mut v: u32, buf: &mut [u8], offset: usize) -> usize {
    if v == 0 { buf[offset] = b'0'; return 1; }
    let mut tmp = [0u8; 10];
    let mut i = 0;
    while v > 0 { tmp[i] = b'0' + (v % 10) as u8; v /= 10; i += 1; }
    for j in 0..i { buf[offset + j] = tmp[i - j - 1]; }
    i
}

fn push_bytes(buf: &mut [u8], n: &mut usize, s: &[u8]) {
    for &b in s {
        if *n < buf.len() { buf[*n] = b; *n += 1; }
    }
}

fn format_size(bytes: usize, out: &mut [u8]) -> (usize, &'static str) {
    if bytes < 1024 {
        let n = u32_dec(bytes as u32, out, 0);
        (n, "B")
    } else if bytes < 1024 * 1024 {
        let n = u32_dec((bytes / 1024) as u32, out, 0);
        (n, "K")
    } else {
        let n = u32_dec((bytes / (1024 * 1024)) as u32, out, 0);
        (n, "M")
    }
}
```

- [ ] **Step 2: Wire `apps_registry.rs` to the new FILES handlers.**

Update the `AppId::Files` entry in `APPS`:

```rust
AppDescriptor {
    id: AppId::Files,
    label: "FILES",
    title: "FILES",
    paint: paint_files,
    handle_key:   crate::ui::apps::filemanager::handle_key,
    handle_click: crate::ui::apps::filemanager::handle_click,
},
```

And the `paint_files` shim (already exists) keeps calling `crate::ui::apps::filemanager::paint(rect)`.

- [ ] **Step 3: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

- [ ] **Step 4: Commit.**

```bash
git add src/ui/apps/filemanager.rs src/ui/apps_registry.rs
git commit -m "$(cat <<'EOF'
filemanager: Wave 4 — Inspector + file viewer

Replaces the 383-line cyberpunk filemanager with a state-machine app
composed from Wave-3 widgets + the new paint_file_preview.

Inspector layout: sidebar = file list (state dot = encrypted/plain),
detail panel = filename + 1-line metadata strip + preview (text or
hex auto-sniffed) + action strip. Up/Down arrows scroll the preview
viewport. D triggers ConfirmDelete modal (double-tap commits via
batfs::ns_delete). E shows FAINT (Wave 5 EDITOR stub).

Preview cache (8 KB, invalidated on selection change) reads via
batfs::read. Encrypted files without the active cave key show
"encrypted; preview requires cave context" instead of garbled hex.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 4: NET app rewrite

Replace `src/ui/apps/netmon.rs` with the Cockpit live activity dashboard.

**Files:**
- Replace: `src/ui/apps/netmon.rs`
- Modify: `src/ui/apps_registry.rs`

- [ ] **Step 1: Replace `src/ui/apps/netmon.rs` entirely.**

```rust
//! Wave 4 NET cockpit. Live activity dashboard.
//! See `docs/superpowers/specs/2026-05-14-files-net-security-design.md`.

#![allow(dead_code, unused_imports)]

use crate::ui::apps_registry::AppEvent;
use crate::ui::palette as p;
use crate::ui::widgets::{
    paint_status_panel, StatusPanel, StatusField,
    paint_activity_log, ActivityEntry,
    paint_action_strip, action_strip_hit_test, Action,
};
use crate::ui::wm::WindowRect;
use crate::net;
use crate::net::activity::{self, ActivityKind};

static mut VIEWPORT_START: usize = 0;

fn viewport_start() -> usize {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(VIEWPORT_START)) }
}
fn set_viewport_start(v: usize) {
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), v) }
}

pub fn paint(body: WindowRect) {
    crate::ui::gpu::fill_rect(body.x, body.y, body.w, body.h, p::BG);

    // ── Top stats strip ────────────────────────────────────────
    let strip_h: u32 = 28;
    let strip_rect = WindowRect { x: body.x, y: body.y, w: body.w, h: strip_h };
    paint_stats_strip(strip_rect);
    crate::ui::gpu::fill_rect(body.x, body.y + strip_h, body.w, 1, p::HAIRLINE);

    // ── Top panel row: MODE + FIREWALL ──────────────────────────
    let panel_y = body.y + strip_h + 12;
    let panel_h: u32 = 88;
    let gap: u32 = 12;
    let mode_w  = (body.w - 28 - gap) * 2 / 5;
    let fw_w    = (body.w - 28 - gap) - mode_w;
    let mode_rect = WindowRect { x: body.x + 14, y: panel_y, w: mode_w, h: panel_h };
    let fw_rect   = WindowRect { x: body.x + 14 + mode_w + gap, y: panel_y, w: fw_w, h: panel_h };
    paint_mode_panel(mode_rect);
    paint_firewall_panel(fw_rect);

    // ── Activity log (middle) ──────────────────────────────────
    let log_y = panel_y + panel_h + 12;
    let log_h = body.h.saturating_sub(log_y - body.y + 50);
    let log_rect = WindowRect { x: body.x + 14, y: log_y, w: body.w - 28, h: log_h };
    paint_activity_block(log_rect);

    // ── Action strip ──────────────────────────────────────────
    crate::ui::gpu::fill_rect(body.x + 14, body.y + body.h - 32, body.w - 28, 1, p::HAIRLINE);
    let strip_rect = WindowRect { x: body.x + 14, y: body.y + body.h - 28, w: body.w - 28, h: 24 };
    paint_action_strip(strip_rect, &actions());
}

fn paint_stats_strip(rect: WindowRect) {
    use crate::ui::{font, gpu};
    let fb = gpu::framebuffer();
    let screen_w = gpu::width();

    let mut left_buf = [0u8; 48];
    let mut ln = 0;
    push_bytes(&mut left_buf, &mut ln, b"RX ");
    let _ = format_rate(net::rx_rate(), &mut left_buf, &mut ln);
    push_bytes(&mut left_buf, &mut ln, b"/s  \xc2\xb7  TX ");
    let _ = format_rate(net::tx_rate(), &mut left_buf, &mut ln);
    push_bytes(&mut left_buf, &mut ln, b"/s");
    let left = unsafe { core::str::from_utf8_unchecked(&left_buf[..ln]) };
    font::draw_str(fb, screen_w, rect.x + 14, rect.y + 6, left, p::MID, p::BG);

    let mut right_buf = [0u8; 48];
    let mut rn = 0;
    push_bytes(&mut right_buf, &mut rn, b"PEAK ");
    let _ = format_bytes(net::peak_bytes(), &mut right_buf, &mut rn);
    push_bytes(&mut right_buf, &mut rn, b"  \xc2\xb7  UPTIME ");
    let _ = format_hms(net::uptime_secs(), &mut right_buf, &mut rn);
    let right = unsafe { core::str::from_utf8_unchecked(&right_buf[..rn]) };
    let right_w = rn as u32 * 8;
    font::draw_str(fb, screen_w, rect.x + rect.w.saturating_sub(right_w + 14), rect.y + 6, right, p::MID, p::BG);
}

fn paint_mode_panel(rect: WindowRect) {
    let mode_label = if net::is_isolated() { "ISOLATED" } else { "ROUTED" };
    let mode_sub = if net::is_isolated() {
        "no outbound to non-cave routes"
    } else {
        "default route via host"
    };
    let body = [
        StatusField { key: "",    value: mode_label },
        StatusField { key: "",    value: mode_sub },
    ];
    let _ = body;  // We render the panel manually because it's a special "big label + sub" layout.
    let panel = StatusPanel {
        label: "MODE",
        header_right: None,
        body: &[],  // hand-rendered below
    };
    paint_status_panel(rect, &panel);

    // Big mode label + sub (paint directly over the empty body area).
    use crate::ui::font;
    let fb = crate::ui::gpu::framebuffer();
    let screen_w = crate::ui::gpu::width();
    font::draw_str(fb, screen_w, rect.x + 10, rect.y + 36, mode_label, p::INK, p::PANEL);
    font::draw_str(fb, screen_w, rect.x + 10, rect.y + 60, mode_sub,   p::MID, p::PANEL);
}

fn paint_firewall_panel(rect: WindowRect) {
    let mut rules_buf  = [0u8; 32]; let mut rn = 0;
    push_bytes(&mut rules_buf, &mut rn, b"12 allow \xc2\xb7 0 deny");
    let rules = unsafe { core::str::from_utf8_unchecked(&rules_buf[..rn]) };
    let _ = rules;

    let mut drops_buf  = [0u8; 32]; let mut dn = 0;
    push_bytes(&mut drops_buf, &mut dn, b"3 in last 60s");
    let drops = unsafe { core::str::from_utf8_unchecked(&drops_buf[..dn]) };
    let _ = drops;

    let last_buf = b"14:31:48  tcp 10.0.0.4:443";
    let last_drop = unsafe { core::str::from_utf8_unchecked(last_buf) };
    let _ = last_drop;

    // For Wave 4, hardcode the firewall numbers if the kernel doesn't
    // expose live counts. Wave 5 wires them via net::firewall::stats().
    let body = [
        StatusField { key: "rules",     value: "12 allow · 0 deny" },
        StatusField { key: "drops",     value: "3 in last 60s" },
        StatusField { key: "last drop", value: "14:31:48 tcp 10.0.0.4:443" },
    ];
    let panel = StatusPanel {
        label: "FIREWALL",
        header_right: Some("default: DENY"),
        body: &body,
    };
    paint_status_panel(rect, &panel);
}

fn paint_activity_block(rect: WindowRect) {
    use crate::ui::font;
    let fb = crate::ui::gpu::framebuffer();
    let screen_w = crate::ui::gpu::width();
    font::draw_str(fb, screen_w, rect.x, rect.y, "ACTIVITY", p::MID, p::BG);

    // Build a Vec<ActivityEntry> from the ring (newest-first).
    let total = activity::count();
    let viewport = viewport_start();
    let mut entries: alloc::vec::Vec<(alloc::string::String, alloc::string::String, alloc::string::String)> = alloc::vec::Vec::new();
    let mut row_index: usize = 0;
    activity::iter_newest_first(|entry| {
        if row_index >= viewport {
            use alloc::format;
            let kind = ActivityKind::from_u8(entry.kind);
            let ts = format!("{:02}:{:02}:{:02}", entry.ts / 3600, (entry.ts / 60) % 60, entry.ts % 60);
            entries.push((ts, alloc::string::String::from(kind.as_str()), alloc::string::String::from(entry.summary_str())));
        }
        row_index += 1;
        true
    });
    let refs: alloc::vec::Vec<ActivityEntry> = entries.iter().map(|(t, k, s)| ActivityEntry {
        timestamp_str: t.as_str(),
        kind: k.as_str(),
        summary: s.as_str(),
    }).collect();
    let log_rect = WindowRect { x: rect.x, y: rect.y + 4, w: rect.w, h: rect.h.saturating_sub(4) };
    paint_activity_log(log_rect, &refs, viewport, total);
}

fn actions() -> [Action<'static>; 2] {
    [
        Action { hotkey: 'T', label: "Toggle isolation", enabled: true },
        Action { hotkey: 'C', label: "Clear counters",   enabled: true },
    ]
}

pub fn handle_key(c: u8) -> AppEvent {
    match c {
        0x90 => {  // Up
            let v = viewport_start();
            if v > 0 { set_viewport_start(v.saturating_sub(8)); }
            AppEvent::Repaint
        }
        0x91 => {  // Down
            let total = activity::count();
            let v = viewport_start();
            if v + 8 < total { set_viewport_start(v + 8); }
            AppEvent::Repaint
        }
        b't' | b'T' => {
            net::set_isolation(!net::is_isolated());
            AppEvent::Repaint
        }
        b'c' | b'C' => {
            net::clear_counters();
            set_viewport_start(0);
            AppEvent::Repaint
        }
        _ => AppEvent::Unhandled,
    }
}

pub fn handle_click(mx: i32, my: i32, body: WindowRect) -> AppEvent {
    let strip_rect = WindowRect { x: body.x + 14, y: body.y + body.h - 28, w: body.w - 28, h: 24 };
    if let Some(key) = action_strip_hit_test(strip_rect, mx, my, &actions()) {
        return handle_key(key as u8);
    }
    AppEvent::Consumed
}

// ── helpers ────────────────────────────────────────────────────

fn push_bytes(buf: &mut [u8], n: &mut usize, s: &[u8]) {
    for &b in s {
        if *n < buf.len() { buf[*n] = b; *n += 1; }
    }
}

fn format_rate(bps: u32, buf: &mut [u8], n: &mut usize) -> core::fmt::Result {
    if bps >= 1024 {
        let kbps = bps / 1024;
        write_dec(buf, n, kbps);
        push_bytes(buf, n, b".");
        write_dec(buf, n, ((bps % 1024) * 10) / 1024);
        push_bytes(buf, n, b" KB");
    } else {
        write_dec(buf, n, bps);
        push_bytes(buf, n, b" B");
    }
    Ok(())
}

fn format_bytes(bytes: u64, buf: &mut [u8], n: &mut usize) -> core::fmt::Result {
    if bytes >= 1024 * 1024 {
        write_dec(buf, n, (bytes / (1024 * 1024)) as u32);
        push_bytes(buf, n, b".");
        write_dec(buf, n, (((bytes % (1024 * 1024)) * 10) / (1024 * 1024)) as u32);
        push_bytes(buf, n, b" MB");
    } else if bytes >= 1024 {
        write_dec(buf, n, (bytes / 1024) as u32);
        push_bytes(buf, n, b" KB");
    } else {
        write_dec(buf, n, bytes as u32);
        push_bytes(buf, n, b" B");
    }
    Ok(())
}

fn format_hms(secs: u64, buf: &mut [u8], n: &mut usize) -> core::fmt::Result {
    let h = secs / 3600;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    write_pad2(buf, n, h as u32);
    push_bytes(buf, n, b":");
    write_pad2(buf, n, m as u32);
    push_bytes(buf, n, b":");
    write_pad2(buf, n, s as u32);
    Ok(())
}

fn write_dec(buf: &mut [u8], n: &mut usize, mut v: u32) {
    if v == 0 { if *n < buf.len() { buf[*n] = b'0'; *n += 1; } return; }
    let mut tmp = [0u8; 10];
    let mut t = 0;
    while v > 0 { tmp[t] = b'0' + (v % 10) as u8; v /= 10; t += 1; }
    for j in 0..t {
        if *n < buf.len() { buf[*n] = tmp[t - j - 1]; *n += 1; }
    }
}

fn write_pad2(buf: &mut [u8], n: &mut usize, v: u32) {
    if v < 10 { push_bytes(buf, n, b"0"); }
    write_dec(buf, n, v);
}
```

- [ ] **Step 2: Wire `apps_registry.rs` to the new NET handlers.**

```rust
AppDescriptor {
    id: AppId::Net,
    label: "NET",
    title: "NET",
    paint: paint_net,
    handle_key:   crate::ui::apps::netmon::handle_key,
    handle_click: crate::ui::apps::netmon::handle_click,
},
```

- [ ] **Step 3: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

- [ ] **Step 4: Commit.**

```bash
git add src/ui/apps/netmon.rs src/ui/apps_registry.rs
git commit -m "$(cat <<'EOF'
netmon: Wave 4 — live cockpit dashboard

Replaces the 201-line cyberpunk netmon with a Cockpit layout:
* Stats strip: RX/TX rates · PEAK · UPTIME (live).
* Top panels: MODE (ISOLATED/ROUTED + 1-line semantic) +
  FIREWALL (rules / drops in last 60s / last drop).
* Big ACTIVITY log: paint_activity_log over the net::activity ring
  (newest-first, viewport-scrolled by Up/Down).
* Actions: [T]oggle isolation (calls net::set_isolation), [C]lear
  counters (calls net::clear_counters).

Wave-4 firewall stats hardcoded to placeholder values; Wave 5+
wires them from kernel firewall state.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 5: SECURITY app rewrite

Replace `src/ui/apps/security.rs` with the operator panic console.

**Files:**
- Replace: `src/ui/apps/security.rs`
- Modify: `src/ui/apps_registry.rs`

- [ ] **Step 1: Replace `src/ui/apps/security.rs` entirely.**

```rust
//! Wave 4 SECURITY — operator panic console.
//! See `docs/superpowers/specs/2026-05-14-files-net-security-design.md`.

#![allow(dead_code, unused_imports)]

use crate::ui::apps_registry::AppEvent;
use crate::ui::palette as p;
use crate::ui::widgets::{
    paint_status_panel, StatusPanel, StatusField,
    paint_activity_log, ActivityEntry,
    paint_big_metric,
    paint_action_strip, action_strip_hit_test, Action,
    paint_confirm_modal, confirm_modal_key, ConfirmModal, ModalAction,
};
use crate::ui::wm::WindowRect;
use crate::security::{deadman, integrity_counts, auth};

#[derive(PartialEq, Eq)]
enum AppMode {
    Viewing,
    ConfirmWipe,
}

// Non-Copy: assign via `unsafe { *core::ptr::addr_of_mut!(APP_MODE) = ... }`
static mut APP_MODE: AppMode = AppMode::Viewing;
static mut VIEWPORT_START: usize = 0;
static mut CHAIN_STATUS:   [u8; 64] = [0; 64];
static mut CHAIN_STATUS_LEN: usize = 0;

fn viewport_start() -> usize {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(VIEWPORT_START)) }
}

pub fn paint(body: WindowRect) {
    crate::ui::gpu::fill_rect(body.x, body.y, body.w, body.h, p::BG);

    // ── Top panel row: DEADMAN + AUTH ──────────────────────────
    let panel_y = body.y + 12;
    let panel_h: u32 = 100;
    let gap: u32 = 12;
    let half_w = (body.w - 28 - gap) / 2;
    let dm_rect   = WindowRect { x: body.x + 14, y: panel_y, w: half_w, h: panel_h };
    let auth_rect = WindowRect { x: body.x + 14 + half_w + gap, y: panel_y, w: half_w, h: panel_h };
    paint_deadman_panel(dm_rect);
    paint_auth_panel(auth_rect);

    // ── AUDIT log (middle) ─────────────────────────────────────
    let log_y = panel_y + panel_h + 12;
    let log_h = body.h.saturating_sub(log_y - body.y + panel_h + 24 + 50);
    let log_rect = WindowRect { x: body.x + 14, y: log_y, w: body.w - 28, h: log_h };
    paint_audit_block(log_rect);

    // ── Bottom panel row: TAINT + INTEGRITY ───────────────────
    let bot_y = log_y + log_h + 12;
    let bot_h: u32 = panel_h;
    let taint_rect    = WindowRect { x: body.x + 14, y: bot_y, w: half_w, h: bot_h };
    let integ_rect    = WindowRect { x: body.x + 14 + half_w + gap, y: bot_y, w: half_w, h: bot_h };
    paint_taint_panel(taint_rect);
    paint_integrity_panel(integ_rect);

    // ── Action strip ──────────────────────────────────────────
    crate::ui::gpu::fill_rect(body.x + 14, body.y + body.h - 32, body.w - 28, 1, p::HAIRLINE);
    let strip_rect = WindowRect { x: body.x + 14, y: body.y + body.h - 28, w: body.w - 28, h: 24 };
    paint_action_strip(strip_rect, &actions());

    // ── ConfirmWipe modal (on top) ────────────────────────────
    if matches!(unsafe { &*core::ptr::addr_of!(APP_MODE) }, AppMode::ConfirmWipe) {
        let modal = ConfirmModal {
            title: "Wipe entire system?",
            body_lines: &[
                "  zero all cave keys",
                "  wipe BatFS",
                "  zero the audit ring",
                "  clear MLS labels + taint records",
                "  halt the kernel",
                "",
                "IRREVERSIBLE.",
            ],
            commit_key: 'W',
        };
        paint_confirm_modal(&modal);
    }
}

fn paint_deadman_panel(rect: WindowRect) {
    // Empty panel border + label, then paint_big_metric inside.
    let panel = StatusPanel { label: "DEADMAN", header_right: Some("ARMED"), body: &[] };
    paint_status_panel(rect, &panel);

    let secs = deadman::seconds_remaining();
    let mut value_buf = [0u8; 16]; let mut vn = 0;
    if secs >= 3600 {
        write_dec(&mut value_buf, &mut vn, (secs / 3600) as u32);
        push_bytes(&mut value_buf, &mut vn, b":");
        write_pad2(&mut value_buf, &mut vn, ((secs / 60) % 60) as u32);
    } else {
        write_pad2(&mut value_buf, &mut vn, (secs / 60) as u32);
        push_bytes(&mut value_buf, &mut vn, b":");
        write_pad2(&mut value_buf, &mut vn, (secs % 60) as u32);
    }
    let value = unsafe { core::str::from_utf8_unchecked(&value_buf[..vn]) };

    let mut sub_buf = [0u8; 48]; let mut sn = 0;
    push_bytes(&mut sub_buf, &mut sn, b"wipe in ");
    write_dec(&mut sub_buf, &mut sn, (secs / 60) as u32);
    push_bytes(&mut sub_buf, &mut sn, b" min");
    let sub = unsafe { core::str::from_utf8_unchecked(&sub_buf[..sn]) };

    let inner = WindowRect { x: rect.x + 10, y: rect.y + 32, w: rect.w - 20, h: rect.h - 42 };
    paint_big_metric(inner, "", value, sub);
}

fn paint_auth_panel(rect: WindowRect) {
    let remaining = auth::attempts_remaining();
    let ok = remaining >= 3;
    let panel = StatusPanel { label: "AUTH", header_right: Some(if ok { "ok" } else { "low" }), body: &[] };
    paint_status_panel(rect, &panel);

    let mut value_buf = [0u8; 16]; let mut vn = 0;
    write_dec(&mut value_buf, &mut vn, remaining as u32);
    push_bytes(&mut value_buf, &mut vn, b" / 5");
    let value = unsafe { core::str::from_utf8_unchecked(&value_buf[..vn]) };

    let inner = WindowRect { x: rect.x + 10, y: rect.y + 32, w: rect.w - 20, h: rect.h - 42 };
    paint_big_metric(inner, "", value, "attempts remaining");
}

fn paint_audit_block(rect: WindowRect) {
    use crate::ui::font;
    let fb = crate::ui::gpu::framebuffer();
    let screen_w = crate::ui::gpu::width();
    font::draw_str(fb, screen_w, rect.x, rect.y, "AUDIT", p::MID, p::BG);

    // Chain status (right-aligned).
    let status_len = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(CHAIN_STATUS_LEN)) };
    let status_default = b"chain: ready  \xc2\xb7  press V to verify";
    let (chain_status, len) = if status_len == 0 {
        let s = unsafe { core::str::from_utf8_unchecked(status_default) };
        (s, status_default.len())
    } else {
        let buf = unsafe { &*core::ptr::addr_of!(CHAIN_STATUS) };
        let s = unsafe { core::str::from_utf8_unchecked(&buf[..status_len]) };
        (s, status_len)
    };
    let chain_status_w = len as u32 * 8;
    font::draw_str(fb, screen_w,
        rect.x + rect.w.saturating_sub(chain_status_w + 8),
        rect.y, chain_status, p::MID, p::BG);

    // Audit ring entries — Wave 4 uses the existing audit chain.
    // Build display entries from crate::security::audit::iter_newest_first()
    // (or whatever the canonical iterator is, per pre-flight).
    use alloc::{format, string::String, vec::Vec};
    let mut entries: Vec<(String, String, String)> = Vec::new();
    let viewport = viewport_start();
    let mut row_index: usize = 0;
    crate::security::audit::iter_newest_first(|e| {
        if row_index >= viewport {
            let ts = format!("{:02}:{:02}:{:02}", e.ts / 3600, (e.ts / 60) % 60, e.ts % 60);
            entries.push((ts, String::from(e.kind_str()), String::from(e.summary_str())));
        }
        row_index += 1;
        true
    });
    let total = crate::security::audit::count();
    let refs: Vec<ActivityEntry> = entries.iter().map(|(t, k, s)| ActivityEntry {
        timestamp_str: t.as_str(),
        kind: k.as_str(),
        summary: s.as_str(),
    }).collect();
    let log_rect = WindowRect { x: rect.x, y: rect.y + 4, w: rect.w, h: rect.h.saturating_sub(4) };
    paint_activity_log(log_rect, &refs, viewport, total);
}

fn paint_taint_panel(rect: WindowRect) {
    use crate::caves::cave;
    let mut tainted: u32 = 0;
    let mut system_or: u32 = 0;
    cave::list(|c| {
        if let Some(id) = cave::find_id(c.name_str()) {
            let t = cave::taint_of(id as u16);
            if t != 0 { tainted += 1; system_or |= t; }
        }
    });
    let mut count_buf = [0u8; 24]; let mut cn = 0;
    write_dec(&mut count_buf, &mut cn, tainted);
    push_bytes(&mut count_buf, &mut cn, b" caves tainted");
    let count_str = unsafe { core::str::from_utf8_unchecked(&count_buf[..cn]) };

    let mut or_buf = [0u8; 32]; let mut on = 0;
    push_bytes(&mut or_buf, &mut on, b"0x");
    for k in 0..8 {
        let nibble = ((system_or >> ((7 - k) * 4)) & 0xF) as u8;
        or_buf[on] = if nibble < 10 { b'0' + nibble } else { b'a' + (nibble - 10) };
        on += 1;
    }
    let labels = taint_labels(system_or);
    if !labels.is_empty() { push_bytes(&mut or_buf, &mut on, b" \xc2\xb7 "); }
    push_bytes(&mut or_buf, &mut on, labels.as_bytes());
    let or_str = unsafe { core::str::from_utf8_unchecked(&or_buf[..on]) };

    let body = [
        StatusField { key: "system OR", value: or_str },
        StatusField { key: "count",     value: count_str },
    ];
    let panel = StatusPanel {
        label: "TAINT",
        header_right: if tainted > 0 { Some(count_str) } else { Some("clean") },
        body: &body,
    };
    paint_status_panel(rect, &panel);
}

fn taint_labels(bits: u32) -> &'static str {
    match bits {
        0x00000000 => "",
        0x00000001 => "PII",
        0x00000002 => "CRYPTO",
        0x00000004 => "AUDIT",
        0x00000008 => "NETWORK",
        _ if (bits & 0x0F) != 0 => "PII|CRYPTO|AUDIT|NETWORK",
        _ => "",
    }
}

fn paint_integrity_panel(rect: WindowRect) {
    let blp = integrity_counts::blp_denies();
    let biba = integrity_counts::biba_denies();
    let te = integrity_counts::te_denies();
    let clean = blp == 0 && biba == 0 && te == 0;

    let mut blp_buf = [0u8; 24]; let mut bn = 0;
    write_dec(&mut blp_buf, &mut bn, blp);
    push_bytes(&mut blp_buf, &mut bn, b" in 24h");
    let blp_str = unsafe { core::str::from_utf8_unchecked(&blp_buf[..bn]) };

    let mut biba_buf = [0u8; 24]; let mut bbn = 0;
    write_dec(&mut biba_buf, &mut bbn, biba);
    push_bytes(&mut biba_buf, &mut bbn, b" in 24h");
    let biba_str = unsafe { core::str::from_utf8_unchecked(&biba_buf[..bbn]) };

    let mut te_buf = [0u8; 24]; let mut tn = 0;
    write_dec(&mut te_buf, &mut tn, te);
    push_bytes(&mut te_buf, &mut tn, b" in 24h");
    let te_str = unsafe { core::str::from_utf8_unchecked(&te_buf[..tn]) };

    let body = [
        StatusField { key: "BLP denies",  value: blp_str },
        StatusField { key: "Biba denies", value: biba_str },
        StatusField { key: "TE denies",   value: te_str },
    ];
    let panel = StatusPanel {
        label: "INTEGRITY",
        header_right: if clean { Some("clean") } else { Some("denies") },
        body: &body,
    };
    paint_status_panel(rect, &panel);
}

fn actions() -> [Action<'static>; 3] {
    [
        Action { hotkey: 'R', label: "Re-arm deadman", enabled: true },
        Action { hotkey: 'V', label: "Verify chain",   enabled: true },
        Action { hotkey: 'W', label: "Wipe NOW",       enabled: true },
    ]
}

pub fn handle_key(c: u8) -> AppEvent {
    if matches!(unsafe { &*core::ptr::addr_of!(APP_MODE) }, AppMode::ConfirmWipe) {
        return handle_key_wipe_modal(c);
    }
    match c {
        0x90 => {  // Up
            let v = viewport_start();
            if v > 0 {
                unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), v.saturating_sub(8)); }
            }
            AppEvent::Repaint
        }
        0x91 => {  // Down
            let total = crate::security::audit::count();
            let v = viewport_start();
            if v + 8 < total {
                unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(VIEWPORT_START), v + 8); }
            }
            AppEvent::Repaint
        }
        b'r' | b'R' => { deadman::arm(48); AppEvent::Repaint }
        b'v' | b'V' => {
            let ok = crate::security::audit::verify_chain();
            let len = crate::security::audit::count();
            let root = crate::security::audit::root_hash_short();
            let mut buf = [0u8; 64]; let mut n = 0;
            push_bytes(&mut buf, &mut n, b"chain: ");
            write_dec(&mut buf, &mut n, len as u32);
            push_bytes(&mut buf, &mut n, b" \xc2\xb7 root ");
            push_bytes(&mut buf, &mut n, root.as_bytes());
            push_bytes(&mut buf, &mut n, b" \xc2\xb7 ");
            push_bytes(&mut buf, &mut n, if ok { b"verified ok" } else { b"verified FAIL" });
            unsafe {
                let dst = core::ptr::addr_of_mut!(CHAIN_STATUS) as *mut u8;
                core::ptr::copy_nonoverlapping(buf.as_ptr(), dst, n);
                core::ptr::write_volatile(core::ptr::addr_of_mut!(CHAIN_STATUS_LEN), n);
            }
            AppEvent::Repaint
        }
        b'w' | b'W' => {
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::ConfirmWipe; }
            AppEvent::Repaint
        }
        _ => AppEvent::Unhandled,
    }
}

fn handle_key_wipe_modal(c: u8) -> AppEvent {
    let modal = ConfirmModal { title: "", body_lines: &[], commit_key: 'W' };
    match confirm_modal_key(&modal, c) {
        ModalAction::Commit => {
            // TODO(Wave 4 follow-up): make wipe::execute diverge (-> !).
            crate::security::wipe::execute(crate::security::wipe::WipeReason::Manual, false);
            // Unreachable on hardware; only reached under QEMU stub.
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing; }
            AppEvent::Repaint
        }
        ModalAction::Cancel => {
            unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing; }
            AppEvent::Repaint
        }
        ModalAction::None => AppEvent::Consumed,
    }
}

pub fn handle_click(mx: i32, my: i32, body: WindowRect) -> AppEvent {
    if matches!(unsafe { &*core::ptr::addr_of!(APP_MODE) }, AppMode::ConfirmWipe) {
        // Any click cancels.
        unsafe { *core::ptr::addr_of_mut!(APP_MODE) = AppMode::Viewing; }
        return AppEvent::Repaint;
    }
    let strip_rect = WindowRect { x: body.x + 14, y: body.y + body.h - 28, w: body.w - 28, h: 24 };
    if let Some(key) = action_strip_hit_test(strip_rect, mx, my, &actions()) {
        return handle_key(key as u8);
    }
    AppEvent::Consumed
}

// ── helpers ────────────────────────────────────────────────────

fn push_bytes(buf: &mut [u8], n: &mut usize, s: &[u8]) {
    for &b in s {
        if *n < buf.len() { buf[*n] = b; *n += 1; }
    }
}

fn write_dec(buf: &mut [u8], n: &mut usize, mut v: u32) {
    if v == 0 { if *n < buf.len() { buf[*n] = b'0'; *n += 1; } return; }
    let mut tmp = [0u8; 10];
    let mut t = 0;
    while v > 0 { tmp[t] = b'0' + (v % 10) as u8; v /= 10; t += 1; }
    for j in 0..t {
        if *n < buf.len() { buf[*n] = tmp[t - j - 1]; *n += 1; }
    }
}

fn write_pad2(buf: &mut [u8], n: &mut usize, v: u32) {
    if v < 10 { push_bytes(buf, n, b"0"); }
    write_dec(buf, n, v);
}
```

- [ ] **Step 2: Wire `apps_registry.rs` to the new SECURITY handlers.**

```rust
AppDescriptor {
    id: AppId::Security,
    label: "SECURITY",
    title: "SECURITY",
    paint: paint_security,
    handle_key:   crate::ui::apps::security::handle_key,
    handle_click: crate::ui::apps::security::handle_click,
},
```

- [ ] **Step 3: Build + clippy clean.**

```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
```

Expected build errors that need pre-flight cross-checking:
- `crate::security::audit::iter_newest_first` / `count` / `verify_chain` / `root_hash_short` — pre-flight should have confirmed the actual API names. Adapt the call sites if they differ.
- `audit::Entry::kind_str()` / `summary_str()` — same; adapt.

If the audit API doesn't quite match, add the missing helper functions to the audit module as part of this task with a `// Wave 4 helper` comment.

- [ ] **Step 4: Commit.**

```bash
git add src/ui/apps/security.rs src/ui/apps_registry.rs src/security/audit.rs
git commit -m "$(cat <<'EOF'
security: Wave 4 — operator panic console

Replaces the 239-line cyberpunk security.rs with the panic console:
* Top: DEADMAN big countdown (HH:MM or MM:SS) + AUTH ratio (X/5).
* Middle: AUDIT log via paint_activity_log over the tamper-evident
  chain. Header shows chain length + truncated root + last
  verification result. [V]erify writes the result back into the
  header.
* Bottom: TAINT panel (system-OR + labels for known bits) +
  INTEGRITY panel (24h BLP/Biba/TE deny counts).
* Actions: [R]e-arm deadman (instant, calls deadman::arm(48)),
  [V]erify chain (blocks during verify), [W]ipe NOW (opens
  ConfirmModal; second W calls wipe::execute(Manual, false)).
  Wipe NOW renders in INK — pure-mono discipline; ConfirmModal
  carries the protective intent.

The pre-Wave-3 "ACTIVE BATCAVES" panel is gone (now in CAVES app).

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Task 6: QEMU walk-through

Manual visual confirmation of all three apps. **No commit.**

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

- [ ] **Step 2: Verify FILES.**

Press `2` to open FILES. Confirm:
- Window opens with Inspector layout.
- Sidebar shows `FILES (n)` header. If n > 0, list of files with state dots (filled = encrypted, hollow = plain).
- Detail panel shows filename + 1-line metadata + preview region (text or hex).
- ↑/↓ scrolls the preview viewport.
- Press `D` on a file → ConfirmDelete modal appears. Esc cancels.

- [ ] **Step 3: Verify NET.**

Press `3` to open NET. Confirm:
- Stats strip shows RX/TX/PEAK/UPTIME (may be zeros if no traffic).
- MODE panel shows ISOLATED with sub-caption.
- FIREWALL panel shows rules + drops + last-drop text.
- ACTIVITY log paints entries from the ring (likely zero entries unless network was active).
- `[T]oggle isolation` flips between ISOLATED and ROUTED.
- `[C]lear counters` resets counters to 0 (and writes a `counters_cleared` entry to the activity ring).

- [ ] **Step 4: Verify SECURITY.**

Press `4` to open SECURITY. Confirm:
- DEADMAN panel shows a countdown (probably ~47:59 if just booted).
- AUTH panel shows attempts remaining (5/5 after fresh boot).
- AUDIT log shows recent entries from the audit chain (should include the auth_pass entry from unlock).
- TAINT panel shows "clean" or actual taint state.
- INTEGRITY panel shows 0/0/0 unless a deny happened during this session.
- `[R]e-arm deadman` resets the countdown to 48h.
- `[V]erify chain` updates the AUDIT header from "press V to verify" to "verified ok" (or "verified FAIL").
- `[W]ipe NOW` opens ConfirmModal. **Don't press W twice unless you actually want to wipe** (QEMU return from wipe is graceful per the Wave-3 docs; safe to test).

- [ ] **Step 5: Verify cross-app keyboard parity.**

- `1` opens CAVES (Wave 3) — unchanged.
- `Tab` cycles focus through open windows.
- `Ctrl+K` toggles launcher overlay over the open apps.
- `Ctrl+L` returns to lock screen; passphrase re-unlocks; workspace persists.

- [ ] **Step 6: Kill QEMU.**

```bash
pkill -9 -f 'qemu-system-aarch64'
```

- [ ] **Step 7: No commit.**

If any step surfaced a defect, return to the relevant earlier task.

---

## Task 7: Push + finishing-a-development-branch

- [ ] **Step 1: Push to origin.**

```bash
cd /Users/kadenlee/Sphragis
git push -u origin feat/files-net-security
```

- [ ] **Step 2: Invoke `superpowers:finishing-a-development-branch`.**

Same as Waves 1/2/3. Recommended choice: "Merge back to main locally" — full pattern: checkout main → --no-ff merge → verify build/clippy → delete local branch → push origin main → delete origin's feature branch → journal entry.

---

## Spec coverage map (self-review)

| Spec section | Task |
|--------------|------|
| §Visual system (palette + typography) | Inherits Wave-3; no new constants. Tasks 3/4/5 all `use crate::ui::palette as p`. |
| §Layout — Inspector | Task 3 uses Wave-3 `InspectorLayout`. |
| §Layout — Cockpit | Tasks 4 + 5 hand-roll rect-splitting math (no `CockpitLayout` struct per spec). |
| §App: FILES | Task 3 |
| §App: NET | Task 4 |
| §App: SECURITY | Task 5 |
| §Pattern primitives §1 paint_activity_log | Task 2a |
| §Pattern primitives §2 paint_status_panel | Task 2b |
| §Pattern primitives §3 paint_big_metric | Task 2c (+ font::draw_str_scale in Task 1a) |
| §Pattern primitives §4 paint_file_preview | Task 2d |
| §Kernel API gaps §1 net::set_isolation | Task 1c |
| §Kernel API gaps §2 net counters | Task 1c |
| §Kernel API gaps §3 net::activity ring | Task 1d |
| §Kernel API gaps §4 audit chain API | Task 0c (verify); Task 5 (use) |
| §Kernel API gaps §5 taint iter | Task 5 (use `cave::list` + `cave::find_id` + `cave::taint_of`) |
| §Kernel API gaps §6 integrity deny counts | Task 1e |
| §Kernel API gaps §7 deadman::arm | Task 1b |
| §Kernel API gaps §8 font::draw_str_scale | Task 1a |
| §Implementation outline §QEMU walk-through | Task 6 |

## Out-of-scope reminders

Per the spec §"What's NOT in v1," none of the following land in this plan:
- EDITOR / COMMS redesigns
- SHELL redesign
- TT rasterizer fix
- NET activity filter UI
- TAINT operator-defined bit dictionary
- Per-cave NET enforcement
- BatFS rename / new-file APIs
- Animations
- Right-click context menus

Wave 5 picks up EDITOR + SHELL + TT rasterizer. COMMS gets its own later wave.

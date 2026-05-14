# Lock Screen Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the cyberpunk boot/lock screen with the quiet, monochromatic, silent-denial design specified in `docs/superpowers/specs/2026-05-14-lock-screen-redesign-design.md`.

**Architecture:** Embed two SIL-OFL TrueType fonts (IBM Plex Serif Italic, IBM Plex Sans Medium) as `include_bytes!` blobs; extend the existing `src/ui/truetype.rs` rasterizer to support multiple font faces and full Unicode codepoints (currently single-font, ASCII-only); rewrite `src/security/boot_screen.rs::paint_lock_screen` end-to-end against a new 5-color palette and the new state machine where `Denied` is pixel-identical to `Idle` and `LockedOut` skips the banner.

**Tech Stack:** Rust 2024 edition, `aarch64-unknown-none` target (no-std, no-alloc kernel code; the `truetype.rs` rasterizer is already no-std and uses fixed-size buffers). Build: `cargo build --release --target aarch64-unknown-none --features gicv3`. Verification: QEMU `-machine virt -cpu max -display cocoa` with virtio-keyboard.

**Verification reality check.** This crate is `#![no_std] #![no_main]` (no `lib.rs`, no test harness). The `#[cfg(test)]` blocks elsewhere in the tree are decorative — `cargo test` never runs them. There is no unit-test step in this plan; every task's verification is "build is clean (no clippy warnings under `-D warnings`)" plus "boot under QEMU and walk the state machine by hand." The Python smoke-test pattern in `scripts/qemu_*_selftest.py` is the closest thing to integration tests, but driving a visual lock-screen redesign with pexpect+serial is more work than it's worth for a single-screen change. Manual visual confirmation against Section 2 of the spec is the contract.

---

## Pre-flight

- [ ] **Step 0a: Confirm clean working tree on a feature branch.**

Run:
```bash
cd /Users/kadenlee/Sphragis
git status --short
git checkout -b feat/lock-screen-redesign
```
Expected: working tree clean before checkout; on branch `feat/lock-screen-redesign` after.

- [ ] **Step 0b: Verify `pyftsubset` is installed (needed by Task 1).**

Run:
```bash
pyftsubset --help 2>&1 | head -3
```
If "command not found", install via:
```bash
pip3 install fonttools
```
Expected after install: `pyftsubset` prints its help banner.

---

### Task 1: Embed IBM Plex font subsets

Replaces the current `fonts/font.ttf` (Verdana, proprietary) with two SIL-OFL fonts subsetted to the 40 codepoints the lock screen actually renders. Resolves both the spec's typography requirement and the incidental Verdana licensing problem.

**Files:**
- Create: `fonts/ibm-plex-serif-italic.ttf` (subsetted, ~120 KB target)
- Create: `fonts/ibm-plex-sans-medium.ttf` (subsetted, ~120 KB target)
- Create: `fonts/README.md`
- Keep: `fonts/font.ttf` (Verdana) — left in place this wave; no callers will reference it after Task 2. Removal happens in a follow-up alongside the polygon Σ deletion.

- [ ] **Step 1: Download the IBM Plex source TTFs into a scratch directory.**

Run:
```bash
mkdir -p /tmp/plex && cd /tmp/plex
curl -fsSL -o plex-serif-italic.ttf \
  https://github.com/IBM/plex/raw/v6.4.2/IBM-Plex-Serif/fonts/complete/ttf/IBMPlexSerif-Italic.ttf
curl -fsSL -o plex-sans-medium.ttf \
  https://github.com/IBM/plex/raw/v6.4.2/IBM-Plex-Sans/fonts/complete/ttf/IBMPlexSans-Medium.ttf
ls -la /tmp/plex/
```
Expected: two files, each 100–250 KB. If `v6.4.2` returns 404, replace with the current default branch tag from https://github.com/IBM/plex/releases.

- [ ] **Step 2: Subset both fonts to the codepoint set from the spec.**

The spec fixes the codepoint set to: `A`–`Z` (26), `0`–`9` (10), `Σ` (U+03A3), space, `.`, `-` — 40 codepoints total. Run:
```bash
cd /tmp/plex
# pyftsubset accepts hex codepoints via --unicodes and literal chars via --text.
# Belt-and-suspenders: pass both, plus retain hinting and basic OpenType tables.
SUBSET_FLAGS='--unicodes=U+0020,U+002D,U+002E,U+0030-U+0039,U+0041-U+005A,U+03A3 \
              --layout-features="" --no-hinting --desubroutinize'
pyftsubset plex-serif-italic.ttf \
  --unicodes=U+0020,U+002D,U+002E,U+0030-U+0039,U+0041-U+005A,U+03A3 \
  --layout-features="" --no-hinting --desubroutinize \
  --output-file=ibm-plex-serif-italic.subset.ttf
pyftsubset plex-sans-medium.ttf \
  --unicodes=U+0020,U+002D,U+002E,U+0030-U+0039,U+0041-U+005A,U+03A3 \
  --layout-features="" --no-hinting --desubroutinize \
  --output-file=ibm-plex-sans-medium.subset.ttf
ls -la *.subset.ttf
```
Expected: two `.subset.ttf` files, each under 200 KB (subsetting takes off most of the 100-KB-per-face hinting tables; a 40-codepoint subset typically lands between 30 KB and 80 KB).

- [ ] **Step 3: Move the subsetted fonts into the repo.**

Run:
```bash
mv /tmp/plex/ibm-plex-serif-italic.subset.ttf \
   /Users/kadenlee/Sphragis/fonts/ibm-plex-serif-italic.ttf
mv /tmp/plex/ibm-plex-sans-medium.subset.ttf \
   /Users/kadenlee/Sphragis/fonts/ibm-plex-sans-medium.ttf
file /Users/kadenlee/Sphragis/fonts/ibm-plex-*.ttf
```
Expected: both files report `TrueType Font data`. If either reports `OpenType Font data`, the rasterizer in `src/ui/truetype.rs` won't parse it (it checks magic `0x00010000` or `'true'`). In that case, re-run pyftsubset with `--no-recalc-bounds` or fall back to OTF→TTF via fontforge; not expected for IBM Plex, which ships TTF.

- [ ] **Step 4: Write `fonts/README.md` with licensing + source.**

Create file `fonts/README.md` with:
````markdown
# Embedded fonts

Two TrueType fonts are embedded into the kernel binary via `include_bytes!` for the boot/lock screen.

| File | Source | License | Use |
|------|--------|---------|-----|
| `ibm-plex-serif-italic.ttf` | [IBM/plex v6.4.2](https://github.com/IBM/plex) | SIL Open Font License 1.1 | Σ glyph on lock screen |
| `ibm-plex-sans-medium.ttf`  | [IBM/plex v6.4.2](https://github.com/IBM/plex) | SIL Open Font License 1.1 | "SPHRAGIS" wordmark |

Both files have been subsetted with `pyftsubset` (from `fonttools`) to the following codepoints to minimize kernel binary footprint:

- `U+0020` (space)
- `U+002D` (hyphen-minus)
- `U+002E` (period)
- `U+0030`–`U+0039` (digits 0–9)
- `U+0041`–`U+005A` (uppercase A–Z)
- `U+03A3` (Greek capital letter sigma — Σ)

The SIL OFL is compatible with this project's AGPL-3.0-or-later license. Per SIL OFL terms, the fonts may not be redistributed under a different name, which is why the filenames preserve `ibm-plex-` upstream branding.

## Legacy file

`font.ttf` is **Verdana** (Microsoft proprietary, bundled with macOS but not redistributable). It is left in place this wave because the rasterizer (`src/ui/truetype.rs`) historically referenced it. After Task 2 of the lock-screen-redesign implementation, nothing in the kernel still references it. Removal is a follow-up task tracked in `docs/superpowers/specs/2026-05-14-lock-screen-redesign-design.md` under *Scope boundary*.
````

- [ ] **Step 5: Commit.**

Run:
```bash
cd /Users/kadenlee/Sphragis
git add fonts/ibm-plex-serif-italic.ttf fonts/ibm-plex-sans-medium.ttf fonts/README.md
git commit -m "$(cat <<'EOF'
fonts: embed SIL-OFL IBM Plex Serif Italic + Plex Sans Medium subsets

40-codepoint subsets (A-Z, 0-9, Σ, space, period, hyphen) of the two
font faces the redesigned lock screen renders. Replaces the
proprietary Verdana that the truetype rasterizer historically
referenced; the Verdana file stays in place this wave because it's
still listed in src/ui/truetype.rs's include_bytes — Task 2 removes
that reference, after which font.ttf has zero callers.

Both subsets are well under 200 KB so the combined kernel footprint
grows by <300 KB. Subsetting done with pyftsubset (fonttools); see
fonts/README.md for codepoint list and licensing.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```
Expected: one commit, three files added.

---

### Task 2: Extend `truetype.rs` for multi-font + non-ASCII

The existing rasterizer is single-font (one `EMBEDDED_FONT` static) and the public draw functions take `&[u8]` filtered to printable ASCII — Σ (U+03A3) would be silently dropped. This task adds a `FontFace` enum, two embedded-font slots, and a Unicode-codepoint-accepting draw API. The existing `draw_text_fb` / `draw_text_fb_styled` / `text_width` keep working unchanged (they continue to use the legacy single-font path for the moment; nothing in the kernel actually calls them, but we preserve the surface to avoid an unrelated diff).

**Files:**
- Modify: `src/ui/truetype.rs`

- [ ] **Step 1: Add the `FontFace` enum + two embedded-font slots.**

Open `src/ui/truetype.rs`. Below the existing `static EMBEDDED_FONT: &[u8] = include_bytes!("../../fonts/font.ttf");` line (around line 961), add:

```rust
/// Which embedded font face to draw with.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FontFace {
    /// IBM Plex Serif Italic — used for the lock-screen Σ glyph.
    PlexSerifItalic,
    /// IBM Plex Sans Medium — used for the lock-screen wordmark.
    PlexSansMedium,
}

static EMBEDDED_PLEX_SERIF_ITALIC: &[u8] =
    include_bytes!("../../fonts/ibm-plex-serif-italic.ttf");
static EMBEDDED_PLEX_SANS_MEDIUM:  &[u8] =
    include_bytes!("../../fonts/ibm-plex-sans-medium.ttf");

static mut PLEX_SERIF_CACHE: Option<TrueTypeFont> = None;
static mut PLEX_SERIF_INIT:  bool = false;
static mut PLEX_SANS_CACHE:  Option<TrueTypeFont> = None;
static mut PLEX_SANS_INIT:   bool = false;
```

- [ ] **Step 2: Add `get_font_face()` — the per-face cached parse.**

Below the new statics, add:

```rust
/// Get a parsed TrueType font for the given face. Parses on first call,
/// caches thereafter. Returns None if the embedded blob is corrupt
/// (which means the build is broken; treat as a hard error at use site).
fn get_font_face(face: FontFace) -> Option<&'static TrueTypeFont> {
    unsafe {
        match face {
            FontFace::PlexSerifItalic => {
                if !core::ptr::read_volatile(core::ptr::addr_of!(PLEX_SERIF_INIT)) {
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(PLEX_SERIF_INIT), true);
                    let parsed = TrueTypeFont::parse(EMBEDDED_PLEX_SERIF_ITALIC);
                    core::ptr::write(core::ptr::addr_of_mut!(PLEX_SERIF_CACHE), parsed);
                }
                let cache_ptr = core::ptr::addr_of!(PLEX_SERIF_CACHE);
                (*cache_ptr).as_ref()
            }
            FontFace::PlexSansMedium => {
                if !core::ptr::read_volatile(core::ptr::addr_of!(PLEX_SANS_INIT)) {
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(PLEX_SANS_INIT), true);
                    let parsed = TrueTypeFont::parse(EMBEDDED_PLEX_SANS_MEDIUM);
                    core::ptr::write(core::ptr::addr_of_mut!(PLEX_SANS_CACHE), parsed);
                }
                let cache_ptr = core::ptr::addr_of!(PLEX_SANS_CACHE);
                (*cache_ptr).as_ref()
            }
        }
    }
}
```

- [ ] **Step 3: Add the Unicode-codepoint glyph-draw API.**

After the new `get_font_face`, add:

```rust
/// Draw a single Unicode codepoint at (x, y) into the framebuffer.
/// Returns the glyph's advance width in pixels (so callers can chain calls
/// to lay out text). Returns 0 if the codepoint isn't in this face's
/// embedded subset (e.g., asking PlexSans for a Greek capital sigma).
pub fn draw_glyph(
    fb: *mut u32,
    screen_w: u32,
    x: i32,
    y: i32,
    codepoint: u32,
    face: FontFace,
    size_px: u16,
    color: u32,
) -> i32 {
    let font = match get_font_face(face) {
        Some(f) => f,
        None => return 0,
    };
    let glyph_id = font.glyph_index(codepoint);
    if glyph_id == 0 { return 0; }

    let mut bitmap = [0u8; MAX_GLYPH_SIZE * MAX_GLYPH_SIZE];
    // Pass the codepoint to render_char directly (it takes char).
    let ch = match char::from_u32(codepoint) {
        Some(c) => c,
        None    => return 0,
    };
    let (gw, gh, advance) = font.render_char(ch, size_px, &mut bitmap);

    let cr = ((color >> 16) & 0xFF) as u32;
    let cg = ((color >> 8) & 0xFF) as u32;
    let cb = (color & 0xFF) as u32;

    for row in 0..gh as i32 {
        let sy = y + row;
        if sy < 0 { continue; }
        for col in 0..gw as i32 {
            let sx = x + col;
            if sx < 0 || sx >= screen_w as i32 { continue; }

            let coverage = bitmap[(row as usize) * (gw as usize) + (col as usize)] as u32;
            if coverage == 0 { continue; }

            let fb_idx = (sy as u32 * screen_w + sx as u32) as usize;
            unsafe {
                let dst = core::ptr::read_volatile(fb.add(fb_idx));
                let dr = (dst >> 16) & 0xFF;
                let dg = (dst >> 8) & 0xFF;
                let db = dst & 0xFF;
                let r = (dr as i32 + (((cr as i32 - dr as i32) * coverage as i32) / 255))
                    .clamp(0, 255) as u32;
                let g = (dg as i32 + (((cg as i32 - dg as i32) * coverage as i32) / 255))
                    .clamp(0, 255) as u32;
                let b = (db as i32 + (((cb as i32 - db as i32) * coverage as i32) / 255))
                    .clamp(0, 255) as u32;
                core::ptr::write_volatile(
                    fb.add(fb_idx),
                    0xFF000000 | (r << 16) | (g << 8) | b,
                );
            }
        }
    }

    advance as i32
}

/// Advance width of a single codepoint in `face` at `size_px`, in pixels.
/// Returns 0 if the codepoint isn't in this face's subset.
pub fn glyph_advance(codepoint: u32, face: FontFace, size_px: u16) -> i32 {
    let font = match get_font_face(face) {
        Some(f) => f,
        None => return 0,
    };
    let glyph_id = font.glyph_index(codepoint);
    if glyph_id == 0 { return 0; }
    let ch = match char::from_u32(codepoint) {
        Some(c) => c,
        None    => return 0,
    };
    let mut dummy = [0u8; 4];
    let (_, _, advance) = font.render_char(ch, size_px, &mut dummy);
    advance as i32
}

/// Total advance width of a string in `face` at `size_px`. Used by the
/// lock screen to center the wordmark.
pub fn text_advance(text: &str, face: FontFace, size_px: u16) -> i32 {
    let mut w = 0i32;
    for ch in text.chars() {
        w += glyph_advance(ch as u32, face, size_px);
    }
    w
}

/// Draw a UTF-8 string at (x, y) in `face`. Returns total advance.
/// Used by the lock screen for the "SPHRAGIS" wordmark.
pub fn draw_text(
    fb: *mut u32,
    screen_w: u32,
    x: i32,
    y: i32,
    text: &str,
    face: FontFace,
    size_px: u16,
    color: u32,
) -> i32 {
    let mut cursor_x = x;
    for ch in text.chars() {
        cursor_x += draw_glyph(fb, screen_w, cursor_x, y, ch as u32, face, size_px, color);
    }
    cursor_x - x
}
```

- [ ] **Step 4: Build to verify the new code compiles.**

Run:
```bash
cd /Users/kadenlee/Sphragis
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -8
```
Expected: clean build. If the new fonts aren't valid TTF (or `include_bytes!` paths are wrong), the compile won't fail (only `parse()` returns None at runtime), but if there's a syntax error in the new code it'll surface here.

- [ ] **Step 5: Clippy with `-D warnings` to catch dead-code warnings.**

Run:
```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -8
```
Expected: clean. The new public functions are not yet called from anywhere, but they're `pub` so they don't trigger dead-code warnings.

- [ ] **Step 6: Commit.**

```bash
cd /Users/kadenlee/Sphragis
git add src/ui/truetype.rs
git commit -m "$(cat <<'EOF'
truetype: add multi-face + Unicode-codepoint draw API

Extends the existing rasterizer with FontFace (PlexSerifItalic /
PlexSansMedium), per-face cached parse, and three new public draw
functions accepting full Unicode codepoints:

  draw_glyph(fb, w, x, y, codepoint, face, size, color) -> advance
  glyph_advance(codepoint, face, size) -> px
  text_advance(text, face, size) -> px
  draw_text(fb, w, x, y, text: &str, face, size, color) -> total advance

The existing draw_text_fb / draw_text_fb_styled / text_width remain
unchanged and continue to use the legacy single-font path. Nothing in
the kernel currently calls those (the rasterizer was wired into the
tree but no boot-screen / WM path actually invoked it), so the legacy
surface stays purely as a fallback.

Sets up the boot_screen rewrite in the next commit.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

### Task 3: Replace `paint_lock_screen` + collapse the palette

Single edit replaces `paint_lock_screen`'s body, swaps the 16-color palette for the 5-color palette from the spec, and deletes every helper enumerated in the spec's kill list. Must be one commit because deleting the old helpers without deleting their callers (or vice versa) breaks the build with dead-code warnings under `-D warnings`.

**Files:**
- Modify: `src/security/boot_screen.rs`

- [ ] **Step 1: Replace the palette block.**

In `src/security/boot_screen.rs`, find the `// ─── Palette ───` section (lines ~71–88). Replace the entire palette block (lines 73–88) with:

```rust
// ─── Palette (Wave 1 lock-screen redesign — pure mono) ─────────────────
//
// 5 named colors. No state-color overlay. See
// docs/superpowers/specs/2026-05-14-lock-screen-redesign-design.md.
// Hex encoding is `(A<<24)|(R<<16)|(G<<8)|B` per the framebuffer note
// at the top of this file.
const BG:       u32 = 0xFF0D0D10; // background
const PANEL:    u32 = 0xFF18181C; // field panel
const HAIRLINE: u32 = 0xFF2A2A30; // field border
const INK:      u32 = 0xFFE5E7EB; // primary text + glyph
const MID:      u32 = 0xFF6B7280; // reserved (unused in v1; see spec)
```

- [ ] **Step 2: Replace the layout constants block.**

The current layout constants block (lines ~90–106) has `MARGIN_X`, `MARGIN_Y`, `HAIRLINE_Y`, `CHAR_W`, `CHAR_H`, `GLYPH_W`, `GLYPH_H`, `FIELD_W`, `FIELD_H`, `DOT_PX`, `DOT_GAP`. Replace the entire block with:

```rust
// ─── Layout constants (1280x800 native; scales with the FB size) ──────

const FIELD_W:  u32 = 480;
const FIELD_H:  u32 = 56;
const DOT_PX:   u32 = 8;
const DOT_GAP:  u32 = 8;

// Σ glyph: 96-px cap height, rendered via Plex Serif Italic.
const GLYPH_SIZE_PX:    u16 = 96;
const WORDMARK_SIZE_PX: u16 = 14;

// Stack offset: how far above screen vertical center the glyph baseline
// sits. ~80 px keeps the composition off dead-middle without crowding
// the top of the screen.
const STACK_ABOVE_CENTER: u32 = 80;

// Wordmark sits 16 px below the glyph baseline; field sits 40 px below
// the wordmark.
const WORDMARK_GAP: u32 = 16;
const FIELD_GAP:    u32 = 40;
```

- [ ] **Step 3: Delete the four helper functions.**

In `src/security/boot_screen.rs`, delete every line of these four functions and the `BOOT_LOG` constant table they reference:

1. `fn draw_corner(...)` (lines ~112–123)
2. `fn draw_status_pill(...)` (lines ~128–159)
3. `const BOOT_LOG: &[(&str, &str)]` (lines ~167–172)
4. `fn draw_boot_log(...)` (lines ~174–181)
5. `fn draw_clock_block(...)` (lines ~183–226)

After deletion, the only function definitions remaining before `paint_lock_screen` should be `hold_ms` and the `LockState` enum.

- [ ] **Step 4: Replace `paint_lock_screen` end-to-end.**

Find `fn paint_lock_screen(fb: *mut u32, w: u32, h: u32, state: LockState, attempts: u8)` (~line 234) and replace its entire body (the `{ ... }` block) with:

```rust
fn paint_lock_screen(fb: *mut u32, w: u32, h: u32, state: LockState, _attempts: u8) {
    // Background — single solid fill, nothing layered on top except the
    // glyph / wordmark / field.
    gpu::fill_screen(BG);

    let cx = (w / 2) as i32;
    let cy = (h / 2) as i32;

    // ── Σ glyph (Plex Serif Italic, 96 px) ────────────────────────────
    //
    // Centered horizontally. Vertical position chosen so the glyph
    // baseline sits STACK_ABOVE_CENTER above screen vertical center.
    // truetype::draw_glyph expects (x, y) as the top-left of the glyph
    // bitmap, so we measure the glyph's advance to center it, and
    // approximate its visual height as size_px (close enough for the
    // single-glyph layout — anti-aliased descender / ascender don't
    // matter at this scale).
    let glyph_advance = crate::ui::truetype::glyph_advance(
        0x03A3, // Σ
        crate::ui::truetype::FontFace::PlexSerifItalic,
        GLYPH_SIZE_PX,
    );
    let glyph_x = cx - glyph_advance / 2;
    let glyph_y = cy - STACK_ABOVE_CENTER as i32 - GLYPH_SIZE_PX as i32;
    crate::ui::truetype::draw_glyph(
        fb, w,
        glyph_x, glyph_y,
        0x03A3,
        crate::ui::truetype::FontFace::PlexSerifItalic,
        GLYPH_SIZE_PX,
        INK,
    );

    // ── Wordmark "SPHRAGIS" (Plex Sans Medium, 14 px, letterspaced) ──
    //
    // Plex Sans doesn't have built-in letterspacing — we add it by
    // drawing each character separately with a fixed extra advance.
    // 0.4em letterspacing at 14px = ~5.6 px per gap; round to 6.
    const WORDMARK_LETTERSPACE: i32 = 6;
    let wordmark = "SPHRAGIS";
    let base_advance = crate::ui::truetype::text_advance(
        wordmark,
        crate::ui::truetype::FontFace::PlexSansMedium,
        WORDMARK_SIZE_PX,
    );
    let extra = (wordmark.len() as i32 - 1) * WORDMARK_LETTERSPACE;
    let wordmark_total_w = base_advance + extra;
    let mut wordmark_x = cx - wordmark_total_w / 2;
    let wordmark_y = glyph_y + GLYPH_SIZE_PX as i32 + WORDMARK_GAP as i32;
    for ch in wordmark.chars() {
        let adv = crate::ui::truetype::draw_glyph(
            fb, w,
            wordmark_x, wordmark_y,
            ch as u32,
            crate::ui::truetype::FontFace::PlexSansMedium,
            WORDMARK_SIZE_PX,
            INK,
        );
        wordmark_x += adv + WORDMARK_LETTERSPACE;
    }

    // ── Field panel ──────────────────────────────────────────────────
    let field_x = (w / 2).saturating_sub(FIELD_W / 2);
    let field_y = (wordmark_y as u32) + WORDMARK_SIZE_PX as u32 + FIELD_GAP;
    gpu::fill_rect(field_x, field_y, FIELD_W, FIELD_H, PANEL);
    draw::draw_border(field_x, field_y, FIELD_W, FIELD_H, HAIRLINE);

    // ── Typed dots (Typing / Granted only). Denied paints same as Idle. ──
    let dots: u8 = match state {
        LockState::Typing(n)  => n,
        LockState::Granted(n) => n,
        // Idle + Denied: zero dots — pixel-identical screens.
        _ => 0,
    };
    if dots > 0 {
        // Center the dot strip horizontally inside the field.
        let strip_w = (dots as u32) * DOT_PX + (dots.saturating_sub(1) as u32) * DOT_GAP;
        let dots_x = field_x + (FIELD_W - strip_w) / 2;
        let dots_y = field_y + (FIELD_H - DOT_PX) / 2;
        for i in 0..dots {
            let dx = dots_x + (i as u32) * (DOT_PX + DOT_GAP);
            gpu::fill_rect(dx, dots_y, DOT_PX, DOT_PX, INK);
        }
    }
}
```

- [ ] **Step 5: Update `run()` to clear dots immediately on Enter and use silent cooldowns.**

In `src/security/boot_screen.rs::run()`, find the `match result` block (~line 528). Replace the entire `match result { ... }` block with:

```rust
        match result {
            auth::AuthResult::Success => {
                paint_lock_screen(fb, w, h, LockState::Granted(len as u8), attempts);
                gpu::flush(0, 0, w, h);
                hold_ms(HOLD_GRANTED_MS);
                deadman::refresh();
                return;
            }
            auth::AuthResult::Failed => {
                // Silent denial. Three things in order:
                //   1. Paint Idle *immediately* so dots vanish the instant
                //      Enter is pressed (otherwise an attacker can time
                //      the field reset).
                //   2. Hold for HOLD_DENIED_MS, draining any keystrokes
                //      typed during the cooldown so they don't burst-paint
                //      when input resumes (which would leak the cooldown
                //      end time).
                //   3. Continue the outer loop, which repaints Idle
                //      (harmless redundant paint).
                paint_lock_screen(fb, w, h, LockState::Idle, attempts);
                gpu::flush(0, 0, w, h);

                let freq: u64;
                let start: u64;
                unsafe {
                    core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
                    core::arch::asm!("mrs {}, cntpct_el0", out(reg) start);
                }
                let target_ticks = (freq / 1000).saturating_mul(HOLD_DENIED_MS);
                loop {
                    let now: u64;
                    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) now); }
                    if now.wrapping_sub(start) >= target_ticks { break; }
                    // Drain + discard any keys typed during the cooldown.
                    crate::drivers::virtio::keyboard::poll();
                    crate::drivers::virtio::tablet::poll();
                    let _ = platform::serial_getc()
                        .or_else(crate::drivers::virtio::keyboard::getc)
                        .or_else(crate::drivers::virtio::tablet::getc_key);
                    core::hint::spin_loop();
                }
                continue;
            }
            auth::AuthResult::Duress => {
                fake_boot_and_wipe(fb, w, h);
            }
            auth::AuthResult::LockedOut => {
                // Silent lockout. Clear the dots before starting the wipe
                // for the same reason as Failed — otherwise the final
                // failed attempt is the only one where dots stay frozen
                // on screen, which is itself a signal. Then no banner,
                // straight to wipe, then halt.
                paint_lock_screen(fb, w, h, LockState::Idle, 0);
                gpu::flush(0, 0, w, h);
                wipe::execute(wipe::WipeReason::Lockout, false);
                loop { unsafe { core::arch::asm!("wfe") }; }
            }
        }
```

- [ ] **Step 6: Update `HOLD_GRANTED_MS` from 900 to 200.**

In `src/security/boot_screen.rs`, find the constants:
```rust
const HOLD_DENIED_MS:  u64 = 2500;
const HOLD_GRANTED_MS: u64 = 900;
```
Change the second line to:
```rust
const HOLD_GRANTED_MS: u64 = 200;
```

- [ ] **Step 7: Build and verify everything compiles.**

Run:
```bash
cd /Users/kadenlee/Sphragis
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -12
```
Expected: clean build. Anything that breaks here will be a missed reference — usually a deleted color constant still used somewhere. Fix by either restoring the constant or removing the caller; the spec's kill list is the authority on what stays.

- [ ] **Step 8: Clippy with `-D warnings` to catch dead-code regressions.**

Run:
```bash
SPHRAGIS_PASSPHRASE=sphragis-dev cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -12
```
Expected: clean. The polygon Σ in `src/ui/draw.rs` is already under file-level `#![allow(dead_code)]` so it won't trip dead-code lints. If clippy flags an unused import in `boot_screen.rs` (e.g., we no longer use `crate::ui::font` since the bitmap font is unused on the new screen), remove it.

- [ ] **Step 9: Verify the spec's kill list is complete with `rg`.**

Run:
```bash
cd /Users/kadenlee/Sphragis
rg 'draw_status_pill|draw_corner|draw_boot_log|draw_clock_block|BOOT_LOG' src/security/boot_screen.rs
rg 'CYAN|GREEN|AMBER|RED|HAIR_HI|FAINT|DIM_TXT' src/security/boot_screen.rs
```
Expected: no matches in either command. If any match remains, delete it.

- [ ] **Step 10: Commit.**

```bash
cd /Users/kadenlee/Sphragis
git add src/security/boot_screen.rs
git commit -m "$(cat <<'EOF'
boot_screen: redesign — pure mono, silent denial, silent lockout

Replaces the 'terminal-cyberpunk meets operator-tactical' lock screen
with the Wave 1 design from
docs/superpowers/specs/2026-05-14-lock-screen-redesign-design.md.

Changes:

- Palette: 16 → 5 named colors (BG, PANEL, HAIRLINE, INK, MID).
  No state-color accent — denied state is pixel-identical to idle.
- Layout: deletes status pills, identity strip, version line, KDF label,
  helper hint row, caps indicator, boot log, fake clock, attempts pill,
  crosshair corners, denial overlay, SYSTEM LOCKED banner.
  What's left: Σ glyph (Plex Serif Italic 96px) + SPHRAGIS wordmark
  (Plex Sans Medium 14px, letterspaced) + 480x56 field with typed dots.
- State machine:
    Failed   → 2.5s silent cooldown, no denial paint
    LockedOut → straight to wipe::execute, no banner
    Granted  → 200ms (down from 900ms — the longer hold existed to let
               the user read 'ACCESS GRANTED', which is gone)

The polygon Σ in src/ui/draw.rs stays (file-level #[allow(dead_code)])
until TT rasterization is proven across all surfaces; deletion is a
follow-up.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

### Task 4: Manual QEMU walkthrough of the happy path

Verifies the new design visually. Spec contract: the screen matches Section 2 of the design doc; idle/denied are visually indistinguishable; granted hold is short; lockout has no banner.

**Files:** none modified.

- [ ] **Step 1: Kill any leftover QEMU + rebuild.**

```bash
cd /Users/kadenlee/Sphragis
pkill -9 -f 'qemu-system-aarch64' 2>/dev/null
SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
```
Expected: clean build.

- [ ] **Step 2: Launch QEMU with Cocoa display.**

```bash
qemu-system-aarch64 \
  -machine virt -cpu max -m 2G \
  -display cocoa \
  -device virtio-gpu-device \
  -device virtio-keyboard-device \
  -netdev user,id=net0 \
  -device virtio-net-device,netdev=net0 \
  -serial none \
  -kernel target/aarch64-unknown-none/release/sphragis &
```
Expected: a QEMU window appears within a few seconds and lands on the new lock screen.

- [ ] **Step 3: Verify Idle state matches spec Section 2.**

Confirm by eye:
- Background is near-black (not pure black), warm dark.
- Σ glyph is centered horizontally, italic serif, white on near-black.
- "SPHRAGIS" wordmark is centered below the glyph, sans-serif, letterspaced.
- Field is below the wordmark: a panel with a 1-px hairline border. Empty.
- Nothing else on screen: no top bar, no boot log, no clock, no version, no hint row.

If anything in the kill list is still visible, return to Task 3 and find the missed deletion.

- [ ] **Step 4: Verify Typing state.**

Type 3 random characters (don't press Enter). Confirm:
- Three white 8×8 dots appear inside the field, centered.
- No cursor (no thin vertical line, no caret).
- Field border stays HAIRLINE color — no accent tint.

Backspace once. Confirm:
- Dot count drops to 2.

Backspace twice more. Confirm:
- Dots vanish entirely. Screen is now visually identical to Idle.

- [ ] **Step 5: Verify silent Denied → Idle return.**

Type any 5-char passphrase that ISN'T `sphragis-dev` (e.g. `wrong`). Press Enter. Confirm:
- The dots clear instantly.
- Screen sits in Idle.
- No red overlay, no shake, no microcopy, no color change anywhere.
- For ~2.5 s, keystrokes are ignored: try mashing 5 random keys during the cooldown, *then take your hands off the keyboard*. When the cooldown ends, **no dots burst onto the screen** — the drained input is discarded, not buffered.
- After ~2.5 s, fresh keystrokes start producing dots again.

This is the silent-denial principle in action. The two timing tests (instant clear on Enter, no burst at cooldown end) together prove the attacker has no signal.

- [ ] **Step 6: Verify Granted transition.**

Type `sphragis-dev` (12 chars — 12 dots should appear), press Enter. Confirm:
- Dots stay visible for ~200 ms (briefly noticeable, not a long hold).
- Screen transitions to the desktop (or whatever lives past `boot_screen::run()`).
- No green "ACCESS GRANTED" text appears in the field at any point.

If the hold feels too short, sanity-check by adjusting `HOLD_GRANTED_MS` upward, but 200 ms is the spec target — only deviate if user feedback indicates it's actually unusable.

- [ ] **Step 7: Kill QEMU.**

```bash
pkill -9 -f 'qemu-system-aarch64'
```

---

### Task 5: Manual QEMU walkthrough of the lockout path

Lockout is destructive — it triggers `wipe::execute(WipeReason::Lockout, …)`. Safe in QEMU because each boot starts fresh. The contract this task verifies: no SYSTEM LOCKED banner, no visible signal that lockout was triggered.

**Files:** none modified.

- [ ] **Step 1: Look up the lockout threshold so we know how many wrong attempts to make.**

```bash
cd /Users/kadenlee/Sphragis
grep -rn 'attempts_remaining\|MAX_ATTEMPTS\|attempts.*-\|attempts_left' src/security/auth.rs | head -10
```
Expected: identify the constant (likely 5). If 5, you'll need 5 wrong attempts before the 6th triggers `LockedOut`.

- [ ] **Step 2: Boot QEMU fresh.**

```bash
pkill -9 -f 'qemu-system-aarch64' 2>/dev/null
qemu-system-aarch64 \
  -machine virt -cpu max -m 2G \
  -display cocoa \
  -device virtio-gpu-device \
  -device virtio-keyboard-device \
  -netdev user,id=net0 \
  -device virtio-net-device,netdev=net0 \
  -serial none \
  -kernel target/aarch64-unknown-none/release/sphragis &
```

- [ ] **Step 3: Exhaust attempts.**

Type a wrong passphrase (e.g. `nope`) + Enter. Wait the 2.5 s cooldown. Repeat **N − 1 times** where N is the threshold from Step 1 (likely 5 — so type `nope` and Enter five times total, waiting the cooldown each time).

Confirm at each step: screen returns to Idle silently. No "4 attempts remaining" indicator. No countdown.

- [ ] **Step 4: Trigger lockout with the Nth wrong attempt.**

Type `nope` + Enter one more time. Confirm:
- Screen stays in Idle (or whatever it was before pressing Enter).
- **No red "SYSTEM LOCKED" banner.**
- No green progress bar.
- No state change visible at all.
- Internally, wipe is running. Eventually QEMU may panic or halt — that's expected. The visible contract is "nothing on screen tells the attacker they've triggered the wipe."

- [ ] **Step 5: Kill QEMU.**

```bash
pkill -9 -f 'qemu-system-aarch64'
```

- [ ] **Step 6: No commit (this task is verification only).**

If any of Steps 3–4 surfaced unexpected visual output, return to Task 3 Step 5 (run() rewiring) and confirm the LockedOut arm no longer paints anything.

---

### Task 6: Push the branch and complete

- [ ] **Step 1: Push to origin.**

```bash
cd /Users/kadenlee/Sphragis
git push -u origin feat/lock-screen-redesign
```

- [ ] **Step 2: Use the `superpowers:finishing-a-development-branch` skill.**

That skill will verify tests pass (here: build is clean), then present options for merging back to `main`, opening a PR, keeping as-is, or discarding. The default for this branch is "merge locally back to main" since the changes are self-contained, but the choice is the operator's.

---

## Spec coverage map (self-review)

| Spec section | Implemented by |
|--------------|---------------|
| Goal — quiet Apple-grade lock screen | Task 3 (full paint replacement) |
| Motivation — clutter removal | Task 3 Steps 3, 4, 9 (deletes + grep verification) |
| Motivation — silent denial property | Task 3 Step 5 (Failed arm), Task 4 Step 5 (verification) |
| Palette — 5 named colors | Task 3 Step 1 |
| Typography — IBM Plex Serif Italic + Plex Sans Medium, subsetted, SIL OFL | Task 1 (download + subset + commit), Task 2 (multi-face API) |
| Glyph rendering — TrueType replaces polygon Σ | Task 3 Step 4 (calls `truetype::draw_glyph` instead of `draw::draw_project_glyph_full`) |
| Layout — centered three-element stack | Task 3 Step 4 |
| What is NOT on screen — full kill list | Task 3 Steps 3, 4, 9 |
| State machine — Idle / Typing / Denied / Granted | Task 3 Step 4 (paint), Task 3 Step 5 (run()) |
| Denied = Idle pixel-identical | Task 3 Step 4 (`_` arm in `match state`) |
| Lockout — silent wipe, no banner | Task 3 Step 5 (LockedOut arm) |
| Granted — 200 ms hold | Task 3 Step 6 |
| Duress unchanged | Task 3 Step 5 (Duress arm preserved) |
| Kill list — `draw_status_pill` etc. | Task 3 Step 3 |
| Polygon Σ stays as dead-code-allowed | No-op (file-level `#![allow(dead_code)]` in `draw.rs` already covers it) |
| Driver-level caps state kept intact | No-op (this plan never touches the driver) |
| Verification — boot QEMU, walk every state | Task 4 + Task 5 |

## Out-of-scope reminders

These appeared in the spec's Scope Boundary section. **Do not** drift into them while executing this plan:

- Wave 2 — Desktop chrome (title bar, taskbar, window decorations).
- Wave 3 — Shell + console palette consistency.
- Wave 4 — Widget pack.
- Wave 5 — Cursor blink / scanline overlay (requires a 1 Hz timer hook).
- Polygon Σ deletion in `src/ui/draw.rs`.
- Removing `fonts/font.ttf` (Verdana) — a follow-up after the rasterizer's legacy single-font path is dropped.

If you find yourself touching any of these, stop and ask. The point of the Wave system is that each wave produces a working, testable change on its own — drifting kills that property.

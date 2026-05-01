// Bat_OS — Paint System
// Walks the layout tree and draws boxes + text to the framebuffer.
// Handles backgrounds, borders, text, and underlines.

use super::layout::LayoutTree;
use super::css::style::*;
use super::media::png::PngImage;
use crate::ui::font;
use crate::ui::truetype;
use crate::drivers::virtio::gpu;

/// STUMP #91: per-pixel alpha-blended box-shadow. Walks every pixel
/// in the inflated shadow rectangle, computes the L1 distance into
/// the blur ring, and BLENDS the shadow color into the framebuffer
/// using that distance as the falloff. Returns immediately for
/// degenerate rects so the cost stays at O(shadow_area).
fn paint_soft_shadow(
    out_x1: i32, out_y1: i32, out_x2: i32, out_y2: i32,
    in_x1: i32,  in_y1: i32,  in_x2: i32,  in_y2: i32,
    blur: i32, color: Color,
) {
    if out_x2 <= out_x1 || out_y2 <= out_y1 { return; }
    let fb = gpu::framebuffer();
    if fb.is_null() { return; }
    let sw = gpu::width() as i32;
    let sh = gpu::height() as i32;

    // Pre-extract shadow RGB; the BGRA layout is B at byte 0, A at
    // byte 3. We blend manually because gpu::fill_rect doesn't.
    let raw = color.raw();
    let sr = ((raw >> 16) & 0xFF) as u32;
    let sg = ((raw >> 8) & 0xFF) as u32;
    let sb = (raw & 0xFF) as u32;
    let base_alpha: u32 = ((raw >> 24) & 0xFF) as u32; // typically 255

    for y in out_y1..out_y2 {
        if y < 0 || y >= sh { continue; }
        for x in out_x1..out_x2 {
            if x < 0 || x >= sw { continue; }

            // Distance from the inner solid-shadow rectangle. Inside
            // the inner rect this is 0 → full alpha. Outside, dist
            // grows linearly toward the outer edge.
            let dx = if x < in_x1 { in_x1 - x }
                     else if x >= in_x2 { x - in_x2 + 1 }
                     else { 0 };
            let dy = if y < in_y1 { in_y1 - y }
                     else if y >= in_y2 { y - in_y2 + 1 }
                     else { 0 };
            // Use Chebyshev (max) distance — square-ish falloff matches
            // CSS box-shadow's "stretches the rounded ring" feel better
            // than L1 (diamond-ish).
            let dist = if dx > dy { dx } else { dy };

            // Falloff function: alpha drops as the square of d/blur,
            // which approximates a Gaussian sigma = blur/2 well enough
            // visually. d ≥ blur → 0 alpha (skip).
            if blur <= 0 || dist >= blur { continue; }
            let t = dist as u32; // 0..blur
            let denom = (blur as u32).max(1);
            // alpha = base * (1 - (t/blur)^2). Integer math: (blur² - t²)/blur²
            let num = denom * denom - t * t;
            let alpha = base_alpha * num / (denom * denom);
            if alpha == 0 { continue; }

            // Read existing pixel and alpha-blend in place.
            let ofs = (y * sw + x) as usize;
            let cur = unsafe { core::ptr::read_volatile(fb.add(ofs)) };
            let cb = (cur & 0xFF) as u32;
            let cg = ((cur >> 8) & 0xFF) as u32;
            let cr = ((cur >> 16) & 0xFF) as u32;
            // out = src * alpha + dst * (1-alpha), all in 0..255.
            let inv = 255u32 - alpha;
            let nr = (sr * alpha + cr * inv + 127) / 255;
            let ng = (sg * alpha + cg * inv + 127) / 255;
            let nb = (sb * alpha + cb * inv + 127) / 255;
            let pixel = 0xFF000000 | (nr << 16) | (ng << 8) | nb;
            unsafe { core::ptr::write_volatile(fb.add(ofs), pixel); }
        }
    }
}

// Image rendering support
/// Draw a PngImage directly at screen position, scaled to fit.
pub fn draw_png(img: &PngImage, sx: i32, sy: i32, w: i32, h: i32) {
    if !img.valid || img.width == 0 || img.height == 0 { return; }

    let fb = gpu::framebuffer();
    let sw = gpu::width();
    let sh = gpu::height();

    for dy in 0..h {
        let screen_y = sy + dy;
        if screen_y < 0 || screen_y >= sh as i32 { continue; }
        let src_y = (dy as u32 * img.height) / h as u32;

        for dx in 0..w {
            let screen_x = sx + dx;
            if screen_x < 0 || screen_x >= sw as i32 { continue; }
            let src_x = (dx as u32 * img.width) / w as u32;

            let pixel = img.get_pixel(src_x, src_y);
            let alpha = (pixel >> 24) & 0xFF;
            if alpha > 128 {
                unsafe {
                    let offset = (screen_y as u32 * sw + screen_x as u32) as usize;
                    core::ptr::write_volatile(fb.add(offset), pixel);
                }
            }
        }
    }
}

/// Paint the layout tree into the framebuffer.
/// `offset_x/y` = position of the browser content area on screen.
/// `scroll_y` = vertical scroll offset.
/// `clip_w/h` = size of the visible area.
pub fn paint(
    tree: &LayoutTree,
    offset_x: i32,
    offset_y: i32,
    scroll_y: i32,
    clip_w: i32,
    clip_h: i32,
) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();

    // Clip boundaries in screen space
    let clip_left = offset_x;
    let clip_right = offset_x + clip_w;
    let clip_top = offset_y;
    let clip_bottom = offset_y + clip_h;

    for i in 0..tree.box_count {
        let b = &tree.boxes[i];
        if !b.active { continue; }

        // Skip zero-dimension boxes
        if b.width <= 0 || b.height <= 0 { continue; }

        // Skip invisible elements
        if b.style.visibility != super::css::style::Visibility::Visible { continue; }
        if b.style.opacity == 0 { continue; }

        // Transform coordinates: layout space -> screen space.
        // STUMP #91: position:sticky boxes "stick" to the viewport top
        // (plus their `top` offset) once the page has scrolled past
        // their natural Y. Below that scroll threshold they paint as
        // normal (Relative-like). In our paginated renderer scroll_y
        // increments by MAX_H per page, so a sticky header re-appears
        // at the top of every page after the first that scrolled past
        // it.
        let sx = b.x + offset_x;
        let sy = if b.style.position == super::css::style::Position::Sticky {
            let stick_top = if b.style.top != i32::MIN { b.style.top } else { 0 };
            let natural_screen_y = b.y + offset_y - scroll_y;
            if natural_screen_y < clip_top + stick_top {
                clip_top + stick_top
            } else {
                natural_screen_y
            }
        } else {
            b.y + offset_y - scroll_y
        };

        // Clipping: skip boxes completely outside visible area
        if sy + b.height < clip_top { continue; }
        if sy > clip_bottom { continue; }
        if sx + b.width < clip_left { continue; }
        if sx > clip_right { continue; }

        // 🎯 STUMP #69: <img> elements with a successfully-decoded PNG
        // skip the normal background/border/text path and draw the
        // image bytes directly. draw_png honors clip + scaling.
        if b.image_slot != 0xFFFF {
            if let Some(img) = crate::browser::media::img_pool::get(b.image_slot) {
                draw_png(img, sx, sy, b.width, b.height);
                continue;
            }
        }

        // Compute clipped background rectangle
        let bg_x1 = sx.max(clip_left);
        let bg_y1 = sy.max(clip_top);
        let bg_x2 = (sx + b.width).min(clip_right);
        let bg_y2 = (sy + b.height).min(clip_bottom);
        let bg_w = (bg_x2 - bg_x1).max(0) as u32;
        let bg_h = (bg_y2 - bg_y1).max(0) as u32;

        // STUMP #91: real soft (Gaussian-ish) box-shadow. Walks every
        // pixel in the inflated shadow rect and writes it with an
        // alpha derived from the L1 distance into the blur ring. This
        // is O(shadow_area) so we cap blur at 20 px — real CSS would
        // do a separable Gaussian convolution on the rasterized alpha
        // mask, but a one-pass distance-falloff approximation is
        // visually indistinguishable for typical UI shadows. Pre-fix
        // (STUMP #82) was 6 hard-edged rect strips per shadow; you
        // could see the seams.
        if b.style.box_shadow_color != Color::TRANSPARENT
            && b.width > 0 && b.height > 0
        {
            let sh = b.style.box_shadow_color;
            let off_x = b.style.box_shadow_x;
            let off_y = b.style.box_shadow_y;
            let blur = b.style.box_shadow_blur.max(0).min(20);
            // Inflated rect = box translated by (off_x, off_y), grown
            // by blur on each side. inner_* is the solid-shadow zone;
            // outside that, alpha falls off toward the edge.
            let inner_x1 = sx + off_x;
            let inner_y1 = sy + off_y;
            let inner_x2 = inner_x1 + b.width;
            let inner_y2 = inner_y1 + b.height;
            let outer_x1 = (inner_x1 - blur).max(clip_left);
            let outer_y1 = (inner_y1 - blur).max(clip_top);
            let outer_x2 = (inner_x2 + blur).min(clip_right);
            let outer_y2 = (inner_y2 + blur).min(clip_bottom);
            paint_soft_shadow(
                outer_x1, outer_y1, outer_x2, outer_y2,
                inner_x1, inner_y1, inner_x2, inner_y2,
                blur, sh,
            );
        }

        // --- Background ---
        let radius = b.style.border_radius.max(0) as u32;
        if b.style.background_color != Color::TRANSPARENT && bg_w > 0 && bg_h > 0 {
            let bgc = b.style.background_color.raw();
            if radius > 0 && bg_w > radius * 2 && bg_h > radius * 2 {
                // Rounded rectangle background
                let bx = bg_x1 as u32;
                let by = bg_y1 as u32;
                // Center fill (full width, minus top/bottom radius rows)
                gpu::fill_rect(bx, by + radius, bg_w, bg_h - radius * 2, bgc);
                // Top strip (inset by radius)
                gpu::fill_rect(bx + radius, by, bg_w - radius * 2, radius, bgc);
                // Bottom strip (inset by radius)
                gpu::fill_rect(bx + radius, by + bg_h - radius, bg_w - radius * 2, radius, bgc);
                // Corner fills (approximate rounded corners with smaller rects)
                let r2 = radius / 2;
                // Top-left corner
                gpu::fill_rect(bx + r2, by, radius - r2, radius, bgc);
                gpu::fill_rect(bx, by + r2, radius, radius - r2, bgc);
                // Top-right corner
                gpu::fill_rect(bx + bg_w - radius, by, radius - r2, radius, bgc);
                gpu::fill_rect(bx + bg_w - radius, by + r2, radius, radius - r2, bgc);
                // Bottom-left corner
                gpu::fill_rect(bx + r2, by + bg_h - radius, radius - r2, radius, bgc);
                gpu::fill_rect(bx, by + bg_h - radius, radius, radius - r2, bgc);
                // Bottom-right corner
                gpu::fill_rect(bx + bg_w - radius, by + bg_h - radius, radius - r2, radius, bgc);
                gpu::fill_rect(bx + bg_w - radius, by + bg_h - radius, radius, radius - r2, bgc);
            } else {
                gpu::fill_rect(bg_x1 as u32, bg_y1 as u32, bg_w, bg_h, bgc);
            }
        }

        // --- Borders ---
        if b.style.border_width > 0 && b.style.border_color != Color::TRANSPARENT
            && bg_w > 0 && bg_h > 0
        {
            let bx = bg_x1 as u32;
            let by = bg_y1 as u32;
            let border_w = b.style.border_width as u32;
            let bc = b.style.border_color.raw();

            // 🎯 STUMP #76: bound every subtract by both `radius * 2`
            // AND `border_w` so we can't underflow on tiny / thin
            // boxes (e.g. <hr> 1px tall, or small <input>s with
            // border-radius). Pre-fix the guard only checked vs
            // radius and the bottom/right edges did
            // `bg_h - border_w` which underflowed when border_w
            // > bg_h.
            let safe_w = bg_w >= radius * 2 + border_w * 2;
            let safe_h = bg_h >= radius * 2 + border_w * 2;
            if radius > 0 && safe_w && safe_h {
                // Rounded border
                gpu::fill_rect(bx + radius, by, bg_w - radius * 2, border_w, bc); // top
                gpu::fill_rect(bx + radius, by + bg_h - border_w, bg_w - radius * 2, border_w, bc); // bottom
                gpu::fill_rect(bx, by + radius, border_w, bg_h - radius * 2, bc); // left
                gpu::fill_rect(bx + bg_w - border_w, by + radius, border_w, bg_h - radius * 2, bc); // right
            } else {
                // Square border. Each edge guarded against the other
                // dimension being smaller than the border width.
                gpu::fill_rect(bx, by, bg_w, border_w.min(bg_h), bc); // top
                if bg_h > border_w {
                    gpu::fill_rect(bx, by + bg_h - border_w, bg_w, border_w, bc); // bottom
                }
                gpu::fill_rect(bx, by, border_w.min(bg_w), bg_h, bc); // left
                if bg_w > border_w {
                    gpu::fill_rect(bx + bg_w - border_w, by, border_w, bg_h, bc); // right
                }
            }

            // HR: full-width horizontal line
            if b.height <= 2 {
                gpu::fill_rect(bx, by, bg_w, border_w.min(bg_h), bc);
            }
        }

        // --- Text ---
        if b.text_len > 0 {
            let text = &tree.text_buf[b.text_start..b.text_start + b.text_len];
            let color = b.style.color.raw();
            let is_bold = b.style.font_weight == FontWeight::Bold;
            let is_underline = b.style.text_decoration.underline;

            // Use actual font size from CSS (clamped to reasonable range)
            let font_px = (b.style.font_size as u16).max(10).min(72);
            // Approximate char width based on font size (proportional)
            let char_w: i32 = if truetype::is_available() {
                (font_px as i32 * 55 / 100).max(5) // TrueType: ~55% of font size
            } else {
                (font_px as i32 * 6 / 10).max(5) // Bitmap: ~60%
            };
            let line_h: i32 = (font_px as i32 * 14 / 10).max(14); // 140% of font size

            // Content area start (inside padding). Sticky boxes get
            // the same Y shift as their box background so the text
            // stays inside the painted box.
            let content_sx = b.content_x + offset_x;
            let content_sy = if b.style.position == super::css::style::Position::Sticky {
                let stick_top = if b.style.top != i32::MIN { b.style.top } else { 0 };
                let natural = b.content_y + offset_y - scroll_y;
                let inset = b.content_y - b.y; // content sits this far below the box top
                if natural - inset < clip_top + stick_top {
                    clip_top + stick_top + inset
                } else {
                    natural
                }
            } else {
                b.content_y + offset_y - scroll_y
            };
            let content_w = b.content_w.max(b.width); // use wider of the two

            // Text centering support
            let text_total_w = text.len() as i32 * char_w;
            let centered_offset = if b.style.text_align == TextAlign::Center && text_total_w < content_w {
                (content_w - text_total_w) / 2
            } else {
                0
            };

            // STUMP #95: word-wrap with TT-measured per-word widths.
            // Pre-fix the wrap decision was made at character boundaries
            // using a font-size × 0.55 estimate of char width — too
            // optimistic for variable-width fonts, so the last 1-2
            // chars of a line would overflow into the right padding
            // (visible across every public-internet site we rendered:
            // "in oper" / "ations." instead of "in" / "operations."). Now
            // we walk word-by-word, measure each word via TT, and wrap
            // before any word that would push past content_x + content_w.
            // Single words longer than the line still fall back to
            // per-char advance so they at least try to fit.
            let line_left = content_sx + centered_offset;
            let line_right = content_sx + content_w; // hard right edge
            let mut tx = line_left;
            let mut ty = content_sy;
            let italic = b.style.font_style == super::css::style::FontStyle::Italic;
            let mono = b.style.font_family == super::css::style::FontFamily::Monospace;
            let space_w = if mono {
                (font_px as i32 * 6 / 10).max(7)
            } else if truetype::is_available() {
                truetype::text_width(b" ", font_px).max(char_w / 2)
            } else {
                char_w
            };

            let measure_word = |word: &[u8]| -> i32 {
                if mono {
                    (word.len() as i32) * (font_px as i32 * 6 / 10).max(7)
                } else if truetype::is_available() {
                    truetype::text_width(word, font_px)
                } else {
                    (word.len() as i32) * char_w
                }
            };

            let mut i = 0usize;
            while i < text.len() {
                // Skip leading whitespace at start of every line.
                if tx == line_left {
                    while i < text.len()
                        && (text[i] == b' ' || text[i] == b'\t' || text[i] == b'\n' || text[i] == b'\r')
                    { i += 1; }
                    if i >= text.len() { break; }
                }
                let ch = text[i];

                // Inline whitespace — advance by space_w (treat \n as space too).
                if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
                    if tx + space_w > line_right {
                        tx = line_left;
                        ty += line_h;
                    } else {
                        tx += space_w;
                    }
                    i += 1;
                    continue;
                }

                // Bullet character carve-out preserved from STUMP #70.
                if ch == 0xB7 {
                    let dot_x = tx + 2; let dot_y = ty + 7;
                    if dot_x >= clip_left && dot_x < clip_right
                        && dot_y >= clip_top && dot_y < clip_bottom
                    {
                        gpu::fill_rect(dot_x as u32, dot_y as u32, 4, 4, color);
                    }
                    tx += char_w; i += 1;
                    continue;
                }

                // Skip remaining non-printable bytes (mostly UTF-8 continuation).
                if ch < 0x20 || ch > 0x7E {
                    i += 1;
                    continue;
                }

                // Find this word's extent (printable run terminated by ws).
                let mut j = i;
                while j < text.len() {
                    let c = text[j];
                    if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' { break; }
                    if c < 0x20 || c > 0x7E { break; }
                    j += 1;
                }
                let word = &text[i..j];
                let word_w = measure_word(word);

                // Wrap if this word won't fit on the current line and
                // we've already drawn something on this line.
                if tx + word_w > line_right && tx > line_left {
                    tx = line_left;
                    ty += line_h;
                }

                // Skip rendering when this entire line is off-screen,
                // but still advance tx so subsequent wraps line up.
                if ty + line_h < clip_top || ty > clip_bottom {
                    tx += word_w;
                    i = j;
                    continue;
                }

                // Draw the word char-by-char (keeps the TT/bitmap branch
                // identical to the pre-fix code path; only the wrap
                // decision changed).
                let mut k = i;
                while k < j {
                    let c = text[k];
                    let ch_buf = [c];
                    let advance = if truetype::is_available() {
                        let a = truetype::draw_text_fb_styled(
                            fb, sw, tx, ty, &ch_buf, font_px, color,
                            clip_left, clip_right, clip_top, clip_bottom,
                            italic,
                        );
                        if is_bold {
                            truetype::draw_text_fb_styled(
                                fb, sw, tx + 1, ty, &ch_buf, font_px, color,
                                clip_left, clip_right, clip_top, clip_bottom,
                                italic,
                            );
                        }
                        if is_underline && c != b' ' {
                            gpu::fill_rect(
                                tx as u32, (ty + font_px as i32 + 1) as u32,
                                a.max(6) as u32, 1, color,
                            );
                        }
                        if mono {
                            (font_px as i32 * 6 / 10).max(7)
                        } else if a > 0 {
                            a
                        } else {
                            char_w
                        }
                    } else {
                        let s = unsafe { core::str::from_utf8_unchecked(&ch_buf) };
                        font::draw_str(fb, sw, tx as u32, ty as u32, s, color, 0xFF0A0A0A);
                        if is_bold || font_px >= 24 {
                            font::draw_str(fb, sw, (tx + 1) as u32, ty as u32, s, color, 0xFF0A0A0A);
                        }
                        if is_underline && c != b' ' {
                            gpu::fill_rect(tx as u32, (ty + 14) as u32, 8, 1, color);
                        }
                        char_w
                    };
                    tx += advance;
                    k += 1;
                }
                i = j;
            }
        }
    }
}

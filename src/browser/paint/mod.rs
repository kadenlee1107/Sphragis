// Bat_OS — Paint System
// Walks the layout tree and draws boxes + text to the framebuffer.
// Handles backgrounds, borders, text, and underlines.

use super::layout::LayoutTree;
use super::css::style::*;
use super::media::png::PngImage;
use crate::ui::font;
use crate::ui::truetype;
use crate::drivers::virtio::gpu;

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

        // Transform coordinates: layout space -> screen space
        let sx = b.x + offset_x;
        let sy = b.y + offset_y - scroll_y;

        // Clipping: skip boxes completely outside visible area
        if sy + b.height < clip_top { continue; }
        if sy > clip_bottom { continue; }
        if sx + b.width < clip_left { continue; }
        if sx > clip_right { continue; }

        // Compute clipped background rectangle
        let bg_x1 = sx.max(clip_left);
        let bg_y1 = sy.max(clip_top);
        let bg_x2 = (sx + b.width).min(clip_right);
        let bg_y2 = (sy + b.height).min(clip_bottom);
        let bg_w = (bg_x2 - bg_x1).max(0) as u32;
        let bg_h = (bg_y2 - bg_y1).max(0) as u32;

        // --- Background ---
        if b.style.background_color != Color::TRANSPARENT && bg_w > 0 && bg_h > 0 {
            gpu::fill_rect(bg_x1 as u32, bg_y1 as u32, bg_w, bg_h,
                b.style.background_color.raw());
        }

        // --- Borders ---
        if b.style.border_width > 0 && b.style.border_color != Color::TRANSPARENT
            && bg_w > 0 && bg_h > 0
        {
            let bx = bg_x1 as u32;
            let by = bg_y1 as u32;
            let border_w = b.style.border_width as u32;
            let bc = b.style.border_color.raw();

            // Top border
            gpu::fill_rect(bx, by, bg_w, border_w.min(bg_h), bc);
            // Bottom border
            if bg_h > border_w {
                gpu::fill_rect(bx, by + bg_h - border_w, bg_w, border_w, bc);
            }
            // Left border
            gpu::fill_rect(bx, by, border_w.min(bg_w), bg_h, bc);
            // Right border
            if bg_w > border_w {
                gpu::fill_rect(bx + bg_w - border_w, by, border_w, bg_h, bc);
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
            let is_big = b.style.font_size >= 28;
            let char_w: i32 = if is_big { 10 } else { 8 };
            let line_h: i32 = 18;

            // Content area start (inside padding)
            let content_sx = b.content_x + offset_x;
            let content_sy = b.content_y + offset_y - scroll_y;
            let content_w = b.content_w.max(b.width); // use wider of the two

            // Compute how many chars fit per line
            let max_chars_per_line = if content_w > 0 { (content_w / char_w).max(1) } else { 80 };

            let mut tx = content_sx;
            let mut ty = content_sy;
            let mut col = 0i32; // character column on current line

            for &ch in text {
                // Wrap to next line if we've exceeded the line width
                if col >= max_chars_per_line {
                    col = 0;
                    tx = content_sx;
                    ty += line_h;
                }

                // Skip rendering if this line is outside clip region
                if ty + line_h < clip_top || ty > clip_bottom {
                    col += 1;
                    tx += char_w;
                    continue;
                }

                if ch < 0x20 || ch > 0x7E {
                    if ch == 0xB7 {
                        // Bullet character -> draw as a small dot
                        let dot_x = tx + 2;
                        let dot_y = ty + 7;
                        if dot_x >= clip_left && dot_x < clip_right
                            && dot_y >= clip_top && dot_y < clip_bottom
                        {
                            gpu::fill_rect(dot_x as u32, dot_y as u32, 4, 4, color);
                        }
                        tx += char_w;
                        col += 1;
                    }
                    continue;
                }

                if tx >= clip_left && tx < clip_right {
                    if truetype::is_available() {
                        // Anti-aliased TrueType rendering
                        let font_px = if is_big { 24u16 } else if is_bold { 16 } else { 14 };
                        let ch_buf = [ch];
                        let advance = truetype::draw_text_fb(
                            fb, sw, tx, ty, &ch_buf, font_px, color,
                            clip_left, clip_right, clip_top, clip_bottom,
                        );

                        // Bold: draw again offset by 1px
                        if is_bold {
                            truetype::draw_text_fb(
                                fb, sw, tx + 1, ty, &ch_buf, font_px, color,
                                clip_left, clip_right, clip_top, clip_bottom,
                            );
                        }

                        // Underline
                        if is_underline && ch != b' ' {
                            gpu::fill_rect(tx as u32, (ty + font_px as i32 + 1) as u32,
                                advance.max(6) as u32, 1, color);
                        }

                        tx += if advance > 0 { advance } else { char_w };
                    } else {
                        // Fallback: monospace bitmap font
                        let ch_buf = [ch];
                        let s = unsafe { core::str::from_utf8_unchecked(&ch_buf) };
                        font::draw_str(fb, sw, tx as u32, ty as u32, s, color, 0xFF0A0A0A);
                        if is_bold || is_big {
                            font::draw_str(fb, sw, (tx + 1) as u32, ty as u32, s, color, 0xFF0A0A0A);
                        }
                        if is_underline && ch != b' ' {
                            gpu::fill_rect(tx as u32, (ty + 14) as u32, 8, 1, color);
                        }
                        tx += char_w;
                    }
                }
                col += 1;
            }
        }
    }
}

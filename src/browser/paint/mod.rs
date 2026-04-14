// Bat_OS — Paint System
// Walks the layout tree and draws boxes + text to the framebuffer.
// Handles backgrounds, borders, text, and underlines.

use super::layout::LayoutTree;
use super::css::style::*;
use super::media::png::PngImage;
use crate::ui::font;
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

    for i in 0..tree.box_count {
        let b = &tree.boxes[i];
        if !b.active { continue; }

        // Transform coordinates: layout space → screen space
        let sx = b.x + offset_x;
        let sy = b.y + offset_y - scroll_y;

        // Clipping: skip boxes outside visible area
        if sy + b.height < offset_y { continue; }
        if sy > offset_y + clip_h { continue; }
        if sx + b.width < offset_x { continue; }
        if sx > offset_x + clip_w { continue; }

        // ─── Background ───
        if b.style.background_color != Color::TRANSPARENT {
            let bx = sx.max(offset_x) as u32;
            let by = sy.max(offset_y) as u32;
            let bw = b.width.min(clip_w) as u32;
            let bh = b.height.min(clip_h) as u32;
            if bw > 0 && bh > 0 {
                gpu::fill_rect(bx, by, bw, bh, b.style.background_color.raw());
            }
        }

        // ─── Left border (for blockquote) ───
        if b.style.border_width > 0 && b.style.border_color != Color::TRANSPARENT {
            let bx = sx as u32;
            let by = sy.max(offset_y) as u32;
            let bh = b.height.min(clip_h) as u32;
            gpu::fill_rect(bx, by, b.style.border_width as u32, bh, b.style.border_color.raw());

            // HR: draw horizontal line
            if b.style.display == Display::Block && b.height == 0 {
                gpu::fill_rect(bx, by, b.width as u32, 1, b.style.border_color.raw());
            }
        }

        // ─── Text ───
        if b.text_len > 0 {
            let text = &tree.text_buf[b.text_start..b.text_start + b.text_len];
            let color = b.style.color.raw();
            let is_bold = b.style.font_weight == FontWeight::Bold;
            let is_underline = b.style.text_decoration.underline;
            let is_big = b.style.font_size >= 28;
            let char_w: i32 = if is_big { 10 } else { 8 }; // h1 slightly wider

            let mut tx = sx;
            let ty = sy;

            if ty >= offset_y && ty < offset_y + clip_h {
                for &ch in text {
                    if ch < 0x20 || ch > 0x7E {
                        if ch == 0xB7 {
                            // Bullet character → draw as a small dot
                            let dot_x = (tx + 2) as u32;
                            let dot_y = (ty + 7) as u32;
                            gpu::fill_rect(dot_x, dot_y, 4, 4, color);
                            tx += char_w as i32;
                            continue;
                        }
                        continue;
                    }

                    if tx >= offset_x && tx < offset_x + clip_w {
                        let ch_buf = [ch];
                        let s = unsafe { core::str::from_utf8_unchecked(&ch_buf) };

                        // Draw character
                        font::draw_str(fb, sw, tx as u32, ty as u32, s, color, 0xFF0A0A0A);

                        // Bold or big: draw again offset by 1px
                        if is_bold || is_big {
                            font::draw_str(fb, sw, (tx + 1) as u32, ty as u32, s, color, 0xFF0A0A0A);
                        }

                        // Underline
                        if is_underline && ch != b' ' {
                            gpu::fill_rect(tx as u32, (ty + 14) as u32, 8, 1, color);
                        }
                    }

                    tx += char_w as i32;
                }
            }
        }
    }
}

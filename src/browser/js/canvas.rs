#![allow(dead_code)]
// Bat_OS — HTML Canvas 2D API
// Implements the CanvasRenderingContext2D interface.
// Allows JavaScript to draw directly to a pixel buffer.

use crate::drivers::virtio::gpu;

pub const CANVAS_MAX_W: usize = 512;
pub const CANVAS_MAX_H: usize = 512;

pub struct Canvas2D {
    pub width: u32,
    pub height: u32,
    pub pixels: [u32; CANVAS_MAX_W * CANVAS_MAX_H],
    // Drawing state
    pub fill_color: u32,
    pub stroke_color: u32,
    pub line_width: i32,
    pub font_size: i32,
    // Transform (simplified — translation only)
    pub translate_x: i32,
    pub translate_y: i32,
    // Path
    pub path_x: [i32; 256],
    pub path_y: [i32; 256],
    pub path_len: usize,
    pub pen_x: i32,
    pub pen_y: i32,
}

impl Canvas2D {
    pub const fn new() -> Self {
        Canvas2D {
            width: 0, height: 0,
            pixels: [0xFF000000; CANVAS_MAX_W * CANVAS_MAX_H],
            fill_color: 0xFF000000,
            stroke_color: 0xFFFFFFFF,
            line_width: 1,
            font_size: 16,
            translate_x: 0, translate_y: 0,
            path_x: [0; 256], path_y: [0; 256],
            path_len: 0,
            pen_x: 0, pen_y: 0,
        }
    }

    pub fn init(&mut self, w: u32, h: u32) {
        self.width = w.min(CANVAS_MAX_W as u32);
        self.height = h.min(CANVAS_MAX_H as u32);
        let total = (self.width * self.height) as usize;
        for i in 0..total {
            self.pixels[i] = 0xFF000000; // black
        }
    }

    // ─── Drawing primitives ───

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let tx = x + self.translate_x;
        let ty = y + self.translate_y;
        for dy in 0..h {
            for dx in 0..w {
                self.set_pixel(tx + dx, ty + dy, self.fill_color);
            }
        }
    }

    pub fn stroke_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let tx = x + self.translate_x;
        let ty = y + self.translate_y;
        // Top + bottom
        for dx in 0..w {
            self.set_pixel(tx + dx, ty, self.stroke_color);
            self.set_pixel(tx + dx, ty + h - 1, self.stroke_color);
        }
        // Left + right
        for dy in 0..h {
            self.set_pixel(tx, ty + dy, self.stroke_color);
            self.set_pixel(tx + w - 1, ty + dy, self.stroke_color);
        }
    }

    pub fn clear_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let saved = self.fill_color;
        self.fill_color = 0x00000000; // transparent
        self.fill_rect(x, y, w, h);
        self.fill_color = saved;
    }

    // ─── Path operations ───

    pub fn begin_path(&mut self) {
        self.path_len = 0;
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        self.pen_x = x + self.translate_x;
        self.pen_y = y + self.translate_y;
    }

    pub fn line_to(&mut self, x: i32, y: i32) {
        let tx = x + self.translate_x;
        let ty = y + self.translate_y;
        self.draw_line(self.pen_x, self.pen_y, tx, ty, self.stroke_color);
        self.pen_x = tx;
        self.pen_y = ty;

        if self.path_len < 256 {
            self.path_x[self.path_len] = tx;
            self.path_y[self.path_len] = ty;
            self.path_len += 1;
        }
    }

    pub fn stroke(&mut self) {
        // Path already drawn by line_to
    }

    pub fn fill(&mut self) {
        // Simplified fill — just draw the path outline with fill color
        let saved = self.stroke_color;
        self.stroke_color = self.fill_color;
        for i in 1..self.path_len {
            self.draw_line(
                self.path_x[i-1], self.path_y[i-1],
                self.path_x[i], self.path_y[i],
                self.fill_color,
            );
        }
        self.stroke_color = saved;
    }

    // ─── Circle / Arc ───

    pub fn arc(&mut self, cx: i32, cy: i32, radius: i32, _start: f32, _end: f32) {
        let tcx = cx + self.translate_x;
        let tcy = cy + self.translate_y;
        // Simple circle drawing (Midpoint circle algorithm)
        let mut x = radius;
        let mut y = 0i32;
        let mut err = 0i32;

        while x >= y {
            self.set_pixel(tcx + x, tcy + y, self.stroke_color);
            self.set_pixel(tcx + y, tcy + x, self.stroke_color);
            self.set_pixel(tcx - y, tcy + x, self.stroke_color);
            self.set_pixel(tcx - x, tcy + y, self.stroke_color);
            self.set_pixel(tcx - x, tcy - y, self.stroke_color);
            self.set_pixel(tcx - y, tcy - x, self.stroke_color);
            self.set_pixel(tcx + y, tcy - x, self.stroke_color);
            self.set_pixel(tcx + x, tcy - y, self.stroke_color);

            if err <= 0 {
                y += 1;
                err += 2 * y + 1;
            }
            if err > 0 {
                x -= 1;
                err -= 2 * x + 1;
            }
        }
    }

    // ─── Text ───

    pub fn fill_text(&mut self, text: &str, x: i32, y: i32) {
        // Use our monospace font to draw text on the canvas
        let tx = x + self.translate_x;
        let ty = y + self.translate_y;
        // For now, just mark the position (real text rendering needs font integration)
        for (i, _ch) in text.bytes().enumerate() {
            let cx = tx + (i as i32) * 8;
            // Simple 1-pixel character placeholder
            for dy in 0..12 {
                self.set_pixel(cx + 2, ty + dy, self.fill_color);
                self.set_pixel(cx + 3, ty + dy, self.fill_color);
            }
        }
    }

    // ─── Transform ───

    pub fn translate(&mut self, x: i32, y: i32) {
        self.translate_x += x;
        self.translate_y += y;
    }

    pub fn save(&self) -> (i32, i32) {
        (self.translate_x, self.translate_y)
    }

    pub fn restore(&mut self, state: (i32, i32)) {
        self.translate_x = state.0;
        self.translate_y = state.1;
    }

    // ─── Color parsing ───

    pub fn set_fill_style(&mut self, color: &str) {
        self.fill_color = parse_color(color);
    }

    pub fn set_stroke_style(&mut self, color: &str) {
        self.stroke_color = parse_color(color);
    }

    // ─── Helpers ───

    fn set_pixel(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            self.pixels[(y as u32 * self.width + x as u32) as usize] = color;
        }
    }

    fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
        // Bresenham's line algorithm
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx: i32 = if x0 < x1 { 1 } else { -1 };
        let sy: i32 = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;

        loop {
            self.set_pixel(x, y, color);
            if x == x1 && y == y1 { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x += sx; }
            if e2 <= dx { err += dx; y += sy; }
        }
    }

    /// Blit canvas to screen framebuffer at given position
    pub fn blit_to_screen(&self, screen_x: u32, screen_y: u32) {
        let fb = gpu::framebuffer();
        let sw = gpu::width();
        let sh = gpu::height();

        for cy in 0..self.height {
            let sy = screen_y + cy;
            if sy >= sh { break; }
            for cx in 0..self.width {
                let sx = screen_x + cx;
                if sx >= sw { continue; }
                let pixel = self.pixels[(cy * self.width + cx) as usize];
                if pixel & 0xFF000000 != 0 { // not transparent
                    unsafe {
                        core::ptr::write_volatile(
                            fb.add((sy * sw + sx) as usize),
                            pixel,
                        );
                    }
                }
            }
        }
    }
}

fn parse_color(s: &str) -> u32 {
    use crate::browser::css::style::Color;
    Color::parse(s).raw()
}

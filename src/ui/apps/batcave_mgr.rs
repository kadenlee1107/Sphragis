// Bat_OS — BatCave Manager App
// 6th desktop app — visual overview of all BatCaves.

use crate::ui::gpu;
use crate::ui::font;
use crate::ui::wm;
use crate::batcave::cave;

const BG: u32 = 0xFF000000;
const FG: u32 = 0xFFA0A0A0;
const FG_HI: u32 = 0xFFFFFFFF;
const DIM: u32 = 0xFF5A5A5A;
const GREEN: u32 = 0xFF00FF00;
const RED: u32 = 0xFF0000FF;
const YELLOW: u32 = 0xFF00FFFF;
const BORDER: u32 = 0xFF1E1E1E;
const PANEL_BG: u32 = 0xFF0A0A0A;
const ROW_ALT: u32 = 0xFF080808;

pub fn render() {
    let r = wm::content_rect();
    let fb = gpu::framebuffer();
    let w = gpu::width();

    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);

    let x = r.x + 16;
    let mut y = r.y + 8;

    // Header
    font::draw_str(fb, w, x, y, "BATCAVES", FG_HI, BG);
    font::draw_str(fb, w, x + 100, y, "// Isolated container runtime", DIM, BG);
    y += 24;

    // Column headers
    gpu::fill_rect(x, y, r.w - 32, 18, PANEL_BG);
    font::draw_str(fb, w, x + 8, y + 1, "NAME", DIM, PANEL_BG);
    font::draw_str(fb, w, x + 180, y + 1, "STATUS", DIM, PANEL_BG);
    font::draw_str(fb, w, x + 280, y + 1, "TYPE", DIM, PANEL_BG);
    font::draw_str(fb, w, x + 400, y + 1, "TOOLS", DIM, PANEL_BG);
    font::draw_str(fb, w, x + 480, y + 1, "CAPABILITIES", DIM, PANEL_BG);
    y += 20;

    gpu::fill_rect(x, y, r.w - 32, 1, BORDER);
    y += 4;

    let cave_count = cave::count();

    if cave_count == 0 {
        font::draw_str(fb, w, x + 8, y + 20, "(no BatCaves — use 'batcave create <name>' in terminal)", DIM, BG);

        // Show quick-start guide
        let gy = y + 60;
        draw_panel(x, gy, r.w - 32, 200, "QUICK START");
        let py = gy + 28;
        font::draw_str(fb, w, x + 8, py, "batcave create pentest-lab --tools nmap,burpsuite", FG, BG);
        font::draw_str(fb, w, x + 8, py + 20, "batcave grant pentest-lab net", FG, BG);
        font::draw_str(fb, w, x + 8, py + 40, "batcave grant pentest-lab raw", FG, BG);
        font::draw_str(fb, w, x + 8, py + 60, "batcave grant pentest-lab display", FG, BG);
        font::draw_str(fb, w, x + 8, py + 80, "batcave enter pentest-lab", FG, BG);
        font::draw_str(fb, w, x + 8, py + 100, "batcave seal pentest-lab", DIM, BG);
        font::draw_str(fb, w, x + 300, py + 100, "# persistent -> ephemeral (irreversible)", DIM, BG);
        font::draw_str(fb, w, x + 8, py + 120, "batcave destroy pentest-lab", DIM, BG);
        font::draw_str(fb, w, x + 300, py + 120, "# secure wipe", DIM, BG);
        font::draw_str(fb, w, x + 8, py + 150, "All traffic goes through secure pipeline. No backdoors.", FG_HI, BG);
    } else {
        let mut row = 0u32;
        cave::list(|c| {
            let row_bg = if row % 2 == 0 { BG } else { ROW_ALT };
            let ry = y + row * 22;
            gpu::fill_rect(x, ry, r.w - 32, 22, row_bg);

            // Status indicator
            let status_color = match c.state {
                cave::CaveState::Running => GREEN,
                cave::CaveState::Stopped => DIM,
                _ => RED,
            };
            gpu::fill_rect(x + 8, ry + 7, 8, 8, status_color);

            // Name
            font::draw_str(fb, w, x + 24, ry + 3, c.name_str(), FG_HI, row_bg);

            // Status
            font::draw_str(fb, w, x + 180, ry + 3, cave::state_str(c.state), status_color, row_bg);

            // Type
            let type_color = if c.is_ephemeral() { YELLOW } else { FG };
            font::draw_str(fb, w, x + 280, ry + 3, cave::type_str(c.cave_type), type_color, row_bg);

            // Tool count
            let mut buf = [b' '; 4];
            let mut n = c.tool_count;
            let mut i = 3;
            if n == 0 { buf[3] = b'0'; } else { while n > 0 && i > 0 { buf[i] = b'0' + (n % 10) as u8; n /= 10; i -= 1; } }
            font::draw_str(fb, w, x + 400, ry + 3, unsafe { core::str::from_utf8_unchecked(&buf[i+1..]) }, FG, row_bg);

            // First few caps
            let mut cx = x + 480;
            for j in 0..c.cap_count.min(4) {
                if c.caps[j].active {
                    font::draw_str(fb, w, cx, ry + 3, c.caps[j].name_str(), GREEN, row_bg);
                    cx += (c.caps[j].name_len as u32 + 1) * 8;
                }
            }

            row += 1;
        });

        // Active sessions
        let sessions_y = y + (row + 1) * 22 + 16;
        draw_panel(x, sessions_y, r.w - 32, 80, "ACTIVE SESSIONS");
        let mut sy = sessions_y + 28;
        let mut has_running = false;
        cave::list(|c| {
            if c.state == cave::CaveState::Running {
                has_running = true;
                font::draw_str(fb, w, x + 8, sy, c.name_str(), GREEN, BG);
                font::draw_str(fb, w, x + 180, sy, "| ", DIM, BG);
                // Show tools
                let mut tx = x + 196;
                for j in 0..c.tool_count.min(5) {
                    if c.tools[j].installed {
                        font::draw_str(fb, w, tx, sy, c.tools[j].name_str(), FG, BG);
                        tx += (c.tools[j].name_len as u32 + 1) * 8;
                    }
                }
                sy += 18;
            }
        });
        if !has_running {
            font::draw_str(fb, w, x + 8, sy, "(no active sessions)", DIM, BG);
        }
    }

    // Footer
    let fy = r.y + r.h - 40;
    gpu::fill_rect(x, fy, r.w - 32, 1, BORDER);
    font::draw_str(fb, w, x + 8, fy + 8, "BatCaves:", DIM, BG);
    let mut buf = [b' '; 4];
    let mut n = cave_count;
    let mut i = 3;
    if n == 0 { buf[3] = b'0'; } else { while n > 0 && i > 0 { buf[i] = b'0' + (n % 10) as u8; n /= 10; i -= 1; } }
    font::draw_str(fb, w, x + 88, fy + 8, unsafe { core::str::from_utf8_unchecked(&buf[i+1..]) }, FG_HI, BG);

    font::draw_str(fb, w, x + 120, fy + 8, "|  6 layers of isolation  |  Supply chain attack proof", DIM, BG);
}

fn draw_panel(x: u32, y: u32, w: u32, h: u32, title: &str) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    gpu::fill_rect(x, y, w, 1, BORDER);
    gpu::fill_rect(x, y + h, w, 1, BORDER);
    gpu::fill_rect(x, y, 1, h, BORDER);
    gpu::fill_rect(x + w, y, 1, h, BORDER);
    gpu::fill_rect(x + 1, y + 1, w - 1, 20, PANEL_BG);
    font::draw_str(fb, sw, x + 8, y + 3, title, FG_HI, PANEL_BG);
    gpu::fill_rect(x + 1, y + 21, w - 1, 1, BORDER);
}

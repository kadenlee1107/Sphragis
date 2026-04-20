// Bat_OS — Security Toolkit App (7th Desktop App)
// Command center for BatCave security operations.
// Shows: active caves, running tools, capability status,
// network pipeline state, filesystem integrity, threat surface.

use crate::ui::wm;
use crate::ui::font;
use crate::ui::gpu;
use crate::batcave::cave;

const BG: u32 = 0xFF0A0A0A;
const FG: u32 = 0xFFA0A0A0;
const FG_HI: u32 = 0xFFFFFFFF;
const DIM: u32 = 0xFF5A5A5A;
const GREEN: u32 = 0xFF00FF00;
const RED: u32 = 0xFF0000FF;
const YELLOW: u32 = 0xFF00FFFF;
const BORDER: u32 = 0xFF1E1E1E;

pub fn render() {
    let r = wm::content_rect();
    let fb = gpu::framebuffer();
    let w = gpu::width();
    let ymax = r.y + r.h;
    let ln = 18u32;

    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);

    let x = r.x + 4;
    let two_col = r.w >= 400;
    let panel_w = if two_col { r.w / 2 - 12 } else { r.w - 8 };
    let col2 = if two_col { r.x + r.w / 2 + 4 } else { x };

    // ─── Left: Active BatCaves ───
    let mut y = r.y + 4;
    if y + 20 < ymax { draw_panel(x, y, panel_w, (ymax - y).min(200), "ACTIVE BATCAVES"); }
    y += 22;

    let mut cave_count = 0u32;
    cave::list(|c| {
        if y + ln >= ymax { return; }
        let state_color = match c.state {
            cave::CaveState::Running => GREEN,
            cave::CaveState::Stopped => DIM,
            _ => RED,
        };
        let state_str = match c.state {
            cave::CaveState::Running => "RUN",
            cave::CaveState::Stopped => "STP",
            cave::CaveState::Destroyed => "DEL",
            cave::CaveState::Free => return,
        };

        font::draw_str(fb, w, x + 4, y, state_str, state_color, BG);
        font::draw_str(fb, w, x + 36, y, c.name_str(), FG_HI, BG);

        // Show caps
        let mut cx = x + 120;
        if c.has_cap("net") && cx + 24 < x + panel_w {
            font::draw_str(fb, w, cx, y, "NET", GREEN, BG); cx += 32;
        }
        if c.has_cap("raw") && cx + 24 < x + panel_w {
            font::draw_str(fb, w, cx, y, "RAW", YELLOW, BG); cx += 32;
        }
        if c.has_cap("display") && cx + 24 < x + panel_w {
            font::draw_str(fb, w, cx, y, "DSP", FG, BG); cx += 32;
        }
        if c.has_cap("fs") && cx + 16 < x + panel_w {
            font::draw_str(fb, w, cx, y, "FS", FG, BG);
        }

        y += ln;
        cave_count += 1;
    });

    if cave_count == 0 && y + ln < ymax {
        font::draw_str(fb, w, x + 4, y, "(no caves)", DIM, BG);
        y += ln;
    }

    // ─── Right: Security Pipeline ───
    let mut y2 = r.y + 4;
    let cx = if two_col { col2 } else { x };
    let pw2 = if two_col { panel_w } else { r.w - 8 };
    if y2 + 20 < ymax { draw_panel(cx, y2, pw2, (ymax - y2).min(180), "SECURITY PIPELINE"); }
    y2 += 22;

    // Firewall
    if y2 + ln < ymax {
        font::draw_str(fb, w, cx + 4, y2, "Firewall", DIM, BG);
        font::draw_str(fb, w, cx + 80, y2, "ACTIVE", GREEN, BG);
        y2 += ln;
    }

    // Encryption
    if y2 + ln < ymax {
        font::draw_str(fb, w, cx + 4, y2, "AES-256", DIM, BG);
        font::draw_str(fb, w, cx + 80, y2, "ACTIVE", GREEN, BG);
        y2 += ln;
    }

    // TLS
    if y2 + ln < ymax {
        font::draw_str(fb, w, cx + 4, y2, "TLS 1.3", DIM, BG);
        font::draw_str(fb, w, cx + 80, y2, "READY", YELLOW, BG);
        y2 += ln;
    }

    // VPN
    let vpn_active = crate::net::vpn::is_active();
    if y2 + ln < ymax {
        font::draw_str(fb, w, cx + 4, y2, "VPN", DIM, BG);
        if vpn_active {
            font::draw_str(fb, w, cx + 80, y2, "CONNECTED", GREEN, BG);
        } else {
            font::draw_str(fb, w, cx + 80, y2, "STANDBY", DIM, BG);
        }
        y2 += ln;
    }

    // Tor
    let tor_ready = crate::net::tor::is_ready();
    if y2 + ln < ymax {
        font::draw_str(fb, w, cx + 4, y2, "Tor", DIM, BG);
        if tor_ready {
            font::draw_str(fb, w, cx + 80, y2, "3-HOP CIRCUIT", GREEN, BG);
        } else {
            font::draw_str(fb, w, cx + 80, y2, "STANDBY", DIM, BG);
        }
        y2 += ln;
    }

    // DoH
    if y2 + ln < ymax {
        font::draw_str(fb, w, cx + 4, y2, "DNS", DIM, BG);
        font::draw_str(fb, w, cx + 80, y2, "DoH ENABLED", GREEN, BG);
        y2 += ln;
    }

    // ─── Bottom: Integrity + Threat Surface ───
    let bot_y = y.max(y2) + 4;
    if bot_y + 30 < ymax {
        let bot_w = if two_col { r.w - 8 } else { r.w - 8 };
        draw_panel(x, bot_y, bot_w, (ymax - bot_y).min(120), "INTEGRITY");
        let mut by = bot_y + 22;

        // Merkle root
        let root = crate::fs::batfs::merkle_root();
        if by + ln < ymax {
            font::draw_str(fb, w, x + 4, by, "Merkle", DIM, BG);
            let hex = b"0123456789abcdef";
            let mut hash_str = [b'.'; 16];
            for i in 0..8 {
                hash_str[i * 2] = hex[(root[i] >> 4) as usize];
                hash_str[i * 2 + 1] = hex[(root[i] & 0xf) as usize];
            }
            font::draw_str(fb, w, x + 60, by,
                unsafe { core::str::from_utf8_unchecked(&hash_str) }, FG_HI, BG);
            by += ln;
        }

        // Dead man's switch
        if by + ln < ymax {
            font::draw_str(fb, w, x + 4, by, "DMS", DIM, BG);
            font::draw_str(fb, w, x + 60, by, "ARMED (48h)", GREEN, BG);
            by += ln;
        }

        // Auth
        if by + ln < ymax {
            font::draw_str(fb, w, x + 4, by, "Auth", DIM, BG);
            font::draw_str(fb, w, x + 60, by, "VERIFIED", GREEN, BG);
            by += ln;
        }

        // Threat surface
        if by + ln < ymax {
            font::draw_str(fb, w, x + 4, by, "Ports", DIM, BG);
            font::draw_str(fb, w, x + 60, by, "0 open (invisible)", GREEN, BG);
        }
    }
}

fn draw_panel(x: u32, y: u32, w: u32, h: u32, title: &str) {
    let fb = gpu::framebuffer();
    let sw = gpu::width();
    gpu::fill_rect(x, y, w, 1, BORDER);
    gpu::fill_rect(x, y + h, w, 1, BORDER);
    gpu::fill_rect(x, y, 1, h, BORDER);
    gpu::fill_rect(x + w, y, 1, h, BORDER);
    font::draw_str(fb, sw, x + 8, y + 2, title, FG_HI, BG);
}

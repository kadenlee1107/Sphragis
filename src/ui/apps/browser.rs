// Bat_OS — BatBrowser: Text-Mode Web Browser
// Downloads HTML over TCP, strips tags, renders text content.
// Supports link navigation, URL bar, page history.
//
// Features:
//   - URL bar with keyboard input
//   - HTTP GET over our TCP stack
//   - HTML tag stripping → readable text
//   - Link extraction (clickable with number keys)
//   - Page history (back/forward)
//   - Status bar (loading, connected, bytes)

use crate::ui::wm;
use crate::ui::font;
use crate::drivers::virtio::gpu;
use crate::drivers::uart;

const BG: u32 = 0xFF0A0A0A;         // page background
const FG: u32 = 0xFFA0A0A0;         // body text
const FG_HI: u32 = 0xFFFFFFFF;      // bright text
const DIM: u32 = 0xFF5A5A5A;        // muted text
const GREEN: u32 = 0xFF00FF00;      // secure indicator
const CYAN: u32 = 0xFFFFFF00;       // links in page
const RED: u32 = 0xFF0000FF;        // error
const BORDER: u32 = 0xFF1E1E1E;     // dividers
const TOOLBAR_BG: u32 = 0xFF1A1A1A; // navigation bar bg
const TAB_BG: u32 = 0xFF141414;     // inactive tab bg
const TAB_ACTIVE: u32 = 0xFF1A1A1A; // active tab bg
const URL_BG: u32 = 0xFF0E0E0E;     // URL bar input bg
const URL_BORDER: u32 = 0xFF2A2A2A; // URL bar border
const BTN_BG: u32 = 0xFF222222;     // button background
const BTN_FG: u32 = 0xFF888888;     // button text
const LINK_COLOR: u32 = 0xFFFF8800; // orange for links
const PROGRESS_BG: u32 = 0xFF1A1A1A;
const PROGRESS_FG: u32 = 0xFF44AA44; // loading bar green
const LOCK_COLOR: u32 = 0xFF00CC00; // HTTPS lock icon color
const BOOKMARK_BG: u32 = 0xFF121212; // bookmarks bar bg

// Browser state
#[derive(Clone, Copy, PartialEq)]
enum BrowserState {
    Idle,
    Loading,
    Loaded,
    Error,
}

// URL bar
const MAX_URL: usize = 128;
static mut URL_BUF: [u8; MAX_URL] = [0; MAX_URL];
static mut URL_LEN: usize = 0;

// Page content (stripped HTML)
const MAX_PAGE: usize = 8192;
static mut PAGE_BUF: [u8; MAX_PAGE] = [0; MAX_PAGE];
static mut PAGE_LEN: usize = 0;

// Extracted links
const MAX_LINKS: usize = 32;
const MAX_LINK_URL: usize = 128;
static mut LINKS: [[u8; MAX_LINK_URL]; MAX_LINKS] = [[0; MAX_LINK_URL]; MAX_LINKS];
static mut LINK_LENS: [usize; MAX_LINKS] = [0; MAX_LINKS];
static mut LINK_COUNT: usize = 0;

// Scroll position
static mut SCROLL_Y: usize = 0;

// State
static mut STATE: BrowserState = BrowserState::Idle;
static mut STATUS_MSG: [u8; 64] = [0; 64];
static mut STATUS_LEN: usize = 0;
static mut BYTES_LOADED: usize = 0;
static mut LOADING_PROGRESS: u8 = 0; // 0-100

// Browser tabs
const MAX_TABS: usize = 4;
static mut TAB_TITLES: [[u8; 20]; MAX_TABS] = [[0; 20]; MAX_TABS];
static mut TAB_TITLE_LENS: [usize; MAX_TABS] = [0; MAX_TABS];
static mut TAB_COUNT: usize = 1;
static mut ACTIVE_TAB: usize = 0;
static mut TABS_INITIALIZED: bool = false;

// Bookmarks
static mut BOOKMARKS: [[u8; 48]; 6] = [[0; 48]; 6];
static mut BM_LENS: [usize; 6] = [0; 6];
static mut BM_COUNT: usize = 0;

fn init_browser() {
    unsafe {
        if !TABS_INITIALIZED {
            TAB_TITLES[0][..7].copy_from_slice(b"New Tab");
            TAB_TITLE_LENS[0] = 7;
            let bms: [&[u8]; 4] = [
                b"https://example.com/",
                b"https://www.google.com/",
                b"http://info.cern.ch/",
                b"https://httpbin.org/ip",
            ];
            for (i, &bm) in bms.iter().enumerate() {
                let l = bm.len().min(48);
                BOOKMARKS[i][..l].copy_from_slice(&bm[..l]);
                BM_LENS[i] = l;
            }
            BM_COUNT = 4;
            TABS_INITIALIZED = true;
        }
    }
}

// History
const MAX_HISTORY: usize = 16;
static mut HISTORY: [[u8; MAX_URL]; MAX_HISTORY] = [[0; MAX_URL]; MAX_HISTORY];
static mut HISTORY_LENS: [usize; MAX_HISTORY] = [0; MAX_HISTORY];
static mut HISTORY_POS: usize = 0;
static mut HISTORY_COUNT: usize = 0;

/// Navigate to a URL.
pub fn navigate(url: &[u8]) {
    unsafe {
        // Save to URL bar
        URL_LEN = url.len().min(MAX_URL);
        URL_BUF[..URL_LEN].copy_from_slice(&url[..URL_LEN]);
        LOADING_PROGRESS = 10;

        // Push to history
        if HISTORY_COUNT < MAX_HISTORY {
            HISTORY_LENS[HISTORY_COUNT] = URL_LEN;
            HISTORY[HISTORY_COUNT][..URL_LEN].copy_from_slice(&url[..URL_LEN]);
            HISTORY_POS = HISTORY_COUNT;
            HISTORY_COUNT += 1;
        }

        STATE = BrowserState::Loading;
        set_status(b"Connecting...");
        SCROLL_Y = 0;
        LINK_COUNT = 0;
        PAGE_LEN = 0;
    }

    // Parse host and path from URL
    let url_str = unsafe { core::str::from_utf8_unchecked(&URL_BUF[..URL_LEN]) };
    let is_https = url_str.starts_with("https://");

    let (host, path, port) = parse_url(url_str);
    let port = if is_https && port == 80 { 443 } else { port };

    // Resolve DNS
    uart::puts("[browser] resolving: ");
    uart::puts(host);
    uart::puts("\n");
    let ip = match crate::net::dns::resolve(host) {
        Ok(ip) => {
            uart::puts("[browser] resolved to ");
            crate::kernel::mm::print_num(((ip >> 24) & 0xFF) as usize);
            uart::putc(b'.');
            crate::kernel::mm::print_num(((ip >> 16) & 0xFF) as usize);
            uart::putc(b'.');
            crate::kernel::mm::print_num(((ip >> 8) & 0xFF) as usize);
            uart::putc(b'.');
            crate::kernel::mm::print_num((ip & 0xFF) as usize);
            uart::puts("\n");
            ip
        }
        Err(e) => {
            uart::puts("[browser] DNS failed: ");
            uart::puts(e);
            uart::puts("\n");
            unsafe { STATE = BrowserState::Error; }
            set_status(b"DNS failed");
            return;
        }
    };

    set_status(b"TCP connecting...");

    // TCP connect
    if crate::net::tcp::connect(ip, port).is_err() {
        unsafe { STATE = BrowserState::Error; }
        set_status(b"Connection failed");
        return;
    }

    // TLS handshake for HTTPS
    if is_https {
        set_status(b"TLS handshake...");
        uart::puts("[browser] TLS handshake with ");
        uart::puts(host);
        uart::puts("\n");
        if crate::net::tls::handshake(host).is_err() {
            unsafe { STATE = BrowserState::Error; }
            set_status(b"TLS handshake failed");
            crate::net::tcp::close();
            return;
        }
    }

    set_status(b"Sending request...");

    // Build HTTP GET
    let mut req = [0u8; 512];
    let mut rlen = 0;
    let get = b"GET ";
    req[rlen..rlen + get.len()].copy_from_slice(get); rlen += get.len();
    req[rlen..rlen + path.len()].copy_from_slice(path.as_bytes()); rlen += path.len();
    let http = b" HTTP/1.0\r\nHost: ";
    req[rlen..rlen + http.len()].copy_from_slice(http); rlen += http.len();
    req[rlen..rlen + host.len()].copy_from_slice(host.as_bytes()); rlen += host.len();
    let trail = b"\r\nUser-Agent: BatBrowser/1.0\r\nAccept: text/html\r\nConnection: close\r\n\r\n";
    req[rlen..rlen + trail.len()].copy_from_slice(trail); rlen += trail.len();

    // Send via TLS or plain TCP
    let send_ok = if is_https {
        crate::net::tls::send_app_data(&req[..rlen]).is_ok()
    } else {
        crate::net::tcp::send_data(&req[..rlen]).is_ok()
    };
    if !send_ok {
        unsafe { STATE = BrowserState::Error; }
        set_status(b"Send failed");
        crate::net::tcp::close();
        return;
    }

    set_status(b"Receiving...");

    // Receive response
    let mut raw = [0u8; 16384];
    let mut total = 0;

    for _ in 0..10 {
        let mut chunk = [0u8; 4096];
        let recv_result = if is_https {
            crate::net::tls::recv_app_data(&mut chunk)
        } else {
            crate::net::tcp::recv_data(&mut chunk)
        };
        match recv_result {
            Ok(n) if n > 0 => {
                let copy = n.min(raw.len() - total);
                raw[total..total + copy].copy_from_slice(&chunk[..copy]);
                total += copy;
                unsafe { BYTES_LOADED = total; }
            }
            _ => break,
        }
    }

    if is_https { crate::net::tls::close(); }
    crate::net::tcp::close();

    if total == 0 {
        unsafe { STATE = BrowserState::Error; }
        set_status(b"No response");
        return;
    }

    // Check for redirect (301, 302, 303, 307, 308)
    // HTTP/1.x 3xx → find Location: header and follow it
    let headers_end = find_header_end(&raw[..total]);
    let status_code = parse_status_code(&raw[..total.min(20)]);

    if status_code >= 300 && status_code < 400 {
        // Extract Location: header
        if let Some(location) = find_header(&raw[..headers_end], b"Location:") {
            uart::puts("[browser] redirect → ");
            uart::puts(unsafe { core::str::from_utf8_unchecked(location) });
            uart::puts("\n");
            // Follow redirect (up to 5 hops)
            static mut REDIRECT_COUNT: u8 = 0;
            unsafe {
                REDIRECT_COUNT += 1;
                if REDIRECT_COUNT < 5 {
                    let mut redir_url = [0u8; MAX_URL];
                    let rlen = location.len().min(MAX_URL);
                    redir_url[..rlen].copy_from_slice(&location[..rlen]);
                    navigate(&redir_url[..rlen]);
                    return;
                }
                REDIRECT_COUNT = 0;
            }
            set_status(b"Too many redirects");
            unsafe { STATE = BrowserState::Error; }
            return;
        }
    }

    // Skip HTTP headers (find \r\n\r\n)
    let body = &raw[headers_end..total];

    // Strip HTML tags and extract text + links
    strip_html(body, host);

    unsafe {
        STATE = BrowserState::Loaded;
        BYTES_LOADED = total;
        LOADING_PROGRESS = 100;

        // Update tab title from hostname
        let hl = host.len().min(20);
        TAB_TITLES[ACTIVE_TAB][..hl].copy_from_slice(&host.as_bytes()[..hl]);
        TAB_TITLE_LENS[ACTIVE_TAB] = hl;
    }
    let mut status = [0u8; 64];
    let mut slen = 0;
    let prefix = b"Loaded ";
    status[..prefix.len()].copy_from_slice(prefix);
    slen += prefix.len();
    slen += write_num(&mut status[slen..], total);
    let suffix = b" bytes";
    status[slen..slen + suffix.len()].copy_from_slice(suffix);
    slen += suffix.len();
    set_status(&status[..slen]);
}

/// Strip HTML tags from body, extract text and links.
fn strip_html(html: &[u8], base_host: &str) {
    unsafe {
        PAGE_LEN = 0;
        LINK_COUNT = 0;
    }

    let mut i = 0;
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut last_was_space = false;

    while i < html.len() {
        if html[i] == b'<' {
            // Check for <script>, <style>, </script>, </style>
            let remaining = &html[i..];
            if starts_with_ci(remaining, b"<script") { in_script = true; }
            if starts_with_ci(remaining, b"</script") { in_script = false; }
            if starts_with_ci(remaining, b"<style") { in_style = true; }
            if starts_with_ci(remaining, b"</style") { in_style = false; }

            // Extract links from <a href="...">
            if starts_with_ci(remaining, b"<a ") || starts_with_ci(remaining, b"<a\t") {
                if let Some(href) = extract_href(remaining) {
                    unsafe {
                        if LINK_COUNT < MAX_LINKS {
                            let len = href.len().min(MAX_LINK_URL);
                            LINKS[LINK_COUNT][..len].copy_from_slice(&href[..len]);
                            LINK_LENS[LINK_COUNT] = len;
                            LINK_COUNT += 1;

                            // Add link marker to page text
                            let marker_start = b"[";
                            let marker_end = b"] ";
                            if PAGE_LEN + 5 < MAX_PAGE {
                                PAGE_BUF[PAGE_LEN] = b'['; PAGE_LEN += 1;
                                PAGE_LEN += write_num(&mut PAGE_BUF[PAGE_LEN..], LINK_COUNT);
                                PAGE_BUF[PAGE_LEN] = b']'; PAGE_LEN += 1;
                            }
                        }
                    }
                }
            }

            // Block-level tags → newline
            if starts_with_ci(remaining, b"<p") || starts_with_ci(remaining, b"<div")
                || starts_with_ci(remaining, b"<br") || starts_with_ci(remaining, b"<h")
                || starts_with_ci(remaining, b"<li") || starts_with_ci(remaining, b"<tr")
            {
                push_page(b'\n');
                last_was_space = true;
            }

            in_tag = true;
            i += 1;
            continue;
        }

        if html[i] == b'>' {
            in_tag = false;
            i += 1;
            continue;
        }

        if in_tag || in_script || in_style {
            i += 1;
            continue;
        }

        // Decode HTML entities
        if html[i] == b'&' {
            if starts_with_ci(&html[i..], b"&amp;") { push_page(b'&'); i += 5; continue; }
            if starts_with_ci(&html[i..], b"&lt;") { push_page(b'<'); i += 4; continue; }
            if starts_with_ci(&html[i..], b"&gt;") { push_page(b'>'); i += 4; continue; }
            if starts_with_ci(&html[i..], b"&nbsp;") { push_page(b' '); i += 6; continue; }
            if starts_with_ci(&html[i..], b"&quot;") { push_page(b'"'); i += 6; continue; }
            // Skip unknown entities
            while i < html.len() && html[i] != b';' { i += 1; }
            i += 1;
            continue;
        }

        // Collapse whitespace
        let ch = html[i];
        if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
            if !last_was_space {
                push_page(b' ');
                last_was_space = true;
            }
            i += 1;
            continue;
        }

        // Regular character
        if ch >= 0x20 && ch <= 0x7E {
            push_page(ch);
            last_was_space = false;
        }
        i += 1;
    }
}

fn push_page(ch: u8) {
    unsafe {
        if PAGE_LEN < MAX_PAGE {
            PAGE_BUF[PAGE_LEN] = ch;
            PAGE_LEN += 1;
        }
    }
}

fn starts_with_ci(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() { return false; }
    for i in 0..needle.len() {
        let a = haystack[i].to_ascii_lowercase();
        let b = needle[i].to_ascii_lowercase();
        if a != b { return false; }
    }
    true
}

fn extract_href(tag: &[u8]) -> Option<&[u8]> {
    // Find href=" or href='
    let mut i = 0;
    while i + 6 < tag.len() {
        if starts_with_ci(&tag[i..], b"href=") {
            i += 5;
            let quote = tag[i];
            if quote == b'"' || quote == b'\'' {
                i += 1;
                let start = i;
                while i < tag.len() && tag[i] != quote { i += 1; }
                return Some(&tag[start..i]);
            }
        }
        if tag[i] == b'>' { break; }
        i += 1;
    }
    None
}

fn parse_url(url: &str) -> (&str, &str, u16) {
    let without_scheme = if url.starts_with("http://") {
        &url[7..]
    } else if url.starts_with("https://") {
        &url[8..]
    } else {
        url
    };

    let (host_port, path) = match without_scheme.find('/') {
        Some(pos) => (&without_scheme[..pos], &without_scheme[pos..]),
        None => (without_scheme, "/"),
    };

    let (host, port) = match host_port.find(':') {
        Some(pos) => {
            let p = host_port[pos+1..].parse::<u16>().unwrap_or(80);
            (&host_port[..pos], p)
        }
        None => (host_port, 80),
    };

    (host, path, port)
}

fn find_header_end(data: &[u8]) -> usize {
    for i in 0..data.len().saturating_sub(3) {
        if data[i] == b'\r' && data[i+1] == b'\n' && data[i+2] == b'\r' && data[i+3] == b'\n' {
            return i + 4;
        }
    }
    0
}

fn parse_status_code(data: &[u8]) -> u16 {
    // "HTTP/1.x NNN ..."
    // Find first space, then parse 3 digits
    let mut i = 0;
    while i < data.len() && data[i] != b' ' { i += 1; }
    i += 1; // skip space
    if i + 3 <= data.len() {
        let h = (data[i] as u16 - b'0' as u16) * 100;
        let t = (data[i+1] as u16 - b'0' as u16) * 10;
        let o = data[i+2] as u16 - b'0' as u16;
        h + t + o
    } else {
        0
    }
}

fn find_header<'a>(headers: &'a [u8], name: &[u8]) -> Option<&'a [u8]> {
    // Case-insensitive search for "Name: value\r\n"
    let mut i = 0;
    while i + name.len() < headers.len() {
        if starts_with_ci(&headers[i..], name) {
            // Found header name, skip to value
            let mut vi = i + name.len();
            // Skip whitespace after colon
            while vi < headers.len() && (headers[vi] == b' ' || headers[vi] == b'\t') { vi += 1; }
            // Value ends at \r\n
            let val_start = vi;
            while vi < headers.len() && headers[vi] != b'\r' && headers[vi] != b'\n' { vi += 1; }
            return Some(&headers[val_start..vi]);
        }
        // Skip to next line
        while i < headers.len() && headers[i] != b'\n' { i += 1; }
        i += 1;
    }
    None
}

fn set_status(msg: &[u8]) {
    unsafe {
        STATUS_LEN = msg.len().min(64);
        STATUS_MSG[..STATUS_LEN].copy_from_slice(&msg[..STATUS_LEN]);
    }
}

fn write_num(buf: &mut [u8], n: usize) -> usize {
    if n == 0 && !buf.is_empty() { buf[0] = b'0'; return 1; }
    let mut digits = [0u8; 10];
    let mut dlen = 0;
    let mut v = n;
    while v > 0 && dlen < 10 { digits[dlen] = b'0' + (v % 10) as u8; dlen += 1; v /= 10; }
    for i in 0..dlen { if i < buf.len() { buf[i] = digits[dlen - 1 - i]; } }
    dlen
}

// ─── Rendering ───

pub fn render() {
    let r = wm::content_rect();
    let fb = gpu::framebuffer();
    let w = gpu::width();
    let ymax = r.y + r.h;
    let ln = 16u32;

    init_browser();
    gpu::fill_rect(r.x, r.y, r.w, r.h, BG);

    let x = r.x;
    let content_w = r.w;
    let mut y = r.y;

    // ═══════════════════════════════════════════
    // TAB BAR
    // ═══════════════════════════════════════════
    if y + 22 < ymax {
        gpu::fill_rect(x, y, content_w, 22, TAB_BG);

        unsafe {
            let tab_w = content_w / (TAB_COUNT.max(1) as u32).min(4);
            for i in 0..TAB_COUNT.min(MAX_TABS) {
                let tx = x + (i as u32) * tab_w;
                let is_active = i == ACTIVE_TAB;
                let bg = if is_active { TAB_ACTIVE } else { TAB_BG };

                // Tab background
                gpu::fill_rect(tx, y, tab_w - 1, 22, bg);
                // Tab bottom border (only inactive)
                if !is_active {
                    gpu::fill_rect(tx, y + 21, tab_w - 1, 1, BORDER);
                }
                // Tab separator
                gpu::fill_rect(tx + tab_w - 1, y + 4, 1, 14, BORDER);

                // Tab title
                let title = core::str::from_utf8_unchecked(&TAB_TITLES[i][..TAB_TITLE_LENS[i]]);
                let max_chars = ((tab_w - 24) / 8) as usize;
                let display_len = title.len().min(max_chars);
                let display = &title[..display_len];
                let color = if is_active { FG_HI } else { DIM };
                font::draw_str(fb, w, tx + 8, y + 3, display, color, bg);

                // Close button on active tab
                if is_active && TAB_COUNT > 1 {
                    font::draw_str(fb, w, tx + tab_w - 18, y + 3, "x", DIM, bg);
                }
            }

            // New tab [+] button
            if TAB_COUNT < MAX_TABS {
                let plus_x = x + (TAB_COUNT as u32) * tab_w + 4;
                font::draw_str(fb, w, plus_x, y + 3, "+", DIM, TAB_BG);
            }
        }
        y += 22;
    }

    // ═══════════════════════════════════════════
    // NAVIGATION TOOLBAR
    // ═══════════════════════════════════════════
    if y + 24 < ymax {
        gpu::fill_rect(x, y, content_w, 24, TOOLBAR_BG);

        let mut bx = x + 4;

        // [<] Back button
        gpu::fill_rect(bx, y + 3, 20, 18, BTN_BG);
        font::draw_str(fb, w, bx + 6, y + 4, "<", BTN_FG, BTN_BG);
        bx += 24;

        // [>] Forward button
        gpu::fill_rect(bx, y + 3, 20, 18, BTN_BG);
        font::draw_str(fb, w, bx + 6, y + 4, ">", BTN_FG, BTN_BG);
        bx += 24;

        // [R] Refresh button
        gpu::fill_rect(bx, y + 3, 20, 18, BTN_BG);
        let refresh_char = unsafe { if STATE == BrowserState::Loading { "X" } else { "R" } };
        font::draw_str(fb, w, bx + 6, y + 4, refresh_char, BTN_FG, BTN_BG);
        bx += 28;

        // URL Bar (the main input area)
        let url_x = bx;
        let url_w = content_w - bx + x - 8;
        gpu::fill_rect(url_x, y + 3, url_w, 18, URL_BG);
        // Rounded-ish border
        gpu::fill_rect(url_x, y + 3, url_w, 1, URL_BORDER);
        gpu::fill_rect(url_x, y + 20, url_w, 1, URL_BORDER);
        gpu::fill_rect(url_x, y + 3, 1, 18, URL_BORDER);
        gpu::fill_rect(url_x + url_w - 1, y + 3, 1, 18, URL_BORDER);

        unsafe {
            let url_str = core::str::from_utf8_unchecked(&URL_BUF[..URL_LEN]);
            let is_https = url_str.starts_with("https");

            // Lock icon for HTTPS
            let text_start = if is_https {
                font::draw_str(fb, w, url_x + 6, y + 4, "##", LOCK_COLOR, URL_BG);
                url_x + 24
            } else if URL_LEN > 0 {
                font::draw_str(fb, w, url_x + 6, y + 4, "i", DIM, URL_BG);
                url_x + 18
            } else {
                url_x + 6
            };

            // URL text
            let max_url_chars = ((url_w - (text_start - url_x) - 16) / 8) as usize;
            let display_len = URL_LEN.min(max_url_chars);
            let url_display = core::str::from_utf8_unchecked(&URL_BUF[..display_len]);
            font::draw_str(fb, w, text_start, y + 4, url_display, FG_HI, URL_BG);

            // Cursor
            if STATE == BrowserState::Idle || STATE == BrowserState::Loaded {
                let cx = text_start + (display_len as u32) * 8;
                if cx < url_x + url_w - 8 {
                    font::draw_str(fb, w, cx, y + 4, "|", FG_HI, URL_BG);
                }
            }
        }
        y += 24;
    }

    // ═══════════════════════════════════════════
    // BOOKMARKS BAR
    // ═══════════════════════════════════════════
    if y + 18 < ymax {
        gpu::fill_rect(x, y, content_w, 18, BOOKMARK_BG);
        gpu::fill_rect(x, y + 17, content_w, 1, BORDER);

        unsafe {
            let mut bmx = x + 4;
            for i in 0..BM_COUNT.min(6) {
                let bm = core::str::from_utf8_unchecked(&BOOKMARKS[i][..BM_LENS[i]]);
                // Extract short domain name for display
                let display = if let Some(start) = bm.find("://") {
                    let after = &bm[start + 3..];
                    let end = after.find('/').unwrap_or(after.len());
                    &after[..end]
                } else {
                    bm
                };
                let short = if display.len() > 14 { &display[..14] } else { display };

                font::draw_str(fb, w, bmx, y + 1, short, DIM, BOOKMARK_BG);
                bmx += (short.len() as u32) * 8 + 16;

                if bmx > x + content_w - 40 { break; }
            }
        }
        y += 18;
    }

    // ═══════════════════════════════════════════
    // LOADING PROGRESS BAR (only when loading)
    // ═══════════════════════════════════════════
    unsafe {
        if STATE == BrowserState::Loading && y + 3 < ymax {
            gpu::fill_rect(x, y, content_w, 3, PROGRESS_BG);
            let progress_w = (content_w as u64 * LOADING_PROGRESS as u64 / 100) as u32;
            gpu::fill_rect(x, y, progress_w, 3, PROGRESS_FG);
            y += 3;
        }
    }

    // Divider before content
    if y < ymax {
        gpu::fill_rect(x, y, content_w, 1, BORDER);
        y += 1;
    }

    // ─── Page Content ───
    unsafe {
        if STATE == BrowserState::Idle {
            if y + ln < ymax {
                font::draw_str(fb, w, x + 8, y + 40, "BatBrowser v1.0", FG_HI, BG);
                font::draw_str(fb, w, x + 8, y + 60, "Type a URL and press Enter", DIM, BG);
                font::draw_str(fb, w, x + 8, y + 80, "e.g. http://example.com/", DIM, BG);
            }
        } else if STATE == BrowserState::Loading {
            if y + ln < ymax {
                font::draw_str(fb, w, x + 8, y + 40, "Loading...", FG_HI, BG);
            }
        } else if STATE == BrowserState::Error {
            if y + ln < ymax {
                let msg = core::str::from_utf8_unchecked(&STATUS_MSG[..STATUS_LEN]);
                font::draw_str(fb, w, x + 8, y + 40, msg, RED, BG);
            }
        } else {
            // Render page text with word wrap
            let chars_per_line = ((r.w - 16) / 8) as usize;
            let max_lines = ((ymax - y - 24) / ln) as usize;

            let text = &PAGE_BUF[..PAGE_LEN];
            let mut line_num = 0usize;
            let mut ti = 0;

            while ti < text.len() && line_num < SCROLL_Y + max_lines {
                // Find end of line (newline or wrap at chars_per_line)
                let line_start = ti;
                let mut line_end = ti;
                let mut chars = 0;

                while line_end < text.len() && text[line_end] != b'\n' && chars < chars_per_line {
                    line_end += 1;
                    chars += 1;
                }

                if line_num >= SCROLL_Y {
                    let draw_y = y + ((line_num - SCROLL_Y) as u32) * ln;
                    if draw_y + ln < ymax {
                        let line_text = core::str::from_utf8_unchecked(&text[line_start..line_end]);
                        // Check for link markers [N]
                        if line_text.contains('[') {
                            font::draw_str(fb, w, x + 4, draw_y, line_text, LINK_COLOR, BG);
                        } else {
                            font::draw_str(fb, w, x + 4, draw_y, line_text, FG, BG);
                        }
                    }
                }

                // Advance past newline
                if line_end < text.len() && text[line_end] == b'\n' {
                    line_end += 1;
                }
                ti = line_end;
                line_num += 1;
            }

            // Show links at bottom if space
            if LINK_COUNT > 0 {
                let links_y = ymax - 20 - (LINK_COUNT.min(3) as u32 * ln);
                if links_y > y + 40 {
                    gpu::fill_rect(x, links_y - 2, r.w - 8, 1, BORDER);
                    let mut ly = links_y;
                    for i in 0..LINK_COUNT.min(3) {
                        if ly + ln < ymax {
                            let link = core::str::from_utf8_unchecked(&LINKS[i][..LINK_LENS[i]]);
                            let mut label = [0u8; 4];
                            label[0] = b'[';
                            label[1] = b'1' + i as u8;
                            label[2] = b']';
                            label[3] = b' ';
                            font::draw_str(fb, w, x + 4, ly,
                                core::str::from_utf8_unchecked(&label[..4]), LINK_COLOR, BG);
                            font::draw_str(fb, w, x + 36, ly, link, CYAN, BG);
                            ly += ln;
                        }
                    }
                }
            }
        }
    }

    // ═══════════════════════════════════════════
    // STATUS BAR (Chrome-style bottom bar)
    // ═══════════════════════════════════════════
    let status_y = ymax - 20;
    if status_y > r.y + 40 {
        gpu::fill_rect(x, status_y, content_w, 20, TOOLBAR_BG);
        gpu::fill_rect(x, status_y, content_w, 1, BORDER);

        unsafe {
            // Left: status message
            let (state_text, state_color) = match STATE {
                BrowserState::Idle => ("Ready", DIM),
                BrowserState::Loading => ("Loading...", CYAN),
                BrowserState::Loaded => ("Secure", GREEN),
                BrowserState::Error => ("Error", RED),
            };

            if STATE == BrowserState::Loaded {
                font::draw_str(fb, w, x + 4, status_y + 2, "##", LOCK_COLOR, TOOLBAR_BG);
                font::draw_str(fb, w, x + 22, status_y + 2, state_text, state_color, TOOLBAR_BG);
            } else {
                font::draw_str(fb, w, x + 4, status_y + 2, state_text, state_color, TOOLBAR_BG);
            }

            // Center: status message detail
            let msg = core::str::from_utf8_unchecked(&STATUS_MSG[..STATUS_LEN]);
            font::draw_str(fb, w, x + 100, status_y + 2, msg, DIM, TOOLBAR_BG);

            // Right: link count + bytes
            if BYTES_LOADED > 0 {
                let mut nbuf = [0u8; 10];
                let nlen = write_num(&mut nbuf, BYTES_LOADED);
                let bx = x + content_w - 80;
                font::draw_str(fb, w, bx, status_y + 2,
                    core::str::from_utf8_unchecked(&nbuf[..nlen]), DIM, TOOLBAR_BG);
                font::draw_str(fb, w, bx + (nlen as u32) * 8, status_y + 2, " B", DIM, TOOLBAR_BG);
            }

            if LINK_COUNT > 0 {
                let mut nbuf = [0u8; 4];
                let nlen = write_num(&mut nbuf, LINK_COUNT);
                let lx = x + content_w - 140;
                font::draw_str(fb, w, lx, status_y + 2, "Links:", DIM, TOOLBAR_BG);
                font::draw_str(fb, w, lx + 52, status_y + 2,
                    core::str::from_utf8_unchecked(&nbuf[..nlen]), LINK_COLOR, TOOLBAR_BG);
            }
        }
    }
}

/// Handle keyboard input for the browser.
pub fn handle_key(ch: u8) {
    unsafe {
        match ch {
            b'\r' | b'\n' => {
                // Navigate to URL
                if URL_LEN > 0 {
                    let url_copy: [u8; MAX_URL] = URL_BUF;
                    let len = URL_LEN;
                    navigate(&url_copy[..len]);
                }
            }
            0x08 | 0x7F => {
                if URL_LEN > 0 { URL_LEN -= 1; }
            }
            // Number keys 1-9: follow link
            b'1'..=b'9' if STATE == BrowserState::Loaded => {
                let link_idx = (ch - b'1') as usize;
                if link_idx < LINK_COUNT {
                    let url = &LINKS[link_idx][..LINK_LENS[link_idx]];
                    let mut url_copy = [0u8; MAX_URL];
                    let len = url.len().min(MAX_URL);
                    url_copy[..len].copy_from_slice(&url[..len]);
                    navigate(&url_copy[..len]);
                }
            }
            // Page up/down (using - and = keys when page loaded)
            b'-' if STATE == BrowserState::Loaded => {
                if SCROLL_Y > 0 { SCROLL_Y -= 5; }
            }
            b'=' if STATE == BrowserState::Loaded => {
                SCROLL_Y += 5;
            }
            // Regular character → URL bar
            c if c >= 0x20 && c <= 0x7E => {
                if URL_LEN < MAX_URL - 1 {
                    URL_BUF[URL_LEN] = c;
                    URL_LEN += 1;
                }
            }
            _ => {}
        }
    }
}

/// Go back in history.
pub fn go_back() {
    unsafe {
        if HISTORY_POS > 0 {
            HISTORY_POS -= 1;
            let len = HISTORY_LENS[HISTORY_POS];
            let mut url = [0u8; MAX_URL];
            url[..len].copy_from_slice(&HISTORY[HISTORY_POS][..len]);
            navigate(&url[..len]);
        }
    }
}

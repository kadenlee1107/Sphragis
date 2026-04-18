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

// Page content with per-character styling
const MAX_PAGE: usize = 8192;
static mut PAGE_BUF: [u8; MAX_PAGE] = [0; MAX_PAGE];
static mut PAGE_LEN: usize = 0;

// Style per character: encodes color + attributes
// Bits: [7:4]=color_index, [3]=bold, [2]=italic, [1]=underline, [0]=big (h1)
static mut PAGE_STYLE: [u8; MAX_PAGE] = [0; MAX_PAGE];

// Color palette for styled rendering
const STYLE_BODY: u8     = 0x00; // gray text
const STYLE_H1: u8       = 0x11; // white + bold + big
const STYLE_H2: u8       = 0x18; // white + bold
const STYLE_H3: u8       = 0x28; // bright gray + bold
const STYLE_LINK: u8     = 0x32; // blue + underline
const STYLE_BOLD: u8     = 0x08; // bold
const STYLE_ITALIC: u8   = 0x04; // italic
const STYLE_CODE: u8     = 0x40; // code (green)
const STYLE_QUOTE: u8    = 0x50; // blockquote (dim)
const STYLE_BULLET: u8   = 0x60; // list bullet (accent)
const STYLE_HR: u8       = 0x70; // horizontal rule

fn style_to_color(style: u8) -> u32 {
    let color_idx = (style >> 4) & 0xF;
    match color_idx {
        0 => 0xFFA0A0A0, // body gray
        1 => 0xFFFFFFFF, // headings white
        2 => 0xFFCCCCCC, // h3 bright gray
        3 => 0xFF4499FF, // links blue
        4 => 0xFF44DD44, // code green
        5 => 0xFF666666, // blockquote dim
        6 => 0xFFFF8800, // bullet/accent orange
        7 => 0xFF3A3A3A, // hr dark
        _ => 0xFFA0A0A0, // default
    }
}

fn style_is_bold(style: u8) -> bool { style & 0x08 != 0 }
fn style_is_underline(style: u8) -> bool { style & 0x02 != 0 }
fn style_is_big(style: u8) -> bool { style & 0x01 != 0 }

// Extracted links
const MAX_LINKS: usize = 32;
const MAX_LINK_URL: usize = 128;
static mut LINKS: [[u8; MAX_LINK_URL]; MAX_LINKS] = [[0; MAX_LINK_URL]; MAX_LINKS];
static mut LINK_LENS: [usize; MAX_LINKS] = [0; MAX_LINKS];
static mut LINK_COUNT: usize = 0;

// Scroll position
static mut SCROLL_Y: usize = 0;

// Level 2 rendering engine (DOM + Layout)
static mut USE_ENGINE: bool = true; // use Level 2 engine vs Level 1 fallback
static mut DOM_DOC: crate::browser::dom::Document = crate::browser::dom::Document::new();
static mut LAYOUT_TREE: crate::browser::layout::LayoutTree = crate::browser::layout::LayoutTree::new();

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

/// V11-state-sweep: scrub the browser's in-memory state on cave switch.
/// URL, page body, styled render buffer, links, scroll, DOM document,
/// layout tree, tab titles, bookmarks — all of it identifies the
/// previous cave's browsing session. Leaving any of it readable is a
/// direct history / DOM leak into the new cave.
pub fn reset_for_cave_switch() {
    let _g = crate::kernel::sync::IrqGuard::new();
    unsafe {
        // URL + page buffers
        for b in (&mut *core::ptr::addr_of_mut!(URL_BUF)).iter_mut() { *b = 0; }
        URL_LEN = 0;
        for b in (&mut *core::ptr::addr_of_mut!(PAGE_BUF)).iter_mut() { *b = 0; }
        PAGE_LEN = 0;
        for b in (&mut *core::ptr::addr_of_mut!(PAGE_STYLE)).iter_mut() { *b = 0; }
        // Links
        for entry in (&mut *core::ptr::addr_of_mut!(LINKS)).iter_mut() {
            for b in entry.iter_mut() { *b = 0; }
        }
        for l in (&mut *core::ptr::addr_of_mut!(LINK_LENS)).iter_mut() { *l = 0; }
        LINK_COUNT = 0;
        SCROLL_Y = 0;
        // Status
        for b in (&mut *core::ptr::addr_of_mut!(STATUS_MSG)).iter_mut() { *b = 0; }
        STATUS_LEN = 0;
        BYTES_LOADED = 0;
        LOADING_PROGRESS = 0;
        STATE = BrowserState::Idle;
        // Tabs + bookmarks
        for entry in (&mut *core::ptr::addr_of_mut!(TAB_TITLES)).iter_mut() {
            for b in entry.iter_mut() { *b = 0; }
        }
        for l in (&mut *core::ptr::addr_of_mut!(TAB_TITLE_LENS)).iter_mut() { *l = 0; }
        TAB_COUNT = 1;
        ACTIVE_TAB = 0;
        TABS_INITIALIZED = false;
        for entry in (&mut *core::ptr::addr_of_mut!(BOOKMARKS)).iter_mut() {
            for b in entry.iter_mut() { *b = 0; }
        }
        for l in (&mut *core::ptr::addr_of_mut!(BM_LENS)).iter_mut() { *l = 0; }
        BM_COUNT = 0;
        // DOM + layout (fresh Document + LayoutTree)
        DOM_DOC = crate::browser::dom::Document::new();
        LAYOUT_TREE = crate::browser::layout::LayoutTree::new();
        // V12 additions: browser HISTORY, the JS VM state (SEED is
        // reachable via Math.random and was acting as a cross-cave
        // timing oracle), and the engine-selector toggle.
        for entry in (&mut *core::ptr::addr_of_mut!(HISTORY)).iter_mut() {
            for b in entry.iter_mut() { *b = 0; }
        }
        for l in (&mut *core::ptr::addr_of_mut!(HISTORY_LENS)).iter_mut() { *l = 0; }
        HISTORY_POS = 0;
        HISTORY_COUNT = 0;
        USE_ENGINE = true;
        // Fresh JS VM — wipes heap, string table, and builtin-PRNG state.
        JS_VM = crate::browser::js::vm::Vm::new();
        JS_VM_INITIALIZED = false;
    }
}

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

    // Response-splitting guard (pentest NET-045 cascade): the Host and path
    // segments come from the URL bar which is user-controlled. Reject any
    // CR/LF/NUL before we paste them into the request.
    if crate::net::http::validate_header_value(host.as_bytes()).is_err()
        || crate::net::http::validate_header_value(path.as_bytes()).is_err()
    {
        unsafe { STATE = BrowserState::Error; }
        set_status(b"Invalid URL (CR/LF)");
        crate::net::tcp::close();
        return;
    }

    // Build HTTP GET
    let mut req = [0u8; 512];
    let mut rlen = 0;
    let get = b"GET ";
    req[rlen..rlen + get.len()].copy_from_slice(get); rlen += get.len();
    req[rlen..rlen + path.len()].copy_from_slice(path.as_bytes()); rlen += path.len();
    let http = b" HTTP/1.1\r\nHost: ";
    req[rlen..rlen + http.len()].copy_from_slice(http); rlen += http.len();
    req[rlen..rlen + host.len()].copy_from_slice(host.as_bytes()); rlen += host.len();
    let trail = b"\r\nUser-Agent: BatBrowser/1.0\r\nAccept: text/html,*/*\r\nAccept-Encoding: gzip, identity\r\nConnection: close\r\n\r\n";
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

    // Receive response — hardened read loop (pentest ATTACK-NET-045 / 046).
    // The old loop spun up to 500 × 5 s = 2500 s on a slow-loris server,
    // wedging the cooperative kernel. Now bounded by:
    //   * 30 s total deadline
    //   * 5 s no-progress idle deadline
    //   * 64 KB header cap, 8 KB per line, 128 header lines
    // See src/net/http.rs for the state machine.
    static mut RAW_BUF: [u8; 131072] = [0u8; 131072];
    let raw = unsafe { &mut *core::ptr::addr_of_mut!(RAW_BUF) };
    // Zero the scratch on each navigation so leftover bytes from the
    // previous request can't poison header scanning.
    for b in raw.iter_mut() { *b = 0; }

    fn recv_https(buf: &mut [u8]) -> Result<usize, &'static str> {
        crate::net::tls::recv_app_data(buf)
    }
    fn recv_plain(buf: &mut [u8]) -> Result<usize, &'static str> {
        crate::net::tcp::recv_data(buf)
    }
    let recv_fn: crate::net::http::RecvFn =
        if is_https { recv_https } else { recv_plain };

    let total = match crate::net::http::read_response(recv_fn, raw) {
        Ok(n) => n,
        Err(crate::net::http::HttpError::DeadlineExceeded)
        | Err(crate::net::http::HttpError::IdleTimeout) => {
            unsafe { STATE = BrowserState::Error; }
            set_status(b"Read timeout (slow-loris?)");
            if is_https { crate::net::tls::close(); }
            crate::net::tcp::close();
            return;
        }
        Err(crate::net::http::HttpError::HeadersTooLarge) => {
            unsafe { STATE = BrowserState::Error; }
            set_status(b"Headers too large");
            if is_https { crate::net::tls::close(); }
            crate::net::tcp::close();
            return;
        }
        Err(crate::net::http::HttpError::BufferFull) => {
            // Partial fetch — we got *something* before the static buffer
            // filled. Use it; the renderer already clamps to the buffer.
            // Recover "total" from the buffer scan. Worst case we render
            // a truncated page, which is existing behaviour.
            raw.len()
        }
        Err(e) => {
            unsafe { STATE = BrowserState::Error; }
            set_status(e.as_str().as_bytes());
            if is_https { crate::net::tls::close(); }
            crate::net::tcp::close();
            return;
        }
    };
    unsafe { BYTES_LOADED = total; }

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

    // Debug: log header parsing
    uart::puts("[browser] total=");
    crate::kernel::mm::print_num(total);
    uart::puts(" headers_end=");
    crate::kernel::mm::print_num(headers_end);
    uart::puts(" status=");
    crate::kernel::mm::print_num(status_code as usize);
    uart::puts("\n");
    // Log content-encoding if present
    if let Some(enc) = find_header(&raw[..headers_end.max(1).min(total)], b"Content-Encoding:") {
        uart::puts("[browser] Content-Encoding: ");
        uart::puts(unsafe { core::str::from_utf8_unchecked(enc) });
        uart::puts("\n");
    }

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

    // Decode body: handle chunked transfer encoding or Content-Length
    static mut DECODED_BUF: [u8; 131072] = [0u8; 131072];
    let decoded = unsafe { &mut *core::ptr::addr_of_mut!(DECODED_BUF) };
    // V12: zero before reuse so a truncated response can't leak the
    // tail of a previous page's body below the truncation boundary.
    for b in decoded.iter_mut() { *b = 0; }
    let mut decoded_len: usize;
    let is_chunked = {
        let hdr_slice = &raw[..headers_end];
        find_header(hdr_slice, b"Transfer-Encoding:").map_or(false, |v| {
            // Check if value contains "chunked"
            let mut found = false;
            if v.len() >= 7 {
                for i in 0..=v.len() - 7 {
                    if starts_with_ci(&v[i..], b"chunked") { found = true; break; }
                }
            }
            found
        })
    };

    if is_chunked {
        // Hardened chunked decoder (ATTACK-DOS-026 cascade): caps each chunk
        // at 4 MiB and the total decoded body at 16 MiB. See src/net/http.rs.
        let chunk_data = &raw[headers_end..total];
        match crate::net::http::decode_chunked(chunk_data, decoded) {
            Ok(n) => {
                decoded_len = n;
                uart::puts("[browser] decoded chunked body: ");
                crate::kernel::mm::print_num(decoded_len);
                uart::puts(" bytes\n");
            }
            Err(e) => {
                uart::puts("[browser] chunked decode rejected: ");
                uart::puts(e.as_str());
                uart::puts("\n");
                unsafe { STATE = BrowserState::Error; }
                set_status(e.as_str().as_bytes());
                return;
            }
        }
    } else {
        // Non-chunked: use body directly (Content-Length or connection close)
        let raw_body = &raw[headers_end..total];
        let copy_len = raw_body.len().min(decoded.len());
        decoded[..copy_len].copy_from_slice(&raw_body[..copy_len]);
        decoded_len = copy_len;
    }

    // Check for gzip Content-Encoding and decompress if needed
    if let Some(enc) = find_header(&raw[..headers_end.max(1).min(total)], b"Content-Encoding:") {
        if starts_with_ci(enc, b"gzip") {
            uart::puts("[browser] gzip detected, decompressing...\n");
            static mut DECOMP_BUF: [u8; 262144] = [0u8; 262144];
            let decompressed = unsafe { &mut *core::ptr::addr_of_mut!(DECOMP_BUF) };
            // V12: zero before use so a short decompress output can't
            // leak a prior response's tail.
            for b in decompressed.iter_mut() { *b = 0; }
            let dec_len = crate::browser::media::gzip::decompress(&decoded[..decoded_len], decompressed);
            if dec_len > 0 {
                // Copy all decompressed data (up to decoded buffer size)
                let copy = dec_len.min(decoded.len());
                uart::puts("[browser] copying ");
                crate::kernel::mm::print_num(copy);
                uart::puts(" bytes to decoded buffer\n");
                decoded[..copy].copy_from_slice(&decompressed[..copy]);
                decoded_len = copy;
                uart::puts("[browser] gzip decompressed: ");
                crate::kernel::mm::print_num(decoded_len);
                uart::puts(" bytes\n");
            } else {
                uart::puts("[browser] gzip decompression failed, using raw data\n");
            }
        }
    }

    // Cap body size to prevent parser from hanging on huge pages
    // (Wikipedia decompresses to 108KB+ — parser gets stuck on complex HTML)
    let body_cap = decoded_len.min(131072); // Use all available decompressed data
    let body = &decoded[..body_cap];

    uart::puts("[browser] body_len=");
    crate::kernel::mm::print_num(body_cap);
    uart::puts("\n");

    // Full rendering pipeline: HTML → DOM → Reader Mode → CSS → Layout → JS → Paint
    unsafe {
        if USE_ENGINE {
            // Step 1: Parse HTML → DOM tree
            crate::browser::html::parser::parse(body, &mut *core::ptr::addr_of_mut!(DOM_DOC));

            // Reader mode disabled — Level 1 text renderer handles content well

            // Step 3: Execute inline <script> tags
            // Wire up the DOM pointer so JS DOM APIs operate on the real tree
            crate::browser::js::dom_api::set_document(&mut *core::ptr::addr_of_mut!(DOM_DOC));
            execute_scripts(&*core::ptr::addr_of!(DOM_DOC));

            // Step 4: Check if scripts mutated the DOM
            let dom_was_dirty = crate::browser::js::dom_api::take_dirty();
            if dom_was_dirty {
                crate::drivers::uart::puts("[browser] DOM mutated by JS — rebuilding layout\n");
            }

            // Step 5: Compute layout (always, or re-layout if DOM dirty)
            let viewport_w = wm::content_rect().w as i32 - 16;
            crate::browser::layout::build(
                &*core::ptr::addr_of!(DOM_DOC),
                &mut *core::ptr::addr_of_mut!(LAYOUT_TREE),
                viewport_w,
            );

            // Debug: log DOM and layout stats
            let dom = &*core::ptr::addr_of!(DOM_DOC);
            let lt = &*core::ptr::addr_of!(LAYOUT_TREE);
            crate::drivers::uart::puts("[browser] DOM nodes=");
            crate::kernel::mm::print_num(dom.node_count);
            crate::drivers::uart::puts(" layout boxes=");
            crate::kernel::mm::print_num(lt.box_count);
            crate::drivers::uart::puts(" body_len=");
            crate::kernel::mm::print_num(body.len());
            crate::drivers::uart::puts("\n");
        }
    }

    // Level 1 text: extract from cleaned DOM tree (reader mode applied)
    // or fall back to raw HTML stripping if engine is off
    unsafe {
        if USE_ENGINE {
            extract_text_from_dom(
                &*core::ptr::addr_of!(DOM_DOC), host
            );
        } else {
            strip_html(body, host);
        }
        crate::drivers::uart::puts("[browser] Level1 text_len=");
        crate::kernel::mm::print_num(PAGE_LEN);
        crate::drivers::uart::puts(" links=");
        crate::kernel::mm::print_num(LINK_COUNT);
        crate::drivers::uart::puts("\n");
    }

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

/// Strip HTML tags, extract styled text and links.
fn strip_html(html: &[u8], base_host: &str) {
    unsafe {
        PAGE_LEN = 0;
        for i in 0..MAX_PAGE { PAGE_STYLE[i] = STYLE_BODY; }
        LINK_COUNT = 0;
    }

    let mut i = 0;
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut last_was_space = false;
    let mut current_style: u8 = STYLE_BODY;
    let mut in_link = false;
    let mut in_bold = false;
    let mut in_italic = false;
    let mut in_code = false;
    let mut in_pre = false;
    let mut in_blockquote = false;
    let mut list_depth: u8 = 0;

    while i < html.len() {
        if html[i] == b'<' {
            let remaining = &html[i..];

            // Script/style exclusion
            if starts_with_ci(remaining, b"<script") { in_script = true; }
            if starts_with_ci(remaining, b"</script") { in_script = false; }
            if starts_with_ci(remaining, b"<style") { in_style = true; }
            if starts_with_ci(remaining, b"</style") { in_style = false; }

            // ─── Style-changing tags ───

            // Headings
            if starts_with_ci(remaining, b"<h1") {
                push_styled(b'\n', current_style); push_styled(b'\n', current_style);
                current_style = STYLE_H1; last_was_space = true;
            } else if starts_with_ci(remaining, b"</h1") {
                push_styled(b'\n', current_style); current_style = STYLE_BODY;
            } else if starts_with_ci(remaining, b"<h2") {
                push_styled(b'\n', current_style); push_styled(b'\n', current_style);
                current_style = STYLE_H2; last_was_space = true;
            } else if starts_with_ci(remaining, b"</h2") {
                push_styled(b'\n', current_style); current_style = STYLE_BODY;
            } else if starts_with_ci(remaining, b"<h3") || starts_with_ci(remaining, b"<h4")
                   || starts_with_ci(remaining, b"<h5") || starts_with_ci(remaining, b"<h6") {
                push_styled(b'\n', current_style);
                current_style = STYLE_H3; last_was_space = true;
            } else if starts_with_ci(remaining, b"</h") {
                push_styled(b'\n', current_style); current_style = STYLE_BODY;
            }

            // Links
            if starts_with_ci(remaining, b"<a ") || starts_with_ci(remaining, b"<a\t") {
                in_link = true;
                current_style = STYLE_LINK;
                if let Some(href) = extract_href(remaining) {
                    unsafe {
                        if LINK_COUNT < MAX_LINKS {
                            let len = href.len().min(MAX_LINK_URL);
                            LINKS[LINK_COUNT][..len].copy_from_slice(&href[..len]);
                            LINK_LENS[LINK_COUNT] = len;
                            LINK_COUNT += 1;
                            // Link marker
                            if PAGE_LEN + 5 < MAX_PAGE {
                                push_styled(b'[', STYLE_BULLET);
                                let mut nbuf = [0u8; 4];
                                let nlen = write_num(&mut nbuf, LINK_COUNT);
                                for n in 0..nlen { push_styled(nbuf[n], STYLE_BULLET); }
                                push_styled(b']', STYLE_BULLET);
                            }
                        }
                    }
                }
            }
            if starts_with_ci(remaining, b"</a") {
                in_link = false;
                current_style = STYLE_BODY;
            }

            // Bold / Strong
            if starts_with_ci(remaining, b"<b>") || starts_with_ci(remaining, b"<b ")
                || starts_with_ci(remaining, b"<strong") {
                in_bold = true; current_style = STYLE_BOLD;
            }
            if starts_with_ci(remaining, b"</b>") || starts_with_ci(remaining, b"</strong") {
                in_bold = false; current_style = STYLE_BODY;
            }

            // Italic / Em
            if starts_with_ci(remaining, b"<i>") || starts_with_ci(remaining, b"<i ")
                || starts_with_ci(remaining, b"<em") {
                in_italic = true; current_style = STYLE_ITALIC;
            }
            if starts_with_ci(remaining, b"</i>") || starts_with_ci(remaining, b"</em") {
                in_italic = false; current_style = STYLE_BODY;
            }

            // Code / Pre
            if starts_with_ci(remaining, b"<code") || starts_with_ci(remaining, b"<pre") {
                in_code = true; current_style = STYLE_CODE;
            }
            if starts_with_ci(remaining, b"</code") || starts_with_ci(remaining, b"</pre") {
                in_code = false; current_style = STYLE_BODY;
            }

            // Blockquote
            if starts_with_ci(remaining, b"<blockquote") {
                in_blockquote = true; current_style = STYLE_QUOTE;
                push_styled(b'\n', current_style);
                // Indent marker
                push_styled(b'|', STYLE_QUOTE);
                push_styled(b' ', STYLE_QUOTE);
            }
            if starts_with_ci(remaining, b"</blockquote") {
                in_blockquote = false; current_style = STYLE_BODY;
                push_styled(b'\n', current_style);
            }

            // Lists
            if starts_with_ci(remaining, b"<ul") || starts_with_ci(remaining, b"<ol") {
                list_depth += 1;
            }
            if starts_with_ci(remaining, b"</ul") || starts_with_ci(remaining, b"</ol") {
                if list_depth > 0 { list_depth -= 1; }
            }
            if starts_with_ci(remaining, b"<li") {
                push_styled(b'\n', current_style);
                // Indent based on list depth
                for _ in 0..list_depth.saturating_sub(1) {
                    push_styled(b' ', current_style);
                    push_styled(b' ', current_style);
                }
                push_styled(b' ', STYLE_BULLET);
                push_styled(0xB7, STYLE_BULLET); // bullet character (·)
                push_styled(b' ', current_style);
                last_was_space = true;
            }

            // Horizontal rule
            if starts_with_ci(remaining, b"<hr") {
                push_styled(b'\n', current_style);
                for _ in 0..40 { push_styled(b'-', STYLE_HR); }
                push_styled(b'\n', current_style);
                last_was_space = true;
            }

            // Block-level tags → newline
            if starts_with_ci(remaining, b"<p") || starts_with_ci(remaining, b"<div")
                || starts_with_ci(remaining, b"<br") || starts_with_ci(remaining, b"<tr")
                || starts_with_ci(remaining, b"<section") || starts_with_ci(remaining, b"<article")
                || starts_with_ci(remaining, b"<header") || starts_with_ci(remaining, b"<footer")
                || starts_with_ci(remaining, b"<nav") || starts_with_ci(remaining, b"<main")
            {
                push_styled(b'\n', current_style);
                last_was_space = true;
            }
            // Double newline for paragraph spacing
            if starts_with_ci(remaining, b"<p") || starts_with_ci(remaining, b"</p") {
                push_styled(b'\n', current_style);
            }

            // Table cells
            if starts_with_ci(remaining, b"<td") || starts_with_ci(remaining, b"<th") {
                push_styled(b'\t', current_style);
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
            // Numeric entities: &#NNN; or &#xHHH;
            if i + 1 < html.len() && html[i + 1] == b'#' {
                let mut val: u32 = 0;
                let mut j = i + 2;
                if j < html.len() && (html[j] == b'x' || html[j] == b'X') {
                    // Hex: &#xHHHH;
                    j += 1;
                    while j < html.len() && html[j] != b';' {
                        let d = html[j];
                        val = val * 16 + match d {
                            b'0'..=b'9' => (d - b'0') as u32,
                            b'a'..=b'f' => (d - b'a' + 10) as u32,
                            b'A'..=b'F' => (d - b'A' + 10) as u32,
                            _ => break,
                        };
                        j += 1;
                    }
                } else {
                    // Decimal: &#NNN;
                    while j < html.len() && html[j] != b';' {
                        if html[j] >= b'0' && html[j] <= b'9' {
                            val = val * 10 + (html[j] - b'0') as u32;
                        } else { break; }
                        j += 1;
                    }
                }
                if j < html.len() && html[j] == b';' { j += 1; }
                // Convert to ASCII (or space for non-ASCII)
                let ch = if val < 128 { val as u8 } else { b' ' };
                push_styled(ch, current_style);
                i = j;
                continue;
            }
            // Named entities
            if starts_with_ci(&html[i..], b"&amp;") { push_styled(b'&', current_style); i += 5; continue; }
            if starts_with_ci(&html[i..], b"&lt;") { push_styled(b'<', current_style); i += 4; continue; }
            if starts_with_ci(&html[i..], b"&gt;") { push_styled(b'>', current_style); i += 4; continue; }
            if starts_with_ci(&html[i..], b"&nbsp;") { push_styled(b' ', current_style); i += 6; continue; }
            if starts_with_ci(&html[i..], b"&quot;") { push_styled(b'"', current_style); i += 6; continue; }
            while i < html.len() && html[i] != b';' { i += 1; }
            i += 1;
            continue;
        }

        // Collapse whitespace (but not in <pre>)
        let ch = html[i];
        if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
            if in_pre {
                push_styled(ch, current_style);
            } else if !last_was_space {
                push_styled(b' ', current_style);
                last_was_space = true;
            }
            i += 1;
            continue;
        }

        // Regular character
        if ch >= 0x20 && ch <= 0x7E {
            push_styled(ch, current_style);
            last_was_space = false;
        }
        i += 1;
    }
}

/// Extract styled text from the cleaned DOM tree (post-reader-mode).
/// This replaces strip_html when the rendering engine is active, ensuring
/// the Level 1 fallback text only contains article content (no nav/sidebar).
fn extract_text_from_dom(doc: &crate::browser::dom::Document, base_host: &str) {
    unsafe {
        PAGE_LEN = 0;
        for i in 0..MAX_PAGE { PAGE_STYLE[i] = STYLE_BODY; }
        LINK_COUNT = 0;
    }

    let body = doc.body();
    dom_text_walk(doc, body, STYLE_BODY, base_host);
}

/// Recursively walk the DOM tree and extract styled text for Level 1 rendering.
fn dom_text_walk(
    doc: &crate::browser::dom::Document,
    node_idx: usize,
    parent_style: u8,
    base_host: &str,
) {
    use crate::browser::dom::NodeType;

    let node = doc.get(node_idx);

    match node.node_type {
        NodeType::Comment | NodeType::Empty => return,
        NodeType::Text => {
            // Render text content with parent's style
            let text = &node.text[..node.text_len];
            for &b in text {
                if b >= 0x20 && b <= 0x7E {
                    push_styled(b, parent_style);
                } else if b == b'\n' || b == b'\r' {
                    // Whitespace — collapse
                }
            }
            return;
        }
        NodeType::Document => {
            // Walk children
            for child_idx in doc.children(node_idx) {
                dom_text_walk(doc, child_idx, parent_style, base_host);
            }
            return;
        }
        NodeType::Element => {
            // Determine style for this element's content
            let tag = node.tag_str();

            // Skip hidden display:none tags
            match tag {
                "script" | "style" | "head" | "meta" | "link" | "title" | "noscript" => return,
                _ => {}
            }

            let mut style = parent_style;

            // Set style based on tag
            match tag {
                "h1" => {
                    push_styled(b'\n', style); push_styled(b'\n', style);
                    style = STYLE_H1;
                }
                "h2" => {
                    push_styled(b'\n', style); push_styled(b'\n', style);
                    style = STYLE_H2;
                }
                "h3" | "h4" | "h5" | "h6" => {
                    push_styled(b'\n', style);
                    style = STYLE_H3;
                }
                "p" | "div" | "section" | "article" | "main" => {
                    push_styled(b'\n', style);
                }
                "br" => {
                    push_styled(b'\n', style);
                }
                "a" => {
                    style = STYLE_LINK;
                    // Extract href for link list
                    if let Some(href) = node.get_attr("href") {
                        unsafe {
                            if LINK_COUNT < MAX_LINKS {
                                let hb = href.as_bytes();
                                let len = hb.len().min(MAX_LINK_URL);
                                LINKS[LINK_COUNT][..len].copy_from_slice(&hb[..len]);
                                LINK_LENS[LINK_COUNT] = len;
                                LINK_COUNT += 1;
                                // Link marker
                                if PAGE_LEN + 5 < MAX_PAGE {
                                    push_styled(b'[', STYLE_BULLET);
                                    let mut nbuf = [0u8; 4];
                                    let nlen = write_num(&mut nbuf, LINK_COUNT);
                                    for n in 0..nlen { push_styled(nbuf[n], STYLE_BULLET); }
                                    push_styled(b']', STYLE_BULLET);
                                }
                            }
                        }
                    }
                }
                "b" | "strong" => { style = STYLE_BOLD; }
                "i" | "em" => { style = STYLE_ITALIC; }
                "code" | "pre" => { style = STYLE_CODE; }
                "blockquote" => {
                    push_styled(b'\n', style);
                    push_styled(b'|', STYLE_QUOTE);
                    push_styled(b' ', STYLE_QUOTE);
                    style = STYLE_QUOTE;
                }
                "li" => {
                    push_styled(b'\n', style);
                    push_styled(b' ', STYLE_BULLET);
                    push_styled(0xB7, STYLE_BULLET);
                    push_styled(b' ', style);
                }
                "hr" => {
                    push_styled(b'\n', style);
                    for _ in 0..40 { push_styled(b'-', STYLE_HR); }
                    push_styled(b'\n', style);
                }
                "td" | "th" => {
                    push_styled(b'\t', style);
                }
                "tr" => {
                    push_styled(b'\n', style);
                }
                "img" => {
                    // Show [Image: alt] placeholder
                    if let Some(alt) = node.get_attr("alt") {
                        if !alt.is_empty() {
                            push_styled(b'[', STYLE_QUOTE);
                            let prefix = b"Image: ";
                            for &b in prefix { push_styled(b, STYLE_QUOTE); }
                            for &b in alt.as_bytes() {
                                if b >= 0x20 && b <= 0x7E {
                                    push_styled(b, STYLE_QUOTE);
                                }
                            }
                            push_styled(b']', STYLE_QUOTE);
                        }
                    }
                    return; // img is void, no children
                }
                _ => {}
            }

            // Walk children
            for child_idx in doc.children(node_idx) {
                dom_text_walk(doc, child_idx, style, base_host);
            }

            // Post-tag actions
            match tag {
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    push_styled(b'\n', parent_style);
                }
                "p" => {
                    push_styled(b'\n', parent_style);
                }
                "blockquote" => {
                    push_styled(b'\n', parent_style);
                }
                "a" => {
                    // Restore parent style after link
                }
                _ => {}
            }
        }
    }
}

// ─── JavaScript Execution (Bytecode VM) ───

static mut JS_VM: crate::browser::js::vm::Vm = crate::browser::js::vm::Vm::new();
static mut JS_VM_INITIALIZED: bool = false;

/// Find and execute all inline <script> tags in the DOM
fn execute_scripts(doc: &crate::browser::dom::Document) {
    unsafe {
        // Initialize VM on first use
        if !core::ptr::read_volatile(core::ptr::addr_of!(JS_VM_INITIALIZED)) {
            (*core::ptr::addr_of_mut!(JS_VM)).init();
            core::ptr::write_volatile(core::ptr::addr_of_mut!(JS_VM_INITIALIZED), true);
            uart::puts("[js] bytecode VM initialized\n");
        }
    }

    // Find all script elements
    doc.find_all_tags("script", |script_idx| {
        // Get the text content of the script (first text child)
        for child_idx in doc.children(script_idx) {
            let child = doc.get(child_idx);
            if child.node_type == crate::browser::dom::NodeType::Text && child.text_len > 0 {
                let source = &child.text[..child.text_len];

                uart::puts("[js] executing script (");
                crate::kernel::mm::print_num(source.len());
                uart::puts(" bytes)\n");

                // Execute using the new bytecode VM
                unsafe {
                    let vm = &mut *core::ptr::addr_of_mut!(JS_VM);
                    match vm.execute(source) {
                        Ok(_val) => {
                            uart::puts("[js] script completed\n");
                        }
                        Err(_) => {
                            uart::puts("[js] script error\n");
                        }
                    }

                    // Check for console output from the VM
                    let cl = vm.console_len;
                    if cl > 0 {
                        uart::puts("[js console] ");
                        for i in 0..cl.min(200) {
                            uart::putc(vm.console_buf[i]);
                        }
                        vm.console_len = 0; // reset
                    }
                }
            }
        }
    });
}

fn push_page(ch: u8) {
    push_styled(ch, STYLE_BODY);
}

fn push_styled(ch: u8, style: u8) {
    unsafe {
        if PAGE_LEN < MAX_PAGE {
            PAGE_BUF[PAGE_LEN] = ch;
            PAGE_STYLE[PAGE_LEN] = style;
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
        } else if USE_ENGINE && (*core::ptr::addr_of!(LAYOUT_TREE)).box_count > 0
            && PAGE_LEN < 200 // Use Level 2 only for simple pages; Level 1 is better for complex ones
            && {
                let lt = &*core::ptr::addr_of!(LAYOUT_TREE);
                let mut ht = false;
                for i in 0..lt.box_count { if lt.boxes[i].active && lt.boxes[i].text_len > 0 { ht = true; break; } }
                ht
            }
        {
            // ═══ Level 2: DOM-based rendering (simple pages only) ═══
            crate::browser::paint::paint(
                &*core::ptr::addr_of!(LAYOUT_TREE),
                x as i32,          // offset_x
                y as i32,          // offset_y
                (SCROLL_Y * 18) as i32,  // scroll_y (convert line scroll to pixels)
                content_w as i32,  // clip_w
                (ymax - y - 24) as i32, // clip_h
            );

            // Also show links at bottom (from Level 1 extraction)
            if LINK_COUNT > 0 {
                let links_y = ymax - 40;
                if links_y > y + 40 {
                    gpu::fill_rect(x, links_y - 2, content_w, 1, BORDER);
                    let mut ly = links_y;
                    for li in 0..LINK_COUNT.min(3) {
                        if ly + ln < ymax - 20 {
                            let link = core::str::from_utf8_unchecked(&LINKS[li][..LINK_LENS[li]]);
                            let mut label = [0u8; 4];
                            label[0] = b'['; label[1] = b'1' + li as u8; label[2] = b']'; label[3] = b' ';
                            font::draw_str(fb, w, x + 4, ly,
                                core::str::from_utf8_unchecked(&label[..4]), LINK_COLOR, BG);
                            font::draw_str(fb, w, x + 36, ly, link, CYAN, BG);
                            ly += ln;
                        }
                    }
                }
            }
        } else {
            // ═══ Level 1: styled text rendering ═══
            // Try TrueType first (proportional, anti-aliased)
            let use_tt = crate::ui::truetype::is_available();
            let tt_size: u16 = 14; // 14px font size for body text
            let chars_per_line = if use_tt {
                ((content_w - 16) / 7) as usize // ~7px avg char width for 14px Verdana
            } else {
                ((content_w - 16) / 8) as usize // 8px monospace
            };
            let max_lines = ((ymax - y - 24) / ln) as usize;

            let text = &PAGE_BUF[..PAGE_LEN];
            let styles = &PAGE_STYLE[..PAGE_LEN];
            let mut line_num = 0usize;
            let mut ti = 0;

            while ti < text.len() && line_num < SCROLL_Y + max_lines {
                let line_start = ti;
                let mut line_end = ti;
                let mut chars = 0;

                // Check if this line starts with a big (h1) style
                let line_is_big = if ti < styles.len() { style_is_big(styles[ti]) } else { false };
                let effective_cpl = if line_is_big { chars_per_line / 2 } else { chars_per_line };

                while line_end < text.len() && text[line_end] != b'\n' && chars < effective_cpl {
                    line_end += 1;
                    chars += 1;
                }

                if line_num >= SCROLL_Y {
                    let draw_y = y + ((line_num - SCROLL_Y) as u32) * ln;
                    if draw_y + ln < ymax {
                        if use_tt {
                            // TrueType rendering: draw the entire line as a string
                            let line_text = &text[line_start..line_end];
                            let line_str = core::str::from_utf8_unchecked(line_text);
                            // Use first char's style for the whole line color
                            let color = if line_start < styles.len() {
                                style_to_color(styles[line_start])
                            } else { FG };
                            let sz = if line_is_big { 22 } else { tt_size };
                            // Get the framebuffer as a byte slice for TrueType rendering
                            let fb_bytes = core::slice::from_raw_parts_mut(
                                fb as *mut u8,
                                (w * gpu::height() * 4) as usize
                            );
                            crate::ui::truetype::draw_truetype(
                                fb_bytes, w, x + 8, draw_y, line_str, sz, color
                            );
                        } else {
                        // Bitmap font rendering: draw each character
                        let mut cx = x + 8;
                        for ci in line_start..line_end {
                            let ch = text[ci];
                            let st = styles[ci];
                            let color = style_to_color(st);

                            if line_is_big {
                                let ch_buf = [ch];
                                let s = core::str::from_utf8_unchecked(&ch_buf);
                                font::draw_str(fb, w, cx, draw_y, s, color, BG);
                                font::draw_str(fb, w, cx + 1, draw_y, s, color, BG);
                                cx += 16;
                            } else {
                                let ch_buf = [ch];
                                let s = core::str::from_utf8_unchecked(&ch_buf);
                                font::draw_str(fb, w, cx, draw_y, s, color, BG);
                                if style_is_bold(st) {
                                    font::draw_str(fb, w, cx + 1, draw_y, s, color, BG);
                                }
                                if style_is_underline(st) && ch != b' ' {
                                    gpu::fill_rect(cx, draw_y + 14, 8, 1, color);
                                }
                                cx += 8;
                            }

                            if cx > x + content_w - 8 { break; }
                        }
                        } // close else (bitmap font)
                    }
                }

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

                    if url.len() > 0 && url[0] == b'/' {
                        // Relative URL — prepend current origin (scheme + host)
                        // Extract scheme+host from current URL_BUF
                        let cur = &URL_BUF[..URL_LEN];
                        // Find end of "https://host" (third /)
                        let mut slash_count = 0;
                        let mut origin_end = 0;
                        for i in 0..cur.len() {
                            if cur[i] == b'/' {
                                slash_count += 1;
                                if slash_count == 3 { origin_end = i; break; }
                            }
                        }
                        if origin_end == 0 { origin_end = cur.len(); }
                        let total = origin_end + url.len();
                        if total <= MAX_URL {
                            url_copy[..origin_end].copy_from_slice(&cur[..origin_end]);
                            url_copy[origin_end..total].copy_from_slice(url);
                            navigate(&url_copy[..total]);
                        }
                    } else if url.starts_with(b"http") {
                        // Absolute URL
                        let len = url.len().min(MAX_URL);
                        url_copy[..len].copy_from_slice(&url[..len]);
                        navigate(&url_copy[..len]);
                    }
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

// Bat_OS — CSS Tokenizer Bridge (Rust side)
// FFI wrapper around Chromium's real CSS3 tokenizer in libblink.a.
// The kernel calls these C functions to get real Blink tokens for CSS.

#![allow(dead_code)]

use crate::drivers::uart;

// Token codes — must match enum CssTokenCode in ports/css_bridge.cpp
pub const CSS_IDENT:       i32 = 1;
pub const CSS_FUNCTION:    i32 = 2;
pub const CSS_AT_KEYWORD:  i32 = 3;
pub const CSS_HASH:        i32 = 4;
pub const CSS_STRING:      i32 = 5;
pub const CSS_NUMBER:      i32 = 6;
pub const CSS_PERCENTAGE:  i32 = 7;
pub const CSS_DIMENSION:   i32 = 8;
pub const CSS_WHITESPACE:  i32 = 9;
pub const CSS_DELIM:       i32 = 10;
pub const CSS_COLON:       i32 = 11;
pub const CSS_SEMICOLON:   i32 = 12;
pub const CSS_COMMA:       i32 = 13;
pub const CSS_LBRACE:      i32 = 14;
pub const CSS_RBRACE:      i32 = 15;
pub const CSS_LPAREN:      i32 = 16;
pub const CSS_RPAREN:      i32 = 17;
pub const CSS_LBRACKET:    i32 = 18;
pub const CSS_RBRACKET:    i32 = 19;
pub const CSS_URL:         i32 = 20;
pub const CSS_COMMENT:     i32 = 23;
pub const CSS_EOF:         i32 = 26;

unsafe extern "C" {
    fn css_tokenizer_create(css: *const u8, len: i32) -> *mut u8;
    fn css_tokenizer_next(
        handle: *mut u8,
        text_buf: *mut u8, text_len: *mut i32,
        text_cap: i32,
        numeric_value: *mut f64,
    ) -> i32;
    fn css_tokenizer_count(handle: *mut u8) -> u32;
    fn css_tokenizer_reset(handle: *mut u8);
    fn css_tokenizer_destroy(handle: *mut u8);
}

/// One token emitted by the real Chromium CSS tokenizer.
pub struct CssToken {
    pub code: i32,
    pub text: [u8; 128],
    pub text_len: usize,
    pub numeric_value: f64,
}

impl CssToken {
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.text[..self.text_len]) }
    }
}

/// RAII wrapper for the C handle.
pub struct BlinkCssTokenizer {
    handle: *mut u8,
}

impl BlinkCssTokenizer {
    pub fn new(css: &[u8]) -> Option<Self> {
        let handle = unsafe { css_tokenizer_create(css.as_ptr(), css.len() as i32) };
        if handle.is_null() { None } else { Some(Self { handle }) }
    }

    pub fn next_token(&mut self) -> CssToken {
        let mut tok = CssToken {
            code: CSS_EOF,
            text: [0u8; 128],
            text_len: 0,
            numeric_value: 0.0,
        };
        let mut tlen: i32 = 0;
        let mut nval: f64 = 0.0;
        tok.code = unsafe {
            css_tokenizer_next(
                self.handle,
                tok.text.as_mut_ptr(),
                &mut tlen,
                tok.text.len() as i32,
                &mut nval,
            )
        };
        tok.text_len = tlen as usize;
        tok.numeric_value = nval;
        tok
    }

    pub fn count(&self) -> u32 {
        unsafe { css_tokenizer_count(self.handle) }
    }

    pub fn reset(&mut self) {
        unsafe { css_tokenizer_reset(self.handle); }
    }
}

impl Drop for BlinkCssTokenizer {
    fn drop(&mut self) {
        unsafe { css_tokenizer_destroy(self.handle); }
    }
}

/// Smoke-test: tokenize a small CSS sample and print the result.
/// Demonstrates that real Chromium CSS tokenization is running on bare metal.
pub fn smoke_test() {
    let css = b"body { color: #fff; font-size: 14px; } .x { padding: 1.5em; }";
    uart::puts("[blink-css] running real Chromium CSS tokenizer...\n");

    let Some(mut tok) = BlinkCssTokenizer::new(css) else {
        uart::puts("[blink-css] FAILED to create tokenizer\n");
        return;
    };

    let mut count = 0u32;
    loop {
        let t = tok.next_token();
        if t.code == CSS_EOF { break; }
        count += 1;
        if count > 200 { break; }
        // Print a brief summary of each token
        uart::puts("[blink-css] tok#");
        print_u32(count);
        uart::puts(" code=");
        print_i32(t.code);
        if t.text_len > 0 && t.text_len < 64 {
            uart::puts(" text=\"");
            for b in &t.text[..t.text_len] {
                if *b >= 0x20 && *b < 0x7f { uart::putc(*b); }
            }
            uart::puts("\"");
        }
        uart::puts("\n");
    }

    uart::puts("[blink-css] total tokens: ");
    print_u32(count);
    uart::puts("\n");
}

fn print_u32(n: u32) {
    let mut buf = [0u8; 12];
    let mut i = 0;
    let mut x = n;
    if x == 0 { uart::putc(b'0'); return; }
    while x > 0 {
        buf[i] = b'0' + (x % 10) as u8;
        x /= 10;
        i += 1;
    }
    while i > 0 { i -= 1; uart::putc(buf[i]); }
}

fn print_i32(n: i32) {
    if n < 0 { uart::putc(b'-'); print_u32((-n) as u32); }
    else { print_u32(n as u32); }
}

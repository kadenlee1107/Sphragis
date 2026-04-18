// Bat_OS — Blink HTML Tokenizer Bridge
// Rust FFI wrapper around Chromium's real HTML5 tokenizer.
// Uses the C bridge in ports/blink_bridge.cpp via libblink.a.

use crate::drivers::uart;

// Token types from blink_bridge.cpp
const TOKEN_START_TAG: i32 = 1;
const TOKEN_END_TAG: i32 = 2;
const TOKEN_CHARACTER: i32 = 3;
const TOKEN_COMMENT: i32 = 4;
const TOKEN_DOCTYPE: i32 = 5;
const TOKEN_EOF: i32 = 6;

unsafe extern "C" {
    fn blink_tokenizer_create() -> *mut u8;
    fn blink_tokenizer_feed(handle: *mut u8, html: *const u8, len: i32);
    fn blink_tokenizer_next(
        handle: *mut u8,
        name_buf: *mut u8, name_len: *mut i32,
        text_buf: *mut u8, text_len: *mut i32,
        name_cap: i32, text_cap: i32,
    ) -> i32;
    fn blink_tokenizer_destroy(handle: *mut u8);
}

/// Parse HTML using Chromium's Blink HTML5 tokenizer and build our DOM tree.
pub fn parse_with_blink(html: &[u8], doc: &mut super::super::dom::Document) {
    doc.init();

    // Reset the Blink memory heap for this session
    super::blink_libc::reset_heap();

    uart::puts("[blink] creating tokenizer...\n");
    let handle = unsafe { blink_tokenizer_create() };
    uart::puts("[blink] tokenizer created\n");
    if handle.is_null() {
        uart::puts("[blink] tokenizer creation failed\n");
        return;
    }

    uart::puts("[blink] feeding HTML...\n");
    unsafe { blink_tokenizer_feed(handle, html.as_ptr(), html.len() as i32) };
    uart::puts("[blink] HTML fed\n");

    uart::puts("[blink] tokenizing with Chromium HTML5 parser...\n");

    // Element stack for building DOM tree
    let mut stack = [0usize; 64];
    let mut stack_depth = 1usize;
    stack[0] = 0; // root

    let mut name_buf = [0u8; 64];
    let mut text_buf = [0u8; 512];
    let mut name_len: i32 = 0;
    let mut text_len: i32 = 0;
    let mut token_count = 0u32;

    loop {
        let tok_type = unsafe {
            blink_tokenizer_next(
                handle,
                name_buf.as_mut_ptr(), &mut name_len,
                text_buf.as_mut_ptr(), &mut text_len,
                64, 512,
            )
        };

        if tok_type == 0 || tok_type == TOKEN_EOF { break; }
        token_count += 1;

        // V8-ROOT-10: FFI returns i32 name_len/text_len. An attacker-influenced
        // Blink side could return -1 (wraps to usize::MAX when cast) or any
        // value larger than the actual buffer; clamp before every slice.
        let name_end = (name_len.max(0) as usize).min(name_buf.len());
        let text_end = (text_len.max(0) as usize).min(text_buf.len());

        match tok_type {
            TOKEN_START_TAG => {
                let tag = &name_buf[..name_end];
                let tag_str = unsafe { core::str::from_utf8_unchecked(tag) };

                // Skip non-visual tags
                if tag_str == "script" || tag_str == "style" || tag_str == "meta"
                    || tag_str == "link" || tag_str == "head" || tag_str == "noscript" {
                    continue;
                }

                let parent = stack[stack_depth - 1];
                if let Some(idx) = doc.add_element(parent, tag_str) {
                    // Push to stack for nesting
                    if stack_depth < 63 {
                        stack[stack_depth] = idx;
                        stack_depth += 1;
                    }
                }
            }
            TOKEN_END_TAG => {
                let tag = &name_buf[..name_end];
                let tag_str = unsafe { core::str::from_utf8_unchecked(tag) };

                // Pop stack
                if stack_depth > 1 {
                    let top = stack[stack_depth - 1];
                    if doc.get(top).tag_str() == tag_str {
                        stack_depth -= 1;
                    } else {
                        // Mismatched tag — try popping until we find it
                        let mut found = false;
                        for d in (1..stack_depth).rev() {
                            if doc.get(stack[d]).tag_str() == tag_str {
                                stack_depth = d;
                                found = true;
                                break;
                            }
                        }
                        if !found && stack_depth > 1 {
                            stack_depth -= 1; // just pop one
                        }
                    }
                }
            }
            TOKEN_CHARACTER => {
                let text = &text_buf[..text_end];
                // Skip whitespace-only nodes
                let mut all_ws = true;
                for &b in text.iter() {
                    if b != b' ' && b != b'\n' && b != b'\r' && b != b'\t' {
                        all_ws = false;
                        break;
                    }
                }
                if all_ws { continue; }

                let parent = stack[stack_depth - 1];
                doc.add_text(parent, text);
            }
            _ => {} // DOCTYPE, COMMENT — ignore
        }

        if token_count > 2000 { break; } // safety limit
    }

    unsafe { blink_tokenizer_destroy(handle) };

    uart::puts("[blink] ");
    crate::kernel::mm::print_num(token_count as usize);
    uart::puts(" tokens → ");
    crate::kernel::mm::print_num(doc.node_count);
    uart::puts(" DOM nodes\n");
}

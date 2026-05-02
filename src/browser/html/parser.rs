// Bat_OS — HTML Parser
// Converts raw HTML bytes into a DOM tree.
// Handles malformed HTML gracefully (like real browsers).

use super::super::dom::{Document, NodeType, MAX_ATTRS, MAX_NAME, MAX_VALUE};

fn starts_with_bytes(hay: &[u8], needle: &[u8]) -> bool {
    if hay.len() < needle.len() { return false; }
    &hay[..needle.len()] == needle
}

/// STUMP #107 — Sprint 3.3: decode the next UTF-8 codepoint from
/// `bytes`. Returns `(codepoint, byte_count)`. On malformed input
/// returns `(U+FFFD, 1)` so the caller advances by exactly one byte
/// and can keep parsing. ASCII passthrough is one byte.
///
/// STUMP #111 (audit H001/H002): rejects overlong encodings AND
/// surrogate codepoints (U+D800..U+DFFF). RFC 3629 §3 requires UTF-8
/// decoders to refuse these — overlongs are a known smuggling vector
/// (NUL-via-overlong bypasses naive byte filters; pre-fix our decoder
/// happily returned U+0000 for `0xC0 0x80` which then got '?'-mapped,
/// silently obscuring the smuggle). Surrogates are reserved for
/// UTF-16 pairs and have no UTF-8 representation; reject as malformed.
fn decode_utf8(bytes: &[u8]) -> (u32, usize) {
    if bytes.is_empty() { return (0, 0); }
    let b0 = bytes[0];
    if b0 < 0x80 { return (b0 as u32, 1); }
    if b0 < 0xC0 { return (0xFFFD, 1); } // stray continuation byte
    let nbytes = if b0 < 0xE0 { 2 } else if b0 < 0xF0 { 3 } else { 4 };
    if bytes.len() < nbytes { return (0xFFFD, 1); }
    let mask: u32 = match nbytes { 2 => 0x1F, 3 => 0x0F, 4 => 0x07, _ => 0x7F };
    let mut cp: u32 = (b0 as u32) & mask;
    for k in 1..nbytes {
        if (bytes[k] & 0xC0) != 0x80 { return (0xFFFD, k); }
        cp = (cp << 6) | (bytes[k] & 0x3F) as u32;
    }
    // Reject overlong encodings — minimum codepoint per byte length:
    // 2 bytes → ≥ 0x80, 3 bytes → ≥ 0x800, 4 bytes → ≥ 0x10000.
    let min_cp: u32 = match nbytes { 2 => 0x80, 3 => 0x800, 4 => 0x10000, _ => 0 };
    if cp < min_cp { return (0xFFFD, nbytes); }
    // Reject UTF-16 surrogate halves and codepoints above Unicode max.
    if (0xD800..=0xDFFF).contains(&cp) || cp > 0x10FFFF { return (0xFFFD, nbytes); }
    (cp, nbytes)
}

/// STUMP #107: best-effort ASCII fallback for a Unicode codepoint.
/// Returns a slice of replacement bytes (usually 1 char, occasionally
/// 3 for ellipsis or 2 for AE/OE). Designed to make non-ASCII text
/// READABLE, not preserve typographic fidelity. Real font fallback /
/// glyph rendering for non-Latin scripts is a follow-up.
fn ascii_fallback(cp: u32) -> &'static [u8] {
    match cp {
        // C1 controls and friends
        0x00A0 => b" ",                                    // nbsp
        0x00A2 => b"c",                                    // ¢
        0x00A3 => b"L",                                    // £
        0x00A5 => b"Y",                                    // ¥
        0x00A7 => b"S",                                    // §
        0x00A9 => b"(c)",                                  // ©
        0x00AB => b"<<",                                   // «
        0x00AD => b"-",                                    // soft hyphen
        0x00AE => b"(R)",                                  // ®
        0x00B0 => b"o",                                    // °
        0x00B1 => b"+/-",                                  // ±
        0x00B2 => b"2",                                    // ²
        0x00B3 => b"3",                                    // ³
        0x00B5 => b"u",                                    // µ
        0x00B6 => b"P",                                    // ¶
        0x00B7 => b"\xB7",                                 // middle dot — paint draws as bullet
        0x00BB => b">>",                                   // »
        0x00BC => b"1/4",
        0x00BD => b"1/2",
        0x00BE => b"3/4",
        0x00BF => b"?",                                    // ¿
        // Latin-1 Supplement: accented Latin letters → unaccented
        0x00C0..=0x00C5 => b"A",
        0x00C6           => b"AE",
        0x00C7           => b"C",
        0x00C8..=0x00CB => b"E",
        0x00CC..=0x00CF => b"I",
        0x00D0           => b"D",
        0x00D1           => b"N",
        0x00D2..=0x00D6 | 0x00D8 => b"O",
        0x00D7           => b"x",
        0x00D9..=0x00DC => b"U",
        0x00DD           => b"Y",
        0x00DE           => b"P",                          // thorn
        0x00DF           => b"ss",                         // eszett
        0x00E0..=0x00E5 => b"a",
        0x00E6           => b"ae",
        0x00E7           => b"c",
        0x00E8..=0x00EB => b"e",
        0x00EC..=0x00EF => b"i",
        0x00F0           => b"d",
        0x00F1           => b"n",
        0x00F2..=0x00F6 | 0x00F8 => b"o",
        0x00F7           => b"/",
        0x00F9..=0x00FC => b"u",
        0x00FD | 0x00FF => b"y",
        0x00FE           => b"p",
        // Common General Punctuation
        0x2010..=0x2015 => b"-",                            // dashes
        0x2018 | 0x2019 => b"'",                            // single quotes
        0x201A           => b",",
        0x201C | 0x201D => b"\"",                           // double quotes
        0x201E           => b",,",
        0x2020           => b"+",                           // dagger
        0x2022           => b"\xB7",                        // bullet
        0x2026           => b"...",                         // ellipsis
        0x2030           => b"%",                           // per mille (close enough)
        0x2039           => b"<",
        0x203A           => b">",
        // Currency
        0x20AC           => b"EUR",
        // Misc that show up a lot
        0x2122           => b"TM",
        0xFFFD           => b"?",                           // replacement char
        // Anything else — single '?' so the layout still reflects a glyph.
        _                => b"?",
    }
}

/// Parse HTML bytes into a DOM tree.
pub fn parse(html: &[u8], doc: &mut Document) {
    doc.init();

    let mut i = 0;
    // Stack of open element indices (for nesting)
    let mut stack = [0usize; 64];
    let mut stack_depth = 0usize;
    stack[0] = 0; // document root
    stack_depth = 1;

    while i < html.len() {
        if html[i] == b'<' {
            // ─── Comment ───
            if i + 4 < html.len() && html[i+1] == b'!' && html[i+2] == b'-' && html[i+3] == b'-' {
                // Skip to -->
                i += 4;
                while i + 2 < html.len() {
                    if html[i] == b'-' && html[i+1] == b'-' && html[i+2] == b'>' {
                        i += 3;
                        break;
                    }
                    i += 1;
                }
                continue;
            }

            // ─── DOCTYPE ───
            if i + 1 < html.len() && html[i+1] == b'!' {
                while i < html.len() && html[i] != b'>' { i += 1; }
                i += 1;
                continue;
            }

            // ─── End tag ───
            if i + 1 < html.len() && html[i+1] == b'/' {
                i += 2;
                // Read tag name
                let mut tag = [0u8; MAX_NAME];
                let mut tlen = 0;
                while i < html.len() && html[i] != b'>' && tlen < MAX_NAME {
                    if html[i] != b' ' && html[i] != b'\t' && html[i] != b'\n' {
                        tag[tlen] = html[i].to_ascii_lowercase();
                        tlen += 1;
                    }
                    i += 1;
                }
                if i < html.len() { i += 1; } // skip >

                // Pop stack until we find the matching open tag
                let tag_str = unsafe { core::str::from_utf8_unchecked(&tag[..tlen]) };
                while stack_depth > 1 {
                    let top = stack[stack_depth - 1];
                    if doc.get(top).tag_str() == tag_str {
                        stack_depth -= 1;
                        break;
                    }
                    // Auto-close mismatched tag
                    stack_depth -= 1;
                }
                continue;
            }

            // ─── Start tag ───
            i += 1;
            let mut tag = [0u8; MAX_NAME];
            let mut tlen = 0;

            // Read tag name
            while i < html.len() && html[i] != b'>' && html[i] != b' '
                && html[i] != b'\t' && html[i] != b'\n' && html[i] != b'/'
                && tlen < MAX_NAME
            {
                tag[tlen] = html[i].to_ascii_lowercase();
                tlen += 1;
                i += 1;
            }

            if tlen == 0 {
                // Empty tag name — skip
                while i < html.len() && html[i] != b'>' { i += 1; }
                if i < html.len() { i += 1; }
                continue;
            }

            // Create element node
            let elem_idx = match doc.create_element(&tag[..tlen]) {
                Some(idx) => idx,
                None => break, // out of nodes
            };

            // Parse attributes
            parse_attributes(html, &mut i, doc, elem_idx);

            // Check for self-closing />
            let self_closing = i > 0 && html[i.saturating_sub(1)] == b'/';

            // Skip to end of tag
            while i < html.len() && html[i] != b'>' { i += 1; }
            if i < html.len() { i += 1; }

            // --- Skip hidden elements ---
            // Check for `hidden` attribute
            {
                let n = doc.get(elem_idx);
                let mut skip = false;
                for a in 0..n.attr_count {
                    let aname = n.attrs[a].name_str();
                    let aval  = n.attrs[a].value_str();
                    // hidden attribute (boolean)
                    if aname == "hidden" { skip = true; break; }
                    // aria-hidden="true"
                    if aname == "aria-hidden" && aval == "true" { skip = true; break; }
                    // style="display:none" or style="...display: none..."
                    if aname == "style" && contains_display_none(aval) { skip = true; break; }
                    // Wikipedia-specific hidden classes
                    if aname == "class" {
                        if str_contains(aval, "mw-jump-link")
                            || str_contains(aval, "noprint")
                            || str_contains(aval, "mw-editsection")
                            || str_contains(aval, "mw-indicators")
                            || str_contains(aval, "mw-hidden-catlinks")
                            || str_contains(aval, "sistersitebox")
                            || str_contains(aval, "mw-empty-elt")
                        {
                            skip = true;
                            break;
                        }
                    }
                }
                if skip {
                    // Skip to matching closing tag (consume all nested content)
                    let _t_tag = doc.get(elem_idx).tag_str();
                    if !doc.get(elem_idx).is_void() {
                        let mut depth = 1u32;
                        while i < html.len() && depth > 0 {
                            if html[i] == b'<' {
                                if i + 1 < html.len() && html[i+1] == b'/' {
                                    // closing tag — check if it matches
                                    depth -= 1;
                                } else if i + 1 < html.len() && html[i+1] != b'!' {
                                    // opening tag (not comment/doctype)
                                    // We approximate: just track depth
                                    depth += 1;
                                }
                                // skip to end of tag
                                while i < html.len() && html[i] != b'>' { i += 1; }
                                if i < html.len() { i += 1; }
                            } else {
                                i += 1;
                            }
                        }
                    }
                    // Reclaim the node slot
                    doc.node_count -= 1;
                    continue;
                }
            }

            // Append to current parent
            let parent = stack[stack_depth - 1];
            doc.append_child(parent, elem_idx);

            // Capture <script> content into Document.js_text and
            // extract <style> content. Pre-fix the parser dropped
            // script content entirely; STUMP #84 wires it through
            // so the JS engine can run after parse.
            //
            // STUMP #88: copy tag bytes into a tiny stack buffer instead
            // of borrowing through `doc.get(...)`. The `<link>` capture
            // below mutates `doc.link_urls`, which collides with the
            // immutable borrow tag_str would otherwise hold across all
            // the `if tag_str == "..."` checks in this block.
            let mut tag_buf = [0u8; 16];
            let tag_buf_len = {
                let n = doc.get(elem_idx);
                let bytes = n.tag_bytes();
                let l = bytes.len().min(tag_buf.len());
                tag_buf[..l].copy_from_slice(&bytes[..l]);
                l
            };
            let tag_str = unsafe { core::str::from_utf8_unchecked(&tag_buf[..tag_buf_len]) };
            if tag_str == "script" {
                let js_start = i;
                let close = b"</script>" as &[u8];
                while i + close.len() <= html.len() {
                    if starts_with_ci(&html[i..], close) {
                        break;
                    }
                    i += 1;
                }
                // Append the script body to doc.js_text, with a `;\n`
                // separator so multiple scripts compose.
                let js_bytes = &html[js_start..i];
                let avail = super::super::dom::MAX_JS - doc.js_len;
                let copy_len = (js_bytes.len() + 2).min(avail);
                if copy_len > 2 {
                    doc.js_text[doc.js_len..doc.js_len + copy_len - 2]
                        .copy_from_slice(&js_bytes[..copy_len - 2]);
                    doc.js_len += copy_len - 2;
                    if doc.js_len + 2 <= super::super::dom::MAX_JS {
                        doc.js_text[doc.js_len] = b';';
                        doc.js_text[doc.js_len + 1] = b'\n';
                        doc.js_len += 2;
                    }
                }
                i += close.len();
                continue;
            }
            // STUMP #88: capture `<link rel="stylesheet" href="...">`
            // hrefs into Document.link_urls so cmd_render can fetch
            // them over HTTP before layout. Only stylesheet rel matches —
            // <link rel=icon> etc. fall through to the regular skip path.
            if tag_str == "link" {
                // Snapshot rel + href into stack-local buffers BEFORE
                // mutating doc.link_urls — same borrow-checker dance as
                // tag_buf above.
                let mut is_stylesheet = false;
                let mut href_buf = [0u8; super::super::dom::MAX_LINK_URL];
                let mut href_len = 0usize;
                {
                    let n = doc.get(elem_idx);
                    for a in 0..n.attr_count {
                        let aname = n.attrs[a].name_str();
                        let aval  = n.attrs[a].value_str();
                        if aname == "rel" && aval == "stylesheet" {
                            is_stylesheet = true;
                        }
                        if aname == "href" {
                            let l = aval.len().min(href_buf.len());
                            href_buf[..l].copy_from_slice(&aval.as_bytes()[..l]);
                            href_len = l;
                        }
                    }
                }
                if is_stylesheet && href_len > 0
                    && doc.link_count < super::super::dom::MAX_LINKS
                {
                    let slot = doc.link_count;
                    doc.link_urls[slot][..href_len]
                        .copy_from_slice(&href_buf[..href_len]);
                    doc.link_lens[slot] = href_len as u16;
                    doc.link_count += 1;
                }
                // <link> is void; nothing to push, fall through to the
                // void-element handling below.
            }

            if tag_str == "style" {
                // Extract CSS text into document's css_text buffer
                let css_start = i;
                let close = b"</style>" as &[u8];
                while i + close.len() <= html.len() {
                    if starts_with_ci(&html[i..], close) {
                        break;
                    }
                    i += 1;
                }
                // Copy CSS content
                let css_end = i;
                let css_bytes = &html[css_start..css_end];
                let avail = super::super::dom::MAX_CSS - doc.css_len;
                let copy_len = css_bytes.len().min(avail);
                if copy_len > 0 {
                    doc.css_text[doc.css_len..doc.css_len + copy_len]
                        .copy_from_slice(&css_bytes[..copy_len]);
                    doc.css_len += copy_len;
                }
                i += close.len();
                continue;
            }

            // Push onto stack if not void/self-closing
            if !doc.get(elem_idx).is_void() && !self_closing {
                if stack_depth < 64 {
                    stack[stack_depth] = elem_idx;
                    stack_depth += 1;
                }
            }
        } else {
            // ─── Text content ───
            let text_start = i;
            while i < html.len() && html[i] != b'<' {
                i += 1;
            }

            let raw_text = &html[text_start..i];
            // 🎯 STUMP #77: pure-whitespace text between two inline
            // tags must collapse to a SINGLE space (not nothing).
            // Pre-fix: <span>A</span> <span>B</span> rendered as
            // "AB" because the inter-tag whitespace was dropped
            // entirely. Browsers render that as "A B" — match.
            let has_visible = raw_text.iter().any(|&b|
                b != b' ' && b != b'\t' && b != b'\n' && b != b'\r');
            let ws_only = !raw_text.is_empty() && !has_visible;
            if ws_only {
                // Emit a single-space text node so the inline run
                // doesn't snap two siblings together.
                if let Some(text_idx) = doc.create_text(b" ") {
                    let parent = stack[stack_depth - 1];
                    doc.append_child(parent, text_idx);
                }
            }
            if has_visible {
                // STUMP #96: stream-decode + collapse + emit so a long
                // text run becomes multiple sibling text nodes instead
                // of being clipped at MAX_TEXT. Pre-fix the parser
                // staged everything in a fixed-size [u8; 2048] decoded
                // buffer then a [u8; 1024] collapsed buffer, so 75 %+
                // of httpbin.org/html's 3.5 KB Moby-Dick excerpt
                // disappeared. Now we collapse into a single MAX_TEXT
                // chunk; when it fills, we emit the chunk as a text
                // node and start a new one. The layout already flows
                // adjacent text nodes inline so the boundary is
                // invisible in the rendered output.
                use super::super::dom::MAX_TEXT;
                let parent = stack[stack_depth - 1];
                let mut chunk = [0u8; MAX_TEXT];
                let mut clen = 0usize;
                let mut last_space = false;

                let flush = |doc: &mut Document, chunk: &[u8]| {
                    if !chunk.is_empty() {
                        if let Some(idx) = doc.create_text(chunk) {
                            doc.append_child(parent, idx);
                        }
                    }
                };

                let mut emit = |doc: &mut Document,
                                 chunk: &mut [u8; MAX_TEXT],
                                 clen: &mut usize,
                                 last_space: &mut bool,
                                 b: u8| {
                    let is_ws = b == b' ' || b == b'\t' || b == b'\n' || b == b'\r';
                    let to_push = if is_ws {
                        if *last_space { return; }
                        *last_space = true;
                        b' '
                    } else {
                        *last_space = false;
                        b
                    };
                    if *clen >= MAX_TEXT - 1 {
                        // STUMP #111 (audit H012): on a >MAX_TEXT word
                        // (no space found in the trailing 64 bytes),
                        // pre-fix `break_at == *clen`, `tail_len == 0`,
                        // and we'd flush the FULL chunk and reset to 0
                        // — eating the next byte we tried to push as the
                        // start of a new word. Now: when no space found,
                        // we explicitly hard-break mid-word at clen-1
                        // (move the last byte into slot 0). This loses
                        // one inter-byte boundary but keeps the byte
                        // stream intact across the chunk boundary.
                        let mut break_at = *clen;
                        let mut found_space = false;
                        let mut k = *clen;
                        while k > 0 {
                            k -= 1;
                            if chunk[k] == b' ' {
                                break_at = k;
                                found_space = true;
                                break;
                            }
                            if *clen - k > 64 { break; }
                        }
                        if !found_space {
                            // No space within last 64 bytes — hard-break
                            // mid-word at clen-1 so the next byte
                            // continues seamlessly.
                            break_at = clen.saturating_sub(1);
                        }
                        flush(doc, &chunk[..break_at]);
                        let tail_len = *clen - break_at;
                        if tail_len > 0 && tail_len < MAX_TEXT {
                            for i in 0..tail_len {
                                chunk[i] = chunk[break_at + i];
                            }
                        }
                        *clen = if tail_len < MAX_TEXT { tail_len } else { 0 };
                    }
                    chunk[*clen] = to_push;
                    *clen += 1;
                };

                let mut j = 0usize;
                while j < raw_text.len() {
                    if raw_text[j] == b'&' {
                        let rest = &raw_text[j..];
                        if starts_with_bytes(rest, b"&nbsp;") { emit(doc, &mut chunk, &mut clen, &mut last_space, b' '); j += 6; }
                        else if starts_with_bytes(rest, b"&amp;") { emit(doc, &mut chunk, &mut clen, &mut last_space, b'&'); j += 5; }
                        else if starts_with_bytes(rest, b"&lt;") { emit(doc, &mut chunk, &mut clen, &mut last_space, b'<'); j += 4; }
                        else if starts_with_bytes(rest, b"&gt;") { emit(doc, &mut chunk, &mut clen, &mut last_space, b'>'); j += 4; }
                        else if starts_with_bytes(rest, b"&quot;") { emit(doc, &mut chunk, &mut clen, &mut last_space, b'"'); j += 6; }
                        else if starts_with_bytes(rest, b"&#39;") || starts_with_bytes(rest, b"&apos;") {
                            emit(doc, &mut chunk, &mut clen, &mut last_space, b'\'');
                            j += if rest[1] == b'#' { 5 } else { 6 };
                        }
                        else if starts_with_bytes(rest, b"&copy;") { emit(doc, &mut chunk, &mut clen, &mut last_space, b'c'); j += 6; }
                        else if starts_with_bytes(rest, b"&mdash;") || starts_with_bytes(rest, b"&#8212;") {
                            emit(doc, &mut chunk, &mut clen, &mut last_space, b'-');
                            j += 7;
                        }
                        else {
                            emit(doc, &mut chunk, &mut clen, &mut last_space, b'&');
                            j += 1;
                        }
                    } else if raw_text[j] >= 0x80 {
                        // STUMP #107 — Sprint 3.3: UTF-8 decode at parse time.
                        // The layout / paint path is ASCII-only because we
                        // ship one Verdana TT font with no codepoints
                        // beyond U+007F. Pre-fix, every multi-byte UTF-8
                        // sequence was either dropped (paint's "ch < 0x20
                        // || ch > 0x7E { skip }") or stamped one
                        // continuation byte at a time as a black square.
                        // Visible result on Wikipedia's language sidebar:
                        // most non-Latin entries were unreadable.
                        // Now we decode the codepoint here and substitute
                        // an ASCII fallback (best-effort transliteration
                        // for accented Latin + common typographic punct;
                        // '?' otherwise). Real font fallback / actual
                        // Unicode rendering is the next milestone.
                        let (cp, n) = decode_utf8(&raw_text[j..]);
                        for &b in ascii_fallback(cp).iter() {
                            emit(doc, &mut chunk, &mut clen, &mut last_space, b);
                        }
                        j += n.max(1);
                    } else {
                        emit(doc, &mut chunk, &mut clen, &mut last_space, raw_text[j]);
                        j += 1;
                    }
                }
                if clen > 0 {
                    flush(doc, &chunk[..clen]);
                }
            }
        }
    }
}

/// Parse attributes from inside a start tag
fn parse_attributes(html: &[u8], pos: &mut usize, doc: &mut Document, elem: usize) {
    let mut i = *pos;
    let node = doc.get_mut(elem);
    let mut count = 0usize;

    // STUMP #111 (audit M-attrs-hardcode): use MAX_ATTRS, not the
    // literal 8. Pre-fix the parser silently capped at 8 even if
    // MAX_ATTRS were ever bumped — the parser would honor only the
    // first 8 attrs and the new slot would never see traffic from
    // real pages. Source-of-truth is dom::MAX_ATTRS.
    while i < html.len() && html[i] != b'>' && count < MAX_ATTRS {
        // Skip whitespace
        while i < html.len() && (html[i] == b' ' || html[i] == b'\t' || html[i] == b'\n') {
            i += 1;
        }
        if i >= html.len() || html[i] == b'>' || html[i] == b'/' { break; }

        // Read attribute name
        let mut name = [0u8; MAX_NAME];
        let mut nlen = 0;
        while i < html.len() && html[i] != b'=' && html[i] != b'>'
            && html[i] != b' ' && html[i] != b'/' && nlen < MAX_NAME
        {
            name[nlen] = html[i].to_ascii_lowercase();
            nlen += 1;
            i += 1;
        }

        if nlen == 0 { i += 1; continue; }

        let mut value = [0u8; MAX_VALUE];
        let mut vlen = 0;

        // Check for = sign
        // Skip whitespace around =
        while i < html.len() && html[i] == b' ' { i += 1; }
        if i < html.len() && html[i] == b'=' {
            i += 1;
            while i < html.len() && html[i] == b' ' { i += 1; }

            if i < html.len() && (html[i] == b'"' || html[i] == b'\'') {
                // Quoted value
                let quote = html[i];
                i += 1;
                while i < html.len() && html[i] != quote && vlen < MAX_VALUE {
                    value[vlen] = html[i];
                    vlen += 1;
                    i += 1;
                }
                if i < html.len() { i += 1; } // skip closing quote
            } else {
                // Unquoted value
                while i < html.len() && html[i] != b' ' && html[i] != b'>'
                    && html[i] != b'/' && vlen < MAX_VALUE
                {
                    value[vlen] = html[i];
                    vlen += 1;
                    i += 1;
                }
            }
        }

        // Store attribute
        node.attrs[count].set_name(&name[..nlen]);
        node.attrs[count].set_value(&value[..vlen]);
        count += 1;
    }
    node.attr_count = count;
    *pos = i;
}

fn starts_with_ci(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() { return false; }
    for i in 0..needle.len() {
        if haystack[i].to_ascii_lowercase() != needle[i].to_ascii_lowercase() {
            return false;
        }
    }
    true
}

/// Check if a style attribute value contains "display:none" or "display: none"
fn contains_display_none(style: &str) -> bool {
    let s = style.as_bytes();
    let pat = b"display";
    let none = b"none";
    let mut i = 0;
    while i + pat.len() < s.len() {
        if starts_with_ci(&s[i..], pat) {
            // Found "display", look for : then "none"
            let mut j = i + pat.len();
            // skip whitespace
            while j < s.len() && (s[j] == b' ' || s[j] == b'\t') { j += 1; }
            if j < s.len() && s[j] == b':' {
                j += 1;
                while j < s.len() && (s[j] == b' ' || s[j] == b'\t') { j += 1; }
                if j + none.len() <= s.len() && starts_with_ci(&s[j..], none) {
                    return true;
                }
            }
        }
        i += 1;
    }
    false
}

/// Simple substring search: does `haystack` contain `needle`?
fn str_contains(haystack: &str, needle: &str) -> bool {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.len() > h.len() { return false; }
    let end = h.len() - n.len() + 1;
    for i in 0..end {
        if &h[i..i + n.len()] == n { return true; }
    }
    false
}

// ─── Reader Mode: Content Extraction ───
//
// After parsing the full HTML into a DOM tree, this function identifies the
// "main content" node and hides everything outside it — navigation, sidebars,
// footers, forms, ads, etc. The result is a clean article-only view.

/// Extract the main article content from a parsed DOM tree.
/// Hides non-content elements by setting their node_type to Comment
/// (which the layout engine already skips).
pub fn extract_content(doc: &mut Document) {
    // Phase 1: Find the main content root node.
    let content_root = find_content_root(doc);

    crate::drivers::uart::puts("[reader] content_root=");
    crate::kernel::mm::print_num(content_root);
    if content_root < doc.node_count {
        let node = &doc.nodes[content_root];
        crate::drivers::uart::puts(" tag=");
        crate::drivers::uart::puts(node.tag_str());
        // Print id if present
        for a in 0..node.attr_count {
            let attr = &node.attrs[a];
            if &attr.name[..attr.name_len] == b"id" {
                crate::drivers::uart::puts(" id=");
                crate::drivers::uart::puts(unsafe { core::str::from_utf8_unchecked(&attr.value[..attr.value_len]) });
            }
        }
    }
    crate::drivers::uart::puts("\n");

    // Phase 2: Hide non-content elements
    hide_non_content(doc, content_root);

    // Count how many visible nodes remain
    let mut visible = 0;
    for i in 0..doc.node_count {
        if doc.nodes[i].node_type != crate::browser::dom::NodeType::Comment { visible += 1; }
    }
    crate::drivers::uart::puts("[reader] visible nodes after extraction: ");
    crate::kernel::mm::print_num(visible);
    crate::drivers::uart::puts("\n");
}

/// Find the best candidate for the main content container.
/// Returns the node index of the content root.
fn find_content_root(doc: &Document) -> usize {
    // 1. <main> element
    if let Some(idx) = doc.find_tag("main") {
        return idx;
    }

    // 2. <article> element
    if let Some(idx) = doc.find_tag("article") {
        return idx;
    }

    // 3. id="mw-content-text" (Wikipedia article body)
    if let Some(idx) = doc.find_by_id("mw-content-text") {
        return idx;
    }

    // 4. id="bodyContent" (Wikipedia)
    if let Some(idx) = doc.find_by_id("bodyContent") {
        return idx;
    }

    // 5. id="content" (common pattern)
    if let Some(idx) = doc.find_by_id("content") {
        return idx;
    }

    // 6. class="post-content" or class="entry-content" (blogs)
    for i in 0..doc.node_count {
        if doc.has_class(i, "post-content") || doc.has_class(i, "entry-content") {
            return i;
        }
    }

    // 7. class="article-body" (news sites)
    for i in 0..doc.node_count {
        if doc.has_class(i, "article-body") {
            return i;
        }
    }

    // 8. role="main"
    for i in 0..doc.node_count {
        if doc.has_role(i, "main") {
            return i;
        }
    }

    // 9. Fallback: <body>
    doc.body()
}

/// Hide non-content elements by reparenting the content root under body.
/// Instead of hiding individual nodes (which breaks tree traversal),
/// we simply make body's first_child point to the content root.
/// All siblings of the content root's ancestor chain become unreachable.
fn hide_non_content(doc: &mut Document, content_root: usize) {
    let body = doc.body();

    if content_root == body {
        // Content root is body itself — just strip noise elements inside
        for i in 1..doc.node_count {
            if doc.nodes[i].node_type != NodeType::Element { continue; }
            if should_strip_inside(doc, i) {
                doc.nodes[i].node_type = NodeType::Comment;
            }
        }
        return;
    }

    // Content root is NOT body — reparent it as body's direct child.
    // This makes the layout walker go body → content_root → article content,
    // skipping all the navigation/sidebar that's between body and content_root.
    doc.nodes[body].first_child = content_root as u16;
    doc.nodes[content_root].parent = body as u16;
    doc.nodes[content_root].next_sibling = 0xFFFF; // no siblings

    // Also strip noise inside the content root
    for i in 1..doc.node_count {
        if doc.nodes[i].node_type != NodeType::Element { continue; }
        if doc.is_descendant_of(i, content_root) {
            if should_strip_inside(doc, i) {
                doc.nodes[i].node_type = NodeType::Comment;
            }
        }
    }
}

/// Check if a node inside the content area should be stripped.
/// These are noise elements that appear even within article content.
fn should_strip_inside(doc: &Document, idx: usize) -> bool {
    let node = &doc.nodes[idx];
    let tag = node.tag_str();

    // Strip by tag name
    match tag {
        "nav" | "aside" | "form" => return true,
        "script" | "style" | "link" | "meta" | "noscript" => return true,
        _ => {}
    }

    // Strip by role attribute
    if doc.has_role(idx, "navigation") || doc.has_role(idx, "banner")
        || doc.has_role(idx, "complementary")
    {
        return true;
    }

    // Strip by specific class names (conservative — only clear noise patterns)
    // NOTE: Do NOT match substrings like "nav" or "menu" — too broad!
    // Wikipedia has classes like "mw-parser-output" containing "put" etc.
    if doc.has_class(idx, "sidebar")
        || doc.has_class(idx, "widget")
        || doc.has_class(idx, "ad-container")
        || doc.has_class(idx, "popup")
        || doc.has_class(idx, "mw-panel")
        || doc.has_class(idx, "catlinks")
        || doc.has_class(idx, "noprint")
        || doc.has_class(idx, "mw-editsection")
    {
        return true;
    }

    // Strip header/footer tags (they are structural noise)
    if tag == "header" || tag == "footer" {
        return true;
    }

    false
}

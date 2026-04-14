// Bat_OS — HTML Parser
// Converts raw HTML bytes into a DOM tree.
// Handles malformed HTML gracefully (like real browsers).

use super::super::dom::{Document, NodeType, Attribute, MAX_NAME, MAX_VALUE};

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

            // Append to current parent
            let parent = stack[stack_depth - 1];
            doc.append_child(parent, elem_idx);

            // Skip <script> and <style> content
            let tag_str = doc.get(elem_idx).tag_str();
            if tag_str == "script" || tag_str == "style" {
                // Find the closing tag
                let close = if tag_str == "script" { b"</script>" as &[u8] } else { b"</style>" };
                while i + close.len() <= html.len() {
                    if starts_with_ci(&html[i..], close) {
                        i += close.len();
                        break;
                    }
                    i += 1;
                }
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
            if raw_text.iter().any(|&b| b != b' ' && b != b'\t' && b != b'\n' && b != b'\r') {
                // Non-whitespace text — create text node
                // Collapse whitespace
                let mut collapsed = [0u8; 256];
                let mut clen = 0;
                let mut last_space = true;
                for &b in raw_text {
                    if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                        if !last_space && clen < 255 {
                            collapsed[clen] = b' ';
                            clen += 1;
                            last_space = true;
                        }
                    } else if clen < 255 {
                        // Decode common entities inline
                        collapsed[clen] = b;
                        clen += 1;
                        last_space = false;
                    }
                }
                // Trim leading space
                let start = if clen > 0 && collapsed[0] == b' ' { 1 } else { 0 };
                if clen > start {
                    if let Some(text_idx) = doc.create_text(&collapsed[start..clen]) {
                        let parent = stack[stack_depth - 1];
                        doc.append_child(parent, text_idx);
                    }
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

    while i < html.len() && html[i] != b'>' && count < 8 {
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

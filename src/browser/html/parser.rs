// Bat_OS — HTML Parser
// Converts raw HTML bytes into a DOM tree.
// Handles malformed HTML gracefully (like real browsers).

use super::super::dom::{Document, NodeType, MAX_NAME, MAX_VALUE};

fn starts_with_bytes(hay: &[u8], needle: &[u8]) -> bool {
    if hay.len() < needle.len() { return false; }
    &hay[..needle.len()] == needle
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

            // Skip <script>, extract <style> content
            let tag_str = doc.get(elem_idx).tag_str();
            if tag_str == "script" {
                let close = b"</script>" as &[u8];
                while i + close.len() <= html.len() {
                    if starts_with_ci(&html[i..], close) {
                        i += close.len();
                        break;
                    }
                    i += 1;
                }
                continue;
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
            if raw_text.iter().any(|&b| b != b' ' && b != b'\t' && b != b'\n' && b != b'\r') {
                // Non-whitespace text — create text node
                // First: decode HTML entities
                let mut decoded = [0u8; 512];
                let mut dlen = 0;
                let mut j = 0;
                while j < raw_text.len() && dlen < 510 {
                    if raw_text[j] == b'&' {
                        // Try to decode entity
                        let rest = &raw_text[j..];
                        if starts_with_bytes(rest, b"&nbsp;") { decoded[dlen] = b' '; dlen += 1; j += 6; }
                        else if starts_with_bytes(rest, b"&amp;") { decoded[dlen] = b'&'; dlen += 1; j += 5; }
                        else if starts_with_bytes(rest, b"&lt;") { decoded[dlen] = b'<'; dlen += 1; j += 4; }
                        else if starts_with_bytes(rest, b"&gt;") { decoded[dlen] = b'>'; dlen += 1; j += 4; }
                        else if starts_with_bytes(rest, b"&quot;") { decoded[dlen] = b'"'; dlen += 1; j += 6; }
                        else if starts_with_bytes(rest, b"&#39;") || starts_with_bytes(rest, b"&apos;") {
                            decoded[dlen] = b'\''; dlen += 1;
                            j += if rest[1] == b'#' { 5 } else { 6 };
                        }
                        else if starts_with_bytes(rest, b"&copy;") { decoded[dlen] = b'c'; dlen += 1; j += 6; }
                        else if starts_with_bytes(rest, b"&mdash;") || starts_with_bytes(rest, b"&#8212;") {
                            decoded[dlen] = b'-'; dlen += 1;
                            j += if rest[1] == b'#' { 7 } else { 7 };
                        }
                        else {
                            // Unknown entity — skip to ; or just output &
                            decoded[dlen] = b'&'; dlen += 1; j += 1;
                        }
                    } else {
                        decoded[dlen] = raw_text[j]; dlen += 1; j += 1;
                    }
                }

                // Collapse whitespace
                let mut collapsed = [0u8; 256];
                let mut clen = 0;
                let mut last_space = true;
                for idx in 0..dlen {
                    let b = decoded[idx];
                    if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                        if !last_space && clen < 255 {
                            collapsed[clen] = b' ';
                            clen += 1;
                            last_space = true;
                        }
                    } else if clen < 255 {
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

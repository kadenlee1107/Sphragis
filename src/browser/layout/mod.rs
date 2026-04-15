// Bat_OS — Layout Engine
// Converts a styled DOM tree into positioned boxes for painting.
//
// Box model: content + padding + border + margin
// Block layout: elements stack vertically
// Inline layout: elements flow left-to-right, wrap at edges

pub mod flex;

use super::dom::{Document, NodeType, MAX_NODES};
use super::css::style::*;

/// Maximum layout boxes
pub const MAX_BOXES: usize = MAX_NODES;

/// A positioned box ready for painting
#[derive(Clone, Copy)]
pub struct LayoutBox {
    pub active: bool,
    pub dom_node: u16,    // index into DOM tree
    pub style: ComputedStyle,
    // Position (relative to page origin)
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    // Content area (inside padding)
    pub content_x: i32,
    pub content_y: i32,
    pub content_w: i32,
    pub content_h: i32,
    // For text nodes: the text to render
    pub text_start: usize, // index into text buffer
    pub text_len: usize,
    // Tree
    pub parent: u16,
    pub first_child: u16,
    pub next_sibling: u16,
}

const NULL: u16 = 0xFFFF;

impl LayoutBox {
    pub const fn empty() -> Self {
        LayoutBox {
            active: false,
            dom_node: NULL,
            style: ComputedStyle::default(),
            x: 0, y: 0, width: 0, height: 0,
            content_x: 0, content_y: 0, content_w: 0, content_h: 0,
            text_start: 0, text_len: 0,
            parent: NULL, first_child: NULL, next_sibling: NULL,
        }
    }
}

/// The layout tree + text buffer
pub struct LayoutTree {
    pub boxes: [LayoutBox; MAX_BOXES],
    pub box_count: usize,
    // Shared text buffer for all text nodes
    pub text_buf: [u8; 16384],
    pub text_len: usize,
    // Total page height (for scrolling)
    pub page_height: i32,
}

impl LayoutTree {
    pub const fn new() -> Self {
        LayoutTree {
            boxes: [LayoutBox::empty(); MAX_BOXES],
            box_count: 0,
            text_buf: [0; 16384],
            text_len: 0,
            page_height: 0,
        }
    }

    fn alloc(&mut self) -> Option<usize> {
        if self.box_count >= MAX_BOXES { return None; }
        let idx = self.box_count;
        self.boxes[idx] = LayoutBox::empty();
        self.boxes[idx].active = true;
        self.box_count += 1;
        Some(idx)
    }

    fn store_text(&mut self, text: &[u8]) -> (usize, usize) {
        let start = self.text_len;
        let len = text.len().min(self.text_buf.len() - self.text_len);
        self.text_buf[start..start + len].copy_from_slice(&text[..len]);
        self.text_len += len;
        (start, len)
    }
}

/// Build a layout tree from a DOM tree.
/// `viewport_w` = available width for layout.
/// Apply common CSS class name hints (for pages without <style> blocks)
fn apply_class_hints(class: &str, style: &mut ComputedStyle) {
    // Common Bootstrap/Tailwind/framework class patterns
    if class.contains("hidden") || class.contains("d-none") || class.contains("invisible") {
        style.display = Display::None;
    }
    if class.contains("bold") || class.contains("fw-bold") || class.contains("font-bold") {
        style.font_weight = FontWeight::Bold;
    }
    if class.contains("text-center") || class.contains("center") {
        style.text_align = TextAlign::Center;
    }
    if class.contains("container") || class.contains("wrapper") {
        style.margin_left = 16;
        style.margin_right = 16;
    }
    if class.contains("btn") || class.contains("button") {
        style.padding_left = 8;
        style.padding_right = 8;
        style.padding_top = 4;
        style.padding_bottom = 4;
        style.background_color = Color::from_rgb(40, 40, 40);
        style.border_width = 1;
        style.border_color = Color::from_rgb(80, 80, 80);
    }
    if class.contains("header") || class.contains("navbar") || class.contains("nav") {
        style.background_color = Color::from_rgb(20, 20, 20);
        style.padding_top = 8;
        style.padding_bottom = 8;
    }
    if class.contains("footer") {
        style.color = Color::from_rgb(120, 120, 120);
        style.margin_top = 16;
    }
}

use super::css::sheet::Stylesheet;

pub fn build(doc: &Document, tree: &mut LayoutTree, viewport_w: i32) {
    tree.box_count = 0;
    tree.text_len = 0;
    tree.page_height = 0;

    // Parse any <style> block CSS into a stylesheet
    let mut sheet = Stylesheet::new();
    if doc.css_len > 0 {
        sheet.parse(&doc.css_text[..doc.css_len]);
    }

    let body = doc.body();

    // Create root layout box
    let root = match tree.alloc() {
        Some(idx) => idx,
        None => return,
    };
    tree.boxes[root].dom_node = body as u16;
    let mut body_style = ComputedStyle::for_tag("body");
    if sheet.has_rules() {
        let body_node = doc.get(body);
        let id = body_node.get_attr("id").unwrap_or("");
        let cls = body_node.get_attr("class").unwrap_or("");
        sheet.apply("body", id, cls, &[], &mut body_style);
    }
    tree.boxes[root].style = body_style;
    tree.boxes[root].x = 0;
    tree.boxes[root].y = 0;
    tree.boxes[root].width = viewport_w;
    tree.boxes[root].content_w = viewport_w - 16; // default padding
    tree.boxes[root].content_x = 8;
    tree.boxes[root].content_y = 8;

    // Recursively lay out children
    let mut cursor_y = 8i32;
    layout_children(doc, tree, &sheet, root, body, 8, &mut cursor_y, viewport_w - 16);

    tree.boxes[root].height = cursor_y + 8;
    tree.boxes[root].content_h = cursor_y;
    tree.page_height = cursor_y + 16;
}

/// Check if a DOM node should be hidden (hidden attr, aria-hidden, etc.)
fn should_hide(node: &super::dom::DomNode) -> bool {
    for i in 0..node.attr_count {
        let aname = node.attrs[i].name_str();
        let aval  = node.attrs[i].value_str();
        if aname == "hidden" { return true; }
        if aname == "aria-hidden" && aval == "true" { return true; }
        if aname == "style" {
            // Check for display:none in inline style
            let s = aval.as_bytes();
            let mut j = 0;
            while j + 12 < s.len() {
                if s[j] == b'd' && s[j+1] == b'i' && s[j+2] == b's' && s[j+3] == b'p'
                    && s[j+4] == b'l' && s[j+5] == b'a' && s[j+6] == b'y' {
                    let mut k = j + 7;
                    while k < s.len() && (s[k] == b' ' || s[k] == b':') { k += 1; }
                    if k + 4 <= s.len() && s[k] == b'n' && s[k+1] == b'o' && s[k+2] == b'n' && s[k+3] == b'e' {
                        return true;
                    }
                }
                j += 1;
            }
        }
        if aname == "class" {
            // Wikipedia-specific hidden classes
            let cls = aval;
            if str_has(cls, "mw-jump-link") || str_has(cls, "noprint")
                || str_has(cls, "mw-editsection") || str_has(cls, "mw-indicators")
                || str_has(cls, "mw-hidden-catlinks") || str_has(cls, "mw-empty-elt")
                || str_has(cls, "sistersitebox")
            {
                return true;
            }
        }
    }
    false
}

/// Simple substring check
fn str_has(haystack: &str, needle: &str) -> bool {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.len() > h.len() { return false; }
    let end = h.len() - n.len() + 1;
    for i in 0..end {
        if &h[i..i + n.len()] == n { return true; }
    }
    false
}

/// Lay out children of a DOM node into a layout box
fn layout_children(
    doc: &Document,
    tree: &mut LayoutTree,
    sheet: &Stylesheet,
    parent_box: usize,
    dom_parent: usize,
    x_offset: i32,
    cursor_y: &mut i32,
    avail_width: i32,
) {
    let mut inline_x = x_offset;
    let char_w = 8i32;
    let line_h = 18i32;

    for child_idx in doc.children(dom_parent) {
        let node = doc.get(child_idx);

        match node.node_type {
            NodeType::Text => {
                // Inline text with proper word wrapping
                let text = &node.text[..node.text_len];
                if text.is_empty() { continue; }

                // Skip whitespace-only text nodes
                let mut all_ws = true;
                for &b in text.iter() {
                    if b != b' ' && b != b'\t' && b != b'\n' && b != b'\r' {
                        all_ws = false;
                        break;
                    }
                }
                if all_ws { continue; }

                let parent_style = tree.boxes[parent_box].style;

                // Create text layout box
                let tbox = match tree.alloc() {
                    Some(idx) => idx,
                    None => return,
                };

                let (ts, tl) = tree.store_text(text);
                tree.boxes[tbox].dom_node = child_idx as u16;
                tree.boxes[tbox].style = parent_style;
                tree.boxes[tbox].text_start = ts;
                tree.boxes[tbox].text_len = tl;
                tree.boxes[tbox].parent = parent_box as u16;

                // Word-wrap layout: break text at word boundaries to fit avail_width
                let max_chars_per_line = if avail_width > 0 { (avail_width / char_w).max(1) } else { 80 };
                let chars_in_text = tl as i32;

                // How many characters fit on the current line from inline_x?
                let remaining_on_line = ((x_offset + avail_width - inline_x) / char_w).max(0);

                if chars_in_text <= remaining_on_line {
                    // Fits on current line
                    let text_w = chars_in_text * char_w;
                    tree.boxes[tbox].x = inline_x;
                    tree.boxes[tbox].y = *cursor_y;
                    tree.boxes[tbox].width = text_w;
                    tree.boxes[tbox].height = line_h;
                    tree.boxes[tbox].content_x = inline_x;
                    tree.boxes[tbox].content_y = *cursor_y;
                    tree.boxes[tbox].content_w = text_w;
                    tree.boxes[tbox].content_h = line_h;
                    inline_x += text_w;
                } else {
                    // Text needs to wrap across multiple lines.
                    // Position the box at the start, compute its total height.
                    let start_y = *cursor_y;

                    // First line: fill remaining space
                    let mut lines = 1i32;
                    let chars_placed = remaining_on_line;

                    // Subsequent full lines
                    let remaining_chars = chars_in_text - chars_placed;
                    if remaining_chars > 0 {
                        lines += (remaining_chars + max_chars_per_line - 1) / max_chars_per_line;
                    }

                    let total_h = lines * line_h;

                    tree.boxes[tbox].x = inline_x;
                    tree.boxes[tbox].y = start_y;
                    tree.boxes[tbox].width = avail_width;
                    tree.boxes[tbox].height = total_h;
                    tree.boxes[tbox].content_x = inline_x;
                    tree.boxes[tbox].content_y = start_y;
                    tree.boxes[tbox].content_w = avail_width;
                    tree.boxes[tbox].content_h = total_h;

                    // After wrapping, cursor moves down
                    *cursor_y += total_h - line_h; // (lines-1) extra lines
                    // inline_x for the last line
                    let last_line_chars = if remaining_chars > 0 {
                        let rem = remaining_chars % max_chars_per_line;
                        if rem == 0 { max_chars_per_line } else { rem }
                    } else {
                        chars_placed
                    };
                    inline_x = x_offset + last_line_chars * char_w;
                }
            }
            NodeType::Element => {
                let mut style = ComputedStyle::for_tag(node.tag_str());

                // Apply CSS stylesheet rules (from <style> blocks)
                if sheet.has_rules() {
                    let tag = node.tag_str();
                    let id = node.get_attr("id").unwrap_or("");
                    let cls = node.get_attr("class").unwrap_or("");

                    // Build ancestor chain for descendant selectors
                    let mut ancestors: [(&str, &str, &str); 8] =
                        [("", "", ""); 8];
                    let mut anc_count = 0;
                    let mut pidx = dom_parent;
                    while anc_count < 8 {
                        let pnode = doc.get(pidx);
                        if pnode.node_type != NodeType::Element { break; }
                        ancestors[anc_count] = (
                            pnode.tag_str(),
                            pnode.get_attr("id").unwrap_or(""),
                            pnode.get_attr("class").unwrap_or(""),
                        );
                        anc_count += 1;
                        if pnode.parent == 0xFFFF { break; }
                        pidx = pnode.parent as usize;
                    }

                    sheet.apply(tag, id, cls, &ancestors[..anc_count], &mut style);
                }

                // Apply inline style="" attribute from HTML (highest priority)
                if let Some(style_attr) = node.get_attr("style") {
                    super::css::parser::apply_inline_style(style_attr, &mut style);
                }

                // Apply class-based color hints from common CSS patterns
                if let Some(class) = node.get_attr("class") {
                    apply_class_hints(class, &mut style);
                }

                if style.display == Display::None { continue; }

                // Skip hidden elements
                if should_hide(node) { continue; }

                // <input> -- render as a text box
                if node.tag_str() == "input" {
                    let input_type = node.get_attr("type").unwrap_or("text");
                    if input_type == "hidden" { continue; }

                    let ibox = match tree.alloc() {
                        Some(idx) => idx,
                        None => return,
                    };

                    let placeholder = node.get_attr("placeholder").unwrap_or(
                        node.get_attr("value").unwrap_or("")
                    );
                    let (ts, tl) = tree.store_text(placeholder.as_bytes());

                    let input_w = if input_type == "submit" || input_type == "button" {
                        ((tl as i32) + 4) * 8
                    } else {
                        200.min(avail_width)
                    };

                    // Force block-like stacking for inputs
                    if inline_x > x_offset {
                        *cursor_y += line_h;
                        inline_x = x_offset;
                    }

                    tree.boxes[ibox].dom_node = child_idx as u16;
                    tree.boxes[ibox].style = style;
                    tree.boxes[ibox].style.background_color = Color::from_rgb(25, 25, 25);
                    tree.boxes[ibox].style.border_width = 1;
                    tree.boxes[ibox].style.border_color = Color::from_rgb(80, 80, 80);
                    tree.boxes[ibox].style.padding_left = 4;
                    tree.boxes[ibox].style.padding_top = 3;
                    tree.boxes[ibox].style.color = Color::from_rgb(150, 150, 150);
                    tree.boxes[ibox].parent = parent_box as u16;
                    tree.boxes[ibox].x = inline_x;
                    tree.boxes[ibox].y = *cursor_y;
                    tree.boxes[ibox].width = input_w;
                    tree.boxes[ibox].height = 22;
                    tree.boxes[ibox].content_x = inline_x + 4;
                    tree.boxes[ibox].content_y = *cursor_y + 3;
                    tree.boxes[ibox].content_w = input_w - 8;
                    tree.boxes[ibox].content_h = 16;
                    tree.boxes[ibox].text_start = ts;
                    tree.boxes[ibox].text_len = tl;

                    if input_type == "submit" || input_type == "button" {
                        tree.boxes[ibox].style.background_color = Color::from_rgb(50, 50, 50);
                        tree.boxes[ibox].style.color = Color::WHITE;
                    }

                    inline_x += input_w + 4;
                    if inline_x > x_offset + avail_width {
                        inline_x = x_offset;
                        *cursor_y += 26;
                    }
                    continue;
                }

                // <button> -- render as a styled button
                if node.tag_str() == "button" {
                    style.background_color = Color::from_rgb(50, 50, 50);
                    style.border_width = 1;
                    style.border_color = Color::from_rgb(100, 100, 100);
                    style.padding_left = 12;
                    style.padding_right = 12;
                    style.padding_top = 4;
                    style.padding_bottom = 4;
                    style.color = Color::WHITE;
                }

                // <textarea> -- render as a larger input box
                if node.tag_str() == "textarea" {
                    style.background_color = Color::from_rgb(25, 25, 25);
                    style.border_width = 1;
                    style.border_color = Color::from_rgb(80, 80, 80);
                    style.padding_left = 4;
                    style.padding_top = 4;
                    style.display = Display::Block;
                    if style.height == Length::Auto {
                        style.height = Length::Px(80);
                    }
                }

                // <img> tags -- allocate space for image
                if node.tag_str() == "img" {
                    let img_w = node.get_attr("width")
                        .and_then(|v| v.parse::<i32>().ok())
                        .unwrap_or(200);
                    let img_h = node.get_attr("height")
                        .and_then(|v| v.parse::<i32>().ok())
                        .unwrap_or(150);

                    if inline_x > x_offset {
                        *cursor_y += line_h;
                        inline_x = x_offset;
                    }

                    let ibox = match tree.alloc() {
                        Some(idx) => idx,
                        None => return,
                    };
                    tree.boxes[ibox].dom_node = child_idx as u16;
                    tree.boxes[ibox].style = style;
                    tree.boxes[ibox].parent = parent_box as u16;
                    tree.boxes[ibox].x = x_offset;
                    tree.boxes[ibox].y = *cursor_y;
                    tree.boxes[ibox].width = img_w.min(avail_width);
                    tree.boxes[ibox].height = img_h;
                    tree.boxes[ibox].content_x = x_offset;
                    tree.boxes[ibox].content_y = *cursor_y;
                    tree.boxes[ibox].content_w = img_w.min(avail_width);
                    tree.boxes[ibox].content_h = img_h;

                    // Store alt text as fallback
                    if let Some(alt) = node.get_attr("alt") {
                        let (ts, tl) = tree.store_text(alt.as_bytes());
                        tree.boxes[ibox].text_start = ts;
                        tree.boxes[ibox].text_len = tl;
                    } else {
                        let (ts, tl) = tree.store_text(b"[image]");
                        tree.boxes[ibox].text_start = ts;
                        tree.boxes[ibox].text_len = tl;
                    }

                    // Draw image border
                    tree.boxes[ibox].style.border_width = 1;
                    tree.boxes[ibox].style.border_color = Color::from_rgb(60, 60, 60);

                    *cursor_y += img_h + 4;
                    inline_x = x_offset;
                    continue;
                }

                let ebox = match tree.alloc() {
                    Some(idx) => idx,
                    None => return,
                };

                tree.boxes[ebox].dom_node = child_idx as u16;
                tree.boxes[ebox].style = style;
                tree.boxes[ebox].parent = parent_box as u16;

                if style.display == Display::Block || style.display == Display::ListItem
                    || style.display == Display::Flex {
                    // Block element: new line, full width
                    if inline_x > x_offset {
                        // End current inline run
                        *cursor_y += line_h;
                        inline_x = x_offset;
                    }

                    *cursor_y += style.margin_top;

                    let block_x = x_offset + style.margin_left + style.padding_left;
                    let block_w = (avail_width - style.margin_left - style.margin_right
                        - style.padding_left - style.padding_right).max(0);

                    tree.boxes[ebox].x = x_offset + style.margin_left;
                    tree.boxes[ebox].y = *cursor_y;
                    tree.boxes[ebox].content_x = block_x;
                    tree.boxes[ebox].content_y = *cursor_y + style.padding_top;

                    // List item bullet
                    if style.display == Display::ListItem {
                        // Store a bullet marker
                        let bullet = match tree.alloc() {
                            Some(idx) => idx,
                            None => return,
                        };
                        let (bs, bl) = tree.store_text(b"\xB7 ");
                        tree.boxes[bullet].text_start = bs;
                        tree.boxes[bullet].text_len = bl;
                        tree.boxes[bullet].style = style;
                        tree.boxes[bullet].style.color = Color::from_rgb(255, 136, 0);
                        tree.boxes[bullet].x = block_x - 16;
                        tree.boxes[bullet].y = *cursor_y + style.padding_top;
                        tree.boxes[bullet].width = 16;
                        tree.boxes[bullet].height = line_h;
                        tree.boxes[bullet].parent = ebox as u16;
                        tree.boxes[bullet].active = true;
                    }

                    let child_y_start = *cursor_y + style.padding_top;
                    let mut child_y = child_y_start;

                    // Recurse into children
                    layout_children(doc, tree, sheet, ebox, child_idx, block_x, &mut child_y, block_w);

                    let content_h = (child_y - child_y_start).max(0);

                    // For fixed-height elements (e.g. textarea)
                    let final_h = match style.height {
                        Length::Px(h) => h.max(content_h),
                        _ => content_h,
                    };

                    tree.boxes[ebox].width = avail_width - style.margin_left - style.margin_right;
                    tree.boxes[ebox].height = final_h + style.padding_top + style.padding_bottom;
                    tree.boxes[ebox].content_w = block_w;
                    tree.boxes[ebox].content_h = final_h;

                    *cursor_y = tree.boxes[ebox].y + tree.boxes[ebox].height + style.margin_bottom;
                    inline_x = x_offset;
                } else {
                    // Inline element -- flow with text
                    let start_x = inline_x;
                    let start_y = *cursor_y;

                    let _saved_inline_x = inline_x;
                    layout_children(doc, tree, sheet, ebox, child_idx, inline_x, cursor_y,
                        avail_width - (inline_x - x_offset));

                    // After inline children, inline_x may have advanced via text nodes.
                    // Approximate: width = amount of text laid out inline.
                    let inline_h = if *cursor_y > start_y {
                        (*cursor_y - start_y) + line_h
                    } else {
                        line_h
                    };

                    tree.boxes[ebox].x = start_x;
                    tree.boxes[ebox].y = start_y;
                    tree.boxes[ebox].width = (inline_x - start_x).max(0);
                    tree.boxes[ebox].height = inline_h;
                    tree.boxes[ebox].content_x = start_x;
                    tree.boxes[ebox].content_y = start_y;
                    tree.boxes[ebox].content_w = (inline_x - start_x).max(0);
                    tree.boxes[ebox].content_h = inline_h;
                }
            }
            _ => {}
        }
    }
}

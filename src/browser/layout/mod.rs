// Bat_OS — Layout Engine
// Converts a styled DOM tree into positioned boxes for painting.
//
// Box model: content + padding + border + margin
// Block layout: elements stack vertically
// Inline layout: elements flow left-to-right, wrap at edges

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
pub fn build(doc: &Document, tree: &mut LayoutTree, viewport_w: i32) {
    tree.box_count = 0;
    tree.text_len = 0;
    tree.page_height = 0;

    let body = doc.body();

    // Create root layout box
    let root = match tree.alloc() {
        Some(idx) => idx,
        None => return,
    };
    tree.boxes[root].dom_node = body as u16;
    tree.boxes[root].style = ComputedStyle::for_tag("body");
    tree.boxes[root].x = 0;
    tree.boxes[root].y = 0;
    tree.boxes[root].width = viewport_w;
    tree.boxes[root].content_w = viewport_w - 16; // default padding
    tree.boxes[root].content_x = 8;
    tree.boxes[root].content_y = 8;

    // Recursively lay out children
    let mut cursor_y = 8i32;
    layout_children(doc, tree, root, body, 8, &mut cursor_y, viewport_w - 16);

    tree.boxes[root].height = cursor_y + 8;
    tree.boxes[root].content_h = cursor_y;
    tree.page_height = cursor_y + 16;
}

/// Lay out children of a DOM node into a layout box
fn layout_children(
    doc: &Document,
    tree: &mut LayoutTree,
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
                // Inline text — word wrap
                let text = &node.text[..node.text_len];
                if text.is_empty() { continue; }

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

                // Simple inline layout: characters flow left-to-right
                let text_w = (tl as i32) * char_w;
                if inline_x + text_w > x_offset + avail_width && inline_x > x_offset {
                    // Wrap to next line
                    inline_x = x_offset;
                    *cursor_y += line_h;
                }

                tree.boxes[tbox].x = inline_x;
                tree.boxes[tbox].y = *cursor_y;
                tree.boxes[tbox].width = text_w.min(avail_width);
                tree.boxes[tbox].height = line_h;
                tree.boxes[tbox].content_x = inline_x;
                tree.boxes[tbox].content_y = *cursor_y;
                tree.boxes[tbox].content_w = text_w;
                tree.boxes[tbox].content_h = line_h;

                inline_x += text_w;
                if inline_x > x_offset + avail_width {
                    inline_x = x_offset;
                    *cursor_y += line_h;
                }
            }
            NodeType::Element => {
                let style = ComputedStyle::for_tag(node.tag_str());

                // Apply inline style attribute
                if let Some(style_attr) = node.get_attr("style") {
                    let mut s = style;
                    super::css::parser::apply_inline_style(style_attr, &mut s);
                }

                if style.display == Display::None { continue; }

                let ebox = match tree.alloc() {
                    Some(idx) => idx,
                    None => return,
                };

                tree.boxes[ebox].dom_node = child_idx as u16;
                tree.boxes[ebox].style = style;
                tree.boxes[ebox].parent = parent_box as u16;

                if style.display == Display::Block || style.display == Display::ListItem {
                    // Block element: new line, full width
                    if inline_x > x_offset {
                        // End current inline run
                        *cursor_y += line_h;
                        inline_x = x_offset;
                    }

                    *cursor_y += style.margin_top;

                    let block_x = x_offset + style.margin_left + style.padding_left;
                    let block_w = avail_width - style.margin_left - style.margin_right
                        - style.padding_left - style.padding_right;

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
                    layout_children(doc, tree, ebox, child_idx, block_x, &mut child_y, block_w);

                    let content_h = child_y - child_y_start;
                    tree.boxes[ebox].width = avail_width - style.margin_left - style.margin_right;
                    tree.boxes[ebox].height = content_h + style.padding_top + style.padding_bottom;
                    tree.boxes[ebox].content_w = block_w;
                    tree.boxes[ebox].content_h = content_h;

                    *cursor_y = child_y + style.padding_bottom + style.margin_bottom;
                    inline_x = x_offset;
                } else {
                    // Inline element — flow with text
                    tree.boxes[ebox].x = inline_x;
                    tree.boxes[ebox].y = *cursor_y;

                    let child_y_start = *cursor_y;
                    let mut child_y = child_y_start;

                    layout_children(doc, tree, ebox, child_idx, inline_x, &mut child_y, avail_width - (inline_x - x_offset));

                    tree.boxes[ebox].width = inline_x - tree.boxes[ebox].x;
                    tree.boxes[ebox].height = line_h;
                    tree.boxes[ebox].content_w = tree.boxes[ebox].width;
                    tree.boxes[ebox].content_h = line_h;
                }
            }
            _ => {}
        }
    }
}

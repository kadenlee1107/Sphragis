// Bat_OS — DOM (Document Object Model)
// Tree structure representing parsed HTML.
// Fixed-size arena allocator — no heap needed.
//
// Every node lives in a flat array (NODES). Parent/child/sibling
// relationships are tracked via indices. This lets us build and
// traverse the tree without Vec or Box.

/// Maximum nodes in one document
pub const MAX_NODES: usize = 2048;
/// Maximum attributes per element
pub const MAX_ATTRS: usize = 8;
/// Maximum tag/attr name length
pub const MAX_NAME: usize = 32;
/// Maximum attribute value length
pub const MAX_VALUE: usize = 128;
/// Maximum text content length per text node
pub const MAX_TEXT: usize = 256;

#[derive(Clone, Copy, PartialEq)]
pub enum NodeType {
    Empty,      // unused slot
    Document,   // root node
    Element,    // <tag>...</tag>
    Text,       // raw text content
    Comment,    // <!-- ... -->
}

#[derive(Clone, Copy)]
pub struct Attribute {
    pub name: [u8; MAX_NAME],
    pub name_len: usize,
    pub value: [u8; MAX_VALUE],
    pub value_len: usize,
}

impl Attribute {
    pub const fn empty() -> Self {
        Attribute {
            name: [0; MAX_NAME],
            name_len: 0,
            value: [0; MAX_VALUE],
            value_len: 0,
        }
    }

    pub fn name_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.name[..self.name_len]) }
    }

    pub fn value_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.value[..self.value_len]) }
    }

    pub fn set_name(&mut self, s: &[u8]) {
        let len = s.len().min(MAX_NAME);
        self.name[..len].copy_from_slice(&s[..len]);
        self.name_len = len;
    }

    pub fn set_value(&mut self, s: &[u8]) {
        let len = s.len().min(MAX_VALUE);
        self.value[..len].copy_from_slice(&s[..len]);
        self.value_len = len;
    }
}

#[derive(Clone, Copy)]
pub struct DomNode {
    pub node_type: NodeType,
    // Element fields
    pub tag: [u8; MAX_NAME],
    pub tag_len: usize,
    pub attrs: [Attribute; MAX_ATTRS],
    pub attr_count: usize,
    // Text node content
    pub text: [u8; MAX_TEXT],
    pub text_len: usize,
    // Tree structure (indices into the node arena)
    pub parent: u16,        // index of parent node
    pub first_child: u16,   // index of first child
    pub last_child: u16,    // index of last child
    pub next_sibling: u16,  // index of next sibling
    pub prev_sibling: u16,  // index of previous sibling
}

const NULL_IDX: u16 = 0xFFFF;

impl DomNode {
    pub const fn empty() -> Self {
        DomNode {
            node_type: NodeType::Empty,
            tag: [0; MAX_NAME],
            tag_len: 0,
            attrs: [Attribute::empty(); MAX_ATTRS],
            attr_count: 0,
            text: [0; MAX_TEXT],
            text_len: 0,
            parent: NULL_IDX,
            first_child: NULL_IDX,
            last_child: NULL_IDX,
            next_sibling: NULL_IDX,
            prev_sibling: NULL_IDX,
        }
    }

    pub fn tag_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.tag[..self.tag_len]) }
    }

    pub fn text_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.text[..self.text_len]) }
    }

    pub fn set_tag(&mut self, s: &[u8]) {
        let len = s.len().min(MAX_NAME);
        self.tag[..len].copy_from_slice(&s[..len]);
        self.tag_len = len;
        // Store lowercase for easy matching
        for i in 0..len {
            if self.tag[i] >= b'A' && self.tag[i] <= b'Z' {
                self.tag[i] += 32;
            }
        }
    }

    pub fn set_text(&mut self, s: &[u8]) {
        let len = s.len().min(MAX_TEXT);
        self.text[..len].copy_from_slice(&s[..len]);
        self.text_len = len;
    }

    pub fn append_text(&mut self, s: &[u8]) {
        let avail = MAX_TEXT - self.text_len;
        let len = s.len().min(avail);
        self.text[self.text_len..self.text_len + len].copy_from_slice(&s[..len]);
        self.text_len += len;
    }

    pub fn get_attr(&self, name: &str) -> Option<&str> {
        for i in 0..self.attr_count {
            if self.attrs[i].name_str() == name {
                return Some(self.attrs[i].value_str());
            }
        }
        None
    }

    pub fn has_children(&self) -> bool {
        self.first_child != NULL_IDX
    }

    /// Check if this is a specific tag
    pub fn is_tag(&self, tag: &str) -> bool {
        self.node_type == NodeType::Element && self.tag_str() == tag
    }

    /// Check if this tag is a block-level element
    pub fn is_block(&self) -> bool {
        if self.node_type != NodeType::Element { return false; }
        match self.tag_str() {
            "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" |
            "ul" | "ol" | "li" | "blockquote" | "pre" | "hr" | "br" |
            "table" | "tr" | "form" | "section" | "article" | "aside" |
            "header" | "footer" | "nav" | "main" | "figure" | "figcaption" |
            "details" | "summary" | "dl" | "dt" | "dd" => true,
            _ => false,
        }
    }

    /// Check if this tag is an inline element
    pub fn is_inline(&self) -> bool {
        if self.node_type != NodeType::Element { return self.node_type == NodeType::Text; }
        !self.is_block()
    }

    /// Check if this is a void element (self-closing, no end tag)
    pub fn is_void(&self) -> bool {
        match self.tag_str() {
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" |
            "input" | "link" | "meta" | "param" | "source" | "track" | "wbr" => true,
            _ => false,
        }
    }
}

/// Maximum CSS text from <style> blocks
pub const MAX_CSS: usize = 4096;

/// The DOM tree — flat arena of nodes
pub struct Document {
    pub nodes: [DomNode; MAX_NODES],
    pub node_count: usize,
    /// Extracted CSS text from <style> blocks
    pub css_text: [u8; MAX_CSS],
    pub css_len: usize,
}

impl Document {
    pub const fn new() -> Self {
        Document {
            nodes: [DomNode::empty(); MAX_NODES],
            node_count: 0,
            css_text: [0; MAX_CSS],
            css_len: 0,
        }
    }

    /// Initialize with a document root node
    pub fn init(&mut self) {
        self.node_count = 0;
        self.css_len = 0;
        let root = self.alloc_node();
        if let Some(idx) = root {
            self.nodes[idx].node_type = NodeType::Document;
        }
    }

    /// Allocate a new node, return its index
    pub fn alloc_node(&mut self) -> Option<usize> {
        if self.node_count >= MAX_NODES { return None; }
        let idx = self.node_count;
        self.nodes[idx] = DomNode::empty();
        self.node_count += 1;
        Some(idx)
    }

    /// Create an element node
    pub fn create_element(&mut self, tag: &[u8]) -> Option<usize> {
        let idx = self.alloc_node()?;
        self.nodes[idx].node_type = NodeType::Element;
        self.nodes[idx].set_tag(tag);
        Some(idx)
    }

    /// Create a text node
    pub fn create_text(&mut self, text: &[u8]) -> Option<usize> {
        let idx = self.alloc_node()?;
        self.nodes[idx].node_type = NodeType::Text;
        self.nodes[idx].set_text(text);
        Some(idx)
    }

    /// Append child to parent
    pub fn append_child(&mut self, parent: usize, child: usize) {
        self.nodes[child].parent = parent as u16;

        let last = self.nodes[parent].last_child;
        if last == NULL_IDX {
            // First child
            self.nodes[parent].first_child = child as u16;
        } else {
            // Append after last child
            self.nodes[last as usize].next_sibling = child as u16;
            self.nodes[child].prev_sibling = last;
        }
        self.nodes[parent].last_child = child as u16;
    }

    /// Add element as child of parent, return index
    pub fn add_element(&mut self, parent: usize, tag: &str) -> Option<usize> {
        let idx = self.create_element(tag.as_bytes())?;
        self.append_child(parent, idx);
        Some(idx)
    }

    /// Add text node as child of parent
    pub fn add_text(&mut self, parent: usize, text: &[u8]) {
        if let Some(idx) = self.create_text(text) {
            self.append_child(parent, idx);
        }
    }

    /// Get node by index
    pub fn get(&self, idx: usize) -> &DomNode {
        &self.nodes[idx]
    }

    /// Get mutable node by index
    pub fn get_mut(&mut self, idx: usize) -> &mut DomNode {
        &mut self.nodes[idx]
    }

    /// Iterate children of a node
    pub fn children(&self, parent: usize) -> ChildIter<'_> {
        ChildIter {
            doc: self,
            current: self.nodes[parent].first_child,
        }
    }

    /// Find first element with given tag name (depth-first)
    pub fn find_tag(&self, tag: &str) -> Option<usize> {
        for i in 0..self.node_count {
            if self.nodes[i].is_tag(tag) {
                return Some(i);
            }
        }
        None
    }

    /// Find all elements with given tag name
    pub fn find_all_tags<F: FnMut(usize)>(&self, tag: &str, mut f: F) {
        for i in 0..self.node_count {
            if self.nodes[i].is_tag(tag) {
                f(i);
            }
        }
    }

    /// Get the <body> element (or document root if no body)
    pub fn body(&self) -> usize {
        self.find_tag("body").unwrap_or(0)
    }

    /// Count total nodes
    pub fn len(&self) -> usize {
        self.node_count
    }

    /// Find first element with a given attribute value (e.g. id="foo")
    pub fn find_by_attr(&self, attr_name: &str, attr_val: &str) -> Option<usize> {
        for i in 0..self.node_count {
            if self.nodes[i].node_type != NodeType::Element { continue; }
            for a in 0..self.nodes[i].attr_count {
                if self.nodes[i].attrs[a].name_str() == attr_name
                    && self.nodes[i].attrs[a].value_str() == attr_val
                {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Find first element by id attribute
    pub fn find_by_id(&self, id: &str) -> Option<usize> {
        self.find_by_attr("id", id)
    }

    /// Check if a node has a given class (substring match in class attribute)
    pub fn has_class(&self, node_idx: usize, class: &str) -> bool {
        if node_idx >= self.node_count { return false; }
        let node = &self.nodes[node_idx];
        for a in 0..node.attr_count {
            if node.attrs[a].name_str() == "class" {
                let cls = node.attrs[a].value_str();
                return str_contains_word(cls, class);
            }
        }
        false
    }

    /// Check if a node has a class containing a substring (partial match)
    pub fn class_contains(&self, node_idx: usize, substr: &str) -> bool {
        if node_idx >= self.node_count { return false; }
        let node = &self.nodes[node_idx];
        for a in 0..node.attr_count {
            if node.attrs[a].name_str() == "class" {
                let cls = node.attrs[a].value_str();
                return str_contains_sub(cls, substr);
            }
        }
        false
    }

    /// Check if a node has a given role attribute value
    pub fn has_role(&self, node_idx: usize, role: &str) -> bool {
        if node_idx >= self.node_count { return false; }
        let node = &self.nodes[node_idx];
        for a in 0..node.attr_count {
            if node.attrs[a].name_str() == "role" {
                return node.attrs[a].value_str() == role;
            }
        }
        false
    }

    /// Check if node_idx is a descendant of ancestor_idx
    pub fn is_descendant_of(&self, node_idx: usize, ancestor_idx: usize) -> bool {
        let mut current = node_idx;
        let mut depth = 0;
        while depth < 100 {
            let parent = self.nodes[current].parent;
            if parent == NULL_IDX { return false; }
            if parent as usize == ancestor_idx { return true; }
            current = parent as usize;
            depth += 1;
        }
        false
    }
}

/// Check if `haystack` contains `needle` as a whole word in a space-separated list
fn str_contains_word(haystack: &str, needle: &str) -> bool {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.len() > h.len() { return false; }
    if n.is_empty() { return false; }
    let end = h.len() - n.len() + 1;
    for i in 0..end {
        if &h[i..i + n.len()] == n {
            // Check word boundaries (start of string or preceded by space)
            let at_start = i == 0 || h[i - 1] == b' ';
            let at_end = i + n.len() == h.len() || h[i + n.len()] == b' ';
            if at_start && at_end { return true; }
        }
    }
    false
}

/// Check if `haystack` contains `needle` as a substring
fn str_contains_sub(haystack: &str, needle: &str) -> bool {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.len() > h.len() { return false; }
    if n.is_empty() { return false; }
    let end = h.len() - n.len() + 1;
    for i in 0..end {
        if &h[i..i + n.len()] == n { return true; }
    }
    false
}

/// Iterator over children of a node
pub struct ChildIter<'a> {
    doc: &'a Document,
    current: u16,
}

impl<'a> Iterator for ChildIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.current == NULL_IDX {
            return None;
        }
        let idx = self.current as usize;
        self.current = self.doc.nodes[idx].next_sibling;
        Some(idx)
    }
}

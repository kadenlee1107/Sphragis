// Bat_OS — CSS Stylesheet Engine
// Parses <style> block CSS and applies rules to DOM elements.
// Supports: tag selectors, .class selectors, #id selectors,
// descendant selectors (e.g. "body p"), and comma groups.

use super::style::*;
use super::parser::apply_property;

/// Maximum CSS rules per stylesheet
const MAX_RULES: usize = 128;
/// Maximum selectors per rule (comma-separated)
const MAX_SELECTORS: usize = 4;
/// Maximum parts in a compound selector (e.g. "body .main p" = 3 parts)
const MAX_PARTS: usize = 4;
/// Maximum declarations per rule
const MAX_DECLS: usize = 16;

/// A single selector part: tag, .class, or #id
#[derive(Clone, Copy)]
struct SelectorPart {
    tag: [u8; 16],
    tag_len: usize,
    class: [u8; 32],
    class_len: usize,
    id: [u8; 32],
    id_len: usize,
}

impl SelectorPart {
    const fn empty() -> Self {
        SelectorPart {
            tag: [0; 16], tag_len: 0,
            class: [0; 32], class_len: 0,
            id: [0; 32], id_len: 0,
        }
    }

    fn tag_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.tag[..self.tag_len]) }
    }
    fn class_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.class[..self.class_len]) }
    }
    fn id_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.id[..self.id_len]) }
    }
}

/// A compound selector (e.g. "body .highlight p")
#[derive(Clone, Copy)]
struct Selector {
    parts: [SelectorPart; MAX_PARTS],
    part_count: usize,
}

impl Selector {
    const fn empty() -> Self {
        Selector {
            parts: [SelectorPart::empty(); MAX_PARTS],
            part_count: 0,
        }
    }
}

/// A CSS property: value pair
#[derive(Clone, Copy)]
struct Declaration {
    prop: [u8; 32],
    prop_len: usize,
    value: [u8; 64],
    value_len: usize,
}

impl Declaration {
    const fn empty() -> Self {
        Declaration {
            prop: [0; 32], prop_len: 0,
            value: [0; 64], value_len: 0,
        }
    }
}

/// A CSS rule: selector(s) + declarations
#[derive(Clone, Copy)]
struct CssRule {
    selectors: [Selector; MAX_SELECTORS],
    sel_count: usize,
    decls: [Declaration; MAX_DECLS],
    decl_count: usize,
}

impl CssRule {
    const fn empty() -> Self {
        CssRule {
            selectors: [Selector::empty(); MAX_SELECTORS],
            sel_count: 0,
            decls: [Declaration::empty(); MAX_DECLS],
            decl_count: 0,
        }
    }
}

/// Parsed CSS stylesheet
pub struct Stylesheet {
    rules: [CssRule; MAX_RULES],
    rule_count: usize,
}

impl Stylesheet {
    pub const fn new() -> Self {
        Stylesheet {
            rules: [CssRule::empty(); MAX_RULES],
            rule_count: 0,
        }
    }

    /// Parse CSS text into rules
    pub fn parse(&mut self, css: &[u8]) {
        self.rule_count = 0;
        let mut i = 0;
        let len = css.len();

        while i < len && self.rule_count < MAX_RULES {
            // Skip whitespace and comments
            i = skip_ws_comments(css, i);
            if i >= len { break; }

            // Find the '{' that starts the declaration block
            let sel_start = i;
            while i < len && css[i] != b'{' { i += 1; }
            if i >= len { break; }

            let sel_end = i;
            i += 1; // skip '{'

            // Find the matching '}'
            let decl_start = i;
            while i < len && css[i] != b'}' { i += 1; }
            if i >= len { break; }
            let decl_end = i;
            i += 1; // skip '}'

            // Parse the rule
            let rule_idx = self.rule_count;
            self.rule_count += 1;
            let rule = &mut self.rules[rule_idx];

            // Parse selectors (comma-separated)
            parse_selectors(&css[sel_start..sel_end], rule);

            // Parse declarations (semicolon-separated)
            parse_declarations(&css[decl_start..decl_end], rule);
        }
    }

    /// Apply matching CSS rules to a ComputedStyle for a given element.
    /// `tag`, `id`, `classes` describe the element.
    /// `ancestors` is a list of (tag, class, id) tuples for ancestor elements.
    pub fn apply(
        &self,
        tag: &str,
        id: &str,
        classes: &str,
        ancestors: &[(&str, &str, &str)],
        style: &mut ComputedStyle,
    ) {
        for ri in 0..self.rule_count {
            let rule = &self.rules[ri];
            let mut matched = false;

            // Check if any selector matches
            for si in 0..rule.sel_count {
                if selector_matches(&rule.selectors[si], tag, id, classes, ancestors) {
                    matched = true;
                    break;
                }
            }

            if matched {
                // Apply all declarations
                for di in 0..rule.decl_count {
                    let d = &rule.decls[di];
                    let prop = unsafe { core::str::from_utf8_unchecked(&d.prop[..d.prop_len]) };
                    let val = unsafe { core::str::from_utf8_unchecked(&d.value[..d.value_len]) };
                    apply_property(prop, val, style);
                }
            }
        }
    }

    pub fn has_rules(&self) -> bool {
        self.rule_count > 0
    }
}

// === Parsing helpers ===

fn skip_ws_comments(css: &[u8], mut i: usize) -> usize {
    let len = css.len();
    loop {
        // Skip whitespace
        while i < len && (css[i] == b' ' || css[i] == b'\n' || css[i] == b'\r' || css[i] == b'\t') {
            i += 1;
        }
        // Skip /* ... */ comments
        if i + 1 < len && css[i] == b'/' && css[i + 1] == b'*' {
            i += 2;
            while i + 1 < len {
                if css[i] == b'*' && css[i + 1] == b'/' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            continue;
        }
        break;
    }
    i
}

fn parse_selectors(sel_text: &[u8], rule: &mut CssRule) {
    rule.sel_count = 0;
    // Split by commas
    let mut start = 0;
    let len = sel_text.len();

    loop {
        // Find next comma or end
        let mut end = start;
        while end < len && sel_text[end] != b',' { end += 1; }

        if rule.sel_count < MAX_SELECTORS {
            let sel = &mut rule.selectors[rule.sel_count];
            parse_one_selector(&sel_text[start..end], sel);
            if sel.part_count > 0 {
                rule.sel_count += 1;
            }
        }

        if end >= len { break; }
        start = end + 1; // skip comma
    }
}

fn parse_one_selector(text: &[u8], sel: &mut Selector) {
    sel.part_count = 0;
    let mut i = 0;
    let len = text.len();

    // Skip leading whitespace
    while i < len && (text[i] == b' ' || text[i] == b'\n' || text[i] == b'\t') { i += 1; }

    while i < len && sel.part_count < MAX_PARTS {
        let part = &mut sel.parts[sel.part_count];
        *part = SelectorPart::empty();
        let mut got_something = false;

        // Parse a simple selector: tag.class#id (or just .class or #id)
        while i < len && text[i] != b' ' && text[i] != b'\n' && text[i] != b'\t' {
            if text[i] == b'.' {
                // Class selector
                i += 1;
                let start = i;
                while i < len && text[i] != b' ' && text[i] != b'.' && text[i] != b'#'
                    && text[i] != b'\n' && text[i] != b'\t'
                {
                    i += 1;
                }
                let clen = (i - start).min(31);
                part.class[..clen].copy_from_slice(&text[start..start + clen]);
                part.class_len = clen;
                got_something = true;
            } else if text[i] == b'#' {
                // ID selector
                i += 1;
                let start = i;
                while i < len && text[i] != b' ' && text[i] != b'.' && text[i] != b'#'
                    && text[i] != b'\n' && text[i] != b'\t'
                {
                    i += 1;
                }
                let ilen = (i - start).min(31);
                part.id[..ilen].copy_from_slice(&text[start..start + ilen]);
                part.id_len = ilen;
                got_something = true;
            } else {
                // Tag name
                let start = i;
                while i < len && text[i] != b' ' && text[i] != b'.' && text[i] != b'#'
                    && text[i] != b'\n' && text[i] != b'\t'
                {
                    i += 1;
                }
                let tlen = (i - start).min(15);
                // Lowercase the tag
                for j in 0..tlen {
                    part.tag[j] = if text[start + j] >= b'A' && text[start + j] <= b'Z' {
                        text[start + j] + 32
                    } else {
                        text[start + j]
                    };
                }
                part.tag_len = tlen;
                got_something = true;
            }
        }

        if got_something {
            sel.part_count += 1;
        }

        // Skip whitespace between parts
        while i < len && (text[i] == b' ' || text[i] == b'\n' || text[i] == b'\t') { i += 1; }
    }
}

fn parse_declarations(text: &[u8], rule: &mut CssRule) {
    rule.decl_count = 0;
    let mut i = 0;
    let len = text.len();

    while i < len && rule.decl_count < MAX_DECLS {
        // Skip whitespace
        while i < len && (text[i] == b' ' || text[i] == b'\n' || text[i] == b'\r' || text[i] == b'\t') {
            i += 1;
        }
        if i >= len { break; }

        // Find colon
        let prop_start = i;
        while i < len && text[i] != b':' && text[i] != b';' { i += 1; }
        if i >= len || text[i] == b';' { i += 1; continue; }

        let prop_end = i;
        i += 1; // skip ':'

        // Skip whitespace after colon
        while i < len && (text[i] == b' ' || text[i] == b'\t') { i += 1; }

        // Find semicolon or end
        let val_start = i;
        while i < len && text[i] != b';' { i += 1; }
        let val_end = i;
        if i < len { i += 1; } // skip ';'

        // Trim whitespace
        let prop = trim(&text[prop_start..prop_end]);
        let val = trim(&text[val_start..val_end]);

        if !prop.is_empty() && !val.is_empty() {
            let d = &mut rule.decls[rule.decl_count];
            let plen = prop.len().min(31);
            d.prop[..plen].copy_from_slice(&prop[..plen]);
            d.prop_len = plen;
            let vlen = val.len().min(63);
            d.value[..vlen].copy_from_slice(&val[..vlen]);
            d.value_len = vlen;
            rule.decl_count += 1;
        }
    }
}

fn trim(s: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = s.len();
    while start < end && (s[start] == b' ' || s[start] == b'\n' || s[start] == b'\r' || s[start] == b'\t') {
        start += 1;
    }
    while end > start && (s[end - 1] == b' ' || s[end - 1] == b'\n' || s[end - 1] == b'\r' || s[end - 1] == b'\t') {
        end -= 1;
    }
    &s[start..end]
}

// === Selector matching ===

/// Check if a selector matches a given element + ancestor chain.
/// The last part of the selector must match the target element.
/// Earlier parts must match ancestor elements (descendant combinator).
fn selector_matches(
    sel: &Selector,
    tag: &str,
    id: &str,
    classes: &str,
    ancestors: &[(&str, &str, &str)],
) -> bool {
    if sel.part_count == 0 { return false; }

    // The last part must match the target element
    let last = &sel.parts[sel.part_count - 1];
    if !part_matches(last, tag, id, classes) {
        return false;
    }

    // If only one part, we're done
    if sel.part_count == 1 { return true; }

    // For descendant selectors: each earlier part must match some ancestor
    // Walk backwards through selector parts and ancestors
    let mut sel_idx = sel.part_count as isize - 2; // start from second-to-last
    let mut anc_idx = 0usize;

    while sel_idx >= 0 && anc_idx < ancestors.len() {
        let part = &sel.parts[sel_idx as usize];
        let (atag, acid, acls) = ancestors[anc_idx];

        if part_matches(part, atag, acid, acls) {
            sel_idx -= 1; // matched this selector part, move to next
        }
        anc_idx += 1; // try next ancestor
    }

    sel_idx < 0 // all parts matched
}

/// Check if a selector part matches an element
fn part_matches(part: &SelectorPart, tag: &str, id: &str, classes: &str) -> bool {
    // Check tag name (if specified)
    if part.tag_len > 0 && part.tag_str() != tag {
        return false;
    }
    // Check ID (if specified)
    if part.id_len > 0 && part.id_str() != id {
        return false;
    }
    // Check class (if specified)
    if part.class_len > 0 {
        let want = part.class_str();
        if !class_list_contains(classes, want) {
            return false;
        }
    }
    true
}

/// Check if a space-separated class list contains a given class
fn class_list_contains(classes: &str, want: &str) -> bool {
    for cls in classes.split(' ') {
        if cls == want { return true; }
    }
    false
}

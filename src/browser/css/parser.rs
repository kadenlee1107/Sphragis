// Bat_OS — CSS Parser
// Parses inline style="" attributes into ComputedStyle modifications.
// Also handles <style> blocks (basic selector matching).

use super::style::*;

/// Apply inline style attribute to a computed style
pub fn apply_inline_style(style_attr: &str, computed: &mut ComputedStyle) {
    // Parse "property: value; property: value;"
    for decl in style_attr.split(';') {
        let decl = decl.trim();
        if decl.is_empty() { continue; }

        if let Some(colon) = decl.find(':') {
            let prop = decl[..colon].trim();
            let val = decl[colon + 1..].trim();
            apply_property(prop, val, computed);
        }
    }
}

/// Apply a single CSS property: value to a computed style
pub fn apply_property(prop: &str, val: &str, style: &mut ComputedStyle) {
    match prop {
        "color" => { style.color = Color::parse(val); }
        "background-color" | "background" => { style.background_color = Color::parse(val); }
        "font-size" => { style.font_size = Length::parse(val).to_px(16, 16); }
        "font-weight" => {
            style.font_weight = if val == "bold" || val == "700" || val == "800" || val == "900" {
                FontWeight::Bold
            } else {
                FontWeight::Normal
            };
        }
        "font-style" => {
            style.font_style = if val == "italic" || val == "oblique" {
                FontStyle::Italic
            } else {
                FontStyle::Normal
            };
        }
        "text-align" => {
            style.text_align = match val {
                "center" => TextAlign::Center,
                "right" => TextAlign::Right,
                _ => TextAlign::Left,
            };
        }
        "text-decoration" => {
            style.text_decoration.underline = val.contains("underline");
            style.text_decoration.line_through = val.contains("line-through");
        }
        "display" => {
            style.display = match val {
                "block" => Display::Block,
                "inline" => Display::Inline,
                "inline-block" => Display::InlineBlock,
                "none" => Display::None,
                "flex" => Display::Flex,
                "list-item" => Display::ListItem,
                _ => style.display,
            };
        }
        "margin" => {
            let px = Length::parse(val).to_px(0, 16);
            style.margin_top = px; style.margin_bottom = px;
            style.margin_left = px; style.margin_right = px;
        }
        "margin-top" => { style.margin_top = Length::parse(val).to_px(0, 16); }
        "margin-bottom" => { style.margin_bottom = Length::parse(val).to_px(0, 16); }
        "margin-left" => { style.margin_left = Length::parse(val).to_px(0, 16); }
        "margin-right" => { style.margin_right = Length::parse(val).to_px(0, 16); }
        "padding" => {
            let px = Length::parse(val).to_px(0, 16);
            style.padding_top = px; style.padding_bottom = px;
            style.padding_left = px; style.padding_right = px;
        }
        "padding-top" => { style.padding_top = Length::parse(val).to_px(0, 16); }
        "padding-bottom" => { style.padding_bottom = Length::parse(val).to_px(0, 16); }
        "padding-left" => { style.padding_left = Length::parse(val).to_px(0, 16); }
        "padding-right" => { style.padding_right = Length::parse(val).to_px(0, 16); }
        "border" => {
            // Simple: "1px solid #color"
            let parts: [&str; 3] = split3(val);
            if !parts[0].is_empty() {
                style.border_width = Length::parse(parts[0]).to_px(0, 16);
            }
            if !parts[2].is_empty() {
                style.border_color = Color::parse(parts[2]);
            }
        }
        "width" => { style.width = Length::parse(val); }
        "height" => { style.height = Length::parse(val); }
        "max-width" => { style.max_width = Length::parse(val); }
        "min-height" => { style.min_height = Length::parse(val); }
        "line-height" => {
            style.line_height = Length::parse(val).to_px(16, style.font_size);
        }
        "border-width" => {
            style.border_width = Length::parse(val).to_px(0, 16);
        }
        "border-color" => {
            style.border_color = Color::parse(val);
        }
        "border-radius" => {
            style.border_radius = Length::parse(val).to_px(0, 16);
        }
        "border-bottom" | "border-top" | "border-left" | "border-right" => {
            let parts: [&str; 3] = split3(val);
            if !parts[0].is_empty() {
                style.border_width = Length::parse(parts[0]).to_px(0, 16);
            }
            if !parts[2].is_empty() {
                style.border_color = Color::parse(parts[2]);
            }
        }
        "overflow" | "overflow-x" | "overflow-y" => {
            style.overflow = match val {
                "hidden" => Overflow::Hidden,
                "scroll" => Overflow::Scroll,
                "auto" => Overflow::Auto,
                _ => Overflow::Visible,
            };
        }
        "visibility" => {
            style.visibility = match val {
                "hidden" => Visibility::Hidden,
                "collapse" => Visibility::Collapse,
                _ => Visibility::Visible,
            };
        }
        "opacity" => {
            // Parse 0.0–1.0 or 0–100
            let bytes = val.as_bytes();
            if bytes.len() > 0 && bytes[0] == b'0' && bytes.len() > 1 && bytes[1] == b'.' {
                // Decimal: 0.5 → 128
                let frac = if bytes.len() > 2 { (bytes[2] - b'0') as u8 } else { 0 };
                style.opacity = (frac as u16 * 255 / 10) as u8;
            } else if val == "1" {
                style.opacity = 255;
            } else if val == "0" {
                style.opacity = 0;
            }
        }
        "text-transform" => {
            style.text_transform = match val {
                "uppercase" => TextTransform::Uppercase,
                "lowercase" => TextTransform::Lowercase,
                "capitalize" => TextTransform::Capitalize,
                _ => TextTransform::None,
            };
        }
        "white-space" => {
            style.white_space = match val {
                "nowrap" => WhiteSpace::NoWrap,
                "pre" => WhiteSpace::Pre,
                "pre-wrap" => WhiteSpace::PreWrap,
                _ => WhiteSpace::Normal,
            };
        }
        "vertical-align" => {
            style.vertical_align = match val {
                "top" => VerticalAlign::Top,
                "middle" => VerticalAlign::Middle,
                "bottom" => VerticalAlign::Bottom,
                _ => VerticalAlign::Baseline,
            };
        }
        "list-style" | "list-style-type" => {
            if val == "none" {
                // Remove list marker
            }
        }
        // Flex properties (store but layout handles separately)
        "flex-direction" => {
            style.flex_direction = match val.trim() {
                "column" => 1,
                "row-reverse" => 2,
                "column-reverse" => 3,
                _ => 0,
            };
        }
        "justify-content" => {
            style.justify_content = match val.trim() {
                "flex-end" | "end" => 1,
                "center" => 2,
                "space-between" => 3,
                "space-around" => 4,
                "space-evenly" => 5,
                _ => 0,
            };
        }
        "align-items" => {
            style.align_items = match val.trim() {
                "flex-start" | "start" => 1,
                "flex-end" | "end" => 2,
                "center" => 3,
                _ => 0, // stretch
            };
        }
        "gap" => {
            style.gap = Length::parse(val).to_px(0, 16);
        }
        "row-gap" | "column-gap" => {
            // Treat as gap until we split row vs column.
            style.gap = Length::parse(val).to_px(0, 16);
        }
        "flex-wrap"
        | "flex" | "flex-grow" | "flex-shrink" | "flex-basis" => {}
        // Transitions/animations — ignore for now
        "transition" | "animation" | "transform" | "cursor" | "user-select"
        | "outline" | "outline-width" | "outline-color" | "outline-style"
        | "box-shadow" | "text-shadow" | "position" | "top" | "left"
        | "right" | "bottom" | "z-index" | "float" | "clear" => {}
        _ => {} // unknown property — ignore
    }
}

/// Split a string into up to 3 parts by spaces
fn split3(s: &str) -> [&str; 3] {
    let mut result = [""; 3];
    let mut idx = 0;
    for part in s.split_whitespace() {
        if idx < 3 {
            result[idx] = part;
            idx += 1;
        }
    }
    result
}

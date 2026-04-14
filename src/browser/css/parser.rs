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

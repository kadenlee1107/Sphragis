// Bat_OS — CSS Computed Style
// Defines CSS properties and their computed values.
// Each DOM element gets a ComputedStyle that determines how it renders.

/// CSS color value (ARGB)
#[derive(Clone, Copy, PartialEq)]
pub struct Color(pub u32);

impl Color {
    pub const TRANSPARENT: Color = Color(0x00000000);
    pub const BLACK: Color = Color(0xFF000000);
    pub const WHITE: Color = Color(0xFFFFFFFF);
    pub const RED: Color = Color(0xFF0000FF);
    pub const GREEN: Color = Color(0xFF00FF00);
    pub const BLUE: Color = Color(0xFFFF0000);
    pub const GRAY: Color = Color(0xFFA0A0A0);

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color(0xFF000000 | (b as u32) << 16 | (g as u32) << 8 | r as u32)
    }

    /// Parse CSS color: #RGB, #RRGGBB, or named color
    pub fn parse(s: &str) -> Color {
        if s.starts_with('#') {
            let hex = &s[1..];
            if hex.len() == 3 {
                let r = parse_hex_digit(hex.as_bytes()[0]) * 17;
                let g = parse_hex_digit(hex.as_bytes()[1]) * 17;
                let b = parse_hex_digit(hex.as_bytes()[2]) * 17;
                return Color::from_rgb(r, g, b);
            } else if hex.len() == 6 {
                let r = parse_hex_digit(hex.as_bytes()[0]) * 16 + parse_hex_digit(hex.as_bytes()[1]);
                let g = parse_hex_digit(hex.as_bytes()[2]) * 16 + parse_hex_digit(hex.as_bytes()[3]);
                let b = parse_hex_digit(hex.as_bytes()[4]) * 16 + parse_hex_digit(hex.as_bytes()[5]);
                return Color::from_rgb(r, g, b);
            }
        }
        match s {
            "black" => Color::BLACK,
            "white" => Color::WHITE,
            "red" => Color::RED,
            "green" => Color::GREEN,
            "blue" => Color::BLUE,
            "gray" | "grey" => Color::GRAY,
            "transparent" => Color::TRANSPARENT,
            "orange" => Color::from_rgb(255, 165, 0),
            "yellow" => Color::from_rgb(255, 255, 0),
            "purple" => Color::from_rgb(128, 0, 128),
            "pink" => Color::from_rgb(255, 192, 203),
            "navy" => Color::from_rgb(0, 0, 128),
            "teal" => Color::from_rgb(0, 128, 128),
            "silver" => Color::from_rgb(192, 192, 192),
            "maroon" => Color::from_rgb(128, 0, 0),
            _ => Color::BLACK,
        }
    }

    pub fn raw(&self) -> u32 { self.0 }
}

fn parse_hex_digit(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}

/// CSS length value
#[derive(Clone, Copy, PartialEq)]
pub enum Length {
    Auto,
    Px(i32),
    Percent(i32), // percent * 100 (so 50% = 5000)
    Em(i32),      // em * 100 (so 1.5em = 150)
}

impl Length {
    pub fn parse(s: &str) -> Length {
        let s = s.trim();
        if s == "auto" { return Length::Auto; }
        if s.ends_with("px") {
            if let Some(n) = parse_int(&s[..s.len()-2]) { return Length::Px(n); }
        }
        if s.ends_with('%') {
            if let Some(n) = parse_int(&s[..s.len()-1]) { return Length::Percent(n * 100); }
        }
        if s.ends_with("em") {
            if let Some(n) = parse_int(&s[..s.len()-2]) { return Length::Em(n * 100); }
        }
        // Bare number → px
        if let Some(n) = parse_int(s) { return Length::Px(n); }
        Length::Auto
    }

    pub fn to_px(&self, parent_px: i32, font_size: i32) -> i32 {
        match self {
            Length::Auto => 0,
            Length::Px(n) => *n,
            Length::Percent(n) => parent_px * n / 10000,
            Length::Em(n) => font_size * n / 100,
        }
    }
}

fn parse_int(s: &str) -> Option<i32> {
    let mut result: i32 = 0;
    let mut neg = false;
    let bytes = s.as_bytes();
    let mut i = 0;
    if i < bytes.len() && bytes[i] == b'-' { neg = true; i += 1; }
    if i >= bytes.len() { return None; }
    while i < bytes.len() {
        if bytes[i] >= b'0' && bytes[i] <= b'9' {
            result = result * 10 + (bytes[i] - b'0') as i32;
            i += 1;
        } else if bytes[i] == b'.' {
            break; // ignore decimal
        } else {
            return None;
        }
    }
    Some(if neg { -result } else { result })
}

/// CSS display property
#[derive(Clone, Copy, PartialEq)]
pub enum Display {
    Block,
    Inline,
    InlineBlock,
    None,
    Flex,
    ListItem,
}

/// CSS font-weight
#[derive(Clone, Copy, PartialEq)]
pub enum FontWeight {
    Normal,  // 400
    Bold,    // 700
}

/// CSS text-align
#[derive(Clone, Copy, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// CSS text-decoration
#[derive(Clone, Copy)]
pub struct TextDecoration {
    pub underline: bool,
    pub line_through: bool,
}

/// Computed style for a DOM element — determines rendering
#[derive(Clone, Copy)]
pub struct ComputedStyle {
    pub display: Display,
    pub color: Color,
    pub background_color: Color,
    pub font_size: i32,       // in pixels
    pub font_weight: FontWeight,
    pub text_align: TextAlign,
    pub text_decoration: TextDecoration,
    pub margin_top: i32,
    pub margin_bottom: i32,
    pub margin_left: i32,
    pub margin_right: i32,
    pub padding_top: i32,
    pub padding_bottom: i32,
    pub padding_left: i32,
    pub padding_right: i32,
    pub border_width: i32,
    pub border_color: Color,
    pub width: Length,
    pub height: Length,
}

impl ComputedStyle {
    pub const fn default() -> Self {
        ComputedStyle {
            display: Display::Inline,
            color: Color::GRAY,
            background_color: Color::TRANSPARENT,
            font_size: 16,
            font_weight: FontWeight::Normal,
            text_align: TextAlign::Left,
            text_decoration: TextDecoration { underline: false, line_through: false },
            margin_top: 0, margin_bottom: 0,
            margin_left: 0, margin_right: 0,
            padding_top: 0, padding_bottom: 0,
            padding_left: 0, padding_right: 0,
            border_width: 0,
            border_color: Color::TRANSPARENT,
            width: Length::Auto,
            height: Length::Auto,
        }
    }

    /// Default style for a given HTML tag
    pub fn for_tag(tag: &str) -> Self {
        let mut s = Self::default();
        match tag {
            "html" | "body" => {
                s.display = Display::Block;
                s.color = Color::from_rgb(180, 180, 180); // light gray
            }
            "div" | "section" | "article" | "aside" | "header" | "footer"
            | "nav" | "main" | "form" | "figure" | "figcaption" => {
                s.display = Display::Block;
            }
            "h1" => {
                s.display = Display::Block;
                s.font_size = 32;
                s.font_weight = FontWeight::Bold;
                s.color = Color::WHITE;
                s.margin_top = 16; s.margin_bottom = 12;
            }
            "h2" => {
                s.display = Display::Block;
                s.font_size = 24;
                s.font_weight = FontWeight::Bold;
                s.color = Color::WHITE;
                s.margin_top = 14; s.margin_bottom = 10;
            }
            "h3" => {
                s.display = Display::Block;
                s.font_size = 20;
                s.font_weight = FontWeight::Bold;
                s.color = Color::from_rgb(220, 220, 220);
                s.margin_top = 12; s.margin_bottom = 8;
            }
            "h4" | "h5" | "h6" => {
                s.display = Display::Block;
                s.font_size = 16;
                s.font_weight = FontWeight::Bold;
                s.color = Color::from_rgb(200, 200, 200);
                s.margin_top = 8; s.margin_bottom = 6;
            }
            "p" => {
                s.display = Display::Block;
                s.margin_top = 8; s.margin_bottom = 8;
            }
            "a" => {
                s.color = Color::from_rgb(68, 153, 255); // blue
                s.text_decoration.underline = true;
            }
            "b" | "strong" => {
                s.font_weight = FontWeight::Bold;
                s.color = Color::WHITE;
            }
            "i" | "em" => {
                s.color = Color::from_rgb(200, 200, 200);
                // Note: italic rendering requires font support
            }
            "code" => {
                s.color = Color::from_rgb(68, 221, 68); // green
                s.background_color = Color::from_rgb(20, 20, 20);
                s.padding_left = 2; s.padding_right = 2;
            }
            "pre" => {
                s.display = Display::Block;
                s.color = Color::from_rgb(68, 221, 68);
                s.background_color = Color::from_rgb(18, 18, 18);
                s.padding_top = 8; s.padding_bottom = 8;
                s.padding_left = 8; s.padding_right = 8;
                s.margin_top = 8; s.margin_bottom = 8;
            }
            "blockquote" => {
                s.display = Display::Block;
                s.color = Color::from_rgb(140, 140, 140);
                s.border_width = 3;
                s.border_color = Color::from_rgb(60, 60, 60);
                s.padding_left = 16;
                s.margin_top = 8; s.margin_bottom = 8;
                s.margin_left = 8;
            }
            "ul" | "ol" => {
                s.display = Display::Block;
                s.margin_top = 4; s.margin_bottom = 4;
                s.padding_left = 20;
            }
            "li" => {
                s.display = Display::ListItem;
                s.margin_top = 2; s.margin_bottom = 2;
            }
            "hr" => {
                s.display = Display::Block;
                s.margin_top = 8; s.margin_bottom = 8;
                s.border_width = 1;
                s.border_color = Color::from_rgb(60, 60, 60);
            }
            "br" => {
                s.display = Display::Block;
            }
            "table" => {
                s.display = Display::Block;
                s.margin_top = 4; s.margin_bottom = 4;
            }
            "tr" => { s.display = Display::Block; }
            "td" | "th" => {
                s.display = Display::Inline;
                s.padding_right = 16;
                if tag == "th" { s.font_weight = FontWeight::Bold; }
            }
            "img" => {
                s.display = Display::InlineBlock;
            }
            "script" | "style" | "head" | "meta" | "link" | "title" => {
                s.display = Display::None;
            }
            _ => {
                // Default inline for unknown tags
                s.display = Display::Inline;
            }
        }
        s
    }
}

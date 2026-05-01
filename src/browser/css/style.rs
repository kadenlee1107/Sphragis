// Bat_OS — CSS Computed Style
// Defines CSS properties and their computed values.
// Each DOM element gets a ComputedStyle that determines how it renders.

/// CSS color value (ARGB)
#[derive(Clone, Copy, PartialEq)]
pub struct Color(pub u32);

impl Color {
    // 🎯 STUMP #67: Color words target the framebuffer's B8G8R8A8 byte
    // order (virtio-gpu's FORMAT_B8G8R8A8). When stored as a u32 LE,
    // that's bytes [B, G, R, A] in memory — i.e. the u32 numeric
    // value is `(A << 24) | (R << 16) | (G << 8) | B`. Pre-fix
    // `from_rgb` had R and B swapped, so #ffd700 (gold) rendered as
    // cyan and #4fc3f7 (sky-blue) rendered as gold — exactly the
    // "H1/H2 colors look swapped" symptom we were seeing.
    pub const TRANSPARENT: Color = Color(0x00000000);
    pub const BLACK: Color = Color(0xFF000000);
    pub const WHITE: Color = Color(0xFFFFFFFF);
    pub const RED:   Color = Color(0xFFFF0000);  // was 0xFF0000FF (blue!)
    pub const GREEN: Color = Color(0xFF00FF00);
    pub const BLUE:  Color = Color(0xFF0000FF);  // was 0xFFFF0000 (red!)
    pub const GRAY:  Color = Color(0xFFA0A0A0);

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color(0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32)
    }

    /// Parse CSS color: #RGB, #RRGGBB, rgb(r,g,b), rgba(r,g,b,a), or named
    pub fn parse(s: &str) -> Color {
        let s = s.trim();
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
        // rgb(r, g, b) and rgba(r, g, b, a)
        if (s.starts_with("rgb(") || s.starts_with("rgba(")) && s.ends_with(')') {
            let inner_start = if s.starts_with("rgba(") { 5 } else { 4 };
            let inner = &s[inner_start..s.len()-1];
            let mut vals = [0u8; 4];
            vals[3] = 255; // default alpha
            let mut vi = 0;
            let mut num: i32 = 0;
            let mut has_num = false;
            for &b in inner.as_bytes() {
                if b >= b'0' && b <= b'9' {
                    num = num * 10 + (b - b'0') as i32;
                    has_num = true;
                } else if (b == b',' || b == b' ') && has_num {
                    if vi < 4 { vals[vi] = num.min(255) as u8; vi += 1; }
                    num = 0;
                    has_num = false;
                }
            }
            if has_num && vi < 4 { vals[vi] = num.min(255) as u8; }
            return Color::from_rgb(vals[0], vals[1], vals[2]);
        }
        match s {
            "black" => Color::BLACK,
            "white" => Color::WHITE,
            "red" => Color::RED,
            "green" | "lime" => Color::GREEN,
            "blue" => Color::BLUE,
            "gray" | "grey" => Color::GRAY,
            "transparent" | "inherit" | "initial" => Color::TRANSPARENT,
            "orange" => Color::from_rgb(255, 165, 0),
            "yellow" => Color::from_rgb(255, 255, 0),
            "purple" => Color::from_rgb(128, 0, 128),
            "pink" => Color::from_rgb(255, 192, 203),
            "navy" => Color::from_rgb(0, 0, 128),
            "teal" => Color::from_rgb(0, 128, 128),
            "silver" => Color::from_rgb(192, 192, 192),
            "maroon" => Color::from_rgb(128, 0, 0),
            "cyan" | "aqua" => Color::from_rgb(0, 255, 255),
            "magenta" | "fuchsia" => Color::from_rgb(255, 0, 255),
            "olive" => Color::from_rgb(128, 128, 0),
            "indigo" => Color::from_rgb(75, 0, 130),
            "coral" => Color::from_rgb(255, 127, 80),
            "salmon" => Color::from_rgb(250, 128, 114),
            "tomato" => Color::from_rgb(255, 99, 71),
            "crimson" => Color::from_rgb(220, 20, 60),
            "gold" => Color::from_rgb(255, 215, 0),
            "khaki" => Color::from_rgb(240, 230, 140),
            "plum" => Color::from_rgb(221, 160, 221),
            "orchid" => Color::from_rgb(218, 112, 214),
            "violet" => Color::from_rgb(238, 130, 238),
            "tan" => Color::from_rgb(210, 180, 140),
            "beige" => Color::from_rgb(245, 245, 220),
            "ivory" => Color::from_rgb(255, 255, 240),
            "linen" => Color::from_rgb(250, 240, 230),
            "snow" => Color::from_rgb(255, 250, 250),
            "darkgray" | "darkgrey" => Color::from_rgb(169, 169, 169),
            "lightgray" | "lightgrey" => Color::from_rgb(211, 211, 211),
            "dimgray" | "dimgrey" => Color::from_rgb(105, 105, 105),
            "darkblue" => Color::from_rgb(0, 0, 139),
            "darkgreen" => Color::from_rgb(0, 100, 0),
            "darkred" => Color::from_rgb(139, 0, 0),
            "darkcyan" => Color::from_rgb(0, 139, 139),
            "darkmagenta" => Color::from_rgb(139, 0, 139),
            "darkorange" => Color::from_rgb(255, 140, 0),
            "lightblue" => Color::from_rgb(173, 216, 230),
            "lightgreen" => Color::from_rgb(144, 238, 144),
            "lightyellow" => Color::from_rgb(255, 255, 224),
            "lightpink" => Color::from_rgb(255, 182, 193),
            "steelblue" => Color::from_rgb(70, 130, 180),
            "royalblue" => Color::from_rgb(65, 105, 225),
            "dodgerblue" => Color::from_rgb(30, 144, 255),
            "skyblue" => Color::from_rgb(135, 206, 235),
            "slategray" | "slategrey" => Color::from_rgb(112, 128, 144),
            "whitesmoke" => Color::from_rgb(245, 245, 245),
            "limegreen" => Color::from_rgb(50, 205, 50),
            "seagreen" => Color::from_rgb(46, 139, 87),
            "forestgreen" => Color::from_rgb(34, 139, 34),
            "firebrick" => Color::from_rgb(178, 34, 34),
            "chocolate" => Color::from_rgb(210, 105, 30),
            "sienna" => Color::from_rgb(160, 82, 45),
            "peru" => Color::from_rgb(205, 133, 63),
            "wheat" => Color::from_rgb(245, 222, 179),
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

/// CSS font-style
#[derive(Clone, Copy, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
}

/// CSS font-family family hint. We don't yet ship multiple fonts —
/// Verdana TrueType is the only one in the initrd — but we can pick
/// a different paint path for `monospace` so `<code>` and `<pre>`
/// content has fixed-width feel even though it's still drawn with
/// the same outlines.
#[derive(Clone, Copy, PartialEq)]
pub enum FontFamily {
    Sans,
    Serif,
    Monospace,
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

/// CSS overflow
#[derive(Clone, Copy, PartialEq)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

/// CSS visibility
#[derive(Clone, Copy, PartialEq)]
pub enum Visibility {
    Visible,
    Hidden,
    Collapse,
}

/// CSS white-space
#[derive(Clone, Copy, PartialEq)]
pub enum WhiteSpace {
    Normal,
    NoWrap,
    Pre,
    PreWrap,
}

/// CSS text-transform
#[derive(Clone, Copy, PartialEq)]
pub enum TextTransform {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

/// CSS vertical-align
#[derive(Clone, Copy, PartialEq)]
pub enum VerticalAlign {
    Baseline,
    Top,
    Middle,
    Bottom,
}

/// Computed style for a DOM element — determines rendering
#[derive(Clone, Copy)]
pub struct ComputedStyle {
    pub display: Display,
    pub color: Color,
    pub background_color: Color,
    pub font_size: i32,       // in pixels
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub font_family: FontFamily,
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
    // Extended properties
    pub max_width: Length,
    pub min_height: Length,
    pub line_height: i32,     // in pixels (0 = auto/normal)
    pub overflow: Overflow,
    pub visibility: Visibility,
    pub opacity: u8,          // 0–255 (255 = fully opaque)
    pub border_radius: i32,   // pixels
    pub text_transform: TextTransform,
    pub white_space: WhiteSpace,
    pub vertical_align: VerticalAlign,
    // 🎯 STUMP #75: minimal flexbox properties on every box. Only used
    // when display: flex; cheap to carry on every ComputedStyle since
    // we already have ~20 fields. Strings parsed in css/parser.rs.
    pub flex_direction: u8,   // 0=row 1=column 2=row-reverse 3=column-reverse
    pub justify_content: u8,  // 0=start 1=end 2=center 3=between 4=around 5=evenly
    pub align_items: u8,      // 0=stretch 1=start 2=end 3=center
    pub gap: i32,
    // 🎯 STUMP #82: minimal box-shadow. Single-color soft drop shadow
    // applied as ~3 stacked rectangles offset by `(box_shadow_x,
    // box_shadow_y)` with `box_shadow_blur`-pixel feather. Setting
    // box_shadow_color to TRANSPARENT (default) skips the paint.
    pub box_shadow_x: i32,
    pub box_shadow_y: i32,
    pub box_shadow_blur: i32,
    pub box_shadow_color: Color,
}

impl ComputedStyle {
    pub const fn default() -> Self {
        ComputedStyle {
            display: Display::Inline,
            color: Color::GRAY,
            background_color: Color::TRANSPARENT,
            font_size: 16,
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            font_family: FontFamily::Sans,
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
            max_width: Length::Auto,
            min_height: Length::Auto,
            flex_direction: 0,
            justify_content: 0,
            align_items: 0,
            gap: 0,
            box_shadow_x: 0,
            box_shadow_y: 0,
            box_shadow_blur: 0,
            box_shadow_color: Color::TRANSPARENT,
            line_height: 0,
            overflow: Overflow::Visible,
            visibility: Visibility::Visible,
            opacity: 255,
            border_radius: 0,
            text_transform: TextTransform::None,
            white_space: WhiteSpace::Normal,
            vertical_align: VerticalAlign::Baseline,
        }
    }

    /// Default style for a given HTML tag — clean reader mode typography
    pub fn for_tag(tag: &str) -> Self {
        let mut s = Self::default();
        // Default body text: white on dark background
        s.color = Color::from_rgb(200, 200, 200);
        match tag {
            "html" | "body" => {
                s.display = Display::Block;
                s.color = Color::from_rgb(232, 234, 237); // Google's text color
                s.background_color = Color::from_rgb(32, 33, 36); // Google dark mode bg
                s.padding_left = 16;
                s.padding_right = 16;
                s.padding_top = 8;
            }
            "div" | "main" | "figure" | "figcaption" => {
                s.display = Display::Block;
                s.margin_top = 6;
                s.margin_bottom = 4;
            }
            "form" => {
                // Forms are hidden by reader mode, but keep display for fallback
                s.display = Display::Block;
                s.margin_top = 4;
                s.margin_bottom = 2;
            }
            "section" | "article" => {
                s.display = Display::Block;
                s.margin_top = 8;
                s.margin_bottom = 4;
                s.padding_top = 4;
                s.padding_bottom = 4;
            }
            "aside" | "nav" => {
                // Typically hidden by reader mode extract_content
                s.display = Display::Block;
                s.margin_top = 4;
                s.margin_bottom = 4;
            }
            "header" => {
                // 🎯 STUMP #80: distinguish <header> visually from
                // a generic block. Subtle bottom border to separate
                // navigation/branding area from page content.
                s.display = Display::Block;
                s.margin_top = 0;
                s.margin_bottom = 12;
                s.padding_top = 8;
                s.padding_bottom = 8;
                s.border_width = 0; // overridden below for bottom-only
            }
            "footer" => {
                s.display = Display::Block;
                s.margin_top = 16;
                s.margin_bottom = 0;
                s.padding_top = 8;
                s.padding_bottom = 8;
                s.color = Color::from_rgb(140, 140, 140);
                s.font_size = 13;
            }
            "h1" => {
                s.display = Display::Block;
                s.font_size = 36;
                s.font_weight = FontWeight::Bold;
                s.color = Color::WHITE;
                s.margin_top = 32; s.margin_bottom = 16;
            }
            "h2" => {
                s.display = Display::Block;
                s.font_size = 24;
                s.font_weight = FontWeight::Bold;
                s.color = Color::WHITE;
                s.margin_top = 20; s.margin_bottom = 10;
                // Subtle separator line
                s.border_width = 1;
                s.border_color = Color::from_rgb(40, 40, 40);
                s.padding_bottom = 6;
            }
            "h3" => {
                s.display = Display::Block;
                s.font_size = 20;
                s.font_weight = FontWeight::Bold;
                s.color = Color::from_rgb(230, 230, 230);
                s.margin_top = 16; s.margin_bottom = 8;
            }
            "h4" | "h5" | "h6" => {
                s.display = Display::Block;
                s.font_size = 16;
                s.font_weight = FontWeight::Bold;
                s.color = Color::from_rgb(210, 210, 210);
                s.margin_top = 12; s.margin_bottom = 6;
            }
            "p" => {
                s.display = Display::Block;
                s.margin_top = 12; s.margin_bottom = 12;
            }
            "a" => {
                // Standard browser-default link styling. Visited
                // distinction would need history; not yet.
                s.color = Color::from_rgb(99, 174, 255); // bright link blue
                s.text_decoration.underline = true;
            }
            "b" | "strong" => {
                s.font_weight = FontWeight::Bold;
                s.color = Color::WHITE;
            }
            "i" | "em" => {
                s.font_style = FontStyle::Italic;
                s.color = Color::from_rgb(210, 210, 210);
            }
            "code" | "kbd" | "samp" | "tt" => {
                s.color = Color::from_rgb(68, 221, 68); // green
                s.background_color = Color::from_rgb(20, 20, 20);
                s.padding_left = 2; s.padding_right = 2;
                s.font_family = FontFamily::Monospace;
            }
            "pre" => {
                s.display = Display::Block;
                s.color = Color::from_rgb(68, 221, 68);
                s.background_color = Color::from_rgb(18, 18, 18);
                s.padding_top = 8; s.padding_bottom = 8;
                s.padding_left = 8; s.padding_right = 8;
                s.margin_top = 8; s.margin_bottom = 8;
                s.font_family = FontFamily::Monospace;
            }
            "blockquote" => {
                s.display = Display::Block;
                s.color = Color::from_rgb(150, 150, 150); // dimmer text
                s.border_width = 3;
                s.border_color = Color::from_rgb(60, 60, 60);
                s.padding_left = 16; // 16px left indent
                s.margin_top = 8; s.margin_bottom = 8;
                s.margin_left = 8;
            }
            "ul" | "ol" => {
                s.display = Display::Block;
                s.margin_top = 4; s.margin_bottom = 4;
                s.margin_left = 16;
                s.padding_left = 20; // 20px left indent for lists
            }
            "li" => {
                s.display = Display::ListItem;
                s.margin_top = 2; s.margin_bottom = 2;
                s.margin_left = 20;
            }
            "dl" => {
                s.display = Display::Block;
                s.margin_top = 4; s.margin_bottom = 4;
            }
            "dt" => {
                s.display = Display::Block;
                s.font_weight = FontWeight::Bold;
                s.color = Color::WHITE;
                s.margin_top = 4;
            }
            "dd" => {
                s.display = Display::Block;
                s.margin_left = 20;
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
                s.margin_top = 8; s.margin_bottom = 8;
                s.border_width = 1;
                s.border_color = Color::from_rgb(50, 50, 50);
            }
            "tr" => { s.display = Display::Block; }
            "td" | "th" => {
                s.display = Display::Inline;
                s.padding_right = 16;
                if tag == "th" {
                    s.font_weight = FontWeight::Bold;
                    s.color = Color::WHITE;
                }
            }
            "img" => {
                s.display = Display::InlineBlock;
            }
            "sup" | "sub" => {
                // Superscript/subscript — render inline, dimmer
                s.display = Display::Inline;
                s.color = Color::from_rgb(140, 140, 140);
            }
            // 🎯 STUMP #80: more HTML5 semantic + inline-emphasis tags.
            "mark" => {
                // Highlight — yellow background, dark text.
                s.display = Display::Inline;
                s.background_color = Color::from_rgb(255, 230, 0);
                s.color = Color::from_rgb(20, 20, 20);
                s.padding_left = 2; s.padding_right = 2;
            }
            "small" => {
                s.display = Display::Inline;
                s.font_size = 12;
                s.color = Color::from_rgb(150, 150, 150);
            }
            "abbr" => {
                s.display = Display::Inline;
                s.text_decoration.underline = true;
                s.color = Color::from_rgb(180, 180, 180);
            }
            "cite" => {
                s.display = Display::Inline;
                s.font_style = FontStyle::Italic;
                s.color = Color::from_rgb(190, 190, 190);
            }
            "q" => {
                // <q> is browser-defaulted to wrap content in quotes.
                // We don't synthesize the quotes (would need ::before
                // pseudo-elements); the user can include them inline.
                s.display = Display::Inline;
                s.color = Color::from_rgb(190, 190, 190);
            }
            "del" | "s" => {
                s.display = Display::Inline;
                s.text_decoration.line_through = true;
                s.color = Color::from_rgb(160, 160, 160);
            }
            "ins" | "u" => {
                s.display = Display::Inline;
                s.text_decoration.underline = true;
            }
            "details" => {
                s.display = Display::Block;
                s.margin_top = 6;
                s.margin_bottom = 6;
                s.padding_left = 12;
                s.background_color = Color::from_rgb(20, 22, 26);
                s.border_width = 1;
                s.border_color = Color::from_rgb(50, 50, 60);
                s.border_radius = 4;
                s.padding_top = 6;
                s.padding_bottom = 6;
            }
            "summary" => {
                // <summary> is a block child of <details> in our
                // current rendering (no toggle interaction yet).
                s.display = Display::Block;
                s.font_weight = FontWeight::Bold;
                s.color = Color::WHITE;
                s.margin_bottom = 4;
            }
            "figure" => {
                s.display = Display::Block;
                s.margin_top = 8;
                s.margin_bottom = 8;
                s.padding_left = 16;
            }
            "figcaption" => {
                s.display = Display::Block;
                s.color = Color::from_rgb(150, 150, 150);
                s.font_size = 13;
                s.margin_top = 4;
            }
            "span" => {
                s.display = Display::Inline;
            }
            "script" | "style" | "head" | "meta" | "link" | "title" | "noscript" => {
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

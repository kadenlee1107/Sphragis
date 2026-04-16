#![allow(dead_code)]
#![allow(unused_assignments)]
// Bat_OS — JavaScript Lexer (Tokenizer)
// Converts JavaScript source code into a stream of tokens.
// Handles: keywords, identifiers, numbers, strings, operators, punctuation.

pub const MAX_TOKENS: usize = 2048;
pub const MAX_STR: usize = 128;

#[derive(Clone, Copy, PartialEq)]
pub enum TokenType {
    // Literals
    Number,        // 42, 3.14
    String,        // "hello", 'world'
    Bool,          // true, false
    Null,          // null
    Undefined,     // undefined

    // Identifiers + Keywords
    Identifier,    // foo, bar, myVar
    Var,           // var
    Let,           // let
    Const,         // const
    Function,      // function
    Return,        // return
    If,            // if
    Else,          // else
    While,         // while
    For,           // for
    Break,         // break
    Continue,      // continue
    New,           // new
    This,          // this
    Typeof,        // typeof
    Void,          // void
    Delete,        // delete
    In,            // in
    Of,            // of
    Switch,        // switch
    Case,          // case
    Default,       // default
    Try,           // try
    Catch,         // catch
    Finally,       // finally
    Throw,         // throw
    Class,         // class
    Extends,       // extends
    Import,        // import
    Export,        // export

    // Operators
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    Percent,       // %
    Assign,        // =
    PlusAssign,    // +=
    MinusAssign,   // -=
    StarAssign,    // *=
    SlashAssign,   // /=
    Equal,         // ==
    StrictEqual,   // ===
    NotEqual,      // !=
    StrictNotEqual,// !==
    Less,          // <
    Greater,       // >
    LessEqual,     // <=
    GreaterEqual,  // >=
    And,           // &&
    Or,            // ||
    Not,           // !
    BitAnd,        // &
    BitOr,         // |
    BitXor,        // ^
    BitNot,        // ~
    ShiftLeft,     // <<
    ShiftRight,    // >>
    Increment,     // ++
    Decrement,     // --
    Arrow,         // =>
    Dot,           // .
    Spread,        // ...
    Question,      // ?
    OptionalChain, // ?.
    Colon,         // :

    // Punctuation
    LeftParen,     // (
    RightParen,    // )
    LeftBrace,     // {
    RightBrace,    // }
    LeftBracket,   // [
    RightBracket,  // ]
    Semicolon,     // ;
    Comma,         // ,

    // Template literals
    TemplateStart,     // `hello ${
    TemplateMid,       // } middle ${
    TemplateEnd,       // } end`
    TemplateNoSub,     // `no substitutions`

    // Special
    Eof,
    Error,
}

#[derive(Clone, Copy)]
pub struct Token {
    pub token_type: TokenType,
    pub text: [u8; MAX_STR],
    pub text_len: usize,
    pub num_value: f64,    // for Number tokens
    pub line: u32,
}

impl Token {
    pub const fn empty() -> Self {
        Token {
            token_type: TokenType::Eof,
            text: [0; MAX_STR],
            text_len: 0,
            num_value: 0.0,
            line: 0,
        }
    }

    pub fn text_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.text[..self.text_len]) }
    }

    fn set_text(&mut self, s: &[u8]) {
        let len = s.len().min(MAX_STR);
        self.text[..len].copy_from_slice(&s[..len]);
        self.text_len = len;
    }
}

/// Tokenize JavaScript source into tokens.
pub fn tokenize(source: &[u8], tokens: &mut [Token; MAX_TOKENS]) -> usize {
    let mut count = 0usize;
    let mut pos = 0usize;
    let mut line = 1u32;

    while pos < source.len() && count < MAX_TOKENS - 1 {
        // Skip whitespace
        while pos < source.len() && is_whitespace(source[pos]) {
            if source[pos] == b'\n' { line += 1; }
            pos += 1;
        }
        if pos >= source.len() { break; }

        let ch = source[pos];

        // Skip comments
        if ch == b'/' && pos + 1 < source.len() {
            if source[pos + 1] == b'/' {
                // Line comment
                while pos < source.len() && source[pos] != b'\n' { pos += 1; }
                continue;
            }
            if source[pos + 1] == b'*' {
                // Block comment
                pos += 2;
                while pos + 1 < source.len() {
                    if source[pos] == b'*' && source[pos + 1] == b'/' { pos += 2; break; }
                    if source[pos] == b'\n' { line += 1; }
                    pos += 1;
                }
                continue;
            }
        }

        let tok = &mut tokens[count];
        tok.line = line;

        // Numbers
        if ch.is_ascii_digit() || (ch == b'.' && pos + 1 < source.len() && source[pos+1].is_ascii_digit()) {
            tok.token_type = TokenType::Number;
            let start = pos;
            while pos < source.len() && (source[pos].is_ascii_digit() || source[pos] == b'.') {
                pos += 1;
            }
            tok.set_text(&source[start..pos]);
            // Parse number
            tok.num_value = parse_f64(&source[start..pos]);
            count += 1;
            continue;
        }

        // Strings
        if ch == b'"' || ch == b'\'' {
            let quote = ch;
            pos += 1;
            let start = pos;
            tok.token_type = TokenType::String;
            while pos < source.len() && source[pos] != quote {
                if source[pos] == b'\\' { pos += 1; } // escape
                if source[pos] == b'\n' { line += 1; }
                pos += 1;
            }
            tok.set_text(&source[start..pos]);
            if pos < source.len() { pos += 1; } // skip closing quote
            count += 1;
            continue;
        }

        // Template literals
        if ch == b'`' {
            pos += 1;
            let start = pos;
            // Scan for ${ or closing backtick
            let mut has_sub = false;
            while pos < source.len() && source[pos] != b'`' {
                if source[pos] == b'$' && pos + 1 < source.len() && source[pos + 1] == b'{' {
                    has_sub = true;
                    break;
                }
                if source[pos] == b'\\' { pos += 1; }
                if pos < source.len() && source[pos] == b'\n' { line += 1; }
                pos += 1;
            }
            if has_sub {
                // Template with substitutions: emit TemplateStart
                tok.token_type = TokenType::TemplateStart;
                tok.set_text(&source[start..pos]);
                pos += 2; // skip ${
                count += 1;

                // Now lex the expression inside ${ ... } and subsequent template parts
                // The rest of template handling happens via the parser re-entering the lexer
                // For simplicity, we'll handle the rest as normal tokens until we hit }
                // then continue template
                let mut brace_depth = 1;
                while pos < source.len() && count < MAX_TOKENS - 2 {
                    // Skip whitespace
                    while pos < source.len() && is_whitespace(source[pos]) {
                        if source[pos] == b'\n' { line += 1; }
                        pos += 1;
                    }
                    if pos >= source.len() { break; }

                    if source[pos] == b'}' {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            pos += 1; // skip closing }
                            // Continue scanning template
                            let mid_start = pos;
                            let mut more_sub = false;
                            while pos < source.len() && source[pos] != b'`' {
                                if source[pos] == b'$' && pos + 1 < source.len() && source[pos + 1] == b'{' {
                                    more_sub = true;
                                    break;
                                }
                                if source[pos] == b'\\' { pos += 1; }
                                if pos < source.len() && source[pos] == b'\n' { line += 1; }
                                pos += 1;
                            }
                            let template_tok = &mut tokens[count];
                            template_tok.line = line;
                            if more_sub {
                                template_tok.token_type = TokenType::TemplateMid;
                                template_tok.set_text(&source[mid_start..pos]);
                                pos += 2; // skip ${
                                count += 1;
                                brace_depth = 1;
                                continue;
                            } else {
                                template_tok.token_type = TokenType::TemplateEnd;
                                template_tok.set_text(&source[mid_start..pos]);
                                if pos < source.len() { pos += 1; } // skip `
                                count += 1;
                                break;
                            }
                        }
                    }
                    if source[pos] == b'{' { brace_depth += 1; }

                    // Lex a normal token for the expression inside ${}
                    // Recursively handle by continuing the outer loop
                    // We break out and let the outer loop handle these tokens
                    break;
                }

                // Re-enter the outer loop for expression tokens
                // The TemplateEnd/TemplateMid tokens mark where template continues
                continue;
            } else {
                // No substitutions — simple template string
                tok.token_type = TokenType::TemplateNoSub;
                tok.set_text(&source[start..pos]);
                if pos < source.len() { pos += 1; } // skip `
                count += 1;
                continue;
            }
        }

        // Identifiers and keywords
        if ch.is_ascii_alphabetic() || ch == b'_' || ch == b'$' {
            let start = pos;
            while pos < source.len() && (source[pos].is_ascii_alphanumeric() || source[pos] == b'_' || source[pos] == b'$') {
                pos += 1;
            }
            let word = &source[start..pos];
            tok.set_text(word);

            tok.token_type = match word {
                b"var" => TokenType::Var,
                b"let" => TokenType::Let,
                b"const" => TokenType::Const,
                b"function" => TokenType::Function,
                b"return" => TokenType::Return,
                b"if" => TokenType::If,
                b"else" => TokenType::Else,
                b"while" => TokenType::While,
                b"for" => TokenType::For,
                b"break" => TokenType::Break,
                b"continue" => TokenType::Continue,
                b"new" => TokenType::New,
                b"this" => TokenType::This,
                b"typeof" => TokenType::Typeof,
                b"void" => TokenType::Void,
                b"delete" => TokenType::Delete,
                b"in" => TokenType::In,
                b"of" => TokenType::Of,
                b"true" | b"false" => TokenType::Bool,
                b"null" => TokenType::Null,
                b"undefined" => TokenType::Undefined,
                b"switch" => TokenType::Switch,
                b"case" => TokenType::Case,
                b"default" => TokenType::Default,
                b"try" => TokenType::Try,
                b"catch" => TokenType::Catch,
                b"finally" => TokenType::Finally,
                b"throw" => TokenType::Throw,
                b"class" => TokenType::Class,
                b"extends" => TokenType::Extends,
                b"import" => TokenType::Import,
                b"export" => TokenType::Export,
                _ => TokenType::Identifier,
            };
            count += 1;
            continue;
        }

        // Operators and punctuation
        tok.set_text(&source[pos..pos+1]);
        match ch {
            b'+' => {
                if pos + 1 < source.len() && source[pos+1] == b'+' { tok.token_type = TokenType::Increment; pos += 2; tok.set_text(b"++"); }
                else if pos + 1 < source.len() && source[pos+1] == b'=' { tok.token_type = TokenType::PlusAssign; pos += 2; tok.set_text(b"+="); }
                else { tok.token_type = TokenType::Plus; pos += 1; }
            }
            b'-' => {
                if pos + 1 < source.len() && source[pos+1] == b'-' { tok.token_type = TokenType::Decrement; pos += 2; }
                else if pos + 1 < source.len() && source[pos+1] == b'=' { tok.token_type = TokenType::MinusAssign; pos += 2; }
                else { tok.token_type = TokenType::Minus; pos += 1; }
            }
            b'*' => {
                if pos + 1 < source.len() && source[pos+1] == b'=' { tok.token_type = TokenType::StarAssign; pos += 2; }
                else { tok.token_type = TokenType::Star; pos += 1; }
            }
            b'/' => {
                if pos + 1 < source.len() && source[pos+1] == b'=' { tok.token_type = TokenType::SlashAssign; pos += 2; }
                else { tok.token_type = TokenType::Slash; pos += 1; }
            }
            b'%' => { tok.token_type = TokenType::Percent; pos += 1; }
            b'=' => {
                if pos + 2 < source.len() && source[pos+1] == b'=' && source[pos+2] == b'=' { tok.token_type = TokenType::StrictEqual; pos += 3; tok.set_text(b"==="); }
                else if pos + 1 < source.len() && source[pos+1] == b'=' { tok.token_type = TokenType::Equal; pos += 2; tok.set_text(b"=="); }
                else if pos + 1 < source.len() && source[pos+1] == b'>' { tok.token_type = TokenType::Arrow; pos += 2; tok.set_text(b"=>"); }
                else { tok.token_type = TokenType::Assign; pos += 1; }
            }
            b'!' => {
                if pos + 2 < source.len() && source[pos+1] == b'=' && source[pos+2] == b'=' { tok.token_type = TokenType::StrictNotEqual; pos += 3; }
                else if pos + 1 < source.len() && source[pos+1] == b'=' { tok.token_type = TokenType::NotEqual; pos += 2; }
                else { tok.token_type = TokenType::Not; pos += 1; }
            }
            b'<' => {
                if pos + 1 < source.len() && source[pos+1] == b'=' { tok.token_type = TokenType::LessEqual; pos += 2; }
                else if pos + 1 < source.len() && source[pos+1] == b'<' { tok.token_type = TokenType::ShiftLeft; pos += 2; }
                else { tok.token_type = TokenType::Less; pos += 1; }
            }
            b'>' => {
                if pos + 1 < source.len() && source[pos+1] == b'=' { tok.token_type = TokenType::GreaterEqual; pos += 2; }
                else if pos + 1 < source.len() && source[pos+1] == b'>' { tok.token_type = TokenType::ShiftRight; pos += 2; }
                else { tok.token_type = TokenType::Greater; pos += 1; }
            }
            b'&' => {
                if pos + 1 < source.len() && source[pos+1] == b'&' { tok.token_type = TokenType::And; pos += 2; }
                else { tok.token_type = TokenType::BitAnd; pos += 1; }
            }
            b'|' => {
                if pos + 1 < source.len() && source[pos+1] == b'|' { tok.token_type = TokenType::Or; pos += 2; }
                else { tok.token_type = TokenType::BitOr; pos += 1; }
            }
            b'^' => { tok.token_type = TokenType::BitXor; pos += 1; }
            b'~' => { tok.token_type = TokenType::BitNot; pos += 1; }
            b'.' => {
                if pos + 2 < source.len() && source[pos+1] == b'.' && source[pos+2] == b'.' { tok.token_type = TokenType::Spread; pos += 3; }
                else { tok.token_type = TokenType::Dot; pos += 1; }
            }
            b'?' => {
                if pos + 1 < source.len() && source[pos+1] == b'.' {
                    tok.token_type = TokenType::OptionalChain; pos += 2; tok.set_text(b"?.");
                } else {
                    tok.token_type = TokenType::Question; pos += 1;
                }
            }
            b':' => { tok.token_type = TokenType::Colon; pos += 1; }
            b'(' => { tok.token_type = TokenType::LeftParen; pos += 1; }
            b')' => { tok.token_type = TokenType::RightParen; pos += 1; }
            b'{' => { tok.token_type = TokenType::LeftBrace; pos += 1; }
            b'}' => { tok.token_type = TokenType::RightBrace; pos += 1; }
            b'[' => { tok.token_type = TokenType::LeftBracket; pos += 1; }
            b']' => { tok.token_type = TokenType::RightBracket; pos += 1; }
            b';' => { tok.token_type = TokenType::Semicolon; pos += 1; }
            b',' => { tok.token_type = TokenType::Comma; pos += 1; }
            _ => { tok.token_type = TokenType::Error; pos += 1; }
        }
        count += 1;
    }

    // EOF token
    tokens[count].token_type = TokenType::Eof;
    tokens[count].line = line;
    count += 1;

    count
}

fn is_whitespace(ch: u8) -> bool {
    ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r'
}

fn parse_f64(bytes: &[u8]) -> f64 {
    let mut result: f64 = 0.0;
    let mut decimal = false;
    let mut decimal_place: f64 = 0.1;
    for &b in bytes {
        if b == b'.' { decimal = true; continue; }
        if b >= b'0' && b <= b'9' {
            let digit = (b - b'0') as f64;
            if decimal {
                result += digit * decimal_place;
                decimal_place *= 0.1;
            } else {
                result = result * 10.0 + digit;
            }
        }
    }
    result
}

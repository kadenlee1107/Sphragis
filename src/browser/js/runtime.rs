// Bat_OS — JavaScript Runtime Values
// Defines the value types that exist during JS execution.

pub const MAX_PROPERTIES: usize = 32;
pub const MAX_STRING: usize = 128;

#[derive(Clone, Copy, PartialEq)]
pub enum JsType {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Object,
    Function,
    Array,
}

#[derive(Clone, Copy)]
pub struct JsValue {
    pub js_type: JsType,
    pub num: f64,
    pub boolean: bool,
    pub str_buf: [u8; MAX_STRING],
    pub str_len: usize,
    pub func_node: u16,     // AST node index for function body
    pub obj_id: u16,        // index into object pool
}

impl JsValue {
    pub const fn undefined() -> Self {
        JsValue {
            js_type: JsType::Undefined,
            num: 0.0, boolean: false,
            str_buf: [0; MAX_STRING], str_len: 0,
            func_node: 0xFFFF, obj_id: 0xFFFF,
        }
    }

    pub fn number(n: f64) -> Self {
        let mut v = Self::undefined();
        v.js_type = JsType::Number;
        v.num = n;
        v
    }

    pub fn boolean(b: bool) -> Self {
        let mut v = Self::undefined();
        v.js_type = JsType::Boolean;
        v.boolean = b;
        v
    }

    pub fn string(s: &[u8]) -> Self {
        let mut v = Self::undefined();
        v.js_type = JsType::String;
        v.str_len = s.len().min(MAX_STRING);
        v.str_buf[..v.str_len].copy_from_slice(&s[..v.str_len]);
        v
    }

    pub fn null() -> Self {
        let mut v = Self::undefined();
        v.js_type = JsType::Null;
        v
    }

    pub fn is_truthy(&self) -> bool {
        match self.js_type {
            JsType::Undefined | JsType::Null => false,
            JsType::Boolean => self.boolean,
            JsType::Number => self.num != 0.0,
            JsType::String => self.str_len > 0,
            _ => true,
        }
    }

    pub fn to_number(&self) -> f64 {
        match self.js_type {
            JsType::Number => self.num,
            JsType::Boolean => if self.boolean { 1.0 } else { 0.0 },
            JsType::String => {
                let _s = unsafe { core::str::from_utf8_unchecked(&self.str_buf[..self.str_len]) };
                // Simple integer parsing
                let mut n: f64 = 0.0;
                let mut neg = false;
                for (i, &b) in self.str_buf[..self.str_len].iter().enumerate() {
                    if i == 0 && b == b'-' { neg = true; continue; }
                    if b >= b'0' && b <= b'9' { n = n * 10.0 + (b - b'0') as f64; }
                }
                if neg { -n } else { n }
            }
            _ => 0.0,
        }
    }

    pub fn str_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.str_buf[..self.str_len]) }
    }

    pub fn to_string_buf(&self, buf: &mut [u8]) -> usize {
        match self.js_type {
            JsType::Undefined => { let s = b"undefined"; let l = s.len().min(buf.len()); buf[..l].copy_from_slice(&s[..l]); l }
            JsType::Null => { let s = b"null"; let l = s.len().min(buf.len()); buf[..l].copy_from_slice(&s[..l]); l }
            JsType::Boolean => {
                let s = if self.boolean { b"true" as &[u8] } else { b"false" };
                let l = s.len().min(buf.len()); buf[..l].copy_from_slice(&s[..l]); l
            }
            JsType::Number => {
                // Simple integer to string
                let n = self.num as i64;
                if n == 0 { buf[0] = b'0'; return 1; }
                let mut digits = [0u8; 20];
                let mut dlen = 0;
                let mut v = if n < 0 { -n } else { n } as u64;
                while v > 0 { digits[dlen] = b'0' + (v % 10) as u8; dlen += 1; v /= 10; }
                let mut pos = 0;
                if n < 0 && pos < buf.len() { buf[pos] = b'-'; pos += 1; }
                for i in (0..dlen).rev() { if pos < buf.len() { buf[pos] = digits[i]; pos += 1; } }
                pos
            }
            JsType::String => {
                let l = self.str_len.min(buf.len());
                buf[..l].copy_from_slice(&self.str_buf[..l]);
                l
            }
            _ => { let s = b"[object]"; let l = s.len().min(buf.len()); buf[..l].copy_from_slice(&s[..l]); l }
        }
    }
}

/// Variable scope (environment)
pub const MAX_VARS: usize = 64;

#[derive(Clone, Copy)]
pub struct Variable {
    pub name: [u8; 32],
    pub name_len: usize,
    pub value: JsValue,
}

impl Variable {
    pub const fn empty() -> Self {
        Variable {
            name: [0; 32], name_len: 0,
            value: JsValue::undefined(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Scope {
    pub vars: [Variable; MAX_VARS],
    pub var_count: usize,
    pub parent: u16, // index into scope stack
}

impl Scope {
    pub const fn new() -> Self {
        Scope {
            vars: [Variable::empty(); MAX_VARS],
            var_count: 0,
            parent: 0xFFFF,
        }
    }

    pub fn get(&self, name: &str) -> Option<JsValue> {
        for i in 0..self.var_count {
            let v = &self.vars[i];
            if v.name_len == name.len() {
                let n = unsafe { core::str::from_utf8_unchecked(&v.name[..v.name_len]) };
                if n == name { return Some(v.value); }
            }
        }
        None
    }

    pub fn set(&mut self, name: &str, value: JsValue) {
        // Check if exists
        for i in 0..self.var_count {
            let v = &mut self.vars[i];
            let n = unsafe { core::str::from_utf8_unchecked(&v.name[..v.name_len]) };
            if n == name {
                v.value = value;
                return;
            }
        }
        // Create new
        if self.var_count < MAX_VARS {
            let v = &mut self.vars[self.var_count];
            let len = name.len().min(32);
            v.name[..len].copy_from_slice(&name.as_bytes()[..len]);
            v.name_len = len;
            v.value = value;
            self.var_count += 1;
        }
    }
}

#![allow(dead_code)]
// Bat_OS — NaN-Boxed JavaScript Value
// Every JS value fits in a single u64. No heap allocation for primitives.
//
// Encoding (IEEE 754 NaN space):
//   Float64:   any non-NaN-boxed f64 passes through untouched
//   Quiet NaN: bits [63:51] = 0x7FF8 (quiet NaN signal)
//              bits [50:48] = tag (3 bits → 8 types)
//              bits [47:0]  = payload (48 bits)
//
// Tags:
//   001 = integer (i32 in lower 32 bits)
//   010 = boolean (bit 0: 0=false, 1=true)
//   011 = null
//   100 = undefined
//   101 = object (ObjId in lower 32 bits)
//   110 = string (StringId in lower 32 bits)
//   111 = symbol (reserved)

/// A JavaScript value packed into 8 bytes via NaN-boxing.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct JsValue(u64);

// NaN-box tag constants
const QNAN: u64        = 0x7FF8_0000_0000_0000;
const TAG_INT: u64     = 0x0001_0000_0000_0000; // 001 << 48
const TAG_BOOL: u64    = 0x0002_0000_0000_0000; // 010 << 48
const TAG_NULL: u64    = 0x0003_0000_0000_0000; // 011 << 48
const TAG_UNDEF: u64   = 0x0004_0000_0000_0000; // 100 << 48
const TAG_OBJ: u64     = 0x0005_0000_0000_0000; // 101 << 48
const TAG_STR: u64     = 0x0006_0000_0000_0000; // 110 << 48
const TAG_MASK: u64    = 0x0007_0000_0000_0000;
const PAYLOAD_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

/// Object handle — index into the object arena.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ObjId(pub u32);

/// String handle — index into the string intern table.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct StringId(pub u32);

impl ObjId {
    pub const NULL: ObjId = ObjId(u32::MAX);
    pub fn is_null(self) -> bool { self.0 == u32::MAX }
}

impl StringId {
    pub const EMPTY: StringId = StringId(0);
}

impl JsValue {
    // ─── Constructors ───

    pub const UNDEFINED: JsValue = JsValue(QNAN | TAG_UNDEF);
    pub const NULL: JsValue = JsValue(QNAN | TAG_NULL);
    pub const TRUE: JsValue = JsValue(QNAN | TAG_BOOL | 1);
    pub const FALSE: JsValue = JsValue(QNAN | TAG_BOOL);

    #[inline]
    pub fn from_f64(v: f64) -> Self {
        JsValue(v.to_bits())
    }

    #[inline]
    pub fn from_i32(v: i32) -> Self {
        JsValue(QNAN | TAG_INT | (v as u32 as u64))
    }

    #[inline]
    pub fn from_bool(v: bool) -> Self {
        JsValue(QNAN | TAG_BOOL | v as u64)
    }

    #[inline]
    pub fn from_obj(id: ObjId) -> Self {
        JsValue(QNAN | TAG_OBJ | id.0 as u64)
    }

    #[inline]
    pub fn from_str(id: StringId) -> Self {
        JsValue(QNAN | TAG_STR | id.0 as u64)
    }

    // ─── Type checks ───

    #[inline]
    fn tag(self) -> u64 {
        if self.is_f64() { 0 } else { self.0 & TAG_MASK }
    }

    #[inline]
    pub fn is_f64(self) -> bool {
        // A value is a float if it's NOT in the NaN-box space
        // (either not a NaN, or the exact canonical NaN)
        (self.0 & QNAN) != QNAN || self.0 == f64::NAN.to_bits()
    }

    #[inline]
    pub fn is_i32(self) -> bool {
        (self.0 & (QNAN | TAG_MASK)) == (QNAN | TAG_INT)
    }

    #[inline]
    pub fn is_bool(self) -> bool {
        (self.0 & (QNAN | TAG_MASK)) == (QNAN | TAG_BOOL)
    }

    #[inline]
    pub fn is_null(self) -> bool {
        self.0 == (QNAN | TAG_NULL)
    }

    #[inline]
    pub fn is_undefined(self) -> bool {
        self.0 == (QNAN | TAG_UNDEF)
    }

    #[inline]
    pub fn is_object(self) -> bool {
        (self.0 & (QNAN | TAG_MASK)) == (QNAN | TAG_OBJ)
    }

    #[inline]
    pub fn is_string(self) -> bool {
        (self.0 & (QNAN | TAG_MASK)) == (QNAN | TAG_STR)
    }

    #[inline]
    pub fn is_number(self) -> bool {
        self.is_f64() || self.is_i32()
    }

    #[inline]
    pub fn is_nullish(self) -> bool {
        self.is_null() || self.is_undefined()
    }

    // ─── Extractors ───

    #[inline]
    pub fn as_f64(self) -> f64 {
        if self.is_i32() {
            self.as_i32() as f64
        } else {
            f64::from_bits(self.0)
        }
    }

    #[inline]
    pub fn as_i32(self) -> i32 {
        (self.0 & 0xFFFF_FFFF) as u32 as i32
    }

    #[inline]
    pub fn as_bool(self) -> bool {
        (self.0 & 1) != 0
    }

    #[inline]
    pub fn as_obj(self) -> ObjId {
        ObjId((self.0 & 0xFFFF_FFFF) as u32)
    }

    #[inline]
    pub fn as_str_id(self) -> StringId {
        StringId((self.0 & 0xFFFF_FFFF) as u32)
    }

    // ─── Conversions (ECMAScript spec) ───

    /// ToNumber: convert any value to a number.
    pub fn to_number(self) -> f64 {
        if self.is_f64() { return f64::from_bits(self.0); }
        if self.is_i32() { return self.as_i32() as f64; }
        if self.is_bool() { return if self.as_bool() { 1.0 } else { 0.0 }; }
        if self.is_null() { return 0.0; }
        // undefined, object, string → NaN (simplified)
        f64::NAN
    }

    /// ToBoolean: the "truthiness" check.
    #[inline]
    pub fn is_truthy(self) -> bool {
        if self.is_bool() { return self.as_bool(); }
        if self.is_null() || self.is_undefined() { return false; }
        if self.is_i32() { return self.as_i32() != 0; }
        if self.is_f64() {
            let f = f64::from_bits(self.0);
            return f != 0.0 && !f.is_nan();
        }
        if self.is_string() { return self.as_str_id().0 != StringId::EMPTY.0; }
        // Objects are always truthy
        true
    }

    /// ToInt32: used for bitwise operations.
    pub fn to_i32(self) -> i32 {
        let n = self.to_number();
        if n.is_nan() || n.is_infinite() || n == 0.0 { return 0; }
        // Truncate and wrap to i32
        n as i64 as i32
    }

    /// ToUint32: used for array indices, unsigned shifts.
    pub fn to_u32(self) -> u32 {
        self.to_i32() as u32
    }

    // ─── Arithmetic helpers ───

    /// Add two values (handles string concatenation in the VM).
    pub fn add_num(self, other: JsValue) -> JsValue {
        // Fast path: both integers
        if self.is_i32() && other.is_i32() {
            let (result, overflow) = self.as_i32().overflowing_add(other.as_i32());
            if !overflow {
                return JsValue::from_i32(result);
            }
        }
        JsValue::from_f64(self.to_number() + other.to_number())
    }

    pub fn sub(self, other: JsValue) -> JsValue {
        if self.is_i32() && other.is_i32() {
            let (result, overflow) = self.as_i32().overflowing_sub(other.as_i32());
            if !overflow {
                return JsValue::from_i32(result);
            }
        }
        JsValue::from_f64(self.to_number() - other.to_number())
    }

    pub fn mul(self, other: JsValue) -> JsValue {
        if self.is_i32() && other.is_i32() {
            let a = self.as_i32() as i64;
            let b = other.as_i32() as i64;
            let r = a * b;
            if r >= i32::MIN as i64 && r <= i32::MAX as i64 {
                return JsValue::from_i32(r as i32);
            }
        }
        JsValue::from_f64(self.to_number() * other.to_number())
    }

    pub fn div(self, other: JsValue) -> JsValue {
        JsValue::from_f64(self.to_number() / other.to_number())
    }

    pub fn rem(self, other: JsValue) -> JsValue {
        let a = self.to_number();
        let b = other.to_number();
        if b == 0.0 { return JsValue::from_f64(f64::NAN); }
        JsValue::from_f64(a % b)
    }

    pub fn neg(self) -> JsValue {
        if self.is_i32() && self.as_i32() != i32::MIN && self.as_i32() != 0 {
            return JsValue::from_i32(-self.as_i32());
        }
        JsValue::from_f64(-self.to_number())
    }

    // ─── Comparison helpers ───

    /// Abstract equality (==) — simplified, no object ToPrimitive.
    pub fn abstract_eq(self, other: JsValue) -> bool {
        // Same type → strict equal
        if self.same_type(other) {
            return self.strict_eq(other);
        }
        // null == undefined (and vice versa)
        if (self.is_null() && other.is_undefined()) ||
           (self.is_undefined() && other.is_null()) {
            return true;
        }
        // Number comparisons
        if self.is_number() && other.is_number() {
            return self.to_number() == other.to_number();
        }
        false
    }

    /// Strict equality (===).
    pub fn strict_eq(self, other: JsValue) -> bool {
        if self.0 == other.0 { return true; }
        // NaN !== NaN
        if self.is_f64() && other.is_f64() {
            let a = f64::from_bits(self.0);
            let b = f64::from_bits(other.0);
            return a == b;
        }
        false
    }

    /// Less than (<).
    pub fn less_than(self, other: JsValue) -> bool {
        self.to_number() < other.to_number()
    }

    fn same_type(self, other: JsValue) -> bool {
        if self.is_f64() && other.is_f64() { return true; }
        if self.is_i32() && other.is_i32() { return true; }
        self.tag() == other.tag()
    }

    // ─── Type name ───

    /// Returns the typeof string for this value.
    pub fn type_name(self) -> &'static str {
        if self.is_undefined() { "undefined" }
        else if self.is_null() { "object" } // yes, typeof null === "object"
        else if self.is_bool() { "boolean" }
        else if self.is_number() { "number" }
        else if self.is_string() { "string" }
        else if self.is_object() { "object" } // TODO: "function" for callable objects
        else { "undefined" }
    }

    /// Raw bits for debugging.
    pub fn raw(self) -> u64 { self.0 }
}

// ─── Display helpers (no std::fmt, so manual) ───

impl JsValue {
    /// Write this value as a string into a buffer. Returns bytes written.
    pub fn write_to(self, buf: &mut [u8], strings: &super::strings::StringTable) -> usize {
        if self.is_undefined() {
            let s = b"undefined";
            let n = s.len().min(buf.len());
            buf[..n].copy_from_slice(&s[..n]);
            return n;
        }
        if self.is_null() {
            let s = b"null";
            let n = s.len().min(buf.len());
            buf[..n].copy_from_slice(&s[..n]);
            return n;
        }
        if self.is_bool() {
            let s = if self.as_bool() { &b"true"[..] } else { &b"false"[..] };
            let n = s.len().min(buf.len());
            buf[..n].copy_from_slice(&s[..n]);
            return n;
        }
        if self.is_i32() {
            return write_i32(buf, self.as_i32());
        }
        if self.is_f64() {
            return write_f64(buf, f64::from_bits(self.0));
        }
        if self.is_string() {
            let s = strings.get(self.as_str_id());
            let n = s.len().min(buf.len());
            buf[..n].copy_from_slice(&s[..n]);
            return n;
        }
        if self.is_object() {
            let s = b"[object Object]";
            let n = s.len().min(buf.len());
            buf[..n].copy_from_slice(&s[..n]);
            return n;
        }
        0
    }
}

// ─── Number formatting (no_std) ───

fn write_i32(buf: &mut [u8], mut v: i32) -> usize {
    if buf.is_empty() { return 0; }
    let mut tmp = [0u8; 12];
    let mut i = 0;
    let neg = v < 0;
    if neg { v = -v; }
    if v == 0 {
        tmp[0] = b'0';
        i = 1;
    } else {
        while v > 0 && i < 11 {
            tmp[i] = b'0' + (v % 10) as u8;
            v /= 10;
            i += 1;
        }
    }
    let total = if neg { i + 1 } else { i };
    if total > buf.len() { return 0; }
    let mut w = 0;
    if neg { buf[w] = b'-'; w += 1; }
    for j in (0..i).rev() {
        buf[w] = tmp[j];
        w += 1;
    }
    w
}

fn write_f64(buf: &mut [u8], v: f64) -> usize {
    if buf.len() < 4 { return 0; }
    if v.is_nan() {
        buf[..3].copy_from_slice(b"NaN");
        return 3;
    }
    if v.is_infinite() {
        if v < 0.0 {
            let s = b"-Infinity";
            let n = s.len().min(buf.len());
            buf[..n].copy_from_slice(&s[..n]);
            return n;
        }
        let s = b"Infinity";
        let n = s.len().min(buf.len());
        buf[..n].copy_from_slice(&s[..n]);
        return n;
    }
    // Simple float formatting: integer part + up to 6 decimal places
    let neg = v < 0.0;
    let v = if neg { -v } else { v };
    let int_part = v as i64;
    let frac_part = ((v - int_part as f64) * 1_000_000.0 + 0.5) as u64;

    let mut w = 0;
    if neg { buf[w] = b'-'; w += 1; }
    w += write_i32(&mut buf[w..], int_part as i32);

    if frac_part > 0 && w < buf.len() - 8 {
        buf[w] = b'.';
        w += 1;
        // Write 6 digits, then trim trailing zeros
        let mut digits = [0u8; 6];
        let mut f = frac_part;
        for d in (0..6).rev() {
            digits[d] = b'0' + (f % 10) as u8;
            f /= 10;
        }
        let mut last_nonzero = 0;
        for d in 0..6 {
            if digits[d] != b'0' { last_nonzero = d; }
        }
        for d in 0..=last_nonzero {
            if w < buf.len() {
                buf[w] = digits[d];
                w += 1;
            }
        }
    }
    w
}

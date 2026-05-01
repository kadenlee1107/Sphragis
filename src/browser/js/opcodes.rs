#![allow(dead_code)]
// Bat_OS — Bytecode Instruction Set
// Stack-based VM with ~80 opcodes. Variable-length encoding:
//   1-byte opcode + 0-4 bytes operand.

// ─── Constants and Literals ───
pub const OP_CONST_I32: u8     = 0x01; // + i32: push integer
pub const OP_CONST_F64: u8     = 0x02; // + u16: push f64 from constant pool[idx]
pub const OP_CONST_STR: u8     = 0x03; // + u16: push string from constant pool[idx]
pub const OP_TRUE: u8          = 0x04; // push true
pub const OP_FALSE: u8         = 0x05; // push false
pub const OP_NULL: u8          = 0x06; // push null
pub const OP_UNDEFINED: u8     = 0x07; // push undefined
pub const OP_CONST_ZERO: u8    = 0x08; // push integer 0 (frequent, saves 4 bytes)
pub const OP_CONST_ONE: u8     = 0x09; // push integer 1

// ─── Stack Manipulation ───
pub const OP_POP: u8           = 0x0A; // discard top
pub const OP_DUP: u8           = 0x0B; // duplicate top
pub const OP_SWAP: u8          = 0x0C; // swap top two

// ─── Arithmetic (pop 2, push 1) ───
pub const OP_ADD: u8           = 0x10; // a + b (handles string concat)
pub const OP_SUB: u8           = 0x11; // a - b
pub const OP_MUL: u8           = 0x12; // a * b
pub const OP_DIV: u8           = 0x13; // a / b
pub const OP_MOD: u8           = 0x14; // a % b
pub const OP_NEG: u8           = 0x15; // -a (unary)
pub const OP_POW: u8           = 0x16; // a ** b

// ─── Bitwise ───
pub const OP_BIT_AND: u8       = 0x18; // a & b
pub const OP_BIT_OR: u8        = 0x19; // a | b
pub const OP_BIT_XOR: u8       = 0x1A; // a ^ b
pub const OP_BIT_NOT: u8       = 0x1B; // ~a
pub const OP_SHL: u8           = 0x1C; // a << b
pub const OP_SHR: u8           = 0x1D; // a >> b (signed)
pub const OP_USHR: u8          = 0x1E; // a >>> b (unsigned)

// ─── Increment / Decrement (top of stack) ───
pub const OP_INC: u8           = 0x1F; // ++
pub const OP_DEC: u8           = 0x20; // --

// ─── Comparison (pop 2, push bool) ───
pub const OP_EQ: u8            = 0x21; // ==
pub const OP_SEQ: u8           = 0x22; // ===
pub const OP_NE: u8            = 0x23; // !=
pub const OP_SNE: u8           = 0x24; // !==
pub const OP_LT: u8            = 0x25; // <
pub const OP_LE: u8            = 0x26; // <=
pub const OP_GT: u8            = 0x27; // >
pub const OP_GE: u8            = 0x28; // >=
pub const OP_INSTANCEOF: u8    = 0x29;
pub const OP_IN: u8            = 0x2A;

// ─── Logical ───
pub const OP_NOT: u8           = 0x30; // !a
pub const OP_TYPEOF: u8        = 0x31; // typeof a → string

// ─── Variable Access ───
pub const OP_GET_LOCAL: u8     = 0x40; // + u16 slot: push local
pub const OP_SET_LOCAL: u8     = 0x41; // + u16 slot: pop → local
pub const OP_GET_GLOBAL: u8    = 0x42; // + u16 name_idx: push global by name
pub const OP_SET_GLOBAL: u8    = 0x43; // + u16 name_idx: pop → global by name
pub const OP_GET_CAPTURE: u8   = 0x44; // + u8 depth + u8 slot: upvalue get
pub const OP_SET_CAPTURE: u8   = 0x45; // + u8 depth + u8 slot: upvalue set

// ─── Control Flow ───
pub const OP_JUMP: u8          = 0x50; // + i16: unconditional relative jump
pub const OP_JUMP_FALSE: u8    = 0x51; // + i16: pop, jump if falsy
pub const OP_JUMP_TRUE: u8     = 0x52; // + i16: pop, jump if truthy
pub const OP_LOOP: u8          = 0x53; // + i16: jump backward (with interrupt check)

// ─── Functions and Calls ───
pub const OP_CALL: u8          = 0x60; // + u8 argc: call function
pub const OP_RETURN: u8        = 0x61; // return top of stack
pub const OP_CLOSURE: u8       = 0x62; // + u16 proto_idx: create closure
pub const OP_NEW: u8           = 0x63; // + u8 argc: new Constructor(args)
// STUMP #93: method call — stack on entry is
//     [func, this, arg0, arg1, ...]   argc = number of REAL args (no this)
// The runtime binds `this = stack[func_pos+1]` and passes
// `args_start = func_pos + 2` to native callees, fixing the
// console.log("x") prints "[object Object] x" bug where the
// receiver leaked into arg[0].
pub const OP_METHOD_CALL: u8   = 0x64;

// ─── Objects and Properties ───
pub const OP_GET_PROP: u8      = 0x70; // + u16 name_idx: obj.prop
pub const OP_SET_PROP: u8      = 0x71; // + u16 name_idx: obj.prop = val
pub const OP_GET_ELEM: u8      = 0x72; // obj[key] (both on stack)
pub const OP_SET_ELEM: u8      = 0x73; // obj[key] = val
pub const OP_NEW_OBJECT: u8    = 0x74; // push empty {}
pub const OP_NEW_ARRAY: u8     = 0x75; // + u16 count: push [el0, el1, ...]
pub const OP_DEFINE_PROP: u8   = 0x76; // + u16 name_idx: obj.name = stack.pop (during literal init)
pub const OP_DELETE_PROP: u8   = 0x77; // + u16 name_idx: delete obj.prop

// ─── This ───
pub const OP_THIS: u8          = 0x78; // push current `this`

// ─── Exception Handling ───
pub const OP_THROW: u8         = 0x80; // throw top of stack
pub const OP_TRY_START: u8     = 0x81; // + i16 catch_offset: push try frame
pub const OP_TRY_END: u8       = 0x82; // pop try frame
pub const OP_CATCH_BIND: u8    = 0x83; // + u16 slot: bind caught value to local

// ─── Iteration ───
pub const OP_FOR_IN_INIT: u8   = 0x88; // init for-in iterator
pub const OP_FOR_IN_NEXT: u8   = 0x89; // + i16 done_offset: next key or jump
pub const OP_FOR_OF_INIT: u8   = 0x8A; // init for-of iterator
pub const OP_FOR_OF_NEXT: u8   = 0x8B; // + i16 done_offset: next value or jump

// ─── Spread / Rest ───
pub const OP_SPREAD: u8        = 0x90; // spread iterable onto stack

// ─── Misc ───
pub const OP_DEBUGGER: u8      = 0xFE; // nop / breakpoint hook
pub const OP_HALT: u8          = 0xFF; // stop execution

// ─── Helpers for reading operands ───

#[inline]
pub fn read_u8(code: &[u8], ip: &mut usize) -> u8 {
    let v = code[*ip];
    *ip += 1;
    v
}

#[inline]
pub fn read_u16(code: &[u8], ip: &mut usize) -> u16 {
    let v = u16::from_le_bytes([code[*ip], code[*ip + 1]]);
    *ip += 2;
    v
}

#[inline]
pub fn read_i16(code: &[u8], ip: &mut usize) -> i16 {
    let v = i16::from_le_bytes([code[*ip], code[*ip + 1]]);
    *ip += 2;
    v
}

#[inline]
pub fn read_i32(code: &[u8], ip: &mut usize) -> i32 {
    let v = i32::from_le_bytes([code[*ip], code[*ip+1], code[*ip+2], code[*ip+3]]);
    *ip += 4;
    v
}

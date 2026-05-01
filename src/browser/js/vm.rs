#![allow(dead_code)]
#![allow(unused_assignments)]
// Bat_OS — Bytecode Virtual Machine
// Stack-based VM that executes compiled bytecode.
// Handles call frames, variable scoping, closures, upvalues, and object operations.

use super::value::{JsValue, ObjId, StringId};
use super::strings::StringTable;
use super::opcodes::*;
use super::compiler::{FunctionProto, Constant, MAX_PROTOS};
use super::object::{JsHeap, ObjFlags};
use super::dom_api::DomApi;

/// Maximum value stack depth.
const MAX_STACK: usize = 1024;
/// Maximum call frame depth.
const MAX_FRAMES: usize = 64;
/// Maximum try/catch nesting.
const MAX_TRY: usize = 32;
/// Maximum global variables.
const MAX_GLOBALS: usize = 256;
/// Interrupt check interval (loop iterations before yielding).
const INTERRUPT_INTERVAL: u32 = 100_000;
/// Maximum open upvalues.
const MAX_UPVALUES: usize = 256;

/// A call frame on the call stack.
#[derive(Clone, Copy)]
struct CallFrame {
    proto_idx: u16,    // index into vm.protos[]
    ip: usize,         // instruction pointer (offset into bytecode)
    stack_base: usize,  // base of this frame's stack window
    this_val: JsValue,  // `this` binding for this frame
    closure_obj: ObjId, // closure object (for upvalue access)
}

/// A try/catch frame.
#[derive(Clone, Copy)]
struct TryFrame {
    catch_ip: usize,       // IP to jump to on exception
    stack_base: usize,     // stack pointer to restore
    frame_idx: usize,      // call frame to unwind to
}

/// A global variable entry.
#[derive(Clone, Copy)]
struct GlobalEntry {
    name: StringId,
    value: JsValue,
}

/// The JavaScript Virtual Machine.
pub struct Vm {
    // Value stack
    pub stack: [JsValue; MAX_STACK],
    pub sp: usize,

    // Call stack
    frames: [CallFrame; MAX_FRAMES],
    frame_count: usize,

    // Try/catch stack
    try_stack: [TryFrame; MAX_TRY],
    try_count: usize,

    // Global variables
    globals: [GlobalEntry; MAX_GLOBALS],
    global_count: usize,

    // Compiled function prototypes
    pub protos: [FunctionProto; MAX_PROTOS],
    pub proto_count: usize,

    // String table
    pub strings: StringTable,

    // Object heap
    pub heap: JsHeap,

    // Interrupt counter (prevent infinite loops)
    loop_counter: u32,

    // Native function table
    natives: [Option<NativeFn>; MAX_NATIVES],
    native_count: usize,

    // Console output buffer
    pub console_buf: [u8; 4096],
    pub console_len: usize,

    // Built-in prototype objects (set by builtins::register_builtins)
    pub array_proto: ObjId,
    pub string_proto: ObjId,
    pub number_proto: ObjId,

    // DOM integration state
    pub dom: DomApi,
}

const MAX_NATIVES: usize = 128;

/// Type for native (built-in) functions.
/// Takes: &mut Vm, args_start (stack index), arg_count
/// Returns: result value or error
type NativeFn = fn(&mut Vm, usize, usize) -> Result<JsValue, JsError>;

/// VM error.
pub enum JsError {
    StackOverflow,
    StackUnderflow,
    InvalidOpcode(u8),
    TypeError(&'static str),
    ReferenceError,
    InternalError(&'static str),
    Thrown(JsValue),
    Interrupted,
}

impl Vm {
    pub const fn new() -> Self {
        Vm {
            stack: [JsValue::UNDEFINED; MAX_STACK],
            sp: 0,
            frames: [CallFrame { proto_idx: 0, ip: 0, stack_base: 0, this_val: JsValue::UNDEFINED, closure_obj: ObjId::NULL }; MAX_FRAMES],
            frame_count: 0,
            try_stack: [TryFrame { catch_ip: 0, stack_base: 0, frame_idx: 0 }; MAX_TRY],
            try_count: 0,
            globals: [GlobalEntry { name: StringId::EMPTY, value: JsValue::UNDEFINED }; MAX_GLOBALS],
            global_count: 0,
            protos: [FunctionProto::UNINIT; MAX_PROTOS],
            proto_count: 0,
            strings: StringTable::new(),
            heap: JsHeap::new(),
            loop_counter: 0,
            natives: [None; MAX_NATIVES],
            native_count: 0,
            console_buf: [0; 4096],
            console_len: 0,
            array_proto: ObjId::NULL,
            string_proto: ObjId::NULL,
            number_proto: ObjId::NULL,
            dom: DomApi::new(),
        }
    }

    /// Initialize the VM (call once at startup).
    pub fn init(&mut self) {
        self.strings.init_well_known();
        self.register_core_globals();
        self.register_native_functions();
        super::builtins::register_builtins(self);
        super::dom_api::register_dom_globals(self);
    }

    /// Compile and execute a JavaScript source string.
    pub fn execute(&mut self, source: &[u8]) -> Result<JsValue, JsError> {
        let mut ast = super::ast::Ast::new();
        let mut tokens = [super::lexer::Token::empty(); super::lexer::MAX_TOKENS];
        let token_count = super::lexer::tokenize(source, &mut tokens);
        super::parser::parse(&tokens[..token_count], &mut ast);

        if ast.count == 0 {
            return Ok(JsValue::UNDEFINED);
        }

        // Step 2: Compile AST → Bytecode
        let (main_proto, child_protos, child_count) = {
            let compiler = super::compiler::Compiler::new(&mut self.strings);
            compiler.compile_script(&ast)
        };

        // Step 3: Store child protos first
        for i in 0..child_count {
            if self.proto_count >= MAX_PROTOS {
                return Err(JsError::InternalError("too many functions"));
            }
            // Need to move child protos; use index arithmetic
            let dst = self.proto_count;
            // Copy bytecode and constants manually since FunctionProto isn't Copy
            self.protos[dst].name = child_protos[i].name;
            self.protos[dst].bytecode_len = child_protos[i].bytecode_len;
            let len = child_protos[i].bytecode_len;
            self.protos[dst].bytecode[..len].copy_from_slice(&child_protos[i].bytecode[..len]);
            self.protos[dst].const_count = child_protos[i].const_count;
            let cc = child_protos[i].const_count;
            for j in 0..cc {
                self.protos[dst].constants[j] = child_protos[i].constants[j];
            }
            self.protos[dst].local_count = child_protos[i].local_count;
            self.protos[dst].param_count = child_protos[i].param_count;
            self.protos[dst].upvalue_count = child_protos[i].upvalue_count;
            let uvc = child_protos[i].upvalue_count as usize;
            for j in 0..uvc {
                self.protos[dst].upvalues[j] = child_protos[i].upvalues[j];
            }
            self.proto_count += 1;
        }

        // Step 4: Store main proto
        if self.proto_count >= MAX_PROTOS {
            return Err(JsError::InternalError("too many functions"));
        }
        let proto_idx = self.proto_count;
        self.protos[proto_idx].name = main_proto.name;
        self.protos[proto_idx].bytecode_len = main_proto.bytecode_len;
        let len = main_proto.bytecode_len;
        self.protos[proto_idx].bytecode[..len].copy_from_slice(&main_proto.bytecode[..len]);
        self.protos[proto_idx].const_count = main_proto.const_count;
        let cc = main_proto.const_count;
        for j in 0..cc {
            self.protos[proto_idx].constants[j] = main_proto.constants[j];
        }
        self.protos[proto_idx].local_count = main_proto.local_count;
        self.protos[proto_idx].param_count = main_proto.param_count;
        self.protos[proto_idx].upvalue_count = main_proto.upvalue_count;
        self.proto_count += 1;

        // Step 5: Set up the initial call frame
        self.sp = 0;
        self.frame_count = 0;

        // Reserve local variable slots
        let local_count = self.protos[proto_idx].local_count as usize;
        for i in 0..local_count {
            self.stack[i] = JsValue::UNDEFINED;
        }
        self.sp = local_count;

        self.frames[0] = CallFrame {
            proto_idx: proto_idx as u16,
            ip: 0,
            stack_base: 0,
            this_val: JsValue::UNDEFINED,
            closure_obj: ObjId::NULL,
        };
        self.frame_count = 1;
        self.loop_counter = 0;

        // Step 6: Run!
        self.run()
    }

    /// The main execution loop.
    fn run(&mut self) -> Result<JsValue, JsError> {
        // 🎯 STUMP #84: hard instruction-count cap. The per-back-jump
        // INTERRUPT_INTERVAL only catches loops; forward bytecode that
        // gets stuck (e.g. a recursive call without progress) would
        // hang the kernel. 10M instructions is plenty for any
        // reasonable page-load script.
        let mut total_ops: u32 = 0;
        const MAX_TOTAL_OPS: u32 = 10_000_000;
        loop {
            total_ops += 1;
            if total_ops >= MAX_TOTAL_OPS {
                return Err(JsError::Interrupted);
            }
            if self.frame_count == 0 {
                return Ok(if self.sp > 0 { self.stack[self.sp - 1] } else { JsValue::UNDEFINED });
            }

            let frame = &self.frames[self.frame_count - 1];
            let pi = frame.proto_idx as usize;
            let ip = frame.ip;
            let base = frame.stack_base;

            if ip >= self.protos[pi].bytecode_len {
                // End of function — implicit return undefined
                self.frame_count -= 1;
                let result = JsValue::UNDEFINED;
                self.sp = base;
                if self.frame_count > 0 {
                    self.push(result)?;
                }
                continue;
            }

            let op = self.protos[pi].bytecode[ip];
            self.frames[self.frame_count - 1].ip = ip + 1;

            match op {
                // ─── Constants ───
                OP_CONST_I32 => {
                    let v = self.read_i32();
                    self.push(JsValue::from_i32(v))?;
                }
                OP_CONST_F64 => {
                    let idx = self.read_u16();
                    let pi = self.frames[self.frame_count - 1].proto_idx as usize;
                    if let Constant::F64(f) = self.protos[pi].constants[idx as usize] {
                        self.push(JsValue::from_f64(f))?;
                    } else {
                        self.push(JsValue::UNDEFINED)?;
                    }
                }
                OP_CONST_STR => {
                    let idx = self.read_u16();
                    let pi = self.frames[self.frame_count - 1].proto_idx as usize;
                    if let Constant::Str(sid) = self.protos[pi].constants[idx as usize] {
                        self.push(JsValue::from_str(sid))?;
                    } else {
                        self.push(JsValue::from_str(StringId::EMPTY))?;
                    }
                }
                OP_TRUE => self.push(JsValue::TRUE)?,
                OP_FALSE => self.push(JsValue::FALSE)?,
                OP_NULL => self.push(JsValue::NULL)?,
                OP_UNDEFINED => self.push(JsValue::UNDEFINED)?,
                OP_CONST_ZERO => self.push(JsValue::from_i32(0))?,
                OP_CONST_ONE => self.push(JsValue::from_i32(1))?,

                // ─── Stack ───
                OP_POP => { self.pop()?; }
                OP_DUP => {
                    let v = self.peek()?;
                    self.push(v)?;
                }
                OP_SWAP => {
                    if self.sp < 2 { return Err(JsError::StackUnderflow); }
                    self.stack.swap(self.sp - 1, self.sp - 2);
                }

                // ─── Arithmetic ───
                OP_ADD => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    // String concatenation check
                    if a.is_string() || b.is_string() {
                        let a_sid = if a.is_string() { a.as_str_id() } else {
                            let mut buf = [0u8; 64];
                            let n = a.write_to(&mut buf, &self.strings);
                            self.strings.intern(&buf[..n])
                        };
                        let b_sid = if b.is_string() { b.as_str_id() } else {
                            let mut buf = [0u8; 64];
                            let n = b.write_to(&mut buf, &self.strings);
                            self.strings.intern(&buf[..n])
                        };
                        let result = self.strings.concat(a_sid, b_sid);
                        self.push(JsValue::from_str(result))?;
                    } else {
                        self.push(a.add_num(b))?;
                    }
                }
                OP_SUB => { let b = self.pop()?; let a = self.pop()?; self.push(a.sub(b))?; }
                OP_MUL => { let b = self.pop()?; let a = self.pop()?; self.push(a.mul(b))?; }
                OP_DIV => { let b = self.pop()?; let a = self.pop()?; self.push(a.div(b))?; }
                OP_MOD => { let b = self.pop()?; let a = self.pop()?; self.push(a.rem(b))?; }
                OP_NEG => { let a = self.pop()?; self.push(a.neg())?; }

                // ─── Bitwise ───
                OP_BIT_AND => { let b = self.pop()?; let a = self.pop()?; self.push(JsValue::from_i32(a.to_i32() & b.to_i32()))?; }
                OP_BIT_OR  => { let b = self.pop()?; let a = self.pop()?; self.push(JsValue::from_i32(a.to_i32() | b.to_i32()))?; }
                OP_BIT_XOR => { let b = self.pop()?; let a = self.pop()?; self.push(JsValue::from_i32(a.to_i32() ^ b.to_i32()))?; }
                OP_BIT_NOT => { let a = self.pop()?; self.push(JsValue::from_i32(!a.to_i32()))?; }
                OP_SHL  => { let b = self.pop()?; let a = self.pop()?; self.push(JsValue::from_i32(a.to_i32() << (b.to_u32() & 31)))?; }
                OP_SHR  => { let b = self.pop()?; let a = self.pop()?; self.push(JsValue::from_i32(a.to_i32() >> (b.to_u32() & 31)))?; }
                OP_USHR => { let b = self.pop()?; let a = self.pop()?; self.push(JsValue::from_i32((a.to_u32() >> (b.to_u32() & 31)) as i32))?; }

                OP_INC => { let a = self.pop()?; self.push(a.add_num(JsValue::from_i32(1)))?; }
                OP_DEC => { let a = self.pop()?; self.push(a.sub(JsValue::from_i32(1)))?; }

                // ─── Comparison ───
                OP_EQ  => { let b = self.pop()?; let a = self.pop()?; self.push(JsValue::from_bool(a.abstract_eq(b)))?; }
                OP_SEQ => { let b = self.pop()?; let a = self.pop()?; self.push(JsValue::from_bool(a.strict_eq(b)))?; }
                OP_NE  => { let b = self.pop()?; let a = self.pop()?; self.push(JsValue::from_bool(!a.abstract_eq(b)))?; }
                OP_SNE => { let b = self.pop()?; let a = self.pop()?; self.push(JsValue::from_bool(!a.strict_eq(b)))?; }
                OP_LT  => { let b = self.pop()?; let a = self.pop()?; self.push(JsValue::from_bool(a.less_than(b)))?; }
                OP_LE  => { let b = self.pop()?; let a = self.pop()?; self.push(JsValue::from_bool(!b.less_than(a)))?; }
                OP_GT  => { let b = self.pop()?; let a = self.pop()?; self.push(JsValue::from_bool(b.less_than(a)))?; }
                OP_GE  => { let b = self.pop()?; let a = self.pop()?; self.push(JsValue::from_bool(!a.less_than(b)))?; }

                // ─── Logical ───
                OP_NOT => {
                    let a = self.pop()?;
                    self.push(JsValue::from_bool(!a.is_truthy()))?;
                }
                OP_TYPEOF => {
                    let a = self.pop()?;
                    let name = if a.is_object() {
                        let flags = self.heap.get_flags(a.as_obj());
                        if (flags & ObjFlags::FUNCTION != 0) || (flags & ObjFlags::NATIVE != 0) {
                            "function"
                        } else {
                            a.type_name()
                        }
                    } else {
                        a.type_name()
                    };
                    let sid = self.strings.intern(name.as_bytes());
                    self.push(JsValue::from_str(sid))?;
                }

                // ─── Variables ───
                OP_GET_LOCAL => {
                    let slot = self.read_u16() as usize;
                    let base = self.frames[self.frame_count - 1].stack_base;
                    let v = self.stack[base + slot];
                    self.push(v)?;
                }
                OP_SET_LOCAL => {
                    let slot = self.read_u16() as usize;
                    let base = self.frames[self.frame_count - 1].stack_base;
                    let v = self.peek()?;
                    self.stack[base + slot] = v;
                }
                OP_GET_GLOBAL => {
                    let name_idx = self.read_u16();
                    let pi = self.frames[self.frame_count - 1].proto_idx as usize;
                    if let Constant::Str(sid) = self.protos[pi].constants[name_idx as usize] {
                        let val = self.get_global(sid);
                        self.push(val)?;
                    } else {
                        self.push(JsValue::UNDEFINED)?;
                    }
                }
                OP_SET_GLOBAL => {
                    let name_idx = self.read_u16();
                    let pi = self.frames[self.frame_count - 1].proto_idx as usize;
                    if let Constant::Str(sid) = self.protos[pi].constants[name_idx as usize] {
                        let val = self.peek()?;
                        self.set_global(sid, val);
                    }
                }
                OP_GET_CAPTURE => {
                    let uv_idx = self.read_u8();
                    let _reserved = self.read_u8();
                    // Get upvalue from closure object
                    let closure = self.frames[self.frame_count - 1].closure_obj;
                    if !closure.is_null() {
                        let _uv_name = self.strings.intern(b"__upv__");
                        // Upvalues stored as properties on closure: __uv0, __uv1, etc.
                        let mut name_buf = [0u8; 8];
                        name_buf[0] = b'_'; name_buf[1] = b'_'; name_buf[2] = b'u';
                        name_buf[3] = b'v'; name_buf[4] = b'0' + uv_idx;
                        let uv_prop_name = self.strings.intern(&name_buf[..5]);
                        let val = self.heap.get_prop(closure, uv_prop_name);
                        self.push(val)?;
                    } else {
                        self.push(JsValue::UNDEFINED)?;
                    }
                }
                OP_SET_CAPTURE => {
                    let uv_idx = self.read_u8();
                    let _reserved = self.read_u8();
                    let val = self.peek()?;
                    let closure = self.frames[self.frame_count - 1].closure_obj;
                    if !closure.is_null() {
                        let mut name_buf = [0u8; 8];
                        name_buf[0] = b'_'; name_buf[1] = b'_'; name_buf[2] = b'u';
                        name_buf[3] = b'v'; name_buf[4] = b'0' + uv_idx;
                        let uv_prop_name = self.strings.intern(&name_buf[..5]);
                        self.heap.set_prop(closure, uv_prop_name, val);
                    }
                }

                // ─── Control Flow ───
                OP_JUMP => {
                    let offset = self.read_i16();
                    let frame = &mut self.frames[self.frame_count - 1];
                    frame.ip = (frame.ip as isize + offset as isize) as usize;
                }
                OP_JUMP_FALSE => {
                    let offset = self.read_i16();
                    let cond = self.pop()?;
                    if !cond.is_truthy() {
                        let frame = &mut self.frames[self.frame_count - 1];
                        frame.ip = (frame.ip as isize + offset as isize) as usize;
                    }
                }
                OP_JUMP_TRUE => {
                    let offset = self.read_i16();
                    let cond = self.pop()?;
                    if cond.is_truthy() {
                        let frame = &mut self.frames[self.frame_count - 1];
                        frame.ip = (frame.ip as isize + offset as isize) as usize;
                    }
                }
                OP_LOOP => {
                    let offset = self.read_i16();
                    let frame = &mut self.frames[self.frame_count - 1];
                    frame.ip = (frame.ip as isize + offset as isize) as usize;
                    // Interrupt check
                    self.loop_counter += 1;
                    if self.loop_counter > INTERRUPT_INTERVAL {
                        self.loop_counter = 0;
                        return Err(JsError::Interrupted);
                    }
                }

                // ─── Functions ───
                OP_CALL => {
                    let argc = self.read_u8() as usize;
                    self.call_function(argc)?;
                }
                OP_METHOD_CALL => {
                    let argc = self.read_u8() as usize;
                    // Stack: [func, this, arg0, ..., arg(argc-1)]
                    self.call_method(argc)?;
                }
                OP_RETURN => {
                    let result = self.pop()?;
                    self.frame_count -= 1;
                    if self.frame_count == 0 {
                        return Ok(result);
                    }
                    let base = self.frames[self.frame_count].stack_base;
                    self.sp = base;
                    self.push(result)?;
                }
                OP_CLOSURE => {
                    let proto_idx = self.read_u16();
                    // Create a closure object that references the function proto
                    let closure_id = self.heap.alloc_closure(proto_idx);
                    // Capture upvalues from current frame
                    let uvc = self.protos[proto_idx as usize].upvalue_count;
                    for i in 0..uvc as usize {
                        let desc = self.protos[proto_idx as usize].upvalues[i];
                        let val = if desc.is_local {
                            // Capture from current frame's locals
                            let base = self.frames[self.frame_count - 1].stack_base;
                            self.stack[base + desc.index as usize]
                        } else {
                            // Capture from enclosing closure's upvalues
                            let enc_closure = self.frames[self.frame_count - 1].closure_obj;
                            if !enc_closure.is_null() {
                                let mut name_buf = [0u8; 8];
                                name_buf[0] = b'_'; name_buf[1] = b'_'; name_buf[2] = b'u';
                                name_buf[3] = b'v'; name_buf[4] = b'0' + desc.index;
                                let uv_prop_name = self.strings.intern(&name_buf[..5]);
                                self.heap.get_prop(enc_closure, uv_prop_name)
                            } else {
                                JsValue::UNDEFINED
                            }
                        };
                        // Store on closure object
                        let mut name_buf = [0u8; 8];
                        name_buf[0] = b'_'; name_buf[1] = b'_'; name_buf[2] = b'u';
                        name_buf[3] = b'v'; name_buf[4] = b'0' + i as u8;
                        let uv_prop_name = self.strings.intern(&name_buf[..5]);
                        self.heap.set_prop(closure_id, uv_prop_name, val);
                    }
                    self.push(JsValue::from_obj(closure_id))?;
                }
                OP_NEW => {
                    let argc = self.read_u8() as usize;
                    // Create new object, call constructor
                    let func_pos = self.sp - argc - 1;
                    let func_val = self.stack[func_pos];
                    let new_obj = self.heap.alloc_object();

                    if func_val.is_object() {
                        let obj = func_val.as_obj();
                        let flags = self.heap.get_flags(obj);

                        if flags & ObjFlags::NATIVE != 0 {
                            let native_idx = self.heap.get_native_idx(obj);
                            if let Some(native_fn) = self.natives[native_idx as usize] {
                                let args_start = func_pos + 1;
                                let result = native_fn(self, args_start, argc)?;
                                self.sp = func_pos;
                                self.push(result)?;
                                return Ok(JsValue::UNDEFINED); // continue execution
                            }
                        }
                        if flags & ObjFlags::FUNCTION != 0 {
                            let proto_idx = self.heap.get_func_proto_idx(obj);
                            // Set up new frame with 'this' = new_obj
                            let new_base = func_pos;
                            self.sp = func_pos;

                            let local_count = self.protos[proto_idx as usize].local_count as usize;
                            let param_count = self.protos[proto_idx as usize].param_count as usize;

                            // Copy args to local param slots, fill rest with undefined
                            for i in 0..local_count {
                                if i < argc && i < param_count {
                                    self.stack[new_base + i] = self.stack[func_pos + 1 + i];
                                } else {
                                    self.stack[new_base + i] = JsValue::UNDEFINED;
                                }
                            }
                            self.sp = new_base + local_count;

                            if self.frame_count >= MAX_FRAMES {
                                return Err(JsError::StackOverflow);
                            }
                            self.frames[self.frame_count] = CallFrame {
                                proto_idx: proto_idx,
                                ip: 0,
                                stack_base: new_base,
                                this_val: JsValue::from_obj(new_obj),
                                closure_obj: obj,
                            };
                            self.frame_count += 1;
                            continue;
                        }
                    }

                    self.sp = func_pos;
                    self.push(JsValue::from_obj(new_obj))?;
                }

                // ─── Objects ───
                OP_NEW_OBJECT => {
                    let obj_id = self.heap.alloc_object();
                    self.push(JsValue::from_obj(obj_id))?;
                }
                OP_NEW_ARRAY => {
                    let count = self.read_u16() as usize;
                    let obj_id = self.heap.alloc_array(count as u32);
                    // Set prototype for method dispatch
                    if !self.array_proto.is_null() {
                        self.heap.set_prototype(obj_id, self.array_proto);
                    }
                    // Pop elements from stack and store in array
                    let start = self.sp - count;
                    for i in 0..count {
                        self.heap.array_set(obj_id, i as u32, self.stack[start + i]);
                    }
                    self.sp = start;
                    self.push(JsValue::from_obj(obj_id))?;
                }
                OP_GET_PROP => {
                    let name_idx = self.read_u16();
                    let obj = self.pop()?;
                    let pi = self.frames[self.frame_count - 1].proto_idx as usize;
                    if let Constant::Str(name) = self.protos[pi].constants[name_idx as usize] {
                        let val = self.get_property(obj, name);
                        self.push(val)?;
                    } else {
                        self.push(JsValue::UNDEFINED)?;
                    }
                }
                OP_SET_PROP => {
                    let name_idx = self.read_u16();
                    let val = self.pop()?;
                    let obj = self.pop()?;
                    let pi = self.frames[self.frame_count - 1].proto_idx as usize;
                    if let Constant::Str(name) = self.protos[pi].constants[name_idx as usize] {
                        if obj.is_object() {
                            self.heap.set_prop(obj.as_obj(), name, val);
                        }
                    }
                    self.push(val)?; // assignment returns the value
                }
                OP_DEFINE_PROP => {
                    let name_idx = self.read_u16();
                    let val = self.pop()?;
                    let obj = self.peek()?; // keep object on stack
                    let pi = self.frames[self.frame_count - 1].proto_idx as usize;
                    if let Constant::Str(name) = self.protos[pi].constants[name_idx as usize] {
                        if obj.is_object() {
                            self.heap.set_prop(obj.as_obj(), name, val);
                        }
                    }
                }
                OP_GET_ELEM => {
                    let key = self.pop()?;
                    let obj = self.pop()?;
                    if obj.is_object() {
                        if key.is_i32() {
                            let val = self.heap.array_get(obj.as_obj(), key.as_i32() as u32);
                            self.push(val)?;
                        } else if key.is_string() {
                            let val = self.heap.get_prop(obj.as_obj(), key.as_str_id());
                            self.push(val)?;
                        } else {
                            self.push(JsValue::UNDEFINED)?;
                        }
                    } else if obj.is_string() && key.is_i32() {
                        // String character access
                        let sid = obj.as_str_id();
                        let s = self.strings.get(sid);
                        let idx = key.as_i32();
                        if idx >= 0 && (idx as usize) < s.len() {
                            let ch = s[idx as usize];
                            let ch_sid = self.strings.intern(&[ch]);
                            self.push(JsValue::from_str(ch_sid))?;
                        } else {
                            self.push(JsValue::UNDEFINED)?;
                        }
                    } else {
                        self.push(JsValue::UNDEFINED)?;
                    }
                }
                OP_SET_ELEM => {
                    let val = self.pop()?;
                    let key = self.pop()?;
                    let obj = self.pop()?;
                    if obj.is_object() {
                        if key.is_i32() {
                            self.heap.array_set(obj.as_obj(), key.as_i32() as u32, val);
                        } else if key.is_string() {
                            self.heap.set_prop(obj.as_obj(), key.as_str_id(), val);
                        }
                    }
                    self.push(val)?;
                }
                OP_DELETE_PROP => {
                    let _name_idx = self.read_u16();
                    // Simplified: just push true
                    let _obj = self.pop()?;
                    self.push(JsValue::TRUE)?;
                }

                // ─── This ───
                OP_THIS => {
                    let this_val = self.frames[self.frame_count - 1].this_val;
                    self.push(this_val)?;
                }

                // ─── Exception Handling ───
                OP_THROW => {
                    let val = self.pop()?;
                    match self.handle_throw(val) {
                        Ok(_) => { continue; }
                        Err(e) => { return Err(e); }
                    }
                }
                OP_TRY_START => {
                    let offset = self.read_i16();
                    let catch_ip = (self.frames[self.frame_count - 1].ip as isize + offset as isize) as usize;
                    if self.try_count < MAX_TRY {
                        self.try_stack[self.try_count] = TryFrame {
                            catch_ip,
                            stack_base: self.sp,
                            frame_idx: self.frame_count - 1,
                        };
                        self.try_count += 1;
                    }
                }
                OP_TRY_END => {
                    if self.try_count > 0 {
                        self.try_count -= 1;
                    }
                }
                OP_CATCH_BIND => {
                    let slot = self.read_u16() as usize;
                    let base = self.frames[self.frame_count - 1].stack_base;
                    // The thrown value should be at the top of stack after unwinding
                    if self.sp > base {
                        self.stack[base + slot] = self.stack[self.sp - 1];
                    }
                }

                // ─── In operator ───
                OP_IN => {
                    let obj = self.pop()?;
                    let key = self.pop()?;
                    if obj.is_object() && key.is_string() {
                        let has = self.heap.has_own_prop(obj.as_obj(), key.as_str_id());
                        self.push(JsValue::from_bool(has))?;
                    } else {
                        self.push(JsValue::FALSE)?;
                    }
                }

                // ─── Instanceof ───
                OP_INSTANCEOF => {
                    let _ctor = self.pop()?;
                    let _obj = self.pop()?;
                    // Simplified instanceof
                    self.push(JsValue::FALSE)?;
                }

                OP_HALT => {
                    return Ok(if self.sp > 0 { self.stack[self.sp - 1] } else { JsValue::UNDEFINED });
                }

                _ => {
                    // Unknown opcode — skip
                    crate::drivers::uart::puts("[js] unknown opcode: 0x");
                    let hex = b"0123456789abcdef";
                    crate::drivers::uart::putc(hex[(op >> 4) as usize]);
                    crate::drivers::uart::putc(hex[(op & 0xf) as usize]);
                    crate::drivers::uart::puts("\n");
                }
            }
        }
    }

    // ─── Stack operations ───

    #[inline]
    fn push(&mut self, val: JsValue) -> Result<(), JsError> {
        if self.sp >= MAX_STACK { return Err(JsError::StackOverflow); }
        self.stack[self.sp] = val;
        self.sp += 1;
        Ok(())
    }

    #[inline]
    fn pop(&mut self) -> Result<JsValue, JsError> {
        if self.sp == 0 { return Err(JsError::StackUnderflow); }
        self.sp -= 1;
        Ok(self.stack[self.sp])
    }

    #[inline]
    fn peek(&self) -> Result<JsValue, JsError> {
        if self.sp == 0 { return Err(JsError::StackUnderflow); }
        Ok(self.stack[self.sp - 1])
    }

    // ─── Operand reading (from current frame's bytecode) ───

    fn read_u8(&mut self) -> u8 {
        let frame = &mut self.frames[self.frame_count - 1];
        let pi = frame.proto_idx as usize;
        let v = self.protos[pi].bytecode[frame.ip];
        frame.ip += 1;
        v
    }

    fn read_u16(&mut self) -> u16 {
        let frame = &mut self.frames[self.frame_count - 1];
        let pi = frame.proto_idx as usize;
        let v = u16::from_le_bytes([
            self.protos[pi].bytecode[frame.ip],
            self.protos[pi].bytecode[frame.ip + 1],
        ]);
        frame.ip += 2;
        v
    }

    fn read_i16(&mut self) -> i16 {
        let frame = &mut self.frames[self.frame_count - 1];
        let pi = frame.proto_idx as usize;
        let v = i16::from_le_bytes([
            self.protos[pi].bytecode[frame.ip],
            self.protos[pi].bytecode[frame.ip + 1],
        ]);
        frame.ip += 2;
        v
    }

    fn read_i32(&mut self) -> i32 {
        let frame = &mut self.frames[self.frame_count - 1];
        let pi = frame.proto_idx as usize;
        let v = i32::from_le_bytes([
            self.protos[pi].bytecode[frame.ip],
            self.protos[pi].bytecode[frame.ip + 1],
            self.protos[pi].bytecode[frame.ip + 2],
            self.protos[pi].bytecode[frame.ip + 3],
        ]);
        frame.ip += 4;
        v
    }

    // ─── Function calls ───

    /// STUMP #93: method-call dispatch. Stack on entry is
    ///     [func, this, arg0, ..., arg(argc-1)]
    /// where `argc` is the number of real arguments (NOT counting
    /// `this`). Native callees receive `args_start = func_pos + 2`
    /// so the receiver doesn't leak into arg[0]. User-defined
    /// functions inherit `this` into their CallFrame, and arguments
    /// are copied from the post-`this` slots into local-param slots.
    fn call_method(&mut self, argc: usize) -> Result<(), JsError> {
        if self.frame_count >= MAX_FRAMES {
            return Err(JsError::StackOverflow);
        }
        // func is at sp - argc - 2 (above it: this, then argc args).
        let func_pos = self.sp - argc - 2;
        let func_val = self.stack[func_pos];
        let this_val = self.stack[func_pos + 1];

        if func_val.is_object() {
            let obj = func_val.as_obj();
            let flags = self.heap.get_flags(obj);

            if flags & ObjFlags::NATIVE != 0 {
                let native_idx = self.heap.get_native_idx(obj);
                if let Some(native_fn) = self.natives[native_idx as usize] {
                    let args_start = func_pos + 2; // skip `this`
                    let result = native_fn(self, args_start, argc)?;
                    self.sp = func_pos;
                    self.push(result)?;
                    return Ok(());
                }
            }

            if flags & ObjFlags::FUNCTION != 0 {
                let proto_idx = self.heap.get_func_proto_idx(obj);
                let new_base = func_pos;
                let local_count = self.protos[proto_idx as usize].local_count as usize;
                let param_count = self.protos[proto_idx as usize].param_count as usize;

                // Snapshot args from the post-`this` slots.
                let mut args_tmp = [JsValue::UNDEFINED; 16];
                let real_argc = argc.min(16);
                for i in 0..real_argc {
                    args_tmp[i] = self.stack[func_pos + 2 + i];
                }
                self.sp = new_base;
                for i in 0..local_count {
                    if i < real_argc && i < param_count {
                        self.stack[new_base + i] = args_tmp[i];
                    } else {
                        self.stack[new_base + i] = JsValue::UNDEFINED;
                    }
                }
                self.sp = new_base + local_count;

                self.frames[self.frame_count] = CallFrame {
                    proto_idx,
                    ip: 0,
                    stack_base: new_base,
                    this_val,
                    closure_obj: obj,
                };
                self.frame_count += 1;
                return Ok(());
            }
        }

        // Not callable — pop everything we set up and push undefined.
        self.sp = func_pos;
        self.push(JsValue::UNDEFINED)?;
        Ok(())
    }

    fn call_function(&mut self, argc: usize) -> Result<(), JsError> {
        if self.frame_count >= MAX_FRAMES {
            return Err(JsError::StackOverflow);
        }

        // The function value is below the arguments on the stack
        let func_pos = self.sp - argc - 1;
        let func_val = self.stack[func_pos];

        if func_val.is_object() {
            let obj = func_val.as_obj();
            let flags = self.heap.get_flags(obj);

            if flags & ObjFlags::NATIVE != 0 {
                // Native function call
                let native_idx = self.heap.get_native_idx(obj);
                if let Some(native_fn) = self.natives[native_idx as usize] {
                    let args_start = func_pos + 1;
                    let result = native_fn(self, args_start, argc)?;
                    self.sp = func_pos;
                    self.push(result)?;
                    return Ok(());
                }
            }

            if flags & ObjFlags::FUNCTION != 0 {
                // User-defined function call
                let proto_idx = self.heap.get_func_proto_idx(obj);
                let new_base = func_pos;
                let local_count = self.protos[proto_idx as usize].local_count as usize;
                let param_count = self.protos[proto_idx as usize].param_count as usize;

                // Determine 'this' value
                // For method calls, 'this' was pushed as the first arg below the function
                // For regular calls, check if there's a 'this' from caller
                let this_val = if func_pos > 0 {
                    // Check if caller set up this for a method call
                    // In method calls, stack is: [obj, func, args...]
                    // func_pos points to func, func_pos-1 is obj
                    // But our method call pattern is: push obj, dup, get_prop, swap
                    // So stack is: [func, obj, args...]
                    // Actually, the method call pushes: obj, func, obj (this), args
                    // So this_val should be the value right after the function
                    if argc > 0 && func_pos + 1 < self.sp {
                        // In method calls, the first "arg" is actually 'this'
                        // We need to check caller's setup
                        JsValue::UNDEFINED
                    } else {
                        JsValue::UNDEFINED
                    }
                } else {
                    JsValue::UNDEFINED
                };

                // Save args temporarily
                let mut args_tmp = [JsValue::UNDEFINED; 16];
                let real_argc = argc.min(16);
                for i in 0..real_argc {
                    args_tmp[i] = self.stack[func_pos + 1 + i];
                }

                // Set up local slots at new_base
                self.sp = new_base;
                for i in 0..local_count {
                    if i < real_argc && i < param_count {
                        self.stack[new_base + i] = args_tmp[i];
                    } else {
                        self.stack[new_base + i] = JsValue::UNDEFINED;
                    }
                }
                self.sp = new_base + local_count;

                self.frames[self.frame_count] = CallFrame {
                    proto_idx: proto_idx,
                    ip: 0,
                    stack_base: new_base,
                    this_val,
                    closure_obj: obj,
                };
                self.frame_count += 1;
                return Ok(());
            }
        }

        // If not callable, just pop args and push undefined
        self.sp = func_pos;
        self.push(JsValue::UNDEFINED)?;
        Ok(())
    }

    // ─── Property access ───

    fn get_property(&mut self, val: JsValue, name: StringId) -> JsValue {
        if val.is_object() {
            let obj_id = val.as_obj();
            let flags = self.heap.get_flags(obj_id);

            // Array.length
            if flags & ObjFlags::ARRAY != 0 {
                if name == super::strings::well_known::LENGTH {
                    return JsValue::from_i32(self.heap.array_len(obj_id) as i32);
                }
                // Check own props first, then Array.prototype
                let own = self.heap.get_prop(obj_id, name);
                if !own.is_undefined() { return own; }
                if !self.array_proto.is_null() {
                    return self.heap.get_prop(self.array_proto, name);
                }
                return JsValue::UNDEFINED;
            }

            return self.heap.get_prop(obj_id, name);
        }
        if val.is_string() {
            // String.length
            if name == super::strings::well_known::LENGTH {
                return JsValue::from_i32(self.strings.len(val.as_str_id()) as i32);
            }
            // String.prototype methods
            if !self.string_proto.is_null() {
                return self.heap.get_prop(self.string_proto, name);
            }
        }
        if val.is_number() {
            // Number.prototype methods (toFixed, toString)
            if !self.number_proto.is_null() {
                return self.heap.get_prop(self.number_proto, name);
            }
        }
        JsValue::UNDEFINED
    }

    // ─── Global variables ───

    pub fn get_global(&self, name: StringId) -> JsValue {
        for i in 0..self.global_count {
            if self.globals[i].name.0 == name.0 {
                return self.globals[i].value;
            }
        }
        JsValue::UNDEFINED
    }

    pub fn set_global(&mut self, name: StringId, val: JsValue) {
        for i in 0..self.global_count {
            if self.globals[i].name.0 == name.0 {
                self.globals[i].value = val;
                return;
            }
        }
        if self.global_count < MAX_GLOBALS {
            self.globals[self.global_count] = GlobalEntry { name, value: val };
            self.global_count += 1;
        }
    }

    // ─── Exception handling ───

    fn handle_throw(&mut self, val: JsValue) -> Result<JsValue, JsError> {
        if self.try_count > 0 {
            self.try_count -= 1;
            let try_frame = self.try_stack[self.try_count];
            // Unwind to the try frame
            self.frame_count = try_frame.frame_idx + 1;
            self.sp = try_frame.stack_base;
            self.push(val)?;
            self.frames[self.frame_count - 1].ip = try_frame.catch_ip;
            Ok(JsValue::UNDEFINED) // Continue execution at catch block
        } else {
            Err(JsError::Thrown(val))
        }
    }

    // ─── Native function registration ───

    fn register_core_globals(&mut self) {
        // Register common global values
        let nan_id = self.strings.intern(b"NaN");
        self.set_global(nan_id, JsValue::from_f64(f64::NAN));

        let inf_id = self.strings.intern(b"Infinity");
        self.set_global(inf_id, JsValue::from_f64(f64::INFINITY));

        let undef_id = self.strings.intern(b"undefined");
        self.set_global(undef_id, JsValue::UNDEFINED);
    }

    fn register_native_functions(&mut self) {
        // console object with log method
        let console_obj = self.heap.alloc_object();
        let log_fn = self.make_native_function(native_console_log);
        let log_name = self.strings.intern(b"log");
        self.heap.set_prop(console_obj, log_name, JsValue::from_obj(log_fn));
        let warn_fn = self.make_native_function(native_console_warn);
        let warn_name = self.strings.intern(b"warn");
        self.heap.set_prop(console_obj, warn_name, JsValue::from_obj(warn_fn));
        let error_fn = self.make_native_function(native_console_warn);
        let error_name = self.strings.intern(b"error");
        self.heap.set_prop(console_obj, error_name, JsValue::from_obj(error_fn));

        let console_name = self.strings.intern(b"console");
        self.set_global(console_name, JsValue::from_obj(console_obj));

        // STUMP #106: synchronous fetch — exposed as both `fetch_sync`
        // (the actual API name) and `fetch` (alias). Both return the
        // response body as a string, "" on error.
        let fetch_fn = self.make_native_function(native_fetch_sync);
        let fetch_sync_name = self.strings.intern(b"fetch_sync");
        self.set_global(fetch_sync_name, JsValue::from_obj(fetch_fn));
        let fetch_name = self.strings.intern(b"fetch");
        self.set_global(fetch_name, JsValue::from_obj(fetch_fn));

        // STUMP #108 — Sprint 3.5: localStorage. Standard Web Storage
        // API surface (getItem, setItem, removeItem, clear, length).
        // Backed by the static singleton in browser::js::storage so
        // values persist across vm.execute calls within the same
        // cave session.
        let ls_obj = self.heap.alloc_object();
        let get_item_fn = self.make_native_function(native_ls_get_item);
        let set_item_fn = self.make_native_function(native_ls_set_item);
        let remove_item_fn = self.make_native_function(native_ls_remove_item);
        let clear_fn = self.make_native_function(native_ls_clear);
        let n = self.strings.intern(b"getItem");
        self.heap.set_prop(ls_obj, n, JsValue::from_obj(get_item_fn));
        let n = self.strings.intern(b"setItem");
        self.heap.set_prop(ls_obj, n, JsValue::from_obj(set_item_fn));
        let n = self.strings.intern(b"removeItem");
        self.heap.set_prop(ls_obj, n, JsValue::from_obj(remove_item_fn));
        let n = self.strings.intern(b"clear");
        self.heap.set_prop(ls_obj, n, JsValue::from_obj(clear_fn));
        let ls_name = self.strings.intern(b"localStorage");
        self.set_global(ls_name, JsValue::from_obj(ls_obj));

        // Math object
        let math_obj = self.heap.alloc_object();
        let pi_name = self.strings.intern(b"PI");
        self.heap.set_prop(math_obj, pi_name, JsValue::from_f64(core::f64::consts::PI));
        let e_name = self.strings.intern(b"E");
        self.heap.set_prop(math_obj, e_name, JsValue::from_f64(core::f64::consts::E));

        let floor_fn = self.make_native_function(native_math_floor);
        let floor_name = self.strings.intern(b"floor");
        self.heap.set_prop(math_obj, floor_name, JsValue::from_obj(floor_fn));
        let ceil_fn = self.make_native_function(native_math_ceil);
        let ceil_name = self.strings.intern(b"ceil");
        self.heap.set_prop(math_obj, ceil_name, JsValue::from_obj(ceil_fn));
        let abs_fn = self.make_native_function(native_math_abs);
        let abs_name = self.strings.intern(b"abs");
        self.heap.set_prop(math_obj, abs_name, JsValue::from_obj(abs_fn));
        let sqrt_fn = self.make_native_function(native_math_sqrt);
        let sqrt_name = self.strings.intern(b"sqrt");
        self.heap.set_prop(math_obj, sqrt_name, JsValue::from_obj(sqrt_fn));
        let min_fn = self.make_native_function(native_math_min);
        let min_name = self.strings.intern(b"min");
        self.heap.set_prop(math_obj, min_name, JsValue::from_obj(min_fn));
        let max_fn = self.make_native_function(native_math_max);
        let max_name = self.strings.intern(b"max");
        self.heap.set_prop(math_obj, max_name, JsValue::from_obj(max_fn));
        let round_fn = self.make_native_function(native_math_round);
        let round_name = self.strings.intern(b"round");
        self.heap.set_prop(math_obj, round_name, JsValue::from_obj(round_fn));
        let pow_fn = self.make_native_function(native_math_pow);
        let pow_name = self.strings.intern(b"pow");
        self.heap.set_prop(math_obj, pow_name, JsValue::from_obj(pow_fn));
        let random_fn = self.make_native_function(native_math_random);
        let random_name = self.strings.intern(b"random");
        self.heap.set_prop(math_obj, random_name, JsValue::from_obj(random_fn));
        let log_fn2 = self.make_native_function(native_math_log);
        let log_name2 = self.strings.intern(b"log");
        // Math.log shadows but that's fine, it's a different object
        // Actually we need to not shadow, the console.log is on a different obj
        self.heap.set_prop(math_obj, log_name2, JsValue::from_obj(log_fn2));

        let math_name = self.strings.intern(b"Math");
        self.set_global(math_name, JsValue::from_obj(math_obj));

        // parseInt / parseFloat / isNaN as globals
        let parse_int_fn = self.make_native_function(native_parse_int);
        let parse_int_name = self.strings.intern(b"parseInt");
        self.set_global(parse_int_name, JsValue::from_obj(parse_int_fn));

        let parse_float_fn = self.make_native_function(native_parse_float);
        let parse_float_name = self.strings.intern(b"parseFloat");
        self.set_global(parse_float_name, JsValue::from_obj(parse_float_fn));

        let is_nan_fn = self.make_native_function(native_is_nan);
        let is_nan_name = self.strings.intern(b"isNaN");
        self.set_global(is_nan_name, JsValue::from_obj(is_nan_fn));

        let is_finite_fn = self.make_native_function(native_is_finite);
        let is_finite_name = self.strings.intern(b"isFinite");
        self.set_global(is_finite_name, JsValue::from_obj(is_finite_fn));

        // alert
        let alert_fn = self.make_native_function(native_alert);
        let alert_name = self.strings.intern(b"alert");
        self.set_global(alert_name, JsValue::from_obj(alert_fn));

        // Array.isArray
        let array_obj = self.heap.alloc_object();
        let is_array_fn = self.make_native_function(native_array_is_array);
        let is_array_name = self.strings.intern(b"isArray");
        self.heap.set_prop(array_obj, is_array_name, JsValue::from_obj(is_array_fn));
        let array_name = self.strings.intern(b"Array");
        self.set_global(array_name, JsValue::from_obj(array_obj));

        // Object.keys, Object.values, Object.entries
        let object_obj = self.heap.alloc_object();
        let keys_fn = self.make_native_function(native_object_keys);
        let keys_name = self.strings.intern(b"keys");
        self.heap.set_prop(object_obj, keys_name, JsValue::from_obj(keys_fn));
        let values_fn = self.make_native_function(native_object_values);
        let values_name = self.strings.intern(b"values");
        self.heap.set_prop(object_obj, values_name, JsValue::from_obj(values_fn));
        let entries_fn = self.make_native_function(native_object_entries);
        let entries_name = self.strings.intern(b"entries");
        self.heap.set_prop(object_obj, entries_name, JsValue::from_obj(entries_fn));
        let assign_fn = self.make_native_function(native_object_assign);
        let assign_name = self.strings.intern(b"assign");
        self.heap.set_prop(object_obj, assign_name, JsValue::from_obj(assign_fn));
        let freeze_fn = self.make_native_function(native_object_freeze);
        let freeze_name = self.strings.intern(b"freeze");
        self.heap.set_prop(object_obj, freeze_name, JsValue::from_obj(freeze_fn));
        let object_name = self.strings.intern(b"Object");
        self.set_global(object_name, JsValue::from_obj(object_obj));

        // JSON.parse, JSON.stringify
        let json_obj = self.heap.alloc_object();
        let json_parse_fn = self.make_native_function(native_json_parse);
        let parse_name = self.strings.intern(b"parse");
        self.heap.set_prop(json_obj, parse_name, JsValue::from_obj(json_parse_fn));
        let json_stringify_fn = self.make_native_function(native_json_stringify);
        let stringify_name = self.strings.intern(b"stringify");
        self.heap.set_prop(json_obj, stringify_name, JsValue::from_obj(json_stringify_fn));
        let json_name = self.strings.intern(b"JSON");
        self.set_global(json_name, JsValue::from_obj(json_obj));
    }

    pub fn make_native_function(&mut self, func: NativeFn) -> ObjId {
        let idx = self.native_count;
        if idx >= MAX_NATIVES { return ObjId::NULL; }
        self.natives[idx] = Some(func);
        self.native_count += 1;
        self.heap.alloc_native_function(idx as u16)
    }

    /// Write to the console output buffer (for console.log).
    pub fn console_write(&mut self, s: &[u8]) {
        let copy = s.len().min(self.console_buf.len() - self.console_len);
        self.console_buf[self.console_len..self.console_len + copy].copy_from_slice(&s[..copy]);
        self.console_len += copy;
    }
}

// ─── Native function implementations ───

fn native_console_log(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    let mut buf = [0u8; 256];
    let mut pos = 0;
    for i in 0..argc {
        if i > 0 && pos < 255 { buf[pos] = b' '; pos += 1; }
        let val = vm.stack[args_start + i];
        pos += val.write_to(&mut buf[pos..], &vm.strings);
    }
    if pos < 255 { buf[pos] = b'\n'; pos += 1; }
    // Console output goes to vm.console_buf only — the shell drains it
    // into the serial log inside the `=== JS console ===` block. We used
    // to also uart-mirror with a `[js] ` prefix during the bring-up of
    // STUMP #86, but that doubled the output and leaked into the
    // pre-console "render:" trace. Single source of truth = console_buf.
    vm.console_write(&buf[..pos]);
    Ok(JsValue::UNDEFINED)
}

fn native_console_warn(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    native_console_log(vm, args_start, argc)
}

/// STUMP #106 — Sprint 3.2: synchronous fetch from JS.
/// Modern web `fetch()` is Promise-based and async; we don't have an
/// event loop in the JS engine yet, so this is `fetch_sync(url)`.
/// Returns the response body as a string. Returns "" on any error.
/// Same SOP and TLS-mode rules apply as Rust-side fetch — JS
/// can't bypass the renderer's security policy.
// STUMP #108 — Sprint 3.5: localStorage natives. Each is a thin
// wrapper that pulls (key, value) strings from the JS stack, copies
// them to local buffers (so we drop the immutable borrow on
// vm.strings before mutating the storage), and forwards to the
// module-level singleton in storage.rs.

fn native_ls_get_item(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::UNDEFINED); }
    let kv = vm.stack[args_start];
    if !kv.is_string() { return Ok(JsValue::UNDEFINED); }
    let kb = vm.strings.get(kv.as_str_id());
    let mut kbuf = [0u8; 32];
    let kl = kb.len().min(kbuf.len());
    kbuf[..kl].copy_from_slice(&kb[..kl]);
    let key = unsafe { core::str::from_utf8_unchecked(&kbuf[..kl]) };
    match crate::browser::js::storage::local_get_item(key) {
        Some(v) => {
            let sid = vm.strings.intern(v.as_bytes());
            Ok(JsValue::from_str(sid))
        }
        None => Ok(JsValue::UNDEFINED),
    }
}

fn native_ls_set_item(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc < 2 { return Ok(JsValue::UNDEFINED); }
    let kv = vm.stack[args_start];
    let vv = vm.stack[args_start + 1];
    if !kv.is_string() || !vv.is_string() { return Ok(JsValue::UNDEFINED); }
    let kb = vm.strings.get(kv.as_str_id());
    let vb = vm.strings.get(vv.as_str_id());
    let mut kbuf = [0u8; 32];
    let mut vbuf = [0u8; 128];
    let kl = kb.len().min(kbuf.len());
    let vl = vb.len().min(vbuf.len());
    kbuf[..kl].copy_from_slice(&kb[..kl]);
    vbuf[..vl].copy_from_slice(&vb[..vl]);
    let k = unsafe { core::str::from_utf8_unchecked(&kbuf[..kl]) };
    let v = unsafe { core::str::from_utf8_unchecked(&vbuf[..vl]) };
    crate::browser::js::storage::local_set_item(k, v);
    Ok(JsValue::UNDEFINED)
}

fn native_ls_remove_item(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::UNDEFINED); }
    let kv = vm.stack[args_start];
    if !kv.is_string() { return Ok(JsValue::UNDEFINED); }
    let kb = vm.strings.get(kv.as_str_id());
    let mut kbuf = [0u8; 32];
    let kl = kb.len().min(kbuf.len());
    kbuf[..kl].copy_from_slice(&kb[..kl]);
    let key = unsafe { core::str::from_utf8_unchecked(&kbuf[..kl]) };
    crate::browser::js::storage::local_remove_item(key);
    Ok(JsValue::UNDEFINED)
}

fn native_ls_clear(_vm: &mut Vm, _args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    crate::browser::js::storage::local_clear();
    Ok(JsValue::UNDEFINED)
}

fn native_fetch_sync(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::from_str(crate::browser::js::value::StringId::EMPTY)); }
    let url_val = vm.stack[args_start];
    if !url_val.is_string() {
        return Ok(JsValue::from_str(crate::browser::js::value::StringId::EMPTY));
    }
    let url_bytes = vm.strings.get(url_val.as_str_id());
    let mut url_buf = [0u8; 512];
    let url_len = url_bytes.len().min(url_buf.len());
    url_buf[..url_len].copy_from_slice(&url_bytes[..url_len]);
    let url = unsafe { core::str::from_utf8_unchecked(&url_buf[..url_len]) };
    // STUMP #106: Same-origin policy applies. Hostile JS can't use
    // fetch() to exfiltrate to a different host.
    if crate::security::origin::check_subresource(url).is_err() {
        return Ok(JsValue::from_str(crate::browser::js::value::StringId::EMPTY));
    }
    static mut FETCH_BUF: [u8; 64 * 1024] = [0; 64 * 1024];
    let buf = unsafe { &mut *core::ptr::addr_of_mut!(FETCH_BUF) };
    let n = match crate::net::fetch::fetch_url(url, buf) {
        Ok(n) => n,
        Err(_) => return Ok(JsValue::from_str(crate::browser::js::value::StringId::EMPTY)),
    };
    let sid = vm.strings.intern(&buf[..n]);
    Ok(JsValue::from_str(sid))
}

fn native_math_floor(_vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::from_f64(f64::NAN)); }
    let v = _vm.stack[args_start].to_number();
    Ok(JsValue::from_f64(floor_f64(v)))
}

fn native_math_ceil(_vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::from_f64(f64::NAN)); }
    let v = _vm.stack[args_start].to_number();
    Ok(JsValue::from_f64(ceil_f64(v)))
}

fn native_math_abs(_vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::from_f64(f64::NAN)); }
    let v = _vm.stack[args_start].to_number();
    Ok(JsValue::from_f64(if v < 0.0 { -v } else { v }))
}

fn native_math_sqrt(_vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::from_f64(f64::NAN)); }
    let v = _vm.stack[args_start].to_number();
    Ok(JsValue::from_f64(sqrt_f64(v)))
}

fn native_math_min(_vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::from_f64(f64::INFINITY)); }
    let mut result = _vm.stack[args_start].to_number();
    for i in 1..argc {
        let v = _vm.stack[args_start + i].to_number();
        if v < result { result = v; }
    }
    Ok(JsValue::from_f64(result))
}

fn native_math_max(_vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::from_f64(f64::NEG_INFINITY)); }
    let mut result = _vm.stack[args_start].to_number();
    for i in 1..argc {
        let v = _vm.stack[args_start + i].to_number();
        if v > result { result = v; }
    }
    Ok(JsValue::from_f64(result))
}

fn native_math_round(_vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::from_f64(f64::NAN)); }
    let v = _vm.stack[args_start].to_number();
    Ok(JsValue::from_f64(floor_f64(v + 0.5)))
}

fn native_math_pow(_vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc < 2 { return Ok(JsValue::from_f64(f64::NAN)); }
    let base = _vm.stack[args_start].to_number();
    let exp = _vm.stack[args_start + 1].to_number();
    Ok(JsValue::from_f64(pow_f64(base, exp)))
}

fn native_math_random(_vm: &mut Vm, _args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    // Simple pseudo-random (LCG). Not cryptographic, but works for JS Math.random()
    static mut SEED: u64 = 12345678901234567;
    let s = unsafe {
        SEED = SEED.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        SEED
    };
    let val = (s >> 12) as f64 / (1u64 << 52) as f64;
    Ok(JsValue::from_f64(val))
}

fn native_math_log(_vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::from_f64(f64::NAN)); }
    let v = _vm.stack[args_start].to_number();
    Ok(JsValue::from_f64(ln_f64(v)))
}

fn native_parse_int(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::from_f64(f64::NAN)); }
    let val = vm.stack[args_start];
    if val.is_number() { return Ok(JsValue::from_i32(val.to_i32())); }
    if val.is_string() {
        let s = vm.strings.get(val.as_str_id());
        let mut i = 0;
        while i < s.len() && s[i] == b' ' { i += 1; }
        let neg = i < s.len() && s[i] == b'-';
        if neg || (i < s.len() && s[i] == b'+') { i += 1; }
        let mut n: i32 = 0;
        let mut found = false;
        while i < s.len() && s[i] >= b'0' && s[i] <= b'9' {
            n = n.wrapping_mul(10).wrapping_add((s[i] - b'0') as i32);
            found = true;
            i += 1;
        }
        if !found { return Ok(JsValue::from_f64(f64::NAN)); }
        return Ok(JsValue::from_i32(if neg { -n } else { n }));
    }
    Ok(JsValue::from_f64(f64::NAN))
}

fn native_parse_float(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::from_f64(f64::NAN)); }
    let val = vm.stack[args_start];
    if val.is_number() { return Ok(JsValue::from_f64(val.to_number())); }
    if val.is_string() {
        let s = vm.strings.get(val.as_str_id());
        let mut i = 0;
        while i < s.len() && s[i] == b' ' { i += 1; }
        let neg = i < s.len() && s[i] == b'-';
        if neg || (i < s.len() && s[i] == b'+') { i += 1; }
        let mut n: f64 = 0.0;
        let mut found = false;
        while i < s.len() && s[i] >= b'0' && s[i] <= b'9' {
            n = n * 10.0 + (s[i] - b'0') as f64;
            found = true;
            i += 1;
        }
        if i < s.len() && s[i] == b'.' {
            i += 1;
            let mut frac = 0.1;
            while i < s.len() && s[i] >= b'0' && s[i] <= b'9' {
                n += (s[i] - b'0') as f64 * frac;
                frac *= 0.1;
                found = true;
                i += 1;
            }
        }
        if !found { return Ok(JsValue::from_f64(f64::NAN)); }
        return Ok(JsValue::from_f64(if neg { -n } else { n }));
    }
    Ok(JsValue::from_f64(f64::NAN))
}

fn native_is_nan(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::TRUE); }
    let v = vm.stack[args_start].to_number();
    Ok(JsValue::from_bool(v.is_nan()))
}

fn native_is_finite(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::FALSE); }
    let v = vm.stack[args_start].to_number();
    Ok(JsValue::from_bool(!v.is_nan() && !v.is_infinite()))
}

fn native_alert(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    native_console_log(vm, args_start, argc)
}

/// Public alias for DOM module to reference.
pub fn native_alert_for_window(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    native_console_log(vm, args_start, argc)
}

fn native_array_is_array(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::FALSE); }
    let val = vm.stack[args_start];
    if val.is_object() {
        let flags = vm.heap.get_flags(val.as_obj());
        return Ok(JsValue::from_bool(flags & ObjFlags::ARRAY != 0));
    }
    Ok(JsValue::FALSE)
}

fn native_object_keys(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::UNDEFINED); }
    let val = vm.stack[args_start];
    if !val.is_object() { return Ok(JsValue::UNDEFINED); }
    let mut keys = [StringId::EMPTY; 32];
    let count = vm.heap.own_keys(val.as_obj(), &mut keys);
    let arr = vm.heap.alloc_array(count as u32);
    for i in 0..count {
        vm.heap.array_set(arr, i as u32, JsValue::from_str(keys[i]));
    }
    Ok(JsValue::from_obj(arr))
}

fn native_object_values(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::UNDEFINED); }
    let val = vm.stack[args_start];
    if !val.is_object() { return Ok(JsValue::UNDEFINED); }
    let mut keys = [StringId::EMPTY; 32];
    let count = vm.heap.own_keys(val.as_obj(), &mut keys);
    let arr = vm.heap.alloc_array(count as u32);
    for i in 0..count {
        let v = vm.heap.get_prop(val.as_obj(), keys[i]);
        vm.heap.array_set(arr, i as u32, v);
    }
    Ok(JsValue::from_obj(arr))
}

fn native_object_entries(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::UNDEFINED); }
    let val = vm.stack[args_start];
    if !val.is_object() { return Ok(JsValue::UNDEFINED); }
    let mut keys = [StringId::EMPTY; 32];
    let count = vm.heap.own_keys(val.as_obj(), &mut keys);
    let arr = vm.heap.alloc_array(count as u32);
    for i in 0..count {
        let v = vm.heap.get_prop(val.as_obj(), keys[i]);
        let entry = vm.heap.alloc_array(2);
        vm.heap.array_set(entry, 0, JsValue::from_str(keys[i]));
        vm.heap.array_set(entry, 1, v);
        vm.heap.array_set(arr, i as u32, JsValue::from_obj(entry));
    }
    Ok(JsValue::from_obj(arr))
}

fn native_object_assign(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::UNDEFINED); }
    let target = vm.stack[args_start];
    if !target.is_object() { return Ok(target); }
    for i in 1..argc {
        let src = vm.stack[args_start + i];
        if src.is_object() {
            let mut keys = [StringId::EMPTY; 32];
            let count = vm.heap.own_keys(src.as_obj(), &mut keys);
            for j in 0..count {
                let v = vm.heap.get_prop(src.as_obj(), keys[j]);
                vm.heap.set_prop(target.as_obj(), keys[j], v);
            }
        }
    }
    Ok(target)
}

fn native_object_freeze(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::UNDEFINED); }
    let val = vm.stack[args_start];
    if val.is_object() {
        vm.heap.set_flags(val.as_obj(), ObjFlags::FROZEN);
    }
    Ok(val)
}

// ─── JSON ───

fn native_json_parse(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::UNDEFINED); }
    let val = vm.stack[args_start];
    if !val.is_string() { return Ok(JsValue::UNDEFINED); }
    let s = vm.strings.get(val.as_str_id());
    let mut buf = [0u8; 4096];
    let len = s.len().min(4096);
    buf[..len].copy_from_slice(&s[..len]);
    let mut pos = 0usize;
    let result = json_parse_value(vm, &buf[..len], &mut pos);
    Ok(result)
}

fn json_parse_value(vm: &mut Vm, s: &[u8], pos: &mut usize) -> JsValue {
    json_skip_ws(s, pos);
    if *pos >= s.len() { return JsValue::NULL; }

    match s[*pos] {
        b'"' => json_parse_string(vm, s, pos),
        b'{' => json_parse_object(vm, s, pos),
        b'[' => json_parse_array(vm, s, pos),
        b't' => { *pos += 4; JsValue::TRUE }
        b'f' => { *pos += 5; JsValue::FALSE }
        b'n' => { *pos += 4; JsValue::NULL }
        _ => json_parse_number(s, pos),
    }
}

fn json_skip_ws(s: &[u8], pos: &mut usize) {
    while *pos < s.len() && (s[*pos] == b' ' || s[*pos] == b'\t' || s[*pos] == b'\n' || s[*pos] == b'\r') {
        *pos += 1;
    }
}

fn json_parse_string(vm: &mut Vm, s: &[u8], pos: &mut usize) -> JsValue {
    *pos += 1; // skip opening quote
    let start = *pos;
    while *pos < s.len() && s[*pos] != b'"' {
        if s[*pos] == b'\\' { *pos += 1; }
        *pos += 1;
    }
    let end = *pos;
    if *pos < s.len() { *pos += 1; } // skip closing quote
    let sid = vm.strings.intern(&s[start..end]);
    JsValue::from_str(sid)
}

fn json_parse_number(s: &[u8], pos: &mut usize) -> JsValue {
    let start = *pos;
    if *pos < s.len() && s[*pos] == b'-' { *pos += 1; }
    while *pos < s.len() && s[*pos].is_ascii_digit() { *pos += 1; }
    if *pos < s.len() && s[*pos] == b'.' {
        *pos += 1;
        while *pos < s.len() && s[*pos].is_ascii_digit() { *pos += 1; }
    }
    // Parse the number
    let mut val: f64 = 0.0;
    let mut i = start;
    let neg = i < s.len() && s[i] == b'-';
    if neg { i += 1; }
    while i < *pos && s[i] != b'.' {
        val = val * 10.0 + (s[i] - b'0') as f64;
        i += 1;
    }
    if i < *pos && s[i] == b'.' {
        i += 1;
        let mut frac = 0.1;
        while i < *pos {
            val += (s[i] - b'0') as f64 * frac;
            frac *= 0.1;
            i += 1;
        }
    }
    if neg { val = -val; }
    JsValue::from_f64(val)
}

fn json_parse_object(vm: &mut Vm, s: &[u8], pos: &mut usize) -> JsValue {
    *pos += 1; // skip {
    let obj = vm.heap.alloc_object();
    json_skip_ws(s, pos);
    while *pos < s.len() && s[*pos] != b'}' {
        // Parse key
        json_skip_ws(s, pos);
        if *pos >= s.len() || s[*pos] != b'"' { break; }
        let key_val = json_parse_string(vm, s, pos);
        let key_sid = key_val.as_str_id();
        // Skip colon
        json_skip_ws(s, pos);
        if *pos < s.len() && s[*pos] == b':' { *pos += 1; }
        // Parse value
        let val = json_parse_value(vm, s, pos);
        vm.heap.set_prop(obj, key_sid, val);
        // Skip comma
        json_skip_ws(s, pos);
        if *pos < s.len() && s[*pos] == b',' { *pos += 1; }
    }
    if *pos < s.len() { *pos += 1; } // skip }
    JsValue::from_obj(obj)
}

fn json_parse_array(vm: &mut Vm, s: &[u8], pos: &mut usize) -> JsValue {
    *pos += 1; // skip [
    let arr = vm.heap.alloc_array(0);
    let mut _idx = 0u32;
    json_skip_ws(s, pos);
    while *pos < s.len() && s[*pos] != b']' {
        let val = json_parse_value(vm, s, pos);
        vm.heap.array_push(arr, val);
        _idx += 1;
        json_skip_ws(s, pos);
        if *pos < s.len() && s[*pos] == b',' { *pos += 1; }
    }
    if *pos < s.len() { *pos += 1; } // skip ]
    JsValue::from_obj(arr)
}

fn native_json_stringify(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::UNDEFINED); }
    let val = vm.stack[args_start];
    let mut buf = [0u8; 4096];
    let len = json_stringify_value(vm, val, &mut buf, 0);
    let sid = vm.strings.intern(&buf[..len]);
    Ok(JsValue::from_str(sid))
}

fn json_stringify_value(vm: &Vm, val: JsValue, buf: &mut [u8], pos: usize) -> usize {
    let mut p = pos;
    if val.is_undefined() || val.is_null() {
        let s = b"null";
        let n = s.len().min(buf.len() - p);
        buf[p..p + n].copy_from_slice(&s[..n]);
        return p + n;
    }
    if val.is_bool() {
        let s = if val.as_bool() { &b"true"[..] } else { &b"false"[..] };
        let n = s.len().min(buf.len() - p);
        buf[p..p + n].copy_from_slice(&s[..n]);
        return p + n;
    }
    if val.is_number() {
        let n = val.write_to(&mut buf[p..], &vm.strings);
        return p + n;
    }
    if val.is_string() {
        if p < buf.len() { buf[p] = b'"'; p += 1; }
        let s = vm.strings.get(val.as_str_id());
        let n = s.len().min(buf.len() - p);
        buf[p..p + n].copy_from_slice(&s[..n]);
        p += n;
        if p < buf.len() { buf[p] = b'"'; p += 1; }
        return p;
    }
    if val.is_object() {
        let obj_id = val.as_obj();
        let flags = vm.heap.get_flags(obj_id);
        if flags & ObjFlags::ARRAY != 0 {
            if p < buf.len() { buf[p] = b'['; p += 1; }
            let len = vm.heap.array_len(obj_id);
            for i in 0..len {
                if i > 0 && p < buf.len() { buf[p] = b','; p += 1; }
                let elem = vm.heap.array_get(obj_id, i);
                p = json_stringify_value(vm, elem, buf, p);
            }
            if p < buf.len() { buf[p] = b']'; p += 1; }
            return p;
        }
        // Object
        if p < buf.len() { buf[p] = b'{'; p += 1; }
        let mut keys = [StringId::EMPTY; 32];
        let count = vm.heap.own_keys(obj_id, &mut keys);
        for i in 0..count {
            if i > 0 && p < buf.len() { buf[p] = b','; p += 1; }
            if p < buf.len() { buf[p] = b'"'; p += 1; }
            let k = vm.strings.get(keys[i]);
            let n = k.len().min(buf.len() - p);
            buf[p..p + n].copy_from_slice(&k[..n]);
            p += n;
            if p < buf.len() { buf[p] = b'"'; p += 1; }
            if p < buf.len() { buf[p] = b':'; p += 1; }
            let v = vm.heap.get_prop(obj_id, keys[i]);
            p = json_stringify_value(vm, v, buf, p);
        }
        if p < buf.len() { buf[p] = b'}'; p += 1; }
        return p;
    }
    p
}

// ─── no_std math helpers ───

fn floor_f64(v: f64) -> f64 {
    if v.is_nan() || v.is_infinite() { return v; }
    let i = v as i64 as f64;
    if v < i { i - 1.0 } else { i }
}

fn ceil_f64(v: f64) -> f64 {
    if v.is_nan() || v.is_infinite() { return v; }
    let i = v as i64 as f64;
    if v > i { i + 1.0 } else { i }
}

fn sqrt_f64(v: f64) -> f64 {
    if v < 0.0 { return f64::NAN; }
    if v == 0.0 { return 0.0; }
    // Newton's method
    let mut x = v;
    for _ in 0..64 {
        let next = 0.5 * (x + v / x);
        if (next - x).abs() < 1e-15 { break; }
        x = next;
    }
    x
}

fn pow_f64(base: f64, exp: f64) -> f64 {
    if exp == 0.0 { return 1.0; }
    if exp == 1.0 { return base; }
    if base == 0.0 { return 0.0; }
    // Integer exponent fast path
    let exp_i = exp as i32;
    if exp == exp_i as f64 && exp_i >= 0 && exp_i <= 100 {
        let mut result = 1.0;
        let mut b = base;
        let mut e = exp_i as u32;
        while e > 0 {
            if e & 1 != 0 { result *= b; }
            b *= b;
            e >>= 1;
        }
        return result;
    }
    // For non-integer exponents, use exp(exp * ln(base))
    exp_f64(exp * ln_f64(base))
}

fn ln_f64(x: f64) -> f64 {
    if x <= 0.0 { return f64::NAN; }
    if x == 1.0 { return 0.0; }
    // Reduce x to [1, 2) and compute ln
    let mut val = x;
    let mut k: i32 = 0;
    while val > 2.0 { val /= 2.0; k += 1; }
    while val < 1.0 { val *= 2.0; k -= 1; }
    // ln(val) where val in [1, 2) using series: ln(1+u) = u - u^2/2 + u^3/3 - ...
    let u = val - 1.0;
    let mut result = 0.0;
    let mut term = u;
    for n in 1..50 {
        result += term / n as f64;
        term *= -u;
    }
    result + k as f64 * 0.6931471805599453 // ln(2)
}

fn exp_f64(x: f64) -> f64 {
    if x == 0.0 { return 1.0; }
    // Taylor series: e^x = 1 + x + x^2/2! + x^3/3! + ...
    let mut result = 1.0;
    let mut term = 1.0;
    for n in 1..50 {
        term *= x / n as f64;
        result += term;
        if term.abs() < 1e-15 { break; }
    }
    result
}

// Absolute value for f64 (no libm)
fn abs_f64(v: f64) -> f64 {
    if v < 0.0 { -v } else { v }
}

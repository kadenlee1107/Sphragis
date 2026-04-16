// Bat_OS — Bytecode Compiler
// Walks the AST and emits stack-based bytecode for the VM.
// Resolves variables to local slots at compile time for fast access.
// Supports closures, upvalues, arrow functions, and this binding.

use super::ast::{AstNode, NodeKind, Operator, Ast};
use super::opcodes::*;
use super::value::StringId;
use super::strings::StringTable;

/// Maximum bytecode size per function.
pub const MAX_BYTECODE: usize = 8192;
/// Maximum constants per function.
pub const MAX_CONSTS: usize = 256;
/// Maximum local variables per function.
pub const MAX_LOCALS: usize = 128;
/// Maximum upvalues per function.
pub const MAX_UPVALUES: usize = 32;
/// Maximum compiled functions.
pub const MAX_PROTOS: usize = 128;

const NULL: u16 = 0xFFFF;

/// A constant pool entry.
#[derive(Clone, Copy)]
pub enum Constant {
    F64(f64),
    Str(StringId),
}

/// Description of a captured upvalue.
#[derive(Clone, Copy)]
pub struct UpvalueDesc {
    pub is_local: bool,
    pub index: u8,
}

/// A compiled function prototype.
pub struct FunctionProto {
    pub name: StringId,
    pub bytecode: [u8; MAX_BYTECODE],
    pub bytecode_len: usize,
    pub constants: [Constant; MAX_CONSTS],
    pub const_count: usize,
    pub local_count: u16,
    pub param_count: u8,
    pub upvalue_count: u8,
    pub upvalues: [UpvalueDesc; MAX_UPVALUES],
}

impl FunctionProto {
    pub const fn new() -> Self {
        FunctionProto {
            name: StringId::EMPTY,
            bytecode: [0; MAX_BYTECODE],
            bytecode_len: 0,
            constants: [Constant::F64(0.0); MAX_CONSTS],
            const_count: 0,
            local_count: 0,
            param_count: 0,
            upvalue_count: 0,
            upvalues: [UpvalueDesc { is_local: false, index: 0 }; MAX_UPVALUES],
        }
    }

    /// Uninitialized sentinel for static array init.
    pub const UNINIT: Self = Self::new();
}

/// A local variable during compilation.
#[derive(Clone, Copy)]
struct Local {
    name: StringId,
    depth: u8,
    slot: u16,
    is_captured: bool,
}

/// Upvalue info during compilation
#[derive(Clone, Copy)]
struct CompilerUpvalue {
    is_local: bool,
    index: u8,
    name: StringId,
}

/// Enclosing compiler state — stored in a static array
struct EnclosingScope {
    locals: [Local; MAX_LOCALS],
    local_count: usize,
    scope_depth: u8,
    upvalues: [CompilerUpvalue; MAX_UPVALUES],
    upvalue_count: usize,
}

/// Static storage for nested compiler contexts (max nesting depth)
const MAX_NESTING: usize = 16;

/// The bytecode compiler.
pub struct Compiler<'a> {
    proto: FunctionProto,
    locals: [Local; MAX_LOCALS],
    local_count: usize,
    scope_depth: u8,
    strings: &'a mut StringTable,
    break_patches: [usize; 32],
    break_count: usize,
    continue_target: usize,
    // Upvalue tracking
    upvalues: [CompilerUpvalue; MAX_UPVALUES],
    upvalue_count: usize,
    // Nested function compilation — store child protos in output array
    output_protos: [FunctionProto; MAX_PROTOS],
    output_proto_count: usize,
    // Enclosing scope stack for nested functions
    enclosing: [Option<EnclosingScope>; MAX_NESTING],
    enclosing_depth: usize,
    // Whether current function is an arrow function (inherits this)
    is_arrow: bool,
}

impl<'a> Compiler<'a> {
    pub fn new(strings: &'a mut StringTable) -> Self {
        Compiler {
            proto: FunctionProto::new(),
            locals: [Local { name: StringId::EMPTY, depth: 0, slot: 0, is_captured: false }; MAX_LOCALS],
            local_count: 0,
            scope_depth: 0,
            strings,
            break_patches: [0; 32],
            break_count: 0,
            continue_target: 0,
            upvalues: [CompilerUpvalue { is_local: false, index: 0, name: StringId::EMPTY }; MAX_UPVALUES],
            upvalue_count: 0,
            output_protos: {
                // Initialize with const fn
                let arr: [FunctionProto; MAX_PROTOS] = [FunctionProto::UNINIT; MAX_PROTOS];
                arr
            },
            output_proto_count: 0,
            enclosing: [const { None }; MAX_NESTING],
            enclosing_depth: 0,
            is_arrow: false,
        }
    }

    /// Compile a script from the AST. Returns (main_proto, child_protos, child_count).
    pub fn compile_script(mut self, ast: &Ast) -> (FunctionProto, [FunctionProto; MAX_PROTOS], usize) {
        if ast.count > 0 {
            self.compile_node(ast, 0); // compile root node (Program)
        }
        self.emit(OP_UNDEFINED);
        self.emit(OP_RETURN);
        self.proto.local_count = self.local_count as u16;
        let count = self.output_proto_count;
        (self.proto, self.output_protos, count)
    }

    /// Compile a function body into a new FunctionProto, return its index in output_protos.
    fn compile_function(&mut self, ast: &Ast, node: &AstNode, is_arrow: bool) -> u16 {
        if self.output_proto_count >= MAX_PROTOS {
            return 0;
        }

        // Save current compiler state
        if self.enclosing_depth < MAX_NESTING {
            let mut saved_locals = [Local { name: StringId::EMPTY, depth: 0, slot: 0, is_captured: false }; MAX_LOCALS];
            for i in 0..self.local_count {
                saved_locals[i] = self.locals[i];
            }
            let mut saved_upvalues = [CompilerUpvalue { is_local: false, index: 0, name: StringId::EMPTY }; MAX_UPVALUES];
            for i in 0..self.upvalue_count {
                saved_upvalues[i] = self.upvalues[i];
            }
            self.enclosing[self.enclosing_depth] = Some(EnclosingScope {
                locals: saved_locals,
                local_count: self.local_count,
                scope_depth: self.scope_depth,
                upvalues: saved_upvalues,
                upvalue_count: self.upvalue_count,
            });
            self.enclosing_depth += 1;
        }

        // Save current proto
        let saved_proto = core::mem::replace(&mut self.proto, FunctionProto::new());
        let saved_local_count = self.local_count;
        let saved_scope_depth = self.scope_depth;
        let saved_break_count = self.break_count;
        let saved_continue_target = self.continue_target;
        let saved_upvalue_count = self.upvalue_count;
        let saved_is_arrow = self.is_arrow;

        // Reset for new function
        self.local_count = 0;
        self.scope_depth = 0;
        self.break_count = 0;
        self.continue_target = 0;
        self.upvalue_count = 0;
        self.is_arrow = is_arrow;
        for i in 0..MAX_LOCALS {
            self.locals[i] = Local { name: StringId::EMPTY, depth: 0, slot: 0, is_captured: false };
        }
        for i in 0..MAX_UPVALUES {
            self.upvalues[i] = CompilerUpvalue { is_local: false, index: 0, name: StringId::EMPTY };
        }

        // Set function name
        self.proto.name = self.intern_name(node);

        // Declare parameters as locals
        let param_count = node.param_count;
        self.proto.param_count = param_count;
        for i in 0..param_count as usize {
            if node.params[i] != NULL {
                let param_node = ast.nodes[node.params[i] as usize];
                let pname = self.intern_name(&param_node);
                self.declare_local(pname);
            }
        }

        // Compile body
        if is_arrow {
            // Arrow function: body can be expression or block
            if node.body != NULL {
                let body_node = ast.nodes[node.body as usize];
                if body_node.kind == NodeKind::BlockStatement {
                    self.compile_node(ast, node.body as usize);
                } else {
                    // Expression body: compile and return result
                    self.compile_node(ast, node.body as usize);
                    self.emit(OP_RETURN);
                }
            }
        } else {
            // Regular function: body is a block
            if node.body != NULL {
                self.compile_node(ast, node.body as usize);
            }
        }

        // Implicit return undefined
        self.emit(OP_UNDEFINED);
        self.emit(OP_RETURN);

        self.proto.local_count = self.local_count as u16;

        // Copy upvalue info to proto
        self.proto.upvalue_count = self.upvalue_count as u8;
        for i in 0..self.upvalue_count {
            self.proto.upvalues[i] = UpvalueDesc {
                is_local: self.upvalues[i].is_local,
                index: self.upvalues[i].index,
            };
        }

        // Store the compiled proto
        let proto_idx = self.output_proto_count;
        let compiled_proto = core::mem::replace(&mut self.proto, saved_proto);
        self.output_protos[proto_idx] = compiled_proto;
        self.output_proto_count += 1;

        // Restore compiler state
        self.local_count = saved_local_count;
        self.scope_depth = saved_scope_depth;
        self.break_count = saved_break_count;
        self.continue_target = saved_continue_target;
        self.upvalue_count = saved_upvalue_count;
        self.is_arrow = saved_is_arrow;

        // Restore enclosing
        if self.enclosing_depth > 0 {
            self.enclosing_depth -= 1;
            if let Some(ref enc) = self.enclosing[self.enclosing_depth] {
                for i in 0..enc.local_count {
                    self.locals[i] = enc.locals[i];
                }
                for i in 0..enc.upvalue_count {
                    self.upvalues[i] = enc.upvalues[i];
                }
            }
            self.enclosing[self.enclosing_depth] = None;
        }

        proto_idx as u16
    }

    fn compile_node(&mut self, ast: &Ast, idx: usize) {
        if idx >= ast.count || idx as u16 == NULL { return; }
        let node = ast.nodes[idx];

        match node.kind {
            NodeKind::Empty => {}

            // ─── Program / Block ───
            NodeKind::Program => {
                let mut child = node.left;
                while child != NULL && (child as usize) < ast.count {
                    self.compile_node(ast, child as usize);
                    child = ast.nodes[child as usize].next;
                }
            }
            NodeKind::BlockStatement => {
                self.begin_scope();
                let mut child = node.left;
                while child != NULL && (child as usize) < ast.count {
                    self.compile_node(ast, child as usize);
                    child = ast.nodes[child as usize].next;
                }
                self.end_scope();
            }

            // ─── Variable Declaration ───
            NodeKind::VarDecl => {
                let name_id = self.intern_name(&node);
                let slot = self.declare_local(name_id);
                if node.left != NULL {
                    self.compile_node(ast, node.left as usize);
                } else {
                    self.emit(OP_UNDEFINED);
                }
                self.emit(OP_SET_LOCAL);
                self.emit_u16(slot);
                self.emit(OP_POP);
            }

            // ─── Expression Statement ───
            NodeKind::ExprStatement => {
                if node.left != NULL {
                    self.compile_node(ast, node.left as usize);
                    self.emit(OP_POP);
                }
            }

            // ─── Number Literal ───
            NodeKind::NumberLiteral => {
                let v = node.num_value;
                if v == 0.0 && !v.is_sign_negative() {
                    self.emit(OP_CONST_ZERO);
                } else if v == 1.0 {
                    self.emit(OP_CONST_ONE);
                } else if v == (v as i32 as f64) && (v as i32) >= -1000000 && (v as i32) <= 1000000 {
                    self.emit(OP_CONST_I32);
                    self.emit_i32(v as i32);
                } else {
                    let ci = self.add_const_f64(v);
                    self.emit(OP_CONST_F64);
                    self.emit_u16(ci);
                }
            }

            // ─── String Literal ───
            NodeKind::StringLiteral => {
                let sid = self.intern_name(&node);
                let ci = self.add_const_str(sid);
                self.emit(OP_CONST_STR);
                self.emit_u16(ci);
            }

            NodeKind::BoolLiteral => {
                if node.bool_value { self.emit(OP_TRUE); } else { self.emit(OP_FALSE); }
            }
            NodeKind::NullLiteral => { self.emit(OP_NULL); }

            // ─── Identifier (variable read) ───
            NodeKind::Identifier => {
                let name_id = self.intern_name(&node);
                // Check for 'this' keyword
                let this_name = self.strings.intern(b"this");
                if name_id.0 == this_name.0 {
                    self.emit(OP_THIS);
                    return;
                }
                if let Some(slot) = self.resolve_local(name_id) {
                    self.emit(OP_GET_LOCAL);
                    self.emit_u16(slot);
                } else if let Some(uv_idx) = self.resolve_upvalue(name_id) {
                    self.emit(OP_GET_CAPTURE);
                    self.emit_byte(uv_idx);
                    self.emit_byte(0); // reserved
                } else {
                    let ci = self.add_const_str(name_id);
                    self.emit(OP_GET_GLOBAL);
                    self.emit_u16(ci);
                }
            }

            // ─── Assignment ───
            NodeKind::AssignExpr => {
                if node.left != NULL {
                    let left = ast.nodes[node.left as usize];

                    // Handle compound assignment operators
                    match node.op {
                        Operator::PlusAssign | Operator::MinusAssign |
                        Operator::StarAssign | Operator::SlashAssign => {
                            // Read current value
                            self.compile_node(ast, node.left as usize);
                            // Compile RHS
                            if node.right != NULL {
                                self.compile_node(ast, node.right as usize);
                            }
                            // Apply operator
                            match node.op {
                                Operator::PlusAssign => self.emit(OP_ADD),
                                Operator::MinusAssign => self.emit(OP_SUB),
                                Operator::StarAssign => self.emit(OP_MUL),
                                Operator::SlashAssign => self.emit(OP_DIV),
                                _ => {}
                            }
                            self.emit(OP_DUP); // keep result
                            // Store
                            match left.kind {
                                NodeKind::Identifier => {
                                    let name_id = self.intern_name(&left);
                                    if let Some(slot) = self.resolve_local(name_id) {
                                        self.emit(OP_SET_LOCAL);
                                        self.emit_u16(slot);
                                    } else if let Some(uv_idx) = self.resolve_upvalue(name_id) {
                                        self.emit(OP_SET_CAPTURE);
                                        self.emit_byte(uv_idx);
                                        self.emit_byte(0);
                                    } else {
                                        let ci = self.add_const_str(name_id);
                                        self.emit(OP_SET_GLOBAL);
                                        self.emit_u16(ci);
                                    }
                                }
                                NodeKind::MemberExpr => {
                                    if left.left != NULL {
                                        self.compile_node(ast, left.left as usize);
                                    }
                                    self.emit(OP_SWAP);
                                    if left.right != NULL {
                                        let prop = ast.nodes[left.right as usize];
                                        let pname = self.intern_name(&prop);
                                        let ci = self.add_const_str(pname);
                                        self.emit(OP_SET_PROP);
                                        self.emit_u16(ci);
                                    }
                                }
                                _ => {}
                            }
                            return;
                        }
                        _ => {}
                    }

                    // Simple assignment
                    if node.right != NULL {
                        self.compile_node(ast, node.right as usize);
                    }
                    self.emit(OP_DUP); // keep for expression result

                    match left.kind {
                        NodeKind::Identifier => {
                            let name_id = self.intern_name(&left);
                            if let Some(slot) = self.resolve_local(name_id) {
                                self.emit(OP_SET_LOCAL);
                                self.emit_u16(slot);
                            } else if let Some(uv_idx) = self.resolve_upvalue(name_id) {
                                self.emit(OP_SET_CAPTURE);
                                self.emit_byte(uv_idx);
                                self.emit_byte(0);
                            } else {
                                let ci = self.add_const_str(name_id);
                                self.emit(OP_SET_GLOBAL);
                                self.emit_u16(ci);
                            }
                        }
                        NodeKind::MemberExpr => {
                            if left.left != NULL {
                                self.compile_node(ast, left.left as usize);
                            }
                            self.emit(OP_SWAP);
                            if left.right != NULL {
                                let prop = ast.nodes[left.right as usize];
                                let pname = self.intern_name(&prop);
                                let ci = self.add_const_str(pname);
                                self.emit(OP_SET_PROP);
                                self.emit_u16(ci);
                            }
                        }
                        _ => {}
                    }
                }
            }

            // ─── Binary Expression ───
            NodeKind::BinaryExpr => {
                // Short-circuit for && and ||
                match node.op {
                    Operator::And => {
                        self.compile_node(ast, node.left as usize);
                        self.emit(OP_DUP);
                        let jump = self.emit_jump(OP_JUMP_FALSE);
                        self.emit(OP_POP);
                        if node.right != NULL {
                            self.compile_node(ast, node.right as usize);
                        }
                        self.patch_jump(jump);
                        return;
                    }
                    Operator::Or => {
                        self.compile_node(ast, node.left as usize);
                        self.emit(OP_DUP);
                        let jump = self.emit_jump(OP_JUMP_TRUE);
                        self.emit(OP_POP);
                        if node.right != NULL {
                            self.compile_node(ast, node.right as usize);
                        }
                        self.patch_jump(jump);
                        return;
                    }
                    _ => {}
                }

                if node.left != NULL { self.compile_node(ast, node.left as usize); }
                if node.right != NULL { self.compile_node(ast, node.right as usize); }

                match node.op {
                    Operator::Add => self.emit(OP_ADD),
                    Operator::Sub => self.emit(OP_SUB),
                    Operator::Mul => self.emit(OP_MUL),
                    Operator::Div => self.emit(OP_DIV),
                    Operator::Mod => self.emit(OP_MOD),
                    Operator::Less => self.emit(OP_LT),
                    Operator::Greater => self.emit(OP_GT),
                    Operator::LessEqual => self.emit(OP_LE),
                    Operator::GreaterEqual => self.emit(OP_GE),
                    Operator::Equal => self.emit(OP_EQ),
                    Operator::StrictEqual => self.emit(OP_SEQ),
                    Operator::NotEqual => self.emit(OP_NE),
                    Operator::StrictNotEqual => self.emit(OP_SNE),
                    Operator::BitAnd => self.emit(OP_BIT_AND),
                    Operator::BitOr => self.emit(OP_BIT_OR),
                    Operator::BitXor => self.emit(OP_BIT_XOR),
                    Operator::ShiftLeft => self.emit(OP_SHL),
                    Operator::ShiftRight => self.emit(OP_SHR),
                    Operator::In => self.emit(OP_IN),
                    _ => self.emit(OP_ADD),
                }
            }

            // ─── Unary Expression ───
            NodeKind::UnaryExpr => {
                if node.left != NULL {
                    self.compile_node(ast, node.left as usize);
                }
                match node.op {
                    Operator::Sub => self.emit(OP_NEG),
                    Operator::Not => self.emit(OP_NOT),
                    Operator::BitNot => self.emit(OP_BIT_NOT),
                    Operator::Typeof => self.emit(OP_TYPEOF),
                    Operator::Increment => {
                        self.emit(OP_INC);
                        // Store back if it's an identifier
                        if node.left != NULL {
                            let left = ast.nodes[node.left as usize];
                            if left.kind == NodeKind::Identifier {
                                self.emit(OP_DUP);
                                let name_id = self.intern_name(&left);
                                if let Some(slot) = self.resolve_local(name_id) {
                                    self.emit(OP_SET_LOCAL);
                                    self.emit_u16(slot);
                                } else {
                                    let ci = self.add_const_str(name_id);
                                    self.emit(OP_SET_GLOBAL);
                                    self.emit_u16(ci);
                                }
                            }
                        }
                    }
                    Operator::Decrement => {
                        self.emit(OP_DEC);
                        if node.left != NULL {
                            let left = ast.nodes[node.left as usize];
                            if left.kind == NodeKind::Identifier {
                                self.emit(OP_DUP);
                                let name_id = self.intern_name(&left);
                                if let Some(slot) = self.resolve_local(name_id) {
                                    self.emit(OP_SET_LOCAL);
                                    self.emit_u16(slot);
                                } else {
                                    let ci = self.add_const_str(name_id);
                                    self.emit(OP_SET_GLOBAL);
                                    self.emit_u16(ci);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // ─── If Statement ───
            NodeKind::IfStatement => {
                if node.left != NULL {
                    self.compile_node(ast, node.left as usize);
                }
                let else_jump = self.emit_jump(OP_JUMP_FALSE);

                if node.right != NULL {
                    self.compile_node(ast, node.right as usize);
                }

                if node.extra != NULL {
                    let end_jump = self.emit_jump(OP_JUMP);
                    self.patch_jump(else_jump);
                    self.compile_node(ast, node.extra as usize);
                    self.patch_jump(end_jump);
                } else {
                    self.patch_jump(else_jump);
                }
            }

            // ─── While Loop ───
            NodeKind::WhileStatement => {
                let loop_start = self.proto.bytecode_len;
                self.continue_target = loop_start;
                let saved_break = self.break_count;

                if node.left != NULL { self.compile_node(ast, node.left as usize); }
                let exit = self.emit_jump(OP_JUMP_FALSE);
                if node.right != NULL { self.compile_node(ast, node.right as usize); }
                self.emit_loop(loop_start);
                self.patch_jump(exit);

                for i in saved_break..self.break_count {
                    self.patch_jump(self.break_patches[i]);
                }
                self.break_count = saved_break;
            }

            // ─── For Loop ───
            NodeKind::ForStatement => {
                self.begin_scope();
                if node.left != NULL { self.compile_node(ast, node.left as usize); }

                let loop_start = self.proto.bytecode_len;
                let saved_break = self.break_count;

                let mut exit = 0;
                let has_cond = node.right != NULL;
                if has_cond {
                    self.compile_node(ast, node.right as usize);
                    exit = self.emit_jump(OP_JUMP_FALSE);
                }

                if node.body != NULL { self.compile_node(ast, node.body as usize); }

                self.continue_target = self.proto.bytecode_len;
                if node.extra != NULL {
                    self.compile_node(ast, node.extra as usize);
                    self.emit(OP_POP);
                }

                self.emit_loop(loop_start);
                if has_cond { self.patch_jump(exit); }

                for i in saved_break..self.break_count {
                    self.patch_jump(self.break_patches[i]);
                }
                self.break_count = saved_break;
                self.end_scope();
            }

            // ─── Return ───
            NodeKind::ReturnStatement => {
                if node.left != NULL {
                    self.compile_node(ast, node.left as usize);
                } else {
                    self.emit(OP_UNDEFINED);
                }
                self.emit(OP_RETURN);
            }

            // ─── Function Declaration ───
            NodeKind::FunctionDecl => {
                let name_id = self.intern_name(&node);
                // Compile function body into a separate proto
                let proto_idx = self.compile_function(ast, &node, false);
                self.emit(OP_CLOSURE);
                self.emit_u16(proto_idx);
                // Bind to local or global
                if self.scope_depth > 0 {
                    let slot = self.declare_local(name_id);
                    self.emit(OP_SET_LOCAL);
                    self.emit_u16(slot);
                    self.emit(OP_POP);
                } else {
                    let slot = self.declare_local(name_id);
                    self.emit(OP_SET_LOCAL);
                    self.emit_u16(slot);
                    self.emit(OP_POP);
                }
            }

            // ─── Arrow Function Expression ───
            NodeKind::ArrowFunc => {
                let proto_idx = self.compile_function(ast, &node, true);
                self.emit(OP_CLOSURE);
                self.emit_u16(proto_idx);
            }

            // ─── Call Expression ───
            NodeKind::CallExpr => {
                let mut is_method = false;
                if node.left != NULL {
                    let callee = ast.nodes[node.left as usize];
                    if callee.kind == NodeKind::MemberExpr {
                        is_method = true;
                        // Method call: stack = [method, this, args...]
                        // 1. Push obj (will become `this`)
                        if callee.left != NULL {
                            self.compile_node(ast, callee.left as usize);
                        }
                        self.emit(OP_DUP);  // [obj, obj]
                        // 2. Get method from obj
                        if callee.right != NULL {
                            let prop = ast.nodes[callee.right as usize];
                            let pname = self.intern_name(&prop);
                            let ci = self.add_const_str(pname);
                            self.emit(OP_GET_PROP);
                            self.emit_u16(ci);
                        }
                        // Stack: [obj, method]
                        self.emit(OP_SWAP); // [method, obj]
                    } else {
                        self.compile_node(ast, node.left as usize);
                    }
                }

                // Compile arguments: linked via right → next chain
                let mut argc = 0u8;
                if is_method { argc = 1; } // count `this` as first arg
                let mut arg = node.right;
                while arg != NULL && (arg as usize) < ast.count {
                    self.compile_node(ast, arg as usize);
                    argc += 1;
                    arg = ast.nodes[arg as usize].next;
                }

                self.emit(OP_CALL);
                self.emit_byte(argc);
            }

            // ─── New Expression ───
            NodeKind::NewExpr => {
                // Compile constructor
                if node.left != NULL {
                    self.compile_node(ast, node.left as usize);
                }
                // Compile arguments
                let mut argc = 0u8;
                let mut arg = node.right;
                while arg != NULL && (arg as usize) < ast.count {
                    self.compile_node(ast, arg as usize);
                    argc += 1;
                    arg = ast.nodes[arg as usize].next;
                }
                self.emit(OP_NEW);
                self.emit_byte(argc);
            }

            // ─── Member Expression ───
            NodeKind::MemberExpr => {
                if node.left != NULL { self.compile_node(ast, node.left as usize); }
                if node.right != NULL {
                    let prop = ast.nodes[node.right as usize];
                    if prop.kind == NodeKind::Identifier {
                        let pname = self.intern_name(&prop);
                        let ci = self.add_const_str(pname);
                        self.emit(OP_GET_PROP);
                        self.emit_u16(ci);
                    } else {
                        // Computed property access: obj[expr]
                        self.compile_node(ast, node.right as usize);
                        self.emit(OP_GET_ELEM);
                    }
                }
            }

            // ─── Object Literal ───
            NodeKind::ObjectLiteral => {
                self.emit(OP_NEW_OBJECT);
                let mut prop = node.left;
                while prop != NULL && (prop as usize) < ast.count {
                    let pnode = ast.nodes[prop as usize];
                    if pnode.left != NULL {
                        self.compile_node(ast, pnode.left as usize);
                        let key_name = self.intern_name(&pnode);
                        let ci = self.add_const_str(key_name);
                        self.emit(OP_DEFINE_PROP);
                        self.emit_u16(ci);
                    }
                    prop = pnode.next;
                }
            }

            // ─── Array Literal ───
            NodeKind::ArrayLiteral => {
                let mut count = 0u16;
                let mut elem = node.left;
                while elem != NULL && (elem as usize) < ast.count {
                    self.compile_node(ast, elem as usize);
                    count += 1;
                    elem = ast.nodes[elem as usize].next;
                }
                self.emit(OP_NEW_ARRAY);
                self.emit_u16(count);
            }

            // ─── Break / Continue ───
            NodeKind::BreakStatement => {
                if self.break_count < 32 {
                    let patch = self.emit_jump(OP_JUMP);
                    self.break_patches[self.break_count] = patch;
                    self.break_count += 1;
                }
            }
            NodeKind::ContinueStatement => {
                self.emit_loop(self.continue_target);
            }

            // ─── Throw ───
            NodeKind::ThrowStatement => {
                if node.left != NULL { self.compile_node(ast, node.left as usize); }
                self.emit(OP_THROW);
            }

            // ─── Try/Catch ───
            NodeKind::TryStatement => {
                let try_start = self.emit_jump(OP_TRY_START);
                if node.left != NULL { self.compile_node(ast, node.left as usize); }
                self.emit(OP_TRY_END);
                let end_jump = self.emit_jump(OP_JUMP);
                self.patch_jump(try_start);

                if node.right != NULL {
                    let catch_node = ast.nodes[node.right as usize];
                    self.begin_scope();
                    let catch_name = self.intern_name(&catch_node);
                    let slot = self.declare_local(catch_name);
                    self.emit(OP_CATCH_BIND);
                    self.emit_u16(slot);
                    if catch_node.left != NULL {
                        self.compile_node(ast, catch_node.left as usize);
                    }
                    self.end_scope();
                }
                self.patch_jump(end_jump);
            }

            // ─── Switch Statement ───
            NodeKind::SwitchStatement => {
                // Compile discriminant
                if node.left != NULL {
                    self.compile_node(ast, node.left as usize);
                }

                // Compile each case
                let saved_break = self.break_count;
                let mut case_idx = node.right;
                let mut end_patches: [usize; 32] = [0; 32];
                let mut end_count = 0;

                while case_idx != NULL && (case_idx as usize) < ast.count {
                    let case_node = ast.nodes[case_idx as usize];

                    if case_node.left != NULL {
                        // case with test
                        self.emit(OP_DUP);
                        self.compile_node(ast, case_node.left as usize);
                        self.emit(OP_SEQ);
                        let skip = self.emit_jump(OP_JUMP_FALSE);

                        // Case body
                        let mut body = case_node.right;
                        while body != NULL && (body as usize) < ast.count {
                            self.compile_node(ast, body as usize);
                            body = ast.nodes[body as usize].next;
                        }
                        if end_count < 32 {
                            end_patches[end_count] = self.emit_jump(OP_JUMP);
                            end_count += 1;
                        }
                        self.patch_jump(skip);
                    } else {
                        // default case
                        let mut body = case_node.right;
                        while body != NULL && (body as usize) < ast.count {
                            self.compile_node(ast, body as usize);
                            body = ast.nodes[body as usize].next;
                        }
                    }

                    case_idx = case_node.next;
                }

                // Patch end jumps
                for i in 0..end_count {
                    self.patch_jump(end_patches[i]);
                }
                // Patch break statements
                for i in saved_break..self.break_count {
                    self.patch_jump(self.break_patches[i]);
                }
                self.break_count = saved_break;
                // Pop discriminant
                self.emit(OP_POP);
            }

            // ─── Class Declaration ───
            // Desugar: class Foo { constructor(x) { this.x = x; } method() {} }
            // becomes: function Foo(x) { this.x = x; } Foo.prototype.method = function() {};
            NodeKind::ClassDecl => {
                let class_name = self.intern_name(&node);
                // Find constructor and compile as main function
                let mut ctor_found = false;
                let mut method = node.left;
                while method != NULL && (method as usize) < ast.count {
                    let mnode = ast.nodes[method as usize];
                    let mname_bytes = &mnode.name[..mnode.name_len];
                    if mname_bytes == b"constructor" {
                        // Compile constructor as the class function
                        let proto_idx = self.compile_function(ast, &mnode, false);
                        self.emit(OP_CLOSURE);
                        self.emit_u16(proto_idx);
                        let slot = self.declare_local(class_name);
                        self.emit(OP_SET_LOCAL);
                        self.emit_u16(slot);
                        self.emit(OP_POP);
                        ctor_found = true;
                        break;
                    }
                    method = mnode.next;
                }

                if !ctor_found {
                    // No constructor — create empty function
                    self.emit(OP_UNDEFINED); // placeholder
                    let slot = self.declare_local(class_name);
                    self.emit(OP_SET_LOCAL);
                    self.emit_u16(slot);
                    self.emit(OP_POP);
                }

                // Compile non-constructor methods onto prototype
                let mut method = node.left;
                while method != NULL && (method as usize) < ast.count {
                    let mnode = ast.nodes[method as usize];
                    let mname_bytes = &mnode.name[..mnode.name_len];
                    if mname_bytes != b"constructor" {
                        // ClassName.prototype.methodName = function() { ... }
                        let mname = self.intern_name(&mnode);
                        // Get class local
                        if let Some(slot) = self.resolve_local(class_name) {
                            self.emit(OP_GET_LOCAL);
                            self.emit_u16(slot);
                        } else {
                            let ci = self.add_const_str(class_name);
                            self.emit(OP_GET_GLOBAL);
                            self.emit_u16(ci);
                        }
                        // Get .prototype
                        let proto_str = self.strings.intern(b"prototype");
                        let ci_proto = self.add_const_str(proto_str);
                        self.emit(OP_GET_PROP);
                        self.emit_u16(ci_proto);
                        // Compile method
                        let proto_idx = self.compile_function(ast, &mnode, false);
                        self.emit(OP_CLOSURE);
                        self.emit_u16(proto_idx);
                        // Set property
                        let ci_mname = self.add_const_str(mname);
                        self.emit(OP_DEFINE_PROP);
                        self.emit_u16(ci_mname);
                        self.emit(OP_POP); // pop prototype obj
                    }
                    method = mnode.next;
                }
            }

            // ─── For...of Statement ───
            NodeKind::ForOfStatement => {
                // for (const x of iterable) { body }
                // Desugar to: var arr = iterable; for (var i = 0; i < arr.length; i++) { var x = arr[i]; body; }
                self.begin_scope();

                // Evaluate iterable into a temp local
                let iter_name = self.strings.intern(b"__iter__");
                let iter_slot = self.declare_local(iter_name);
                if node.right != NULL {
                    self.compile_node(ast, node.right as usize);
                } else {
                    self.emit(OP_UNDEFINED);
                }
                self.emit(OP_SET_LOCAL);
                self.emit_u16(iter_slot);
                self.emit(OP_POP);

                // Counter variable
                let idx_name = self.strings.intern(b"__idx__");
                let idx_slot = self.declare_local(idx_name);
                self.emit(OP_CONST_ZERO);
                self.emit(OP_SET_LOCAL);
                self.emit_u16(idx_slot);
                self.emit(OP_POP);

                // Loop variable from VarDecl
                let mut loop_var_name = StringId::EMPTY;
                if node.left != NULL {
                    let var_node = ast.nodes[node.left as usize];
                    loop_var_name = self.intern_name(&var_node);
                }
                let loop_slot = self.declare_local(loop_var_name);
                self.emit(OP_UNDEFINED);
                self.emit(OP_SET_LOCAL);
                self.emit_u16(loop_slot);
                self.emit(OP_POP);

                let loop_start = self.proto.bytecode_len;
                let saved_break = self.break_count;
                self.continue_target = loop_start;

                // Condition: __idx__ < __iter__.length
                self.emit(OP_GET_LOCAL);
                self.emit_u16(idx_slot);
                self.emit(OP_GET_LOCAL);
                self.emit_u16(iter_slot);
                let len_str = self.strings.intern(b"length");
                let ci_len = self.add_const_str(len_str);
                self.emit(OP_GET_PROP);
                self.emit_u16(ci_len);
                self.emit(OP_LT);
                let exit = self.emit_jump(OP_JUMP_FALSE);

                // Set loop variable: x = __iter__[__idx__]
                self.emit(OP_GET_LOCAL);
                self.emit_u16(iter_slot);
                self.emit(OP_GET_LOCAL);
                self.emit_u16(idx_slot);
                self.emit(OP_GET_ELEM);
                self.emit(OP_SET_LOCAL);
                self.emit_u16(loop_slot);
                self.emit(OP_POP);

                // Body
                if node.body != NULL {
                    self.compile_node(ast, node.body as usize);
                }

                // Increment index
                self.emit(OP_GET_LOCAL);
                self.emit_u16(idx_slot);
                self.emit(OP_INC);
                self.emit(OP_SET_LOCAL);
                self.emit_u16(idx_slot);
                self.emit(OP_POP);

                self.emit_loop(loop_start);
                self.patch_jump(exit);

                for i in saved_break..self.break_count {
                    self.patch_jump(self.break_patches[i]);
                }
                self.break_count = saved_break;
                self.end_scope();
            }

            // ─── Conditional (ternary) ───
            NodeKind::ConditionalExpr => {
                if node.left != NULL { self.compile_node(ast, node.left as usize); }
                let else_jump = self.emit_jump(OP_JUMP_FALSE);
                if node.right != NULL { self.compile_node(ast, node.right as usize); }
                let end_jump = self.emit_jump(OP_JUMP);
                self.patch_jump(else_jump);
                if node.extra != NULL { self.compile_node(ast, node.extra as usize); }
                self.patch_jump(end_jump);
            }

            _ => {}
        }
    }

    // ─── Emit helpers ───

    fn emit(&mut self, op: u8) {
        if self.proto.bytecode_len < MAX_BYTECODE {
            self.proto.bytecode[self.proto.bytecode_len] = op;
            self.proto.bytecode_len += 1;
        }
    }

    fn emit_byte(&mut self, v: u8) { self.emit(v); }

    fn emit_u16(&mut self, v: u16) {
        let b = v.to_le_bytes();
        self.emit(b[0]); self.emit(b[1]);
    }

    fn emit_i32(&mut self, v: i32) {
        let b = v.to_le_bytes();
        for byte in b { self.emit(byte); }
    }

    fn emit_jump(&mut self, op: u8) -> usize {
        self.emit(op);
        let patch = self.proto.bytecode_len;
        self.emit(0); self.emit(0);
        patch
    }

    fn patch_jump(&mut self, patch: usize) {
        let offset = (self.proto.bytecode_len as isize - patch as isize - 2) as i16;
        let b = offset.to_le_bytes();
        self.proto.bytecode[patch] = b[0];
        self.proto.bytecode[patch + 1] = b[1];
    }

    fn emit_loop(&mut self, target: usize) {
        self.emit(OP_LOOP);
        let offset = (target as isize - self.proto.bytecode_len as isize - 2) as i16;
        let b = offset.to_le_bytes();
        self.emit(b[0]); self.emit(b[1]);
    }

    // ─── Constants ───

    fn add_const_f64(&mut self, v: f64) -> u16 {
        for i in 0..self.proto.const_count {
            if let Constant::F64(f) = self.proto.constants[i] {
                if f.to_bits() == v.to_bits() { return i as u16; }
            }
        }
        if self.proto.const_count >= MAX_CONSTS { return 0; }
        let i = self.proto.const_count;
        self.proto.constants[i] = Constant::F64(v);
        self.proto.const_count += 1;
        i as u16
    }

    fn add_const_str(&mut self, sid: StringId) -> u16 {
        for i in 0..self.proto.const_count {
            if let Constant::Str(s) = self.proto.constants[i] {
                if s.0 == sid.0 { return i as u16; }
            }
        }
        if self.proto.const_count >= MAX_CONSTS { return 0; }
        let i = self.proto.const_count;
        self.proto.constants[i] = Constant::Str(sid);
        self.proto.const_count += 1;
        i as u16
    }

    // ─── Scoping ───

    fn begin_scope(&mut self) { self.scope_depth += 1; }

    fn end_scope(&mut self) {
        while self.local_count > 0 && self.locals[self.local_count - 1].depth >= self.scope_depth {
            if self.locals[self.local_count - 1].is_captured {
                // Close upvalue — for now just pop
                self.emit(OP_POP);
            } else {
                self.emit(OP_POP);
            }
            self.local_count -= 1;
        }
        if self.scope_depth > 0 { self.scope_depth -= 1; }
    }

    fn declare_local(&mut self, name: StringId) -> u16 {
        let slot = self.local_count as u16;
        if self.local_count < MAX_LOCALS {
            self.locals[self.local_count] = Local { name, depth: self.scope_depth, slot, is_captured: false };
            self.local_count += 1;
        }
        slot
    }

    fn resolve_local(&self, name: StringId) -> Option<u16> {
        for i in (0..self.local_count).rev() {
            if self.locals[i].name.0 == name.0 { return Some(self.locals[i].slot); }
        }
        None
    }

    /// Resolve a variable in enclosing scopes as an upvalue.
    fn resolve_upvalue(&mut self, name: StringId) -> Option<u8> {
        if self.enclosing_depth == 0 {
            return None;
        }

        // Check if the variable is in the immediately enclosing scope's locals
        let enc_idx = self.enclosing_depth - 1;
        if let Some(ref enc) = self.enclosing[enc_idx] {
            for i in (0..enc.local_count).rev() {
                if enc.locals[i].name.0 == name.0 {
                    // Found in enclosing locals — add upvalue that captures a local
                    return Some(self.add_upvalue(true, enc.locals[i].slot as u8, name));
                }
            }
            // Check enclosing upvalues
            for i in 0..enc.upvalue_count {
                if enc.upvalues[i].name.0 == name.0 {
                    return Some(self.add_upvalue(false, i as u8, name));
                }
            }
        }
        None
    }

    fn add_upvalue(&mut self, is_local: bool, index: u8, name: StringId) -> u8 {
        // Check if we already have this upvalue
        for i in 0..self.upvalue_count {
            if self.upvalues[i].is_local == is_local && self.upvalues[i].index == index {
                return i as u8;
            }
        }
        if self.upvalue_count >= MAX_UPVALUES {
            return 0;
        }
        let idx = self.upvalue_count;
        self.upvalues[idx] = CompilerUpvalue { is_local, index, name };
        self.upvalue_count += 1;
        idx as u8
    }

    fn intern_name(&mut self, node: &AstNode) -> StringId {
        self.strings.intern(&node.name[..node.name_len])
    }
}

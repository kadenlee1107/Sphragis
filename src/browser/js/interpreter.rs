#![allow(dead_code)]
// Bat_OS — JavaScript Interpreter
// Tree-walking interpreter that executes the AST.
// Evaluates expressions, executes statements, manages scopes.

use super::ast::*;
use super::runtime::*;
use crate::drivers::uart;

const MAX_SCOPES: usize = 16;
const MAX_CALL_DEPTH: usize = 32;

/// Console output buffer (for console.log)
pub static mut CONSOLE_OUTPUT: [u8; 4096] = [0; 4096];
pub static mut CONSOLE_LEN: usize = 0;

/// Execution result
pub enum ExecResult {
    Value(JsValue),
    Return(JsValue),
    Break,
    Continue,
    Error,
}

/// JavaScript execution engine
pub struct Engine {
    pub scopes: [Scope; MAX_SCOPES],
    pub scope_count: usize,
    pub current_scope: usize,
    pub call_depth: usize,
}

impl Engine {
    pub const fn new() -> Self {
        Engine {
            scopes: [Scope::new(); MAX_SCOPES],
            scope_count: 0,
            current_scope: 0,
            call_depth: 0,
        }
    }

    pub fn init(&mut self) {
        self.scope_count = 1;
        self.current_scope = 0;
        self.scopes[0] = Scope::new();
        self.call_depth = 0;

        // Built-in globals
        self.set_var("undefined", JsValue::undefined());
        self.set_var("NaN", JsValue::number(f64::NAN));
        self.set_var("Infinity", JsValue::number(f64::INFINITY));
    }

    fn push_scope(&mut self) {
        if self.scope_count < MAX_SCOPES {
            let parent = self.current_scope as u16;
            self.scopes[self.scope_count] = Scope::new();
            self.scopes[self.scope_count].parent = parent;
            self.current_scope = self.scope_count;
            self.scope_count += 1;
        }
    }

    fn pop_scope(&mut self) {
        if self.current_scope > 0 {
            let parent = self.scopes[self.current_scope].parent;
            self.current_scope = parent as usize;
        }
    }

    fn get_var(&self, name: &str) -> JsValue {
        let mut scope_idx = self.current_scope;
        loop {
            if let Some(val) = self.scopes[scope_idx].get(name) {
                return val;
            }
            let parent = self.scopes[scope_idx].parent;
            if parent == 0xFFFF || scope_idx == 0 { break; }
            scope_idx = parent as usize;
        }
        JsValue::undefined()
    }

    fn set_var(&mut self, name: &str, value: JsValue) {
        self.scopes[self.current_scope].set(name, value);
    }

    /// Execute an AST program
    pub fn execute(&mut self, ast: &Ast) -> JsValue {
        self.init();
        if AstNode::is_null(ast.root) { return JsValue::undefined(); }
        self.exec_node(ast, ast.root)
    }

    fn exec_node(&mut self, ast: &Ast, idx: u16) -> JsValue {
        if AstNode::is_null(idx) { return JsValue::undefined(); }
        let node = ast.get(idx);

        match node.kind {
            NodeKind::Program => self.exec_statements(ast, node.left),
            NodeKind::ExprStatement => self.eval_expr(ast, node.left),
            NodeKind::VarDecl => {
                let init = if !AstNode::is_null(node.left) {
                    self.eval_expr(ast, node.left)
                } else {
                    JsValue::undefined()
                };
                self.set_var(node.name_str(), init);
                JsValue::undefined()
            }
            NodeKind::BlockStatement => {
                self.push_scope();
                let result = self.exec_statements(ast, node.left);
                self.pop_scope();
                result
            }
            NodeKind::IfStatement => {
                let cond = self.eval_expr(ast, node.left);
                if cond.is_truthy() {
                    self.exec_node(ast, node.right)
                } else if !AstNode::is_null(node.extra) {
                    self.exec_node(ast, node.extra)
                } else {
                    JsValue::undefined()
                }
            }
            NodeKind::WhileStatement => {
                let mut result = JsValue::undefined();
                for _ in 0..10000 { // safety limit
                    let cond = self.eval_expr(ast, node.left);
                    if !cond.is_truthy() { break; }
                    result = self.exec_node(ast, node.right);
                }
                result
            }
            NodeKind::ForStatement => {
                self.push_scope();
                if !AstNode::is_null(node.left) { self.exec_node(ast, node.left); }
                let mut result = JsValue::undefined();
                for _ in 0..10000 {
                    if !AstNode::is_null(node.right) {
                        let cond = self.eval_expr(ast, node.right);
                        if !cond.is_truthy() { break; }
                    }
                    result = self.exec_node(ast, node.body);
                    if !AstNode::is_null(node.extra) { self.eval_expr(ast, node.extra); }
                }
                self.pop_scope();
                result
            }
            NodeKind::ReturnStatement => {
                let val = if !AstNode::is_null(node.left) {
                    self.eval_expr(ast, node.left)
                } else {
                    JsValue::undefined()
                };
                val // Note: proper return handling needs unwinding
            }
            NodeKind::FunctionDecl => {
                let mut func_val = JsValue::undefined();
                func_val.js_type = JsType::Function;
                func_val.func_node = idx;
                if node.name_len > 0 {
                    self.set_var(node.name_str(), func_val);
                }
                func_val
            }
            _ => self.eval_expr(ast, idx),
        }
    }

    fn exec_statements(&mut self, ast: &Ast, first: u16) -> JsValue {
        let mut result = JsValue::undefined();
        let mut current = first;
        while !AstNode::is_null(current) {
            result = self.exec_node(ast, current);
            current = ast.get(current).next;
        }
        result
    }

    fn eval_expr(&mut self, ast: &Ast, idx: u16) -> JsValue {
        if AstNode::is_null(idx) { return JsValue::undefined(); }
        let node = ast.get(idx);

        match node.kind {
            NodeKind::NumberLiteral => JsValue::number(node.num_value),
            NodeKind::StringLiteral => JsValue::string(&node.name[..node.name_len]),
            NodeKind::BoolLiteral => JsValue::boolean(node.bool_value),
            NodeKind::NullLiteral => JsValue::null(),
            NodeKind::Identifier => self.get_var(node.name_str()),
            NodeKind::BinaryExpr => {
                let left = self.eval_expr(ast, node.left);
                let right = self.eval_expr(ast, node.right);
                self.eval_binary(node.op, &left, &right)
            }
            NodeKind::UnaryExpr => {
                let operand = self.eval_expr(ast, node.left);
                match node.op {
                    Operator::Not => JsValue::boolean(!operand.is_truthy()),
                    Operator::Sub => JsValue::number(-operand.to_number()),
                    Operator::Typeof => {
                        let t = match operand.js_type {
                            JsType::Undefined => b"undefined" as &[u8],
                            JsType::Null => b"object",
                            JsType::Boolean => b"boolean",
                            JsType::Number => b"number",
                            JsType::String => b"string",
                            JsType::Function => b"function",
                            _ => b"object",
                        };
                        JsValue::string(t)
                    }
                    _ => operand,
                }
            }
            NodeKind::AssignExpr => {
                let right = self.eval_expr(ast, node.right);
                let left_node = ast.get(node.left);
                if left_node.kind == NodeKind::Identifier {
                    let val = match node.op {
                        Operator::Assign => right,
                        Operator::PlusAssign => {
                            let cur = self.get_var(left_node.name_str());
                            self.eval_binary(Operator::Add, &cur, &right)
                        }
                        Operator::MinusAssign => {
                            let cur = self.get_var(left_node.name_str());
                            self.eval_binary(Operator::Sub, &cur, &right)
                        }
                        _ => right,
                    };
                    self.set_var(left_node.name_str(), val);
                    val
                } else {
                    right
                }
            }
            NodeKind::CallExpr => self.eval_call(ast, idx),
            _ => JsValue::undefined(),
        }
    }

    fn eval_binary(&self, op: Operator, left: &JsValue, right: &JsValue) -> JsValue {
        // String concatenation
        if op == Operator::Add && (left.js_type == JsType::String || right.js_type == JsType::String) {
            let mut buf = [0u8; MAX_STRING];
            let l1 = left.to_string_buf(&mut buf);
            let mut buf2 = [0u8; MAX_STRING];
            let l2 = right.to_string_buf(&mut buf2);
            let mut result = [0u8; MAX_STRING];
            let total = (l1 + l2).min(MAX_STRING);
            result[..l1].copy_from_slice(&buf[..l1]);
            let r2 = (total - l1).min(l2);
            result[l1..l1+r2].copy_from_slice(&buf2[..r2]);
            return JsValue::string(&result[..l1+r2]);
        }

        let l = left.to_number();
        let r = right.to_number();

        match op {
            Operator::Add => JsValue::number(l + r),
            Operator::Sub => JsValue::number(l - r),
            Operator::Mul => JsValue::number(l * r),
            Operator::Div => JsValue::number(if r != 0.0 { l / r } else { f64::NAN }),
            Operator::Mod => JsValue::number(if r != 0.0 { l % r } else { f64::NAN }),
            Operator::Equal | Operator::StrictEqual => JsValue::boolean(l == r),
            Operator::NotEqual | Operator::StrictNotEqual => JsValue::boolean(l != r),
            Operator::Less => JsValue::boolean(l < r),
            Operator::Greater => JsValue::boolean(l > r),
            Operator::LessEqual => JsValue::boolean(l <= r),
            Operator::GreaterEqual => JsValue::boolean(l >= r),
            Operator::And => JsValue::boolean(left.is_truthy() && right.is_truthy()),
            Operator::Or => JsValue::boolean(left.is_truthy() || right.is_truthy()),
            Operator::BitAnd => JsValue::number((l as i64 & r as i64) as f64),
            Operator::BitOr => JsValue::number((l as i64 | r as i64) as f64),
            Operator::BitXor => JsValue::number((l as i64 ^ r as i64) as f64),
            Operator::ShiftLeft => JsValue::number(((l as i64) << (r as u32)) as f64),
            Operator::ShiftRight => JsValue::number(((l as i64) >> (r as u32)) as f64),
            _ => JsValue::undefined(),
        }
    }

    fn eval_call(&mut self, ast: &Ast, call_idx: u16) -> JsValue {
        let call = ast.get(call_idx);
        let callee_idx = call.left;
        let callee_node = ast.get(callee_idx);

        // Built-in: console.log
        if callee_node.kind == NodeKind::MemberExpr {
            let obj_node = ast.get(callee_node.left);
            if obj_node.name_str() == "console" && callee_node.name_str() == "log" {
                // Collect arguments and output
                let mut arg_idx = call.extra;
                while !AstNode::is_null(arg_idx) {
                    let val = self.eval_expr(ast, arg_idx);
                    let mut buf = [0u8; 128];
                    let len = val.to_string_buf(&mut buf);
                    // Write to console output buffer
                    unsafe {
                        let cl = core::ptr::read_volatile(core::ptr::addr_of!(CONSOLE_LEN));
                        if cl + len < 4096 {
                            for j in 0..len {
                                core::ptr::write_volatile(
                                    core::ptr::addr_of_mut!(CONSOLE_OUTPUT).cast::<u8>().add(cl + j),
                                    buf[j],
                                );
                            }
                            core::ptr::write_volatile(
                                core::ptr::addr_of_mut!(CONSOLE_OUTPUT).cast::<u8>().add(cl + len),
                                b' ',
                            );
                            core::ptr::write_volatile(core::ptr::addr_of_mut!(CONSOLE_LEN), cl + len + 1);
                        }
                    }
                    arg_idx = ast.get(arg_idx).next;
                }
                unsafe {
                    let cl = core::ptr::read_volatile(core::ptr::addr_of!(CONSOLE_LEN));
                    if cl < 4096 {
                        core::ptr::write_volatile(
                            core::ptr::addr_of_mut!(CONSOLE_OUTPUT).cast::<u8>().add(cl),
                            b'\n',
                        );
                        core::ptr::write_volatile(core::ptr::addr_of_mut!(CONSOLE_LEN), cl + 1);
                    }
                }
                return JsValue::undefined();
            }

            // Built-in: document.getElementById
            if obj_node.name_str() == "document" && callee_node.name_str() == "getElementById" {
                // Return a placeholder object
                return JsValue::string(b"[Element]");
            }
        }

        // User-defined function call
        let func_val = self.eval_expr(ast, callee_idx);
        if func_val.js_type == JsType::Function && !AstNode::is_null(func_val.func_node) {
            if self.call_depth >= MAX_CALL_DEPTH { return JsValue::undefined(); }
            self.call_depth += 1;

            let func_node = ast.get(func_val.func_node);

            self.push_scope();

            // Bind parameters
            let mut arg_idx = call.extra;
            for i in 0..func_node.param_count as usize {
                let param_idx = func_node.params[i];
                if !AstNode::is_null(param_idx) {
                    let param_name = ast.get(param_idx).name_str();
                    let arg_val = if !AstNode::is_null(arg_idx) {
                        let v = self.eval_expr(ast, arg_idx);
                        arg_idx = ast.get(arg_idx).next;
                        v
                    } else {
                        JsValue::undefined()
                    };
                    self.set_var(param_name, arg_val);
                }
            }

            // Execute function body
            let result = self.exec_node(ast, func_node.body);

            self.pop_scope();
            self.call_depth -= 1;

            return result;
        }

        // Built-in: alert
        if callee_node.kind == NodeKind::Identifier && callee_node.name_str() == "alert" {
            let arg_idx = call.extra;
            if !AstNode::is_null(arg_idx) {
                let val = self.eval_expr(ast, arg_idx);
                let mut buf = [0u8; 128];
                let len = val.to_string_buf(&mut buf);
                uart::puts("[js alert] ");
                uart::puts(unsafe { core::str::from_utf8_unchecked(&buf[..len]) });
                uart::puts("\n");
            }
            return JsValue::undefined();
        }

        JsValue::undefined()
    }
}

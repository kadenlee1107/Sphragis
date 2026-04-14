// Bat_OS — JavaScript AST (Abstract Syntax Tree)
// Defines the node types produced by the parser.
// The interpreter walks this tree to execute JavaScript.

pub const MAX_AST_NODES: usize = 1024;
pub const MAX_IDENT: usize = 32;

#[derive(Clone, Copy, PartialEq)]
pub enum NodeKind {
    Empty,
    // Literals
    NumberLiteral,    // 42, 3.14
    StringLiteral,    // "hello"
    BoolLiteral,      // true, false
    NullLiteral,      // null
    ArrayLiteral,     // [1, 2, 3]
    ObjectLiteral,    // { a: 1 }

    // Expressions
    Identifier,       // foo
    BinaryExpr,       // a + b, a == b
    UnaryExpr,        // !a, -b, typeof x
    AssignExpr,       // a = b
    CallExpr,         // foo(1, 2)
    MemberExpr,       // obj.prop, obj["prop"]
    ConditionalExpr,  // a ? b : c
    ArrowFunc,        // (x) => x + 1
    NewExpr,          // new Foo()

    // Statements
    ExprStatement,    // expr;
    VarDecl,          // var/let/const x = expr;
    BlockStatement,   // { ... }
    IfStatement,      // if (cond) { ... } else { ... }
    WhileStatement,   // while (cond) { ... }
    ForStatement,     // for (init; cond; update) { ... }
    ReturnStatement,  // return expr;
    FunctionDecl,     // function foo(a, b) { ... }
    BreakStatement,   // break;
    ContinueStatement,// continue;
    TryStatement,     // try { ... } catch (e) { ... }
    ThrowStatement,   // throw expr;
    SwitchStatement,  // switch (expr) { case ... }

    // Program
    Program,          // top-level list of statements
}

#[derive(Clone, Copy, PartialEq)]
pub enum Operator {
    None,
    Add, Sub, Mul, Div, Mod,
    Equal, StrictEqual, NotEqual, StrictNotEqual,
    Less, Greater, LessEqual, GreaterEqual,
    And, Or, Not,
    BitAnd, BitOr, BitXor, BitNot,
    ShiftLeft, ShiftRight,
    Assign, PlusAssign, MinusAssign, StarAssign, SlashAssign,
    Typeof, Void, Delete, In, Of,
    Increment, Decrement,
}

#[derive(Clone, Copy)]
pub struct AstNode {
    pub kind: NodeKind,
    pub op: Operator,

    // Value storage
    pub name: [u8; MAX_IDENT],
    pub name_len: usize,
    pub num_value: f64,
    pub bool_value: bool,

    // Children (indices into AST array)
    pub left: u16,       // left operand / condition / callee
    pub right: u16,      // right operand / then-body
    pub extra: u16,      // else-body / update / arguments list
    pub next: u16,       // next statement in sequence / next arg

    // For function declarations
    pub param_count: u8,
    pub params: [u16; 8], // indices of parameter name nodes
    pub body: u16,        // function body block
}

const NULL: u16 = 0xFFFF;

impl AstNode {
    pub const fn empty() -> Self {
        AstNode {
            kind: NodeKind::Empty,
            op: Operator::None,
            name: [0; MAX_IDENT],
            name_len: 0,
            num_value: 0.0,
            bool_value: false,
            left: NULL, right: NULL, extra: NULL, next: NULL,
            param_count: 0,
            params: [NULL; 8],
            body: NULL,
        }
    }

    pub fn name_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.name[..self.name_len]) }
    }

    pub fn set_name(&mut self, s: &[u8]) {
        let len = s.len().min(MAX_IDENT);
        self.name[..len].copy_from_slice(&s[..len]);
        self.name_len = len;
    }

    pub fn is_null(idx: u16) -> bool { idx == NULL }
}

/// AST arena
pub struct Ast {
    pub nodes: [AstNode; MAX_AST_NODES],
    pub count: usize,
    pub root: u16, // index of Program node
}

impl Ast {
    pub const fn new() -> Self {
        Ast {
            nodes: [AstNode::empty(); MAX_AST_NODES],
            count: 0,
            root: NULL,
        }
    }

    pub fn alloc(&mut self) -> Option<u16> {
        if self.count >= MAX_AST_NODES { return None; }
        let idx = self.count;
        self.nodes[idx] = AstNode::empty();
        self.count += 1;
        Some(idx as u16)
    }

    pub fn get(&self, idx: u16) -> &AstNode {
        &self.nodes[idx as usize]
    }

    pub fn get_mut(&mut self, idx: u16) -> &mut AstNode {
        &mut self.nodes[idx as usize]
    }
}

// Bat_OS — JavaScript Engine
// Bytecode-compiled VM with NaN-boxed values, string interning,
// arena-allocated objects, and native function bindings.

// ─── Core engine (new bytecode VM) ───
pub mod value;      // NaN-boxed JsValue (8 bytes per value)
pub mod strings;    // String intern table
pub mod opcodes;    // Bytecode instruction set (~80 opcodes)
pub mod object;     // Object heap (arena-allocated, property storage)
pub mod compiler;   // AST → bytecode compiler
pub mod vm;         // Stack-based bytecode VM

// ─── Built-in methods ───
pub mod builtins;   // Array/String/Object/Number prototype methods
pub mod dom_api;    // DOM integration (document, Element, window)

// ─── Frontend (shared with old interpreter) ───
pub mod ast;        // AST node types + arena
pub mod lexer;      // Tokenizer
pub mod parser;     // Recursive descent parser

// ─── Web APIs ───
pub mod canvas;     // Canvas 2D API
pub mod storage;    // Web Storage API

// ─── Legacy (kept for compatibility, will be removed) ───
pub mod interpreter;
pub mod runtime;

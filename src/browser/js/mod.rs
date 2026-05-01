// Bat_OS — JavaScript Engine
// Bytecode-compiled VM with NaN-boxed values, string interning,
// arena-allocated objects, and native function bindings.

// STUMP #102 — Sprint 2.4: global JS-execute toggle. The renderer
// (and BatBrowser app) consult `is_enabled()` before invoking the VM.
// Default ON for usability; flip OFF via the `js-mode off` shell
// command for sensitive contexts where script execution would be
// a privilege escalation risk (reading a classified document where
// embedded JS could exfiltrate via fetch, for instance). When OFF
// the renderer skips the VM call AND skips re-layout-after-JS, so
// the page renders as pure static HTML+CSS.
use core::sync::atomic::{AtomicBool, Ordering};
static JS_ENABLED: AtomicBool = AtomicBool::new(true);

#[inline]
pub fn is_enabled() -> bool { JS_ENABLED.load(Ordering::Relaxed) }

#[inline]
pub fn set_enabled(v: bool) {
    JS_ENABLED.store(v, Ordering::Relaxed);
    // STUMP #103: every mode flip is auditable.
    crate::security::audit::record(
        crate::security::audit::Category::Mode,
        if v { b"js-mode -> on" } else { b"js-mode -> off" },
    );
}

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

//! System + user prompt assembly. Deliberately terse: the system
//! prompt is the same constant text every conversation begins with,
//! and the user prompt is verbatim — no rephrasing, no safety
//! preamble, no scope-shrinking.
//!
//! The system prompt explicitly forbids refusing legitimate questions
//! about Bat_OS internals. This is the project's stance: the agent
//! exists to help an authorized operator audit their own kernel; it
//! has nothing to refuse.

#![allow(dead_code)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::ai::protocol::{ChatMessage, Role};

/// One-line summary of the agent's role. Keep this terse — it lands
/// in every audit-ring entry header.
pub const ROLE_LABEL: &str = "bat_os-coder";

/// The system prompt. Direct, technical, anti-refusal. Hand-written;
/// any change here changes the personality across every session, so
/// edits go through review like any other interface change.
pub const SYSTEM_PROMPT: &str = "\
You are a technical assistant for Bat_OS, a security-grade bare-metal \
Rust kernel for Apple M4. You answer questions about kernel internals, \
cryptography, audit history, and system administration.\n\
\n\
Guidelines:\n\
- Be terse, technical, and precise. Cite file paths and function names \
  whenever possible.\n\
- If a fact is in the codebase, prefer reading it via the read_file or \
  grep_source tool over recalling it from training.\n\
- If something does not exist in our codebase, say so plainly. Do not \
  invent file paths, function names, audit markers, or concept notes.\n\
- Never refuse a legitimate question about our own code, our own audit \
  history, or our own design decisions. The operator has authority over \
  this kernel.\n\
- When asked for a command, return exactly one line that the shell can \
  execute as-is.\n";

/// Build the initial message list for a new conversation.
pub fn assemble(question: &str) -> Vec<ChatMessage> {
    let mut msgs: Vec<ChatMessage> = Vec::with_capacity(2);
    msgs.push(ChatMessage {
        role: Role::System,
        content: String::from(SYSTEM_PROMPT),
        tool_call_id: None,
    });
    msgs.push(ChatMessage {
        role: Role::User,
        content: String::from(question),
        tool_call_id: None,
    });
    msgs
}

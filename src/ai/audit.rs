//! Per-session audit-ring helpers for the AI agent.
//!
//! Three event shapes:
//!
//! - **session-start** — emitted by `AgentSession::new()` once we
//!   have an outbound TLS pcb and a session id. Carries the user's
//!   first question (truncated to fit the 192-byte msg slot).
//! - **tool-call** — emitted by `tools::dispatch` for every tool the
//!   model invokes. Carries `name + truncated args`.
//! - **session-end** — emitted by `AgentSession::close()`. Carries
//!   total tokens generated and a 0/1 ok flag. Always written, even
//!   on errors, so the ring records "session existed but failed."
//!
//! All three call into `crate::security::audit::record(Category::Ai, ...)`,
//! so they're sealed under the same master-key AEAD as every other
//! audit event.

#![allow(dead_code)]

use crate::security::audit::{record, Category};

/// Emit one session-start entry. `session_id` is the correlation
/// token the operator uses to group all entries for one conversation.
pub fn log_session_start(session_id: u64, question: &str) {
    let mut buf = [0u8; 160];
    let n = render(&mut buf, b"start", session_id, question.as_bytes());
    record(Category::Ai, &buf[..n]);
}

/// Emit one tool-call entry. Args are truncated to fit the 192-byte
/// audit-msg slot; the model can always re-issue the call if a
/// reviewer needs the full args.
pub fn log_tool_call(session_id: u64, name: &str, args: &str) {
    let mut buf = [0u8; 160];
    // Compose "<name>: <args>" then hand off to render.
    let mut combined = [0u8; 140];
    let mut k = 0;
    for &b in name.as_bytes() {
        if k >= combined.len() { break; }
        combined[k] = b;
        k += 1;
    }
    if k + 2 < combined.len() {
        combined[k] = b':';
        combined[k + 1] = b' ';
        k += 2;
    }
    for &b in args.as_bytes() {
        if k >= combined.len() { break; }
        combined[k] = b;
        k += 1;
    }
    let n = render(&mut buf, b"tool", session_id, &combined[..k]);
    record(Category::Ai, &buf[..n]);
}

/// Emit one session-end entry. `ok` is false if the session ended
/// via interrupt or error, true on a clean Done.
pub fn log_session_end(session_id: u64, tokens: u32, ok: bool) {
    let mut buf  = [0u8; 160];
    let mut tail = [0u8; 32];
    let mut k = 0;
    for &b in b"toks=" {
        if k >= tail.len() { break; }
        tail[k] = b;
        k += 1;
    }
    k = push_u32(&mut tail, k, tokens);
    if k + 4 < tail.len() {
        tail[k] = b' ';
        tail[k + 1] = b'o';
        tail[k + 2] = b'k';
        tail[k + 3] = b'=';
        k += 4;
    }
    if k < tail.len() {
        tail[k] = if ok { b'1' } else { b'0' };
        k += 1;
    }
    let n = render(&mut buf, b"end", session_id, &tail[..k]);
    record(Category::Ai, &buf[..n]);
}

/// Render `event=<event> sid=<sid> <body>` into `out`. Truncates
/// silently if the body is longer than the remaining space — the
/// event tag and sid always survive.
fn render(out: &mut [u8], event: &[u8], session_id: u64, body: &[u8]) -> usize {
    let mut k = 0;
    for &b in b"event=" {
        if k >= out.len() { return k; }
        out[k] = b;
        k += 1;
    }
    for &b in event {
        if k >= out.len() { return k; }
        out[k] = b;
        k += 1;
    }
    if k + 5 < out.len() {
        out[k]     = b' ';
        out[k + 1] = b's';
        out[k + 2] = b'i';
        out[k + 3] = b'd';
        out[k + 4] = b'=';
        k += 5;
    }
    k = push_u64(out, k, session_id);
    if k < out.len() {
        out[k] = b' ';
        k += 1;
    }
    for &b in body {
        if k >= out.len() { return k; }
        out[k] = b;
        k += 1;
    }
    k
}

fn push_u32(out: &mut [u8], mut k: usize, mut n: u32) -> usize {
    if n == 0 {
        if k < out.len() { out[k] = b'0'; k += 1; }
        return k;
    }
    let mut tmp = [0u8; 10];
    let mut i = 0;
    while n > 0 {
        tmp[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        if k >= out.len() { return k; }
        out[k] = tmp[i];
        k += 1;
    }
    k
}

fn push_u64(out: &mut [u8], mut k: usize, mut n: u64) -> usize {
    if n == 0 {
        if k < out.len() { out[k] = b'0'; k += 1; }
        return k;
    }
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while n > 0 {
        tmp[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        if k >= out.len() { return k; }
        out[k] = tmp[i];
        k += 1;
    }
    k
}

//! Bat_OS AI agent — domain-narrow assistant backed by a locally
//! hosted, fine-tuned LLM. Lives in `src/ai/`. See
//! `DESIGN_AI_AGENT.md` and `docs/PLAN_AI_AGENT.md` for the full
//! architecture.
//!
//! Outline:
//!
//! - `protocol`  — wire types (`ChatRequest`, `ChatDelta`, etc.)
//! - `prompt`    — system + user prompt assembly
//! - `client`    — HTTPS client over `crate::net::https`, pinned cert
//! - `stream`    — line-buffered SSE framer
//! - `tools`     — read-only tool dispatch (6 tools)
//! - `rag`       — compile-time BM25 over Concept notes + DESIGN docs
//! - `audit`     — per-session audit-ring helpers (Category::Ai)
//! - `policy`    — cave-policy entry that allowlists the agent's pcb
//!
//! No async runtime. No serde. No std. Cooperative blocking via
//! `StreamingResponse::poll`. See `DESIGN_AI_AGENT.md` §Inference
//! host for the network protocol; we speak the OpenAI-compatible
//! `/v1/chat/completions` endpoint with `Accept: text/event-stream`.
//!
//! Status: Phase 2 scaffolding. Most submodules are stubs that
//! `cargo check` accepts but do not yet do useful work. They are
//! filled in successive PRs per `docs/PLAN_AI_AGENT.md`.

#![allow(dead_code)] // scaffolding — submodules wire up incrementally

pub mod audit;
pub mod client;
pub mod policy;
pub mod prompt;
pub mod protocol;
pub mod rag;
pub(crate) mod rag_corpus;
pub mod stream;
pub mod tools;

use alloc::string::String;

/// Failure modes surfaced to callers. Kept narrow on purpose so the
/// shell `cmd_ai` arm can pretty-print without a giant match.
#[derive(Debug)]
pub enum AgentError {
    /// Caller invoked `interrupt()` while a response was streaming.
    Interrupted,
    /// HTTPS open / TLS handshake / write / read failed. Carries the
    /// underlying `&'static str` from `crate::net::https`.
    Network(&'static str),
    /// Server returned a non-2xx status or a malformed SSE frame.
    Protocol(&'static str),
    /// A tool dispatch failed (e.g. `read_file` on a path that does
    /// not exist). Surfaced as a tool-call result, not a session
    /// kill — the model can recover by trying again.
    Tool(&'static str),
    /// Request exceeded the per-session token budget.
    TokenBudget,
    /// The agent's cave-policy entry rejected the connection (e.g.
    /// pin mismatch, host mismatch). Constant-cost — no detail
    /// leaked to the cave caller.
    PolicyDenied,
}

/// One logical conversation. Carries TLS pcb, audit session id,
/// streaming framer state, and the interrupt flag. Construct via
/// `AgentSession::new()`; tear down via `close()`.
pub struct AgentSession {
    /// `crate::net::https` connection handle. `None` after close().
    pcb: Option<usize>,
    /// Monotonic session id, used as the audit-ring correlation
    /// token across `log_session_start` / `log_tool_call` / `log_session_end`.
    session_id: u64,
    /// Total tokens generated this session. Bumped by `stream::feed`.
    tokens_seen: u32,
    /// Set true by `interrupt()`. Polled by `StreamingResponse::poll`.
    interrupt: bool,
}

impl AgentSession {
    pub fn new() -> Result<Self, AgentError> {
        // Phase 2 stub: returns an inert session. Wire-up happens
        // when `client::open()` lands in Phase 5.
        Ok(Self {
            pcb: None,
            session_id: 0,
            tokens_seen: 0,
            interrupt: false,
        })
    }

    /// Issue one question, returning a streaming response handle.
    /// Caller drives it by polling — see `StreamingResponse::poll`.
    pub fn ask(&mut self, _question: &str) -> StreamingResponse<'_> {
        // Phase 2 stub: emits one `Done` event immediately. The real
        // implementation builds a `protocol::ChatRequest`, sends it
        // via `client::write_request`, and feeds bytes into the
        // `stream::StreamFramer`.
        StreamingResponse {
            session: self,
            done: false,
        }
    }

    /// Best-effort cancel of the in-flight request. The next
    /// `StreamingResponse::poll` will return `StreamEvent::Error(Interrupted)`.
    pub fn interrupt(&mut self) {
        self.interrupt = true;
    }

    /// Tear down the TLS pcb and emit the closing audit entry.
    pub fn close(self) {
        // Phase 2 stub. Real impl: `https::close_pcb(self.pcb)` +
        // `audit::log_session_end(self.tokens_seen, !self.interrupt)`.
    }
}

/// One streaming response. Borrows the session for its lifetime so
/// only one in-flight request is permitted per session — matches the
/// shell's interactive single-line model.
pub struct StreamingResponse<'a> {
    session: &'a mut AgentSession,
    done: bool,
}

/// Events surfaced to the caller as the model generates.
#[derive(Debug)]
pub enum StreamEvent {
    /// One or more text deltas arrived. The shell concatenates these
    /// into a single line for display; richer UIs may stream byte-by-byte.
    Text(String),
    /// The model decided to invoke a tool. The agent layer dispatches,
    /// captures the result, and feeds it back as the next user
    /// message; this event is informational for UIs that want to
    /// show "the agent is searching" hints.
    ToolCall { name: &'static str },
    /// Stream completed normally.
    Done,
    /// Stream completed with an error.
    Error(AgentError),
}

impl<'a> StreamingResponse<'a> {
    /// Cooperative-blocking poll. Returns the next event or
    /// `Done`/`Error` to terminate the loop. Caller should keep
    /// calling until a terminal event.
    pub fn poll(&mut self) -> StreamEvent {
        if self.done {
            return StreamEvent::Done;
        }
        if self.session.interrupt {
            self.done = true;
            return StreamEvent::Error(AgentError::Interrupted);
        }
        // Phase 2 stub: report Done immediately so callers can be
        // wired up + tested without an inference server present.
        self.done = true;
        StreamEvent::Done
    }
}

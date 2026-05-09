//! HTTPS client wrapping `crate::net::https`. Pinned to the
//! operator's self-signed cert via `policy`. Speaks
//! `POST /v1/chat/completions` with `Accept: text/event-stream`.
//!
//! Phase 2 stub: the type shapes exist so `mod.rs` compiles; the
//! actual `open_kernel` call is wired in Phase 5 once the request
//! builder is exercised end-to-end against a local ollama.

#![allow(dead_code)]

use crate::ai::AgentError;

/// Configured at compile time via `BAT_OS_AI_INFERENCE_HOST` and
/// `BAT_OS_AI_INFERENCE_PORT` (defaulted in `build.rs`). Defaults
/// shown here are placeholders; real values come from env at build
/// time per `DESIGN_AI_AGENT.md` §Inference host.
pub const HOST: &str = "10.0.2.42";
pub const PORT: u16  = 443;

/// Open a TLS connection to the inference host. Caller is responsible
/// for `https::close_pcb` on the returned handle.
pub fn open() -> Result<usize, AgentError> {
    // Phase 2 stub. Real impl:
    //   crate::ai::policy::ensure_allowlisted(HOST, PORT)?;
    //   crate::net::https::open_kernel(HOST, PORT)
    //       .map_err(AgentError::Network)
    Err(AgentError::Network("client::open not wired yet"))
}

/// Write the chat-completions request as one HTTP/1.1 message.
/// `body` is the JSON body produced by `protocol::serialize_request`.
pub fn write_request(_pcb: usize, _body: &str) -> Result<(), AgentError> {
    Err(AgentError::Network("client::write_request not wired yet"))
}

/// Read up to `buf.len()` bytes off the response stream. Returns the
/// number of bytes written; 0 means the server half-closed.
pub fn read_chunk(_pcb: usize, _buf: &mut [u8]) -> Result<usize, AgentError> {
    Err(AgentError::Network("client::read_chunk not wired yet"))
}

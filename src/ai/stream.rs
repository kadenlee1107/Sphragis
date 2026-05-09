//! Line-buffered SSE framer. Bytes-in, `ChatDelta`s-out. The
//! transport is HTTP/1.1 chunked + SSE, so the framer holds an
//! internal buffer until it sees a `\n\n` event boundary, splits
//! out the `data: <json>` line, and feeds it to
//! `protocol::parse_delta_line`.
//!
//! Phase 2 stub: the type and `feed`/`drain` shapes are present so
//! `mod.rs` compiles; the actual byte machinery lands in Phase 4.

#![allow(dead_code)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::ai::protocol::{parse_delta_line, ChatDelta};

/// Holds bytes between frame boundaries. One framer per session.
/// Maximum reasonable retained-bytes is the size of one delta line
/// — a few KB. We cap explicitly to detect runaway servers.
pub struct StreamFramer {
    buf: String,
    /// Hard cap on retained bytes (defense-in-depth). If exceeded we
    /// drop the buffer and emit no events for that flush.
    cap: usize,
}

impl StreamFramer {
    pub fn new() -> Self {
        Self {
            buf: String::with_capacity(4096),
            cap: 16 * 1024,
        }
    }

    /// Feed a chunk of network bytes into the framer. Returns any
    /// fully-formed `ChatDelta`s decoded from the chunk.
    pub fn feed(&mut self, chunk: &[u8]) -> Vec<ChatDelta> {
        // Phase 2 stub: just append to the buffer and try to drain
        // line-by-line. Real impl strips `data: ` prefix, splits on
        // `\n\n`, and handles split-mid-frame chunks. Verified
        // against an ollama capture in Phase 4.
        if let Ok(s) = core::str::from_utf8(chunk) {
            self.buf.push_str(s);
        }
        if self.buf.len() > self.cap {
            self.buf.clear();
            return Vec::new();
        }

        let mut events: Vec<ChatDelta> = Vec::new();
        while let Some(idx) = self.buf.find('\n') {
            let line: String = self.buf[..idx].into();
            // Drop the consumed line (and its newline).
            self.buf.replace_range(..=idx, "");
            // Strip the SSE `data: ` prefix if present.
            let payload = line.strip_prefix("data:").unwrap_or(&line).trim();
            if payload.is_empty() { continue; }
            if let Some(d) = parse_delta_line(payload) {
                events.push(d);
            }
        }
        events
    }

    /// Discard buffered bytes — used on stream cancel to avoid
    /// leaking half-frames into the next session.
    pub fn reset(&mut self) {
        self.buf.clear();
    }
}

impl Default for StreamFramer {
    fn default() -> Self { Self::new() }
}

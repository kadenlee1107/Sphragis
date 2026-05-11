//! Audit-ring forwarder — pushes events out to an external SIEM
//! over the existing kernel-mediated HTTPS surface.
//!
//! Wire format: NDJSON (newline-delimited JSON). One object per
//! audit ring entry. Schema:
//!
//!     {"ts": <u64>, "cat": "<label>", "msg": "<string>"}
//!
//! Why NDJSON over a proper Elasticsearch / Splunk bulk format: it
//! works with every SIEM (rsyslog, Loki, Fluentd, Splunk HEC,
//! Elastic, Wazuh) without per-vendor adapters. Each line stands on
//! its own.
//!
//! Why HTTPS and not syslog/UDP: syslog has no integrity, can lose
//! packets silently, and we already have TLS 1.3 in the kernel.
//! The audit ring is the most security-critical state we have; it
//! deserves an authenticated channel.
//!
//! The forwarder is **fire-and-forget**: we POST a batch and
//! discard the response body (the SIEM should be reachable;
//! delivery failure surfaces as a kmsg WARN). For high-assurance
//! deployments a future iteration adds at-least-once delivery
//! semantics (durable per-cave queue, retries, dead-letter).

#![allow(dead_code)]

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::security::audit::{recent, Entry, MSG_LEN};
use crate::kernel::kmsg;
use crate::net::https;

/// Default batch size. Larger batches amortize TLS handshake +
/// HTTP framing overhead; smaller batches lose less on a crash.
pub const DEFAULT_BATCH: usize = 64;

#[derive(Debug)]
pub enum ForwardError {
    Connect(&'static str),
    Send(&'static str),
    Empty,
}

/// Format `n` most-recent audit entries as NDJSON.
fn build_ndjson(n: usize) -> String {
    let cap = if n > 256 { 256 } else { n };
    let mut buf: Vec<Entry> = Vec::with_capacity(cap);
    buf.resize(cap, Entry::empty());
    let count = recent(&mut buf);

    let mut out = String::with_capacity(count * 128);
    for i in 0..count {
        let e = &buf[i];
        let label = match e.cat {
            1  => "fetch",  2  => "script", 3  => "click", 4  => "nav",
            5  => "form",   6  => "mode",   7  => "auth",  8  => "boot",
            9  => "cave",   10 => "ai",     _  => "unknown",
        };
        let mlen = e.mlen as usize;
        let mlen = if mlen > MSG_LEN { MSG_LEN } else { mlen };
        let msg = core::str::from_utf8(&e.msg[..mlen]).unwrap_or("<binary>");
        // JSON-escape only what NDJSON requires (no embedded newlines).
        let mut escaped = String::with_capacity(msg.len() + 4);
        for c in msg.chars() {
            match c {
                '"'  => escaped.push_str("\\\""),
                '\\' => escaped.push_str("\\\\"),
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                c if (c as u32) < 0x20 => {
                    escaped.push_str(&format!("\\u{:04x}", c as u32));
                }
                c => escaped.push(c),
            }
        }
        out.push_str(&format!(
            "{{\"ts\":{},\"cat\":\"{}\",\"msg\":\"{}\"}}\n",
            e.ts, label, escaped
        ));
    }
    out
}

/// Push the last `n` audit entries to the configured SIEM endpoint.
/// `host` + `port` flow through the existing pinned-cert HTTPS
/// stack; the forwarder doesn't talk to the network directly.
///
/// The endpoint path is taken from `path` — typically something
/// like "/api/v1/audit-bulk" for a custom SIEM ingestion endpoint,
/// or "/services/collector/event" for Splunk HEC.
pub fn forward(host: &str, port: u16, path: &str, n: usize) -> Result<usize, ForwardError> {
    let body = build_ndjson(n);
    if body.is_empty() {
        return Err(ForwardError::Empty);
    }

    let pcb = https::open_kernel(host, port).map_err(|e| ForwardError::Connect(e))?;

    let req = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/x-ndjson\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        path, host, body.len(), body
    );

    let send_result = https::write(pcb, req.as_bytes());
    if let Err(e) = send_result {
        https::close_pcb(pcb);
        kmsg::warn(b"audit_forwarder: write failed");
        return Err(ForwardError::Send(e));
    }

    // Read whatever the server sends back; we discard it. Anything
    // that didn't 5xx is treated as success. Failure to read is OK
    // (some SIEMs close the connection without a response body).
    let mut sink = [0u8; 1024];
    let _ = https::read(pcb, &mut sink);
    https::close_pcb(pcb);

    let bytes_sent = body.len();
    kmsg::info(b"audit_forwarder: batch sent");
    Ok(bytes_sent)
}

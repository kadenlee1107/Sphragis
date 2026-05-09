//! Wire types for the OpenAI-compatible `/v1/chat/completions`
//! endpoint exposed by ollama. We hand-roll JSON serialization to
//! avoid pulling `serde` into the kernel image.
//!
//! Only the fields we actually use are modeled. The protocol is
//! stable enough that this can stay narrow.

#![allow(dead_code)]

use alloc::string::String;
use alloc::vec::Vec;

/// Role on a chat message. `Tool` is used to feed tool results back
/// to the model.
#[derive(Debug, Clone, Copy)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::System    => "system",
            Role::User      => "user",
            Role::Assistant => "assistant",
            Role::Tool      => "tool",
        }
    }
}

/// One message in the chat history. `tool_call_id` is set on `Tool`
/// messages to bind a result back to the call that produced it.
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub tool_call_id: Option<String>,
}

/// Tool definition broadcast to the model. JSON Schema-shaped.
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    /// Pre-rendered parameters JSON Schema as a string literal so we
    /// don't reconstruct it every request.
    pub parameters_json: &'static str,
}

/// `tool_choice` field on the request — `None` means the server's
/// default behavior; `Required` forces tool use; `Auto` lets the
/// model decide. We send `Auto` for free-form questions and
/// `Required` for scripted ones (e.g. `cmd_ai_selftest`).
#[derive(Debug, Clone, Copy)]
pub enum ToolChoice {
    Auto,
    Required,
    None,
}

/// Outbound request. Built by `prompt::assemble` and serialized by
/// `serialize_request`.
pub struct ChatRequest<'a> {
    pub model: &'a str,
    pub messages: &'a [ChatMessage],
    pub tools: &'a [ToolDef],
    pub tool_choice: ToolChoice,
    pub temperature: f32,
    pub max_tokens: u32,
    pub stream: bool,
}

/// One delta chunk parsed off the SSE stream. Either text content,
/// a tool-call fragment, or the finish marker.
#[derive(Debug)]
pub struct ChatDelta {
    pub text: Option<String>,
    pub tool_call: Option<ToolCallDelta>,
    pub finish_reason: Option<FinishReason>,
}

/// Tool calls stream in fragments — name first, then arguments
/// piece-by-piece. We accumulate them in `client::SessionState`.
#[derive(Debug)]
pub struct ToolCallDelta {
    /// Index of the tool call in the assistant message (always 0
    /// today; multi-call is future scope).
    pub index: u32,
    pub id: Option<String>,
    pub name: Option<String>,
    pub args_fragment: Option<String>,
}

/// Why the stream ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    /// Model returned an unrecognized reason; surfaced for forensics.
    Other,
}

/// Build the JSON body for a request. Hand-rolled so we don't drag
/// in `serde`/`serde_json`. The output is a `String` because TLS
/// `write` consumes a borrowed slice — caller passes `body.as_bytes()`.
pub fn serialize_request(req: &ChatRequest<'_>) -> String {
    let mut s = String::with_capacity(512 + req.messages.len() * 256);
    s.push('{');

    s.push_str("\"model\":");
    push_json_string(&mut s, req.model);
    s.push(',');

    s.push_str("\"stream\":");
    s.push_str(if req.stream { "true" } else { "false" });
    s.push(',');

    s.push_str("\"temperature\":");
    push_f32(&mut s, req.temperature);
    s.push(',');

    s.push_str("\"max_tokens\":");
    push_u32(&mut s, req.max_tokens);
    s.push(',');

    s.push_str("\"messages\":[");
    for (i, m) in req.messages.iter().enumerate() {
        if i > 0 { s.push(','); }
        s.push('{');
        s.push_str("\"role\":");
        push_json_string(&mut s, m.role.as_str());
        s.push(',');
        s.push_str("\"content\":");
        push_json_string(&mut s, &m.content);
        if let Some(tcid) = &m.tool_call_id {
            s.push(',');
            s.push_str("\"tool_call_id\":");
            push_json_string(&mut s, tcid);
        }
        s.push('}');
    }
    s.push(']');

    if !req.tools.is_empty() {
        s.push_str(",\"tools\":[");
        for (i, t) in req.tools.iter().enumerate() {
            if i > 0 { s.push(','); }
            s.push_str("{\"type\":\"function\",\"function\":{");
            s.push_str("\"name\":");
            push_json_string(&mut s, t.name);
            s.push(',');
            s.push_str("\"description\":");
            push_json_string(&mut s, t.description);
            s.push(',');
            s.push_str("\"parameters\":");
            // parameters_json is pre-validated string; emit verbatim.
            s.push_str(t.parameters_json);
            s.push_str("}}");
        }
        s.push(']');

        s.push_str(",\"tool_choice\":");
        match req.tool_choice {
            ToolChoice::Auto     => s.push_str("\"auto\""),
            ToolChoice::Required => s.push_str("\"required\""),
            ToolChoice::None     => s.push_str("\"none\""),
        }
    }

    s.push('}');
    s
}

/// Parse one decoded SSE `data:` line into a `ChatDelta`. Returns
/// `None` on `[DONE]` (the server's stream-end marker) or on lines
/// we don't understand — caller treats both as "no event for this line."
///
/// This is a hand-written shallow parser; it only inspects the
/// fields we care about (`choices[0].delta.content`,
/// `choices[0].delta.tool_calls[0]`, `choices[0].finish_reason`)
/// and ignores everything else. Robust against unknown future fields.
pub fn parse_delta_line(line: &str) -> Option<ChatDelta> {
    let line = line.trim();
    if line.is_empty() || line == "[DONE]" {
        return None;
    }

    let text = scan_string_field(line, "\"content\"");
    let finish = scan_finish_reason(line);
    let tool_call = scan_tool_call_delta(line);

    if text.is_none() && finish.is_none() && tool_call.is_none() {
        return None;
    }
    Some(ChatDelta { text, tool_call, finish_reason: finish })
}

/// Find `"<key>":"..."` and return the unescaped value. Returns
/// `None` if the key is absent or the value is `null`.
fn scan_string_field(haystack: &str, key: &str) -> Option<String> {
    let pos = haystack.find(key)?;
    let after = &haystack[pos + key.len()..];
    let after = skip_colon_ws(after)?;
    if after.starts_with("null") {
        return None;
    }
    if !after.starts_with('"') {
        return None;
    }
    decode_json_string(&after[1..])
}

/// Pop ASCII whitespace and a single `:`.
fn skip_colon_ws(s: &str) -> Option<&str> {
    let s = s.trim_start();
    let s = s.strip_prefix(':')?;
    Some(s.trim_start())
}

/// Decode a JSON string starting just after the opening quote. Stops
/// at the closing quote. Returns `None` on truncation.
fn decode_json_string(after_open: &str) -> Option<String> {
    let mut out = String::new();
    let mut iter = after_open.chars();
    while let Some(c) = iter.next() {
        match c {
            '"' => return Some(out),
            '\\' => {
                let esc = iter.next()?;
                match esc {
                    '"'  => out.push('"'),
                    '\\' => out.push('\\'),
                    '/'  => out.push('/'),
                    'n'  => out.push('\n'),
                    't'  => out.push('\t'),
                    'r'  => out.push('\r'),
                    'b'  => out.push('\u{0008}'),
                    'f'  => out.push('\u{000C}'),
                    'u'  => {
                        // Read 4 hex digits.
                        let mut code: u32 = 0;
                        for _ in 0..4 {
                            let d = iter.next()?;
                            let n = d.to_digit(16)?;
                            code = (code << 4) | n;
                        }
                        if let Some(ch) = char::from_u32(code) {
                            out.push(ch);
                        }
                    }
                    _ => out.push(esc),
                }
            }
            _ => out.push(c),
        }
    }
    None
}

fn scan_finish_reason(haystack: &str) -> Option<FinishReason> {
    let s = scan_string_field(haystack, "\"finish_reason\"")?;
    Some(match s.as_str() {
        "stop"        => FinishReason::Stop,
        "length"      => FinishReason::Length,
        "tool_calls"  => FinishReason::ToolCalls,
        _             => FinishReason::Other,
    })
}

/// Hardcoded `index: 0` per design — multi-tool-call streams are
/// future scope. Pulls fragments out of the first tool-call slot.
fn scan_tool_call_delta(haystack: &str) -> Option<ToolCallDelta> {
    if !haystack.contains("\"tool_calls\"") {
        return None;
    }
    let id   = scan_string_field(haystack, "\"id\"");
    let name = scan_string_field(haystack, "\"name\"");
    let args = scan_string_field(haystack, "\"arguments\"");
    if id.is_none() && name.is_none() && args.is_none() {
        return None;
    }
    Some(ToolCallDelta {
        index: 0,
        id,
        name,
        args_fragment: args,
    })
}

/// Push a JSON-escaped string literal (including surrounding quotes).
fn push_json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"'  => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                push_u4_hex(out, c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn push_u4_hex(out: &mut String, code: u32) {
    out.push_str("\\u");
    for shift in (0..4).rev() {
        let nib = ((code >> (shift * 4)) & 0xf) as u8;
        let c = if nib < 10 { b'0' + nib } else { b'a' + (nib - 10) };
        out.push(c as char);
    }
}

fn push_u32(out: &mut String, n: u32) {
    if n == 0 { out.push('0'); return; }
    let mut buf = [0u8; 10];
    let mut i = buf.len();
    let mut x = n;
    while x > 0 {
        i -= 1;
        buf[i] = b'0' + (x % 10) as u8;
        x /= 10;
    }
    for &b in &buf[i..] {
        out.push(b as char);
    }
}

/// Crude f32 -> string. Three decimal places, no scientific notation.
/// Adequate for `temperature` which is always a small dial (0.0..=2.0).
fn push_f32(out: &mut String, x: f32) {
    let neg = x < 0.0;
    let v = if neg { -x } else { x };
    let int_part  = v as u32;
    let frac_part = ((v - int_part as f32) * 1000.0) as u32;
    if neg { out.push('-'); }
    push_u32(out, int_part);
    out.push('.');
    if frac_part < 100 { out.push('0'); }
    if frac_part < 10  { out.push('0'); }
    push_u32(out, frac_part);
}

/// Preserve a list of `Vec<ChatMessage>` callers may want to build
/// up incrementally. Re-export here for convenience.
pub type Messages = Vec<ChatMessage>;

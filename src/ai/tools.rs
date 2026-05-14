//! Tool dispatch table. Six read-only tools; the model can invoke
//! any of them on its own initiative, but none of them mutate the
//! kernel or the filesystem.
//!
//! 1. `read_file`         — return file contents (UTF-8 only).
//! 2. `grep_source`       — substring search across `src/`.
//! 3. `query_audit_ring`  — return the last N audit entries.
//! 4. `suggest_command`   — propose one shell command for a given context.
//! 5. `read_concept_note` — return one note from the vault corpus.
//! 6. `list_caves`        — enumerate live caves and their policy.
//!
//! Phase 2 stub. Each `exec_*` function returns a placeholder
//! string so `dispatch` compiles; the real implementations land in
//! Phase 7. Per the design doc, EVERY response from a tool MUST be
//! truthful: if a path does not exist, return "not found"; never
//! synthesize content.

#![allow(dead_code)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::ai::protocol::ToolDef;

/// All six tools, in the order the model sees them. Stable for the
/// life of a session — the model relies on this ordering when
/// streaming partial tool-call payloads.
pub const TOOLS: &[ToolDef] = &[
    ToolDef {
        name: "read_file",
        description: "Read a UTF-8 file from the Sphragis source tree.",
        parameters_json: r#"{"type":"object","properties":{"path":{"type":"string","description":"path relative to repo root"}},"required":["path"]}"#,
    },
    ToolDef {
        name: "grep_source",
        description: "Search src/ for a substring. Returns matches with file:line.",
        parameters_json: r#"{"type":"object","properties":{"pattern":{"type":"string"},"path_glob":{"type":"string"}},"required":["pattern"]}"#,
    },
    ToolDef {
        name: "query_audit_ring",
        description: "Return up to 'limit' recent audit-ring entries; optionally filter by category.",
        parameters_json: r#"{"type":"object","properties":{"limit":{"type":"integer","minimum":1,"maximum":256},"category":{"type":"string"}},"required":["limit"]}"#,
    },
    ToolDef {
        name: "suggest_command",
        description: "Given an operator goal, return one shell command that accomplishes it. Single line, no preamble.",
        parameters_json: r#"{"type":"object","properties":{"context":{"type":"string"}},"required":["context"]}"#,
    },
    ToolDef {
        name: "read_concept_note",
        description: "Return one Concept note by name from the vault corpus.",
        parameters_json: r#"{"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}"#,
    },
    ToolDef {
        name: "list_caves",
        description: "List currently live caves with their pid, name, capability set, and egress policy.",
        parameters_json: r#"{"type":"object","properties":{}}"#,
    },
];

/// Symbolic tool name. Lookup is by string match against the wire
/// payload because the model emits raw strings. This `enum` exists
/// so the dispatch site has a single match instead of a pile of
/// string compares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolName {
    ReadFile,
    GrepSource,
    QueryAuditRing,
    SuggestCommand,
    ReadConceptNote,
    ListCaves,
}

impl ToolName {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "read_file"         => ToolName::ReadFile,
            "grep_source"       => ToolName::GrepSource,
            "query_audit_ring"  => ToolName::QueryAuditRing,
            "suggest_command"   => ToolName::SuggestCommand,
            "read_concept_note" => ToolName::ReadConceptNote,
            "list_caves"        => ToolName::ListCaves,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ToolName::ReadFile        => "read_file",
            ToolName::GrepSource      => "grep_source",
            ToolName::QueryAuditRing  => "query_audit_ring",
            ToolName::SuggestCommand  => "suggest_command",
            ToolName::ReadConceptNote => "read_concept_note",
            ToolName::ListCaves       => "list_caves",
        }
    }
}

/// Dispatch one tool call. `args_json` is the raw JSON object the
/// model emitted; each `exec_*` parses what it needs.
pub fn dispatch(tool: ToolName, args_json: &str) -> String {
    match tool {
        ToolName::ReadFile        => exec_read_file(args_json),
        ToolName::GrepSource      => exec_grep_source(args_json),
        ToolName::QueryAuditRing  => exec_query_audit_ring(args_json),
        ToolName::SuggestCommand  => exec_suggest_command(args_json),
        ToolName::ReadConceptNote => exec_read_concept_note(args_json),
        ToolName::ListCaves       => exec_list_caves(args_json),
    }
}

// ─── Tool implementations ─────────────────────────────────────────
//
// These run inside the kernel image. The model lives on the
// inference host and emits tool_call payloads over the agent's TLS
// channel; the agent dispatches here. Every tool is read-only and
// operates over compile-time-bundled corpus + live kernel state.

/// Extract a string field from a hand-rolled JSON `{"key":"value"}`.
/// We don't carry serde into the kernel image; the dispatch payloads
/// are simple enough to scan for one expected key.
fn scan_string_arg(args: &str, key: &str) -> Option<String> {
    let needle = alloc::format!("\"{}\"", key);
    let pos = args.find(&needle)?;
    let after = &args[pos + needle.len()..];
    let after = after.trim_start();
    let after = after.strip_prefix(':')?;
    let after = after.trim_start();
    if !after.starts_with('"') { return None; }
    let rest = &after[1..];
    // Find unescaped closing quote.
    let mut out = String::new();
    let mut iter = rest.chars();
    while let Some(c) = iter.next() {
        match c {
            '"' => return Some(out),
            '\\' => {
                if let Some(esc) = iter.next() {
                    match esc {
                        '"'  => out.push('"'),
                        '\\' => out.push('\\'),
                        'n'  => out.push('\n'),
                        't'  => out.push('\t'),
                        _    => out.push(esc),
                    }
                }
            }
            _ => out.push(c),
        }
    }
    None
}

/// read_file — return one entry from the compile-time RAG corpus.
/// The kernel image only carries the RAG corpus and the audit ring;
/// it has no general filesystem at inference time. The model gets
/// the corpus body when it asks for a path that maps to a corpus
/// slug.
fn exec_read_file(args: &str) -> String {
    let path = match scan_string_arg(args, "path") {
        Some(p) => p,
        None => return String::from("{\"error\":\"missing path\"}"),
    };
    use crate::ai::rag_corpus::CORPUS;
    // Match by slug fragment OR by exact title (case-insensitive on
    // both sides). Path "concept_audit_ring_contract.md" matches the
    // slug "concept_audit_ring_contract" generated by build_rag_corpus.
    let needle = path.to_lowercase();
    let needle = needle.trim_end_matches(".md");
    for entry in CORPUS.iter() {
        let title_norm = entry.title.to_lowercase().replace(' ', "_");
        if title_norm == needle || needle.ends_with(&title_norm) {
            // Return JSON with body capped at 4 KB.
            let body = if entry.body.len() > 4096 {
                &entry.body[..4096]
            } else {
                entry.body
            };
            return alloc::format!(
                "{{\"path\":\"{}\",\"content\":{}}}",
                entry.title,
                json_string(body)
            );
        }
    }
    String::from("{\"error\":\"path not in corpus\"}")
}

/// grep_source — search across the RAG corpus body text for a
/// substring. Returns up to 20 line-level matches.
fn exec_grep_source(args: &str) -> String {
    let pattern = match scan_string_arg(args, "pattern") {
        Some(p) => p,
        None => return String::from("{\"error\":\"missing pattern\"}"),
    };
    use crate::ai::rag_corpus::CORPUS;
    let mut out = String::from("{\"matches\":[");
    let mut count = 0usize;
    let max_matches = 20usize;
    'outer: for entry in CORPUS.iter() {
        for (i, line) in entry.body.lines().enumerate() {
            if line.contains(&pattern) {
                if count > 0 { out.push(','); }
                out.push_str(&alloc::format!(
                    "{{\"path\":\"{}\",\"line\":{},\"content\":{}}}",
                    entry.title, i + 1, json_string(line)
                ));
                count += 1;
                if count >= max_matches {
                    out.push_str("],\"truncated\":true}");
                    break 'outer;
                }
            }
        }
    }
    if count < max_matches {
        out.push_str("]}");
    }
    out
}

/// query_audit_ring — return the last N audit entries. Bounded by
/// the model so it can't dump the full ring in one call.
fn exec_query_audit_ring(args: &str) -> String {
    use crate::security::audit::{recent, Entry, Category, MSG_LEN};
    let limit_str = scan_string_arg(args, "limit").unwrap_or_default();
    let limit = limit_str.parse::<usize>().unwrap_or(16);
    let limit = if limit > 32 { 32 } else { limit };
    let mut buf: alloc::vec::Vec<Entry> = alloc::vec::Vec::with_capacity(limit);
    buf.resize(limit, Entry::empty());
    let n = recent(&mut buf);
    let mut out = String::from("{\"entries\":[");
    for i in 0..n {
        if i > 0 { out.push(','); }
        let e = &buf[i];
        let label = match e.cat {
            1  => "fetch", 2  => "script", 3  => "click", 4  => "nav",
            5  => "form",  6  => "mode",   7  => "auth",  8  => "boot",
            9  => "cave",  10 => "ai",     _  => "unknown",
        };
        let _ = Category::Boot;  // silence unused-import warning
        let mlen = e.mlen as usize;
        let mlen = if mlen > MSG_LEN { MSG_LEN } else { mlen };
        let msg = core::str::from_utf8(&e.msg[..mlen]).unwrap_or("<binary>");
        out.push_str(&alloc::format!(
            "{{\"ts\":{},\"cat\":\"{}\",\"msg\":{}}}",
            e.ts, label, json_string(msg)
        ));
    }
    out.push_str("]}");
    out
}

/// suggest_command — extract the context and return a structured
/// "I would suggest" wrapper. The actual suggestion comes from the
/// model itself; this tool exists so the model can hand a recommended
/// command back through a structured channel rather than free-form
/// text, which the shell can render distinctly.
fn exec_suggest_command(args: &str) -> String {
    let context = scan_string_arg(args, "context").unwrap_or_default();
    // We just echo the context back as a placeholder; the caller is
    // expected to populate the actual command in their text turn.
    alloc::format!(
        "{{\"context\":{},\"note\":\"caller proposes the command in the assistant text turn\"}}",
        json_string(&context)
    )
}

/// read_concept_note — return one Concept note by name. Same source
/// (the compile-time RAG corpus) as read_file, but only matches the
/// `concept_*` slugs.
fn exec_read_concept_note(args: &str) -> String {
    let name = match scan_string_arg(args, "name") {
        Some(n) => n.to_lowercase().replace(' ', "_"),
        None => return String::from("{\"error\":\"missing name\"}"),
    };
    use crate::ai::rag_corpus::CORPUS;
    for entry in CORPUS.iter() {
        let title_norm = entry.title.to_lowercase().replace(' ', "_");
        if title_norm == name {
            let body = if entry.body.len() > 4096 { &entry.body[..4096] } else { entry.body };
            return alloc::format!(
                "{{\"name\":\"{}\",\"content\":{}}}",
                entry.title, json_string(body)
            );
        }
    }
    String::from("{\"error\":\"no concept note matched name\"}")
}

/// list_caves — enumerate live caves. We don't expose internal
/// pointers; just the labels the operator would see.
fn exec_list_caves(_args: &str) -> String {
    // The cave registry surface lives in `crate::caves`. For Phase 2
    // we return a placeholder reflecting kernel availability; the
    // real enumeration lands when `caves::list()` is exposed.
    String::from("{\"caves\":[],\"note\":\"cave enumeration pending caves::list() surface\"}")
}

/// JSON-quote a string, escaping the four characters the inner JSON
/// must not contain raw: `\"`, `\\`, control chars 0-0x1f. Used by
/// every tool's body-rendering path so we don't ship hand-built JSON
/// fragments that break on quote.
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"'  => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&alloc::format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Convenience: the names of all tools, for use in audit log
/// pre-allocation and the `cmd_ai_selftest` happy path.
pub fn all_names() -> Vec<&'static str> {
    TOOLS.iter().map(|t| t.name).collect()
}

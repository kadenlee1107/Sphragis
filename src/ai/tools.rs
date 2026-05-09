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
        description: "Read a UTF-8 file from the Bat_OS source tree.",
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

fn exec_read_file(_args: &str) -> String {
    // Phase 7 stub.
    String::from("{\"error\":\"read_file not yet implemented\"}")
}

fn exec_grep_source(_args: &str) -> String {
    String::from("{\"error\":\"grep_source not yet implemented\"}")
}

fn exec_query_audit_ring(_args: &str) -> String {
    // Will need a `crate::security::audit::recent(n: usize) -> &[Entry]`
    // helper that doesn't exist yet — flagged in PLAN_AI_AGENT.md
    // Task 7.1.5.
    String::from("{\"error\":\"query_audit_ring not yet implemented\"}")
}

fn exec_suggest_command(_args: &str) -> String {
    String::from("{\"error\":\"suggest_command not yet implemented\"}")
}

fn exec_read_concept_note(_args: &str) -> String {
    String::from("{\"error\":\"read_concept_note not yet implemented\"}")
}

fn exec_list_caves(_args: &str) -> String {
    String::from("{\"error\":\"list_caves not yet implemented\"}")
}

/// Convenience: the names of all tools, for use in audit log
/// pre-allocation and the `cmd_ai_selftest` happy path.
pub fn all_names() -> Vec<&'static str> {
    TOOLS.iter().map(|t| t.name).collect()
}

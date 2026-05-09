# Bat_OS AI Agent — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land a Bat_OS-native AI assistant accessible via shell command and desktop panel, backed by a fine-tuned Qwen2.5-Coder-7B model running on the operator's RTX 5070, with mandatory tool-use, RAG, and per-session audit-ring entries.

**Architecture:** Two-host split. The 5070 runs `ollama serve` permanently with the fine-tuned model; Bat_OS reaches it via the existing kernel-mediated HTTPS syscall (`https::open_kernel`) under a single dedicated cave-policy egress entry. The Bat_OS-side agent is native Rust under `src/ai/` — no `tokio`, no `reqwest`, no `async`. Tool calls are mandatory for any factual claim, and the agent cites the file:line every claim came from. Streaming responses render token-by-token into the operator's UI.

**Tech Stack:** Rust no_std (Bat_OS side), Python 3 + transformers + peft + bitsandbytes (training side), ollama (inference serving), QEMU + pexpect (smoke testing), the kernel's existing TLS 1.3 stack with the post-PR-#24 X.509 hardening, the existing audit-ring AEAD.

**Reference spec:** `DESIGN_AI_AGENT.md` (root). Read it before starting. This document is the *how*.

**Pre-deletion HEAD:** Tag `pre-ai-agent-2026-05-08` will be applied to `main` HEAD before any code lands. Branch `feat/ai-agent` off `main`.

**Existing patterns to mirror** (referenced repeatedly in tasks below):
- `src/ui/shell.rs::cmd_x509_selftest` — selftest command structure
- `src/ui/shell.rs::cmd_scheduler_selftest` — same pattern, also good reference
- `src/security/audit.rs::record(cat, msg)` — how to write an audit entry
- `src/net/https.rs::{open_kernel, write, read, close_pcb}` — HTTPS surface
- `scripts/qemu_pq_interop_smoke.py` — smoke test harness pattern
- `scripts/qemu_selftests_smoke.py` (post-scheduler-block-on) — multi-selftest harness

---

## Phase 0: Pre-flight

### Task 0.1: Verify clean tree on `main`

**Files:** none (git operation only).

- [ ] **Step 1: Confirm on main + clean tree**

Run: `git status --short && git branch --show-current`
Expected: empty status output, `main`.

- [ ] **Step 2: Confirm latest is post-PR-#24**

Run: `git log -1 --format='%H %s'`
Expected: hash of either `c5faa735 ... (#23)` or later. PR #24 (anchor-aware pathLen) should be on `main`. If it isn't, STOP — pull main first.

- [ ] **Step 3: Run baseline smokes**

Run:
```sh
cargo build --release --target aarch64-unknown-none --features gicv3
python3 scripts/qemu_boot_smoke.py
python3 scripts/qemu_pq_interop_smoke.py
```
Expected: all PASS. If anything fails, STOP — fix before starting.

### Task 0.2: Tag the pre-deletion commit

- [ ] **Step 1: Apply rescue tag**

Run:
```sh
git tag -a pre-ai-agent-2026-05-08 -m "Last commit before AI agent module lands. See DESIGN_AI_AGENT.md."
```

- [ ] **Step 2: Push tag**

Run: `git push origin pre-ai-agent-2026-05-08`

### Task 0.3: Create feature branch

- [ ] **Step 1: Branch off main**

Run: `git switch -c feat/ai-agent`
Expected: `Switched to a new branch 'feat/ai-agent'`.

- [ ] **Step 2: Push the branch (no PR yet)**

Run: `git push -u origin feat/ai-agent`

---

## Phase 1: External infrastructure (training + inference host)

This phase is **operator-driven** — most steps happen on the RTX 5070 box, not in the Bat_OS source tree. The deliverables of this phase are: (a) the LoRA adapter weights merged into a quantized GGUF available via ollama, (b) `scripts/build_lora_dataset.py` committed to the repo, (c) `docs/AI_AGENT_DEPLOY.md` operator setup guide.

### Task 1.1: Write `scripts/build_lora_dataset.py`

**Files:**
- Create: `scripts/build_lora_dataset.py`

The script walks the repo + the Obsidian vault, builds instruction-pair training examples, and writes a JSONL dataset suitable for HuggingFace SFTTrainer.

- [ ] **Step 1: Create the file with the full script**

```python
#!/usr/bin/env python3
"""Build LoRA training dataset from Bat_OS source + docs + vault.

Produces a JSONL file at out/bat_os_lora_dataset.jsonl with one
{"instruction": ..., "input": ..., "output": ...} record per line.
Intended for HuggingFace SFTTrainer or trl's LoRA fine-tune flow.

Dataset composition (see DESIGN_AI_AGENT.md "Training data"):
  * Source files: synthesize "What does function X do?" → docstring + body
  * Audit markers: "What is V8-ROOT-1?" → surrounding comment + linked code
  * Concept notes: "Tell me about <topic>" → Concept note body
  * Commit messages: subject → body + diff stat
"""
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

REPO  = Path(__file__).resolve().parent.parent
VAULT = Path.home() / "BAT_OS_VAULT"
OUT   = REPO / "out" / "bat_os_lora_dataset.jsonl"

AUDIT_RE = re.compile(r'(V\d+-[A-Z]+(?:-\d+)?|STUMP\s*#\s*\d+)')
RUST_FN_RE = re.compile(r'^(///[^\n]*\n)*\s*(pub\s+(?:async\s+|unsafe\s+|const\s+)*fn\s+\w+[^{]+)\{', re.MULTILINE)

def collect_source_pairs() -> list[dict]:
    """For each pub fn in src/, emit a (signature → body+docstring) pair."""
    out = []
    for p in (REPO / "src").rglob("*.rs"):
        text = p.read_text(encoding="utf-8", errors="replace")
        for m in RUST_FN_RE.finditer(text):
            sig = m.group(2).strip()
            # crude: take 30 lines after the match as the body
            start = m.start()
            body_end = min(start + 2000, len(text))
            body = text[start:body_end]
            rel = p.relative_to(REPO)
            out.append({
                "instruction": f"In Bat_OS, what does the following function do?",
                "input": sig,
                "output": f"From `{rel}`:\n\n```rust\n{body}\n```",
            })
    return out

def collect_audit_pairs() -> list[dict]:
    """For each V-marker, emit a (marker → surrounding-comment) pair."""
    out = []
    seen: set[str] = set()
    for p in (REPO / "src").rglob("*.rs"):
        text = p.read_text(encoding="utf-8", errors="replace")
        for m in AUDIT_RE.finditer(text):
            marker = m.group(1)
            if marker in seen: continue
            seen.add(marker)
            # context window: 800 chars
            ctx = text[max(0, m.start() - 200): m.end() + 600]
            rel = p.relative_to(REPO)
            out.append({
                "instruction": f"What does the audit marker {marker} refer to in Bat_OS?",
                "input": "",
                "output": f"From `{rel}`:\n\n{ctx}",
            })
    return out

def collect_concept_pairs() -> list[dict]:
    """Each Concept note → its full body."""
    out = []
    if not (VAULT / "Concepts").exists():
        return out
    for p in (VAULT / "Concepts").glob("*.md"):
        text = p.read_text(encoding="utf-8")
        title = p.stem
        out.append({
            "instruction": f"Explain the Bat_OS concept '{title}'.",
            "input": "",
            "output": text,
        })
    return out

def collect_commit_pairs() -> list[dict]:
    """Each commit on main → (subject → body)."""
    out = []
    r = subprocess.run(
        ["git", "log", "main", "--format=%s%n----BODY----%n%b%n----END----"],
        cwd=REPO, capture_output=True, text=True, timeout=30,
    )
    if r.returncode != 0: return out
    blocks = r.stdout.split("----END----")
    for blk in blocks:
        blk = blk.strip()
        if "----BODY----" not in blk: continue
        subject, body = blk.split("----BODY----", 1)
        subject, body = subject.strip(), body.strip()
        if not subject or not body: continue
        out.append({
            "instruction": "Expand on this Bat_OS commit subject.",
            "input": subject,
            "output": body,
        })
    return out

def main() -> int:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    pairs = []
    pairs.extend(collect_source_pairs())
    pairs.extend(collect_audit_pairs())
    pairs.extend(collect_concept_pairs())
    pairs.extend(collect_commit_pairs())
    with OUT.open("w", encoding="utf-8") as f:
        for p in pairs:
            f.write(json.dumps(p, ensure_ascii=False) + "\n")
    print(f"[lora-dataset] wrote {len(pairs)} pairs to {OUT.relative_to(REPO)}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
```

- [ ] **Step 2: Make executable + run it**

```sh
chmod +x scripts/build_lora_dataset.py
python3 scripts/build_lora_dataset.py
```
Expected: `[lora-dataset] wrote NNNN pairs to out/bat_os_lora_dataset.jsonl` where NNNN is in the low thousands.

- [ ] **Step 3: Spot-check the output**

```sh
head -3 out/bat_os_lora_dataset.jsonl | python3 -c "import json,sys; [print(json.dumps(json.loads(l), indent=2)) for l in sys.stdin]"
```
Expected: three valid JSON records with `instruction`, `input`, `output` fields, each making sense as a training pair.

- [ ] **Step 4: Add `out/` to .gitignore**

Edit `.gitignore`, add line: `/out/`.

- [ ] **Step 5: Commit script + gitignore**

```sh
git add scripts/build_lora_dataset.py .gitignore
git commit -m "ai-agent: scripts/build_lora_dataset.py — assemble LoRA training pairs"
```

### Task 1.2: Write `docs/AI_AGENT_DEPLOY.md` operator deploy guide

**Files:**
- Create: `docs/AI_AGENT_DEPLOY.md`

- [ ] **Step 1: Create the deploy guide**

```markdown
# AI Agent — Deploy Guide

This is the operator's how-to for the **inference host side** of the AI
agent feature. It assumes one RTX 5070 desktop running Linux that's
reachable from the M4 Mac over the LAN.

## Hardware

- RTX 5070 (or any NVIDIA card with ≥8 GB VRAM)
- Linux host (Ubuntu 24.04 tested)
- Static LAN IP

## One-time setup

1. **Install ollama**

   ```sh
   curl -fsSL https://ollama.com/install.sh | sh
   sudo systemctl enable --now ollama
   ```

2. **Pull the base model**

   ```sh
   ollama pull qwen2.5-coder:7b
   ```

3. **Generate the training dataset (on the Mac)**

   ```sh
   cd /Users/kadenlee/Bat_OS
   python3 scripts/build_lora_dataset.py
   # Copy out/bat_os_lora_dataset.jsonl to the 5070
   scp out/bat_os_lora_dataset.jsonl 5070-host:/tmp/
   ```

4. **Train the LoRA (on the 5070)**

   ```sh
   pip install transformers peft trl bitsandbytes accelerate
   ```

   Then run a training script following HuggingFace's `SFTTrainer`
   pattern — see https://huggingface.co/docs/trl for current examples.
   Hyperparameters (initial guess):
   - LoRA rank 16, alpha 32
   - Sequence length 4096
   - Effective batch size 32 (batch 4, grad accum 8)
   - Learning rate 2e-4
   - Epochs 3
   Wall time ~6-12 hours on the 5070.

5. **Merge the LoRA + quantize to GGUF**

   ```sh
   # Merge:
   python3 -m peft.merge_adapter --adapter ./out/lora --base qwen2.5-coder-7b --output ./out/merged
   # Quantize via llama.cpp:
   git clone https://github.com/ggerganov/llama.cpp
   cd llama.cpp && make
   ./convert-hf-to-gguf.py ../out/merged --outfile ../out/bat-os-coder.gguf
   ./quantize ../out/bat-os-coder.gguf ../out/bat-os-coder-q4.gguf Q4_K_M
   ```

6. **Register with ollama**

   ```sh
   cat > /tmp/Modelfile <<EOF
   FROM /path/to/bat-os-coder-q4.gguf
   TEMPLATE """{{ .System }}
   <|im_start|>user
   {{ .Prompt }}<|im_end|>
   <|im_start|>assistant
   """
   PARAMETER temperature 0.7
   PARAMETER num_ctx 8192
   EOF
   ollama create bat-os-coder -f /tmp/Modelfile
   ollama run bat-os-coder "test"
   ```

7. **Set up TLS for ollama**

   ollama by default serves plain HTTP. Bat_OS reaches it via TLS, so
   put a TLS terminator (caddy / nginx / a small Rust proxy) in front
   of ollama:

   ```sh
   # caddy example
   cat > /etc/caddy/Caddyfile <<EOF
   ai.bat-os.local {
     reverse_proxy localhost:11434
     tls /etc/ssl/bat-os-ai.crt /etc/ssl/bat-os-ai.key
   }
   EOF
   sudo systemctl reload caddy
   ```

   Generate a self-signed cert; pin its SHA-256 fingerprint into
   `src/net/tls_pinning.rs` per the existing pinning convention.

## Verifying the host

From the Mac:

```sh
curl -k https://10.0.2.42:443/api/tags
```

Should list `bat-os-coder` among the available models.

## Retraining cadence

Re-run dataset build + LoRA train after substantive code or doc changes.
Cheap (~6h on the 5070); can be cron'd weekly.

## Backup / rollback

The base Qwen2.5-Coder-7B GGUF stays in `~/.ollama/models/blobs/` — if
the fine-tuned `bat-os-coder` model produces bad outputs, fall back to
plain `qwen2.5-coder:7b` by changing the Modelfile.
```

- [ ] **Step 2: Commit**

```sh
git add docs/AI_AGENT_DEPLOY.md
git commit -m "ai-agent: docs/AI_AGENT_DEPLOY.md — operator setup guide for the 5070 inference host"
```

### Task 1.3: Operator runs the deploy

This is **manual operator work**, not engineer work. The plan tracks it as a checkbox so the prerequisite is recorded:

- [ ] Operator runs `scripts/build_lora_dataset.py` and copies the JSONL to the 5070.
- [ ] Operator trains the LoRA on the 5070.
- [ ] Operator merges + quantizes the model.
- [ ] Operator registers the model in ollama.
- [ ] Operator stands up TLS-fronted ollama on the LAN.
- [ ] Operator verifies `curl -k https://<host>/api/tags` returns the model list.

The Bat_OS-side phases (2 onwards) can begin in parallel with operator deploy work. They depend on operator deploy only at the smoke-test phase.

---

## Phase 2: `src/ai/` module scaffold

Goal: empty module that compiles, with all sub-files in place. No real logic yet; subsequent phases fill in each sub-file.

### Task 2.1: Add `mod ai;` to `src/main.rs`

**Files:**
- Modify: `src/main.rs` (add module declaration)

- [ ] **Step 1: Find the existing module list**

Run: `grep -n '^mod ' src/main.rs`
Expected:
```
6:mod batcave;
7:mod boot;
8:mod crypto;
9:mod drivers;
10:mod fs;
11:mod kernel;
12:mod net;
13:mod platform;
14:mod security;
15:mod ui;
```

- [ ] **Step 2: Insert `mod ai;` alphabetically (line 6)**

Edit `src/main.rs`. After `mod ai;` is added the list is:

```rust
mod ai;
mod batcave;
mod boot;
mod crypto;
mod drivers;
mod fs;
mod kernel;
mod net;
mod platform;
mod security;
mod ui;
```

- [ ] **Step 3: Verify it doesn't break the build (it will, until Task 2.2)**

Run: `cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -5`
Expected: error like `couldn't find module 'ai' in main.rs context`. That's expected — Task 2.2 creates the file.

### Task 2.2: Create `src/ai/mod.rs` skeleton

**Files:**
- Create: `src/ai/mod.rs`

- [ ] **Step 1: Create the directory + skeleton mod.rs**

```rust
//! Bat_OS AI agent — locally-hosted, domain-narrow, kernel-mediated.
//!
//! See DESIGN_AI_AGENT.md for the why.
//! See docs/PLAN_AI_AGENT.md for the how.
//!
//! Public API exposed by this module:
//!
//! * [`AgentSession::new`] — open a session
//! * [`AgentSession::ask`]  — send a question, get a streaming response
//! * [`AgentSession::interrupt`] — cancel the in-flight response
//! * [`AgentSession::close`] — close the session and write the audit entry
//!
//! All other items are crate-private. Production callers go through the
//! shell command (`ai <question>`) or the desktop panel; both use the
//! public API above.

#![allow(dead_code)]   // skeleton phase — most items unused until Phase 11

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;

mod audit;
mod client;
mod policy;
mod prompt;
mod protocol;
mod rag;
mod stream;
mod tools;

#[derive(Debug, Clone, Copy)]
pub enum AgentError {
    NetworkUnreachable,
    TlsHandshakeFailed,
    ProtocolError,
    ToolDispatchError,
    Interrupted,
    PolicyDenied,
}

pub struct AgentSession {
    // populated in Phase 11
    _placeholder: (),
}

pub struct StreamingResponse {
    // populated in Phase 11
    _placeholder: (),
}

impl AgentSession {
    pub fn new() -> Result<Self, AgentError> {
        Err(AgentError::NetworkUnreachable)   // Phase 11 implements
    }

    pub fn ask(&mut self, _question: &str) -> StreamingResponse {
        StreamingResponse { _placeholder: () }   // Phase 11 implements
    }

    pub fn interrupt(&mut self) {}   // Phase 11 implements

    pub fn close(self) {}   // Phase 11 implements
}
```

- [ ] **Step 2: Create empty placeholder files for the other modules**

```sh
mkdir -p src/ai
for f in audit client policy prompt protocol rag stream tools; do
  cat > "src/ai/$f.rs" <<EOF
//! \`src/ai/$f.rs\` — see DESIGN_AI_AGENT.md and docs/PLAN_AI_AGENT.md.
//! Filled in during the corresponding plan phase.
#![allow(dead_code)]
EOF
done
```

- [ ] **Step 3: cargo check**

Run: `cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3`
Expected: `Finished` line (no errors). Warnings about dead code are OK because of `#![allow(dead_code)]`.

- [ ] **Step 4: Commit**

```sh
git add src/main.rs src/ai/
git commit -m "ai-agent: scaffold src/ai/ module skeleton"
```

---

## Phase 3: `protocol.rs` — chat-completions API types

Goal: serializable Rust types matching ollama's OpenAI-compatible JSON shapes.

### Task 3.1: Define request/response structs

**Files:**
- Modify: `src/ai/protocol.rs` (replace skeleton with real content)

- [ ] **Step 1: Replace `src/ai/protocol.rs` with the type definitions**

```rust
//! `src/ai/protocol.rs` — chat-completion API request/response shapes
//! matching ollama's OpenAI-compatible JSON schema.
//!
//! ollama serves an HTTP API with two relevant endpoints:
//! * `POST /v1/chat/completions` — same JSON contract as OpenAI's
//! * `GET  /api/tags`            — list registered models
//!
//! We use the OpenAI-compatible endpoint because it gives us native
//! tool-call formatting (Qwen 2.5 supports it) and standard streaming.
//!
//! These types are minimal — only the fields we read or write. They
//! are NOT a complete OpenAI schema; many optional fields are skipped.

#![allow(dead_code)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// Chat-completion request body sent to ollama.
#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,                  // e.g. "bat-os-coder"
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<ToolDef>,
    pub tool_choice: ToolChoice,
    pub stream: bool,                   // we always set true
    pub temperature: f32,               // we always send 0.4 for determinism
    pub max_tokens: u32,                // we cap at 1024
}

/// One message in the conversation history.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub tool_calls: Vec<ToolCall>,      // Assistant role only
    pub tool_call_id: Option<String>,   // Tool role only
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role { System, User, Assistant, Tool }

/// Tool definition advertised to the model so it knows what's callable.
/// `parameters_json` is the JSON-schema string describing the args.
#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters_json: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ToolChoice {
    /// "no tool unless we ask"
    None,
    /// "you may call tools as you see fit" — our default
    Auto,
}

/// One tool invocation the model wants to make.
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args_json: String,              // raw JSON; parsed by `tools.rs`
}

/// One streaming chunk delta from the model.
#[derive(Debug, Clone)]
pub struct ChatDelta {
    pub content_delta: String,          // empty if this chunk is a tool-call
    pub tool_call_delta: Option<ToolCallDelta>,
    pub finish_reason: Option<FinishReason>,
}

#[derive(Debug, Clone)]
pub struct ToolCallDelta {
    pub index: u32,
    pub id_delta: String,
    pub name_delta: String,
    pub args_delta: String,
}

#[derive(Debug, Clone, Copy)]
pub enum FinishReason { Stop, ToolCalls, Length, Other }
```

- [ ] **Step 2: cargo check**

Run: `cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3`
Expected: `Finished`.

- [ ] **Step 3: Commit**

```sh
git add src/ai/protocol.rs
git commit -m "ai-agent: protocol.rs — OpenAI-compatible chat-completion types"
```

### Task 3.2: Add minimal JSON serialization helpers

The kernel doesn't ship serde. We hand-roll JSON for the small surface we need.

**Files:**
- Modify: `src/ai/protocol.rs` (extend with serialize functions)

- [ ] **Step 1: Add `serialize_request` to the bottom of `protocol.rs`**

Append to `src/ai/protocol.rs`:

```rust
// ─── Serialization ──────────────────────────────────────────────────────

/// Manually serialize a ChatRequest to JSON. Hand-rolled because the
/// kernel doesn't ship serde and we only need to write 5-10 fields.
pub fn serialize_request(req: &ChatRequest) -> String {
    let mut s = String::with_capacity(2048);
    s.push('{');
    push_field_str(&mut s, "model", &req.model, true);
    s.push(',');
    s.push_str("\"messages\":[");
    for (i, m) in req.messages.iter().enumerate() {
        if i > 0 { s.push(','); }
        serialize_message(&mut s, m);
    }
    s.push(']');
    if !req.tools.is_empty() {
        s.push_str(",\"tools\":[");
        for (i, t) in req.tools.iter().enumerate() {
            if i > 0 { s.push(','); }
            s.push_str("{\"type\":\"function\",\"function\":{");
            push_field_str(&mut s, "name", &t.name, true);
            s.push(',');
            push_field_str(&mut s, "description", &t.description, true);
            s.push(',');
            // parameters_json is RAW JSON — no escape, no quotes
            s.push_str("\"parameters\":");
            s.push_str(&t.parameters_json);
            s.push_str("}}");
        }
        s.push(']');
        s.push_str(",\"tool_choice\":\"auto\"");
    }
    s.push_str(",\"stream\":");
    s.push_str(if req.stream { "true" } else { "false" });
    s.push_str(",\"temperature\":");
    push_f32(&mut s, req.temperature);
    s.push_str(",\"max_tokens\":");
    push_u32(&mut s, req.max_tokens);
    s.push('}');
    s
}

fn serialize_message(s: &mut String, m: &ChatMessage) {
    s.push('{');
    let role = match m.role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    };
    push_field_str(s, "role", role, true);
    s.push(',');
    push_field_str(s, "content", &m.content, true);
    if let Some(tcid) = &m.tool_call_id {
        s.push(',');
        push_field_str(s, "tool_call_id", tcid, true);
    }
    if !m.tool_calls.is_empty() {
        s.push_str(",\"tool_calls\":[");
        for (i, tc) in m.tool_calls.iter().enumerate() {
            if i > 0 { s.push(','); }
            s.push_str("{\"id\":");
            push_quoted(s, &tc.id);
            s.push_str(",\"type\":\"function\",\"function\":{\"name\":");
            push_quoted(s, &tc.name);
            s.push_str(",\"arguments\":");
            push_quoted(s, &tc.args_json);   // args_json is a JSON string field — quote it
            s.push_str("}}");
        }
        s.push(']');
    }
    s.push('}');
}

fn push_field_str(s: &mut String, k: &str, v: &str, _quote_v: bool) {
    push_quoted(s, k);
    s.push(':');
    push_quoted(s, v);
}

fn push_quoted(s: &mut String, v: &str) {
    s.push('"');
    for c in v.chars() {
        match c {
            '"'  => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use core::fmt::Write;
                let _ = write!(s, "\\u{:04x}", c as u32);
            }
            c => s.push(c),
        }
    }
    s.push('"');
}

fn push_u32(s: &mut String, n: u32) {
    use core::fmt::Write;
    let _ = write!(s, "{}", n);
}

fn push_f32(s: &mut String, f: f32) {
    // Two decimals is enough for our parameters.
    use core::fmt::Write;
    let _ = write!(s, "{:.2}", f);
}
```

- [ ] **Step 2: cargo check**

Run: `cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3`
Expected: `Finished`.

- [ ] **Step 3: Commit**

```sh
git add src/ai/protocol.rs
git commit -m "ai-agent: protocol.rs — hand-rolled JSON serializer for ChatRequest"
```

### Task 3.3: JSON deserialization helpers for streaming chunks

The streaming response gives us `data: { "choices": [{ "delta": { ... } }] }` lines. We need a tiny parser that extracts content + tool-call deltas. We don't need a full JSON parser — just enough to walk known shapes.

**Files:**
- Modify: `src/ai/protocol.rs` (extend with deserialize_delta)

- [ ] **Step 1: Append a minimal scan-based parser**

Append to `src/ai/protocol.rs`:

```rust
// ─── Deserialization (streaming chunks only) ────────────────────────────

/// Parse one ollama SSE data-line payload (the JSON object after `data: `)
/// into a ChatDelta. Returns None if the line is `[DONE]` or unparseable.
///
/// We intentionally do NOT use a general-purpose JSON parser. The
/// expected shape is fixed:
/// {"choices":[{"delta":{"content":"...","tool_calls":[{"index":N,
///  "id":"...","function":{"name":"...","arguments":"..."}}]},
///  "finish_reason":null}]}
pub fn parse_delta_line(line: &[u8]) -> Option<ChatDelta> {
    let s = core::str::from_utf8(line).ok()?;
    if s.trim() == "[DONE]" {
        return Some(ChatDelta {
            content_delta: String::new(),
            tool_call_delta: None,
            finish_reason: Some(FinishReason::Stop),
        });
    }
    let content = scan_string_field(s, "\"content\":");
    let finish = scan_string_field(s, "\"finish_reason\":");
    let finish_reason = finish.and_then(|f| match f.as_str() {
        "stop" => Some(FinishReason::Stop),
        "tool_calls" => Some(FinishReason::ToolCalls),
        "length" => Some(FinishReason::Length),
        _ => None,
    });
    let tool_call_delta = scan_tool_call_delta(s);
    Some(ChatDelta {
        content_delta: content.unwrap_or_default(),
        tool_call_delta,
        finish_reason,
    })
}

/// Find a "key":"value" string field. Naive — does not handle nested
/// objects or escaped quotes inside the value gracefully. Sufficient
/// for the simple top-level fields we read.
fn scan_string_field(s: &str, key: &str) -> Option<String> {
    let p = s.find(key)?;
    let rest = &s[p + key.len()..].trim_start();
    if !rest.starts_with('"') {
        // could be null / number / object — for our cases that means absent
        return None;
    }
    let body = &rest[1..];
    let mut out = String::new();
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"'  => return Some(out),
            '\\' => match chars.next()? {
                '"'  => out.push('"'),
                '\\' => out.push('\\'),
                'n'  => out.push('\n'),
                'r'  => out.push('\r'),
                't'  => out.push('\t'),
                _    => return Some(out),  // unsupported escape — bail with what we have
            },
            c    => out.push(c),
        }
    }
    None
}

fn scan_tool_call_delta(s: &str) -> Option<ToolCallDelta> {
    if !s.contains("\"tool_calls\"") {
        return None;
    }
    let id = scan_string_field(s, "\"id\":").unwrap_or_default();
    let name = scan_string_field(s, "\"name\":").unwrap_or_default();
    let args = scan_string_field(s, "\"arguments\":").unwrap_or_default();
    Some(ToolCallDelta {
        index: 0,                       // assume single-tool-call streams; multi is future scope
        id_delta: id,
        name_delta: name,
        args_delta: args,
    })
}
```

- [ ] **Step 2: cargo check**

Run: `cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3`
Expected: `Finished`.

- [ ] **Step 3: Commit**

```sh
git add src/ai/protocol.rs
git commit -m "ai-agent: protocol.rs — scan-based delta-line parser for streaming chunks"
```

---

## Phase 4: `stream.rs` — SSE chunk parser

Goal: turn raw byte chunks from the kernel HTTPS read into a sequence of `ChatDelta`s.

### Task 4.1: Implement the streaming framer

**Files:**
- Modify: `src/ai/stream.rs`

- [ ] **Step 1: Replace skeleton with the framer**

```rust
//! `src/ai/stream.rs` — SSE-flavored chunk framer for ollama responses.
//!
//! ollama's streaming endpoint sends server-sent-events:
//!   data: {"choices":[{"delta":{"content":"hello"}}]}\n\n
//!   data: {"choices":[{"delta":{"content":" world"}}]}\n\n
//!   data: [DONE]\n\n
//!
//! Each `data: ` line is one delta. Two newlines separate events.
//!
//! The framer is byte-fed (the HTTPS layer reads in fixed-size chunks)
//! and yields complete deltas as they arrive.

#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;

use super::protocol::{parse_delta_line, ChatDelta};

pub struct StreamFramer {
    buf: Vec<u8>,
}

impl StreamFramer {
    pub const fn new() -> Self {
        Self { buf: Vec::new() }
    }

    /// Feed one chunk of bytes from the HTTPS read. Returns any complete
    /// deltas that are now ready.
    pub fn feed(&mut self, chunk: &[u8]) -> Vec<ChatDelta> {
        self.buf.extend_from_slice(chunk);
        let mut out = Vec::new();
        // Look for "\n\n" delimiters — each event is "data: <line>\n\n"
        loop {
            let pos = self
                .buf
                .windows(2)
                .position(|w| w == b"\n\n");
            let Some(end) = pos else { break };
            let event = self.buf[..end].to_vec();
            self.buf.drain(..end + 2);
            if let Some(delta) = parse_event(&event) {
                out.push(delta);
            }
        }
        out
    }

    /// True if the framer has consumed a [DONE] sentinel.
    pub fn is_done(&self, deltas: &[ChatDelta]) -> bool {
        deltas.iter().any(|d| matches!(
            d.finish_reason,
            Some(super::protocol::FinishReason::Stop)
        ))
    }
}

/// Parse one event-buffer (everything between two \n\n boundaries).
/// Strips the leading "data: " prefix.
fn parse_event(event: &[u8]) -> Option<ChatDelta> {
    const PREFIX: &[u8] = b"data: ";
    let line = if event.starts_with(PREFIX) { &event[PREFIX.len()..] } else { event };
    parse_delta_line(line)
}
```

- [ ] **Step 2: cargo check**

Run: `cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3`
Expected: `Finished`.

- [ ] **Step 3: Commit**

```sh
git add src/ai/stream.rs
git commit -m "ai-agent: stream.rs — SSE chunk framer for ollama streaming responses"
```

---

## Phase 5: `client.rs` — HTTPS client

Goal: open a TLS connection to the inference host using the kernel's HTTPS syscall, write the chat request, read+frame chunks.

### Task 5.1: Implement the client

**Files:**
- Modify: `src/ai/client.rs`

- [ ] **Step 1: Replace skeleton**

```rust
//! `src/ai/client.rs` — HTTPS client over the kernel-mediated TLS path.
//!
//! Uses `crate::net::https::{open_kernel, write, read, close_pcb}`,
//! which are the only network primitives this module touches. The TLS
//! handshake, chain validation, and PQ key agreement are all kernel
//! responsibilities — see DESIGN_TLS_HARDENING.md and PRs #21-#24 for
//! the path they go through.
//!
//! Cave-policy: the agent's connection runs under a dedicated allowlist
//! entry that names the inference host and nothing else. See
//! `super::policy` for the entry construction.

#![allow(dead_code)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use super::protocol::{ChatRequest, serialize_request};
use super::stream::StreamFramer;
use super::AgentError;
use crate::net::https;

const READ_CHUNK_SIZE: usize = 4096;

pub struct AgentClient {
    pcb: usize,
    framer: StreamFramer,
}

impl AgentClient {
    /// Open a TLS connection to the inference host.
    pub fn new(host: &str, port: u16) -> Result<Self, AgentError> {
        let pcb = https::open_kernel(host, port)
            .map_err(|_| AgentError::TlsHandshakeFailed)?;
        Ok(Self { pcb, framer: StreamFramer::new() })
    }

    /// Send a chat-completion request body. Returns once the request
    /// has been written; responses are read separately.
    pub fn send_request(&self, host: &str, req: &ChatRequest) -> Result<(), AgentError> {
        let body = serialize_request(req);
        let mut http = String::with_capacity(body.len() + 256);
        http.push_str("POST /v1/chat/completions HTTP/1.1\r\n");
        http.push_str("Host: "); http.push_str(host); http.push_str("\r\n");
        http.push_str("Content-Type: application/json\r\n");
        http.push_str("Accept: text/event-stream\r\n");
        // Length is in bytes, not chars
        let len = body.as_bytes().len();
        http.push_str("Content-Length: ");
        push_u32(&mut http, len as u32);
        http.push_str("\r\n\r\n");
        http.push_str(&body);
        https::write(self.pcb, http.as_bytes())
            .map_err(|_| AgentError::ProtocolError)?;
        Ok(())
    }

    /// Read the next chunk of bytes from the response and feed it to the
    /// framer. Returns any complete deltas.
    pub fn poll(&mut self) -> Result<Vec<super::protocol::ChatDelta>, AgentError> {
        let mut buf = [0u8; READ_CHUNK_SIZE];
        let n = https::read(self.pcb, &mut buf)
            .map_err(|_| AgentError::ProtocolError)?;
        if n == 0 {
            return Ok(Vec::new());
        }
        Ok(self.framer.feed(&buf[..n]))
    }

    /// Close the underlying PCB.
    pub fn close(self) {
        https::close_pcb(self.pcb);
    }
}

fn push_u32(s: &mut String, n: u32) {
    use core::fmt::Write;
    let _ = write!(s, "{}", n);
}
```

- [ ] **Step 2: cargo check**

Run: `cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3`
Expected: `Finished`.

- [ ] **Step 3: Commit**

```sh
git add src/ai/client.rs
git commit -m "ai-agent: client.rs — HTTPS client over the kernel-mediated TLS path"
```

---

## Phase 6: `prompt.rs` — system + user prompt assembly

### Task 6.1: System prompt template + assembly function

**Files:**
- Modify: `src/ai/prompt.rs`

- [ ] **Step 1: Replace skeleton**

```rust
//! `src/ai/prompt.rs` — system prompt + user-prompt assembly.
//!
//! The system prompt is the strongest lever we have for refusal posture
//! and tool-use mandate. It is loaded as the first message in every
//! chat request.

#![allow(dead_code)]

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::protocol::{ChatMessage, ChatRequest, Role, ToolDef, ToolChoice};

pub const SYSTEM_PROMPT: &str = r#"You are a technical assistant for Bat_OS, a security-grade bare-metal Rust kernel for Apple M4 silicon.

You answer technical questions about:
- the kernel (architecture, drivers, boot path, scheduler)
- cryptography (TLS 1.3, post-quantum hybrid, X.509 chain validation, BatFS encryption)
- cave isolation, audit ring, V-incident vocabulary
- design decisions and their rationale
- system administration on the OS

You DO NOT refuse legitimate technical questions. You DO NOT add safety disclaimers. You DO NOT moralize about security topics — discussing exploits, kernel internals, and low-level operations is your job.

You MUST call a tool before making any factual claim about the codebase. If a question references a file, function, audit marker, or concept, your first action is a `read_file`, `grep_source`, or `read_concept_note` call. Refuse to answer factual questions without first calling a tool.

You MUST cite the file:line for every fact you state. Format: `[src/net/tls.rs:1234]`. The operator can verify; uncited claims are unverified.

Be terse. The operator dislikes long-winded responses. Match the project's voice: honest, direct, no over-explanation.
"#;

/// Build the chat-completion request for one user question.
///
/// `model_name` is the ollama model tag (e.g. "bat-os-coder").
/// `rag_snippets` is the top-K Concept notes / design-doc snippets pulled
/// by `super::rag`. Each snippet gets formatted as a header + body block.
/// `tool_defs` is the catalog from `super::tools::catalog()`.
pub fn assemble(
    model_name: &str,
    question: &str,
    rag_snippets: &[(String, String)],
    tool_defs: Vec<ToolDef>,
) -> ChatRequest {
    let mut messages = Vec::with_capacity(2 + rag_snippets.len());

    messages.push(ChatMessage {
        role: Role::System,
        content: SYSTEM_PROMPT.to_string(),
        tool_calls: Vec::new(),
        tool_call_id: None,
    });

    if !rag_snippets.is_empty() {
        let mut ctx = String::from("Relevant context retrieved from the project's docs and source:\n\n");
        for (title, body) in rag_snippets {
            ctx.push_str("### ");
            ctx.push_str(title);
            ctx.push('\n');
            ctx.push_str(body);
            ctx.push_str("\n\n");
        }
        messages.push(ChatMessage {
            role: Role::System,
            content: ctx,
            tool_calls: Vec::new(),
            tool_call_id: None,
        });
    }

    messages.push(ChatMessage {
        role: Role::User,
        content: question.to_string(),
        tool_calls: Vec::new(),
        tool_call_id: None,
    });

    ChatRequest {
        model: model_name.to_string(),
        messages,
        tools: tool_defs,
        tool_choice: ToolChoice::Auto,
        stream: true,
        temperature: 0.4,
        max_tokens: 1024,
    }
}
```

- [ ] **Step 2: cargo check**

Run: `cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3`
Expected: `Finished`.

- [ ] **Step 3: Commit**

```sh
git add src/ai/prompt.rs
git commit -m "ai-agent: prompt.rs — system prompt + chat-request assembly"
```

---

## Phase 7: `tools.rs` — tool catalog + dispatch

The agent calls these via JSON tool-use. Every tool is read-only.

### Task 7.1: Define `ToolName` enum + dispatch entry

**Files:**
- Modify: `src/ai/tools.rs`

- [ ] **Step 1: Replace skeleton with the dispatch scaffold**

```rust
//! `src/ai/tools.rs` — tool catalog + dispatch.
//!
//! The agent advertises six tools to the model via `catalog()`. When
//! the model issues a tool call, `dispatch(name, args_json)` runs the
//! corresponding tool and returns the result as a JSON string.
//!
//! ALL TOOLS ARE READ-ONLY. No write_file, no run_command, no policy
//! mutation. Adding mutating tools is deliberate future scope.

#![allow(dead_code)]

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::protocol::ToolDef;

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
    pub const fn as_str(self) -> &'static str {
        match self {
            ToolName::ReadFile        => "read_file",
            ToolName::GrepSource      => "grep_source",
            ToolName::QueryAuditRing  => "query_audit_ring",
            ToolName::SuggestCommand  => "suggest_command",
            ToolName::ReadConceptNote => "read_concept_note",
            ToolName::ListCaves       => "list_caves",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "read_file"         => Some(Self::ReadFile),
            "grep_source"       => Some(Self::GrepSource),
            "query_audit_ring"  => Some(Self::QueryAuditRing),
            "suggest_command"   => Some(Self::SuggestCommand),
            "read_concept_note" => Some(Self::ReadConceptNote),
            "list_caves"        => Some(Self::ListCaves),
            _ => None,
        }
    }
}

/// The tool catalog advertised to the model in every request.
pub fn catalog() -> Vec<ToolDef> {
    let mut out = Vec::with_capacity(6);
    out.push(ToolDef {
        name: "read_file".to_string(),
        description: "Read a file from the Bat_OS source tree. Read-only. Path is relative to the repo root.".to_string(),
        parameters_json: r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#.to_string(),
    });
    out.push(ToolDef {
        name: "grep_source".to_string(),
        description: "Search the Bat_OS source tree with a regex pattern. Returns matching file:line:content rows.".to_string(),
        parameters_json: r#"{"type":"object","properties":{"pattern":{"type":"string"},"path_glob":{"type":"string"}},"required":["pattern"]}"#.to_string(),
    });
    out.push(ToolDef {
        name: "query_audit_ring".to_string(),
        description: "Read recent audit-ring entries. Returns the most recent N entries with category and message.".to_string(),
        parameters_json: r#"{"type":"object","properties":{"limit":{"type":"integer"}},"required":["limit"]}"#.to_string(),
    });
    out.push(ToolDef {
        name: "suggest_command".to_string(),
        description: "Suggest a Bat_OS shell command for a given context. Operator must confirm before running.".to_string(),
        parameters_json: r#"{"type":"object","properties":{"context":{"type":"string"}},"required":["context"]}"#.to_string(),
    });
    out.push(ToolDef {
        name: "read_concept_note".to_string(),
        description: "Read a hand-written Concept note from the Obsidian vault. Use when the question is conceptual rather than code.".to_string(),
        parameters_json: r#"{"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}"#.to_string(),
    });
    out.push(ToolDef {
        name: "list_caves".to_string(),
        description: "List active caves with their PIDs, names, capabilities, and policies.".to_string(),
        parameters_json: r#"{"type":"object","properties":{}}"#.to_string(),
    });
    out
}

/// Dispatch a tool call. `args_json` is the raw arguments JSON the model
/// produced; we extract the fields we need with the same scan-based
/// approach as `protocol::parse_delta_line`.
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

// Stubs filled in by tasks 7.2-7.7
fn exec_read_file(_args: &str) -> String { error_response("read_file: not implemented") }
fn exec_grep_source(_args: &str) -> String { error_response("grep_source: not implemented") }
fn exec_query_audit_ring(_args: &str) -> String { error_response("query_audit_ring: not implemented") }
fn exec_suggest_command(_args: &str) -> String { error_response("suggest_command: not implemented") }
fn exec_read_concept_note(_args: &str) -> String { error_response("read_concept_note: not implemented") }
fn exec_list_caves(_args: &str) -> String { error_response("list_caves: not implemented") }

fn error_response(msg: &str) -> String {
    let mut s = String::with_capacity(msg.len() + 16);
    s.push_str("{\"error\":\"");
    s.push_str(msg);
    s.push_str("\"}");
    s
}
```

- [ ] **Step 2: cargo check**

Run: `cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3`
Expected: `Finished`.

- [ ] **Step 3: Commit**

```sh
git add src/ai/tools.rs
git commit -m "ai-agent: tools.rs — catalog + dispatch scaffold (six tools, all stubbed)"
```

### Task 7.1.5: Pre-research for tool implementations

Before writing tool implementations, document the exact APIs each tool needs to call. Spend 30-60 minutes mapping the surface; commit the findings as comments at the top of `tools.rs`.

- [ ] **Step 1: Read the relevant existing code**

```sh
# BatFS read API
rg -nE 'pub fn read|pub fn open|pub fn list' src/fs/ src/batfs/ 2>/dev/null

# Audit ring read API (record() exists; recent-entries reader may need to be added)
grep -nE 'pub fn|^impl' src/security/audit.rs | head -20

# Cave list API
rg -nE 'pub fn list|pub fn iter|pub fn count' src/batcave/cave.rs 2>/dev/null

# Shell command name table
grep -nE 'COMMAND_NAMES' src/ui/shell_completion.rs
```

- [ ] **Step 2: Document findings as a comment block at the top of `src/ai/tools.rs`**

After the existing module doc, add a section:

```rust
//! ## Underlying APIs used by tool implementations
//!
//! - `read_file`         calls `crate::fs::<X>::read(path)`         at `src/fs/<X>.rs:<line>`
//! - `grep_source`       walks `crate::fs::<X>::ls()`               at `src/fs/<X>.rs:<line>`
//! - `query_audit_ring`  calls `crate::security::audit::recent(N)`  at `src/security/audit.rs:<line>` (NOTE: may need to be added)
//! - `suggest_command`   reads `crate::ui::shell_completion::COMMAND_NAMES` at `src/ui/shell_completion.rs:<line>`
//! - `read_concept_note` reads from `super::rag::CORPUS`            (in-tree, see Phase 8)
//! - `list_caves`        calls `crate::batcave::cave::list()`       at `src/batcave/cave.rs:<line>`
```

Replace `<X>` and `<line>` with what you actually find.

- [ ] **Step 3: If `audit::recent(n)` doesn't exist, add it as a separate sub-task**

That helper takes `n` and returns the most recent N audit entries (decrypted with the master key). Pattern-match against `audit::record` in `src/security/audit.rs` for the lock + ring-buffer access pattern.

- [ ] **Step 4: Commit the documentation pass**

```sh
git add src/ai/tools.rs
git commit -m "ai-agent: tools.rs — document the underlying APIs each tool will call"
```

### Task 7.2-7.7: Implement each tool

Once 7.1.5 is done you have concrete API surface. For each tool, the pattern is:

1. Parse args from `args_json` using `super::protocol::scan_string_field` (already in protocol.rs).
2. Call the underlying API found in 7.1.5.
3. Format the result as a JSON string.

Each tool gets its own task and its own commit. The TDD pattern per tool:

- [ ] Task 7.2: Implement `exec_read_file` — read a file from BatFS, cap at 32 KiB output.
- [ ] Task 7.3: Implement `exec_grep_source` — naive substring match over `src/`, cap at 50 results.
- [ ] Task 7.4: Implement `exec_query_audit_ring` — read recent entries via audit ring (may need to add a read helper to `src/security/audit.rs`).
- [ ] Task 7.5: Implement `exec_suggest_command` — return the closest match from `COMMAND_NAMES` plus a one-line description.
- [ ] Task 7.6: Implement `exec_read_concept_note` — read from a known set of concept-note names; fail gracefully if the name isn't in the set (vault is on the host disk, not in BatFS — this tool may be a stub on the kernel side and only fully functional under selftest-on-boot, where the vault is bundled at build time).
- [ ] Task 7.7: Implement `exec_list_caves` — call `cave::list()` and serialize.

After all six are implemented:

- [ ] **cargo check + cargo clippy --features gicv3 -- -D warnings clean**
- [ ] **Commit each tool separately** (six small commits)

---

## Phase 8: `rag.rs` — retrieval-augmented generation

### Task 8.1: Implement BM25 over a fixed corpus

**Files:**
- Modify: `src/ai/rag.rs`

**Note:** The kernel can't read arbitrary files at runtime efficiently; we ship the RAG corpus as a `static` table built at compile time via `include_str!`. The build script bundles every Concept note and every `DESIGN_*.md` into the kernel image.

- [ ] **Step 1: Replace skeleton with corpus + BM25**

```rust
//! `src/ai/rag.rs` — retrieval over a compile-time-bundled corpus.
//!
//! The RAG corpus is built into the kernel image via `include_str!`.
//! `top_k(query, k)` returns the most-relevant snippets via BM25.
//!
//! Why compile-time: the kernel doesn't have a great way to lazy-load
//! many small files at runtime, and the corpus is small (~10 Concept
//! notes + 8 design docs ≈ 100 KB). Embedding it in the binary keeps
//! retrieval simple, predictable, and audit-friendly.

#![allow(dead_code)]

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// One corpus entry. `title` is what the snippet is shown as in the
/// prompt; `body` is the content. Both are static — bundled at compile
/// time.
struct CorpusEntry {
    title: &'static str,
    body:  &'static str,
}

static CORPUS: &[CorpusEntry] = &[
    // Concept notes from ~/BAT_OS_VAULT/Concepts/ — bundled at build time
    CorpusEntry {
        title: "M4 Boot Path",
        body: include_str!("../../docs/concept_notes/m4_boot_path.md"),
    },
    CorpusEntry {
        title: "TLS Hardening Journey",
        body: include_str!("../../docs/concept_notes/tls_hardening_journey.md"),
    },
    // ...similar entries for every Concept note + DESIGN_*.md
    // (see Task 8.2 for the bundled-corpus generation script)
];

pub fn top_k(query: &str, k: usize) -> Vec<(String, String)> {
    let q_terms = tokenize(query);
    let mut scored: Vec<(f32, usize)> = CORPUS
        .iter()
        .enumerate()
        .map(|(i, e)| (bm25(&q_terms, e.body), i))
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(core::cmp::Ordering::Equal));
    scored.truncate(k);
    scored
        .into_iter()
        .filter(|(s, _)| *s > 0.0)
        .map(|(_, i)| (CORPUS[i].title.to_string(), CORPUS[i].body.to_string()))
        .collect()
}

fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2)
        .map(|w| {
            let mut s = String::with_capacity(w.len());
            for c in w.chars() { s.push(c.to_ascii_lowercase()); }
            s
        })
        .collect()
}

/// Minimal BM25 — k1=1.2, b=0.75. Doc length is approximated by character count / 5.
fn bm25(q_terms: &[String], doc: &str) -> f32 {
    const K1: f32 = 1.2;
    const B:  f32 = 0.75;
    let avg_doc_len: f32 = 4000.0;            // hand-tuned average across our docs
    let doc_lower = {
        let mut s = String::with_capacity(doc.len());
        for c in doc.chars() { s.push(c.to_ascii_lowercase()); }
        s
    };
    let dl = (doc.len() as f32) / 5.0;        // rough word count
    let mut score = 0.0;
    for term in q_terms {
        let tf = doc_lower.matches(term.as_str()).count() as f32;
        if tf == 0.0 { continue; }
        let numer = tf * (K1 + 1.0);
        let denom = tf + K1 * (1.0 - B + B * (dl / avg_doc_len));
        score += numer / denom;
    }
    score
}
```

- [ ] **Step 2: Skip cargo check until Task 8.2 — the include_str! paths will fail until we generate them**

### Task 8.2: Generate the bundled-corpus files

**Files:**
- Create: `scripts/build_rag_corpus.sh`
- Create: `docs/concept_notes/*.md` (one per vault Concept note)

- [ ] **Step 1: Write the bundle script**

Create `scripts/build_rag_corpus.sh`:

```sh
#!/usr/bin/env bash
# Bundle the Obsidian Concept notes + design docs into a directory the
# kernel can include_str!() at compile time.
set -euo pipefail

REPO="$(git rev-parse --show-toplevel)"
VAULT="${HOME}/BAT_OS_VAULT"
OUT="$REPO/docs/concept_notes"

mkdir -p "$OUT"
rm -f "$OUT"/*.md

# Concept notes — slugify titles
if [ -d "$VAULT/Concepts" ]; then
  for f in "$VAULT"/Concepts/*.md; do
    base="$(basename "$f" .md | tr '[:upper:] ' '[:lower:]_' | tr -dc 'a-z0-9_')"
    cp "$f" "$OUT/$base.md"
  done
fi

# Design docs — copy in
for f in "$REPO"/DESIGN_*.md; do
  base="$(basename "$f" .md | tr '[:upper:]' '[:lower:]')"
  cp "$f" "$OUT/$base.md"
done

ls "$OUT"
```

- [ ] **Step 2: Run the script**

```sh
chmod +x scripts/build_rag_corpus.sh
sh scripts/build_rag_corpus.sh
```

- [ ] **Step 3: Update `rag.rs` CORPUS array to match the actual filenames**

Run `ls docs/concept_notes` to see the actual generated names; edit `CORPUS` in `rag.rs` so each `include_str!` path matches.

- [ ] **Step 4: cargo check**

Run: `cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3`
Expected: `Finished`.

- [ ] **Step 5: Commit**

```sh
git add scripts/build_rag_corpus.sh docs/concept_notes/ src/ai/rag.rs
git commit -m "ai-agent: rag.rs — BM25 retrieval over compile-time-bundled corpus"
```

---

## Phase 9: `audit.rs` — audit ring integration

### Task 9.1: Add `Category::Ai` to the audit ring

**Files:**
- Modify: `src/security/audit.rs`

- [ ] **Step 1: Add `Ai` variant + label**

In `src/security/audit.rs`, edit `pub enum Category` to add:

```rust
    /// AI-agent session events: prompt, tool calls, response.
    Ai          = 10,
```

And in `impl Category::label`:

```rust
            Category::Ai         => "ai",
```

- [ ] **Step 2: cargo check**

Run: `cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3`
Expected: `Finished`.

- [ ] **Step 3: Commit**

```sh
git add src/security/audit.rs
git commit -m "ai-agent: audit.rs — Category::Ai for AI-agent session events"
```

### Task 9.2: Implement `src/ai/audit.rs` helpers

**Files:**
- Modify: `src/ai/audit.rs`

- [ ] **Step 1: Replace skeleton with helpers**

```rust
//! `src/ai/audit.rs` — audit-ring helpers for AI-agent sessions.

#![allow(dead_code)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::security::audit::{record, Category};

pub fn log_session_start(question: &str) {
    let mut buf = Vec::with_capacity(64 + question.len());
    buf.extend_from_slice(b"agent.start q=\"");
    push_summary(&mut buf, question, 80);
    buf.extend_from_slice(b"\"");
    record(Category::Ai, &buf);
}

pub fn log_tool_call(tool: &str, args: &str) {
    let mut buf = Vec::with_capacity(64 + tool.len() + args.len());
    buf.extend_from_slice(b"agent.tool t=");
    buf.extend_from_slice(tool.as_bytes());
    buf.extend_from_slice(b" args=\"");
    push_summary(&mut buf, args, 80);
    buf.extend_from_slice(b"\"");
    record(Category::Ai, &buf);
}

pub fn log_session_end(token_count: u32, ok: bool) {
    let mut buf = Vec::with_capacity(64);
    if ok { buf.extend_from_slice(b"agent.end ok=true tok="); }
    else  { buf.extend_from_slice(b"agent.end ok=false tok="); }
    push_u32(&mut buf, token_count);
    record(Category::Ai, &buf);
}

fn push_summary(buf: &mut Vec<u8>, s: &str, max: usize) {
    let n = s.len().min(max);
    buf.extend_from_slice(s.as_bytes()[..n].as_ref());
    if s.len() > max { buf.extend_from_slice(b"…"); }
}

fn push_u32(buf: &mut Vec<u8>, n: u32) {
    let mut s = String::with_capacity(16);
    use core::fmt::Write;
    let _ = write!(s, "{}", n);
    buf.extend_from_slice(s.as_bytes());
}
```

- [ ] **Step 2: cargo check + commit**

```sh
cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
git add src/ai/audit.rs
git commit -m "ai-agent: audit.rs — session-start, tool-call, session-end audit helpers"
```

---

## Phase 10: `policy.rs` — cave policy entry

### Task 10.1: Define the agent's policy entry

**Files:**
- Modify: `src/ai/policy.rs`

- [ ] **Step 1: Replace skeleton**

```rust
//! `src/ai/policy.rs` — cave policy entry construction for the AI agent.
//!
//! At deploy time the operator configures the inference host's address
//! via a build-time env var (`BAT_OS_AI_INFERENCE_HOST`). This entry is
//! the only egress allowance the agent's connection has; everything
//! else stays default-deny.

#![allow(dead_code)]

pub const DEFAULT_AI_INFERENCE_HOST: &str = "10.0.2.42";
pub const DEFAULT_AI_INFERENCE_PORT: u16 = 443;

pub fn inference_host() -> &'static str {
    option_env!("BAT_OS_AI_INFERENCE_HOST").unwrap_or(DEFAULT_AI_INFERENCE_HOST)
}

pub fn inference_port() -> u16 {
    match option_env!("BAT_OS_AI_INFERENCE_PORT") {
        Some(s) => s.parse().unwrap_or(DEFAULT_AI_INFERENCE_PORT),
        None    => DEFAULT_AI_INFERENCE_PORT,
    }
}
```

- [ ] **Step 2: Wire the env-change rerun into `build.rs`**

Edit `build.rs`. After existing `cargo:rerun-if-env-changed=…` lines, add:

```rust
    println!("cargo:rerun-if-env-changed=BAT_OS_AI_INFERENCE_HOST");
    println!("cargo:rerun-if-env-changed=BAT_OS_AI_INFERENCE_PORT");
```

- [ ] **Step 3: cargo check + commit**

```sh
cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
git add src/ai/policy.rs build.rs
git commit -m "ai-agent: policy.rs — inference-host config via build-time env"
```

---

## Phase 11: `mod.rs` — AgentSession orchestrator

### Task 11.1: Wire all the modules into AgentSession

**Files:**
- Modify: `src/ai/mod.rs`

- [ ] **Step 1: Replace the skeleton with the real orchestrator**

```rust
//! Bat_OS AI agent — locally-hosted, domain-narrow, kernel-mediated.
//!
//! See DESIGN_AI_AGENT.md for the why; docs/PLAN_AI_AGENT.md for the how.

#![allow(dead_code)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

mod audit;
mod client;
mod policy;
mod prompt;
mod protocol;
mod rag;
mod stream;
mod tools;

use client::AgentClient;
use protocol::{ChatDelta, ChatMessage, ChatRequest, FinishReason, Role};
use tools::ToolName;

#[derive(Debug, Clone, Copy)]
pub enum AgentError {
    NetworkUnreachable,
    TlsHandshakeFailed,
    ProtocolError,
    ToolDispatchError,
    Interrupted,
    PolicyDenied,
}

const MODEL_NAME: &str = "bat-os-coder";

pub struct AgentSession {
    client: AgentClient,
    history: Vec<ChatMessage>,
    interrupt: bool,
}

impl AgentSession {
    pub fn new() -> Result<Self, AgentError> {
        let client = AgentClient::new(policy::inference_host(), policy::inference_port())?;
        Ok(Self { client, history: Vec::new(), interrupt: false })
    }

    /// Ask one question. Returns an iterator-like StreamingResponse the
    /// caller polls until completion or interruption.
    pub fn ask(&mut self, question: &str) -> StreamingResponse<'_> {
        audit::log_session_start(question);
        let snippets = rag::top_k(question, 5);
        let req = prompt::assemble(MODEL_NAME, question, &snippets, tools::catalog());
        // We'll send the request lazily on first poll() call so the
        // caller can set up streaming UI first.
        StreamingResponse {
            session: self,
            request: Some(req),
            tokens: 0,
            done: false,
        }
    }

    pub fn interrupt(&mut self) {
        self.interrupt = true;
    }

    pub fn close(self) {
        self.client.close();
    }
}

pub struct StreamingResponse<'a> {
    session: &'a mut AgentSession,
    request: Option<ChatRequest>,
    tokens: u32,
    done: bool,
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Text(String),
    ToolCall { name: String, args: String, result: String },
    Done,
    Error(AgentError),
}

impl<'a> StreamingResponse<'a> {
    /// Poll for the next event. Returns Done when complete.
    pub fn poll(&mut self) -> StreamEvent {
        if self.done { return StreamEvent::Done; }
        if self.session.interrupt {
            self.done = true;
            audit::log_session_end(self.tokens, false);
            return StreamEvent::Error(AgentError::Interrupted);
        }
        // First poll: send the request
        if let Some(req) = self.request.take() {
            if let Err(e) = self.session.client.send_request(policy::inference_host(), &req) {
                self.done = true;
                audit::log_session_end(0, false);
                return StreamEvent::Error(e);
            }
        }
        // Subsequent polls: read chunks
        match self.session.client.poll() {
            Ok(deltas) if deltas.is_empty() => StreamEvent::Text(String::new()),
            Ok(deltas) => {
                let mut text = String::new();
                let mut tool_event: Option<StreamEvent> = None;
                for d in deltas {
                    text.push_str(&d.content_delta);
                    if let Some(tcd) = d.tool_call_delta {
                        let tool = ToolName::from_str(&tcd.name_delta);
                        if let Some(t) = tool {
                            audit::log_tool_call(t.as_str(), &tcd.args_delta);
                            let result = tools::dispatch(t, &tcd.args_delta);
                            tool_event = Some(StreamEvent::ToolCall {
                                name: tcd.name_delta,
                                args: tcd.args_delta,
                                result,
                            });
                        }
                    }
                    if matches!(d.finish_reason, Some(FinishReason::Stop)) {
                        self.done = true;
                        audit::log_session_end(self.tokens, true);
                    }
                }
                if let Some(ev) = tool_event { return ev; }
                self.tokens += text.chars().count() as u32;
                StreamEvent::Text(text)
            }
            Err(e) => {
                self.done = true;
                audit::log_session_end(self.tokens, false);
                StreamEvent::Error(e)
            }
        }
    }
}
```

- [ ] **Step 2: cargo clippy + check**

```sh
cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings 2>&1 | tail -3
cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
```
Expected: both `Finished`.

- [ ] **Step 3: Commit**

```sh
git add src/ai/mod.rs
git commit -m "ai-agent: mod.rs — AgentSession orchestrator + StreamingResponse"
```

---

## Phase 12: Shell integration

### Task 12.1: Add `ai` shell command

**Files:**
- Modify: `src/ui/shell.rs`

- [ ] **Step 1: Find the dispatch table** (around line 335 per existing pattern)

Run: `grep -n '"x509-selftest" => cmd_x509_selftest()' src/ui/shell.rs`
Expected: a line near the shell command-dispatch match arm.

- [ ] **Step 2: Add the new arm**

In the dispatch match, add (alphabetically around the other commands):

```rust
        "ai"             => cmd_ai(args),
        "ai-selftest"    => cmd_ai_selftest(),
```

- [ ] **Step 3: Implement `cmd_ai`**

After the existing `cmd_x509_selftest` function, add:

```rust
pub(crate) fn cmd_ai(args: &str) {
    use crate::ai::{AgentSession, StreamEvent, AgentError};
    let question = args.trim();
    if question.is_empty() {
        crate::drivers::uart::puts("usage: ai <question>\n");
        return;
    }

    let mut session = match AgentSession::new() {
        Ok(s) => s,
        Err(_) => {
            crate::drivers::uart::puts("ai: inference host unreachable. check 5070 + LAN policy.\n");
            return;
        }
    };

    let mut resp = session.ask(question);
    loop {
        match resp.poll() {
            StreamEvent::Text(t) => {
                if !t.is_empty() {
                    crate::drivers::uart::puts(&t);
                }
            }
            StreamEvent::ToolCall { name, .. } => {
                crate::drivers::uart::puts("\n[tool ");
                crate::drivers::uart::puts(&name);
                crate::drivers::uart::puts("]\n");
            }
            StreamEvent::Done => {
                crate::drivers::uart::puts("\n");
                break;
            }
            StreamEvent::Error(_) => {
                crate::drivers::uart::puts("\n[ai: error]\n");
                break;
            }
        }
    }
    drop(resp);
    session.close();
}
```

- [ ] **Step 4: Implement `cmd_ai_selftest`**

After `cmd_ai`, add:

```rust
pub(crate) fn cmd_ai_selftest() {
    use crate::drivers::uart;

    // Subtest 1: prompt assembly
    let req = crate::ai::__test_assemble("hello", &[]);
    if req.messages.len() >= 2 && matches!(req.messages[0].role, crate::ai::__TestRole::System) {
        uart::puts("[ai-selftest] PASS: prompt-assembly\n");
    } else {
        uart::puts("[ai-selftest] FAIL: prompt-assembly\n");
    }

    // Subtest 2: stream framer
    let mut framer = crate::ai::__TestFramer::new();
    let chunk = b"data: {\"choices\":[{\"delta\":{\"content\":\"hi\"}}]}\n\n";
    let deltas = framer.feed(chunk);
    if deltas.len() == 1 && deltas[0].content_delta == "hi" {
        uart::puts("[ai-selftest] PASS: stream-framer\n");
    } else {
        uart::puts("[ai-selftest] FAIL: stream-framer\n");
    }

    // Subtest 3: tool dispatch (read_file with bogus path → error response)
    let result = crate::ai::__test_dispatch_read_file("{\"path\":\"nonexistent.xyz\"}");
    if result.contains("error") {
        uart::puts("[ai-selftest] PASS: tool-dispatch\n");
    } else {
        uart::puts("[ai-selftest] FAIL: tool-dispatch\n");
    }
}
```

- [ ] **Step 5: Add the `__test_*` helpers to `src/ai/mod.rs`**

These are minimal pub-crate test helpers — gated by `#[cfg(feature = "selftest-on-boot")]`:

```rust
#[cfg(feature = "selftest-on-boot")]
pub(crate) use protocol::Role as __TestRole;

#[cfg(feature = "selftest-on-boot")]
pub(crate) fn __test_assemble(q: &str, snippets: &[(String, String)]) -> ChatRequest {
    prompt::assemble(MODEL_NAME, q, snippets, tools::catalog())
}

#[cfg(feature = "selftest-on-boot")]
pub(crate) use stream::StreamFramer as __TestFramer;

#[cfg(feature = "selftest-on-boot")]
pub(crate) fn __test_dispatch_read_file(args: &str) -> String {
    tools::dispatch(tools::ToolName::ReadFile, args)
}
```

- [ ] **Step 6: cargo check + cargo build with selftest-on-boot**

```sh
cargo check --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -3
cargo build --release --target aarch64-unknown-none --features gicv3,selftest-on-boot 2>&1 | tail -3
```
Both expected: `Finished`.

- [ ] **Step 7: Commit**

```sh
git add src/ui/shell.rs src/ai/mod.rs
git commit -m "ai-agent: shell.rs — 'ai' + 'ai-selftest' commands"
```

### Task 12.2: Wire selftest into `selftest-on-boot`

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Find the existing selftest block**

```sh
grep -nA3 'selftest-on-boot' src/main.rs | head
```

- [ ] **Step 2: Add the AI-selftest invocation**

After the existing `cmd_x509_selftest` and `cmd_scheduler_selftest` calls inside the `selftest-on-boot` block:

```rust
    drivers::uart::puts("[selftest] running ai-selftest before auth gate...\n");
    ui::shell::cmd_ai_selftest();
```

- [ ] **Step 3: Build with selftest-on-boot, verify it boots**

```sh
cargo build --release --target aarch64-unknown-none --features gicv3,selftest-on-boot
python3 scripts/qemu_selftests_smoke.py 2>&1 | tail -10
```
Expected: ai-selftest PASS lines among the existing x509 + scheduler ones.

- [ ] **Step 4: Commit**

```sh
git add src/main.rs
git commit -m "ai-agent: wire ai-selftest into selftest-on-boot"
```

---

## Phase 13: Desktop panel

This phase is OPTIONAL for the first ship — the shell command alone is usable. Add the panel in a follow-up if you want a richer UI surface.

If you're doing it now:

- [ ] Task 13.1: Add `App::Ai` variant to the desktop apps enum in `src/ui/desktop.rs`.
- [ ] Task 13.2: Implement `draw_ai_panel(scene)` rendering the chat UI (mirror the existing `draw_files_panel` pattern).
- [ ] Task 13.3: Wire input handling — Enter sends, Esc cancels, Ctrl+C interrupts.
- [ ] Task 13.4: Add the panel to the main desktop draw loop.
- [ ] Task 13.5: Smoke test via the boot-screen UI.

Each task is its own commit.

---

## Phase 14: QEMU smoke

### Task 14.1: Stub ollama responder

**Files:**
- Create: `scripts/serve_stub_ollama.py`

- [ ] **Step 1: Create the stub server**

```python
#!/usr/bin/env python3
"""Tiny stub of the ollama chat-completion endpoint for QEMU smoke.

Listens on localhost:11434 (or port from env). Responds to
POST /v1/chat/completions with a fixed canned streaming response so
the smoke test can verify the Bat_OS-side end-to-end path without a
real model running.
"""
from http.server import HTTPServer, BaseHTTPRequestHandler
import os, sys, json, time

CANNED = [
    {"choices":[{"delta":{"content":"From "},"finish_reason":None}]},
    {"choices":[{"delta":{"content":"`src/net/tls.rs:1234`"},"finish_reason":None}]},
    {"choices":[{"delta":{"content":": the kernel handles handshake."},"finish_reason":"stop"}]},
]

class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path != "/v1/chat/completions":
            self.send_response(404); self.end_headers(); return
        n = int(self.headers.get("Content-Length","0"))
        _ = self.rfile.read(n)
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.end_headers()
        for ev in CANNED:
            self.wfile.write(b"data: ")
            self.wfile.write(json.dumps(ev).encode())
            self.wfile.write(b"\n\n")
            self.wfile.flush()
            time.sleep(0.01)
        self.wfile.write(b"data: [DONE]\n\n")

    def log_message(self, *a, **k): pass

def main():
    port = int(os.environ.get("STUB_PORT", "11434"))
    HTTPServer(("127.0.0.1", port), Handler).serve_forever()

if __name__ == "__main__":
    sys.exit(main())
```

- [ ] **Step 2: Make executable + commit**

```sh
chmod +x scripts/serve_stub_ollama.py
git add scripts/serve_stub_ollama.py
git commit -m "ai-agent: serve_stub_ollama.py — stub responder for QEMU smoke"
```

### Task 14.2: AI smoke test

**Files:**
- Create: `scripts/qemu_ai_smoke.py`

- [ ] **Step 1: Write the smoke**

Mirror `scripts/qemu_pq_interop_smoke.py` structure. The smoke:
1. Starts the stub ollama responder on `127.0.0.1:11434`
2. Builds Bat_OS with `--features gicv3,selftest-on-boot,ai-stub-host` (the `ai-stub-host` Cargo feature points policy.rs at the stub instead of the LAN host)
3. Boots in QEMU with user-mode networking that maps host:11434 to guest's 10.0.2.2:11434
4. Reads serial output for `[ai-selftest] PASS:` lines
5. Asserts ≥3 sub-test PASSes, 0 FAILs, no panic

- [ ] **Step 2: Add `ai-stub-host` Cargo feature**

Edit `Cargo.toml`'s `[features]` section, add:

```toml
ai-stub-host = []
```

And in `src/ai/policy.rs`, gate the host:

```rust
pub fn inference_host() -> &'static str {
    if cfg!(feature = "ai-stub-host") {
        "10.0.2.2"   // QEMU's default user-net host alias
    } else {
        option_env!("BAT_OS_AI_INFERENCE_HOST").unwrap_or(DEFAULT_AI_INFERENCE_HOST)
    }
}
```

- [ ] **Step 3: Run the smoke**

Verify it passes.

- [ ] **Step 4: Commit**

```sh
git add scripts/qemu_ai_smoke.py Cargo.toml src/ai/policy.rs
git commit -m "ai-agent: qemu_ai_smoke.py — end-to-end smoke against stub responder"
```

---

## Phase 15: Eval suite

### Task 15.1: Build the pinned eval set

**Files:**
- Create: `scripts/ai_eval_set.json` (50 Q+A pairs about Bat_OS)
- Create: `scripts/ai_eval.py` (runs the eval against the live 5070)

This is operator-curated content. The plan only sketches scope; engineer judgment + collaboration with operator drives the actual content.

- [ ] Task 15.1a: Draft 50 Q+A pairs covering: 5 questions per major subsystem (kernel, TLS, BatFS, cave isolation, audit ring) + 5 questions per recent V-incident set + 10 cross-cutting questions. Each Q has a known-correct answer that cites a specific file:line. Commit `scripts/ai_eval_set.json`.
- [ ] Task 15.1b: Write `scripts/ai_eval.py` — runs each Q against the live 5070, scores answers by checking that the expected file:line citation appears.
- [ ] Task 15.1c: Run the eval. Establish a baseline pass rate. Aim for ≥80% on first run; document.
- [ ] Task 15.1d: Commit eval script + baseline results.

---

## Phase 16: Real-hardware acceptance

- [ ] Task 16.1: Boot Bat_OS on M4 via chainload.
- [ ] Task 16.2: Run `ai how does the cave-switch TLS wipe work` against the live 5070. Verify response cites `src/net/tls.rs` and includes the V5-XLAYER-001 marker.
- [ ] Task 16.3: Run `ai-selftest` interactively.
- [ ] Task 16.4: Verify audit ring shows the AI session entries (`audit Ai`).
- [ ] Task 16.5: Document the run in `docs/SESSION_JOURNAL.md`.

---

## Phase 17: Final commit + PR

- [ ] **Step 1: Verify all acceptance criteria**

- `cargo build --release --features gicv3` clean
- `cargo build --release --features gicv3,selftest-on-boot` clean
- `cargo clippy --release --features gicv3 -- -D warnings` clean
- `qemu_boot_smoke.py` PASS (regression)
- `qemu_pq_interop_smoke.py` PASS (regression)
- `qemu_selftests_smoke.py` PASS — ai-selftest sub-tests included
- `qemu_ai_smoke.py` PASS — end-to-end against stub responder
- Real-hardware acceptance steps in Phase 16 done

- [ ] **Step 2: Update DESIGN_AI_AGENT.md if anything changed during implementation**

Reflect any deviations from the original spec — e.g., if a tool's signature changed or a parameter was renamed.

- [ ] **Step 3: Push the branch**

```sh
git push origin feat/ai-agent
```

- [ ] **Step 4: Open the PR**

Use the `superpowers:finishing-a-development-branch` skill. Recommended: **Push and create a Pull Request** against `main` (matches PRs #18-#25).

The PR description should reference DESIGN_AI_AGENT.md, list the verification layers above, and link any smoke logs.

---

## Decisions explicitly out of scope

Per design §"Out of scope":

- In-kernel inference (no llama.cpp port; revisit when M4 NPU/AMX drivers land).
- Mutating tools (write_file, run_command, set_policy) — read-only v1.
- Voice input.
- Cloud fallback.
- Multi-tenant / multi-operator sessions.
- Model versioning UI in Bat_OS.

If any of these surface during implementation, they spawn their own thread; do not let scope creep collapse this PR.

🦇

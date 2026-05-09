# DESIGN: Bat_OS AI Agent — Locally-Hosted, Domain-Narrow, In-Kernel-Mediated

**Status:** Active proposal as of 2026-05-08.
**Follows:** `DESIGN_HTTPS_SYSCALL.md`, `DESIGN_TLS_HARDENING.md`, `DESIGN_NO_BROWSER.md`.
**Touches:** new `src/ai/` module, `src/ui/shell.rs` (new `ai` command), `src/ui/desktop.rs` (new agent panel), `src/net/cave_policy.rs` (new policy entry for the inference host), `scripts/qemu_ai_smoke.py` (new), `Cargo.toml` (likely no new in-kernel deps — agent is plain Rust).
**Adds:** native AI assistant integrated into the operator's Bat_OS workflow.

## Goal

Give the operator an AI assistant that lives inside Bat_OS — accessible via a shell command and a desktop panel — and is genuinely useful for working on this specific OS (kernel internals, cave policies, audit-ring entries, design docs, V-incident archaeology). The assistant is fast, locally-hosted on operator-controlled hardware, and refuses nothing on legitimate technical questions.

The differentiator from "use ChatGPT" is twofold: **the model knows this project specifically** (fine-tuned on the source), and **every byte of every conversation stays on hardware the operator owns**. No OpenAI, no Anthropic, no inference-as-a-service.

## Why now

The project keeps accumulating context — the V-incident vocabulary, the Concept notes in the vault, the design docs at the top of the repo. A new operator (or future-you, returning after a month) needs hours to re-orient. A locally-hosted assistant fine-tuned on all of that material drops re-orientation time to minutes.

This is also the natural test of the kernel-mediated HTTPS syscall (`bat_https_open`) — a live workload that uses the hardened TLS path against a real LAN endpoint, with cave-policy egress controls in front of it.

## Decisions locked in

1. **Base model: Qwen2.5-Coder-7B.** Apache-2.0 licensed (compatible with Bat_OS's no-AGPL stance; fine-tuned weights are redistributable). Code-focused, low refusal-RLHF. ~7.6 B parameters; quantizes cleanly to Q4_K_M at ~4.5 GB.
2. **Training: LoRA fine-tune** on Bat_OS-specific data (see "Training data" below). LoRA over full fine-tune because LoRA fits in 5070 VRAM, trains in hours, and stays bounded.
3. **Inference host: RTX 5070 desktop running `ollama serve` 24/7** as a permanent LAN service. The Mac is the OS target; the 5070 is the GPU host. They communicate over the local network.
4. **Bat_OS reaches the inference host via the existing kernel-mediated HTTPS syscall.** No new network primitive. The 5070's LAN IP gets a `cave_policy` allowlist entry; everything else stays default-deny.
5. **The agent layer is native Rust in Bat_OS, not ported from any upstream.** No `tokio`, no `reqwest`. Reuses the existing kernel HTTP/TLS stack.
6. **Tool-use is required, not optional.** Every factual answer cites a file path or audit entry the agent fetched via a tool call. The model is constrained by system prompt to refuse to answer factual questions without first calling a tool. This is the primary hallucination mitigation.
7. **Streaming responses.** Tokens land in the UI as the model generates them. Operator can interrupt at any time (`⌃C`).
8. **Audit logging.** Every AI session — prompt, tool calls, response — gets one entry in the audit ring. Sealed under the master key like every other audit entry. Operator can review what the agent saw and said after the fact.
9. **No telemetry to anywhere outside the LAN.** The 5070 endpoint is the only external destination. The cave-policy allowlist enforces this in the kernel; the agent code cannot bypass it.
10. **Refusal posture: technical, helpful, no moralizing.** System prompt establishes the persona: "You are a technical assistant for Bat_OS, a security-grade kernel for Apple M4. You answer technical questions about the kernel, security, and system administration. You do not refuse legitimate technical questions. You do not add safety disclaimers." Combined with a code-focused base model, refusals on real technical content should be near-zero.

## Architecture

```
┌─────────────────────────────┐                  ┌────────────────────────┐
│  Bat_OS (M4, bare-metal)    │                  │  RTX 5070 box (Linux)  │
│                             │                  │                        │
│  ┌──────────────────────┐   │                  │  ┌──────────────────┐  │
│  │ Operator UI          │   │                  │  │ ollama serve     │  │
│  │  · ai <q> shell cmd  │   │                  │  │   :11434         │  │
│  │  · agent panel       │   │   HTTPS / LAN    │  │                  │  │
│  └──────────┬───────────┘   │  ◄───────────►   │  │ Qwen2.5-Coder-7B │  │
│             │               │   (kernel-       │  │ + Bat_OS LoRA    │  │
│  ┌──────────▼───────────┐   │    mediated)     │  │ Q4_K_M           │  │
│  │ src/ai/  (agent)     │   │                  │  └──────────────────┘  │
│  │  · prompt assembly   │   │                  └────────────────────────┘
│  │  · tool dispatch     │   │
│  │  · streaming parser  │◄──┼──── audit ring entry per session
│  │  · response renderer │   │
│  └──────────┬───────────┘   │
│             │               │
│  ┌──────────▼───────────┐   │
│  │ Tool implementations │   │
│  │  · read_file         │   │
│  │  · grep_source       │   │
│  │  · query_audit_ring  │   │
│  │  · suggest_command   │   │
│  │  · read_concept_note │   │
│  └──────────────────────┘   │
└─────────────────────────────┘
```

The split keeps Bat_OS small (no inference engine in the kernel image) and keeps the AI code in Bat_OS native (no `tokio`, no upstream-port headaches). The expensive piece — the model — runs on hardware that already has the right drivers (CUDA on the 5070 via ollama).

## Components

### Model + training

**Base:** Qwen2.5-Coder-7B-Instruct. Pulled via `ollama pull qwen2.5-coder:7b` for inference; pulled via Hugging Face for fine-tuning.

**Training data** (assembled from the repo + vault):

- **`src/`** — every Rust file. Cleaned of `target/`, generated code. ~155 files, ~30k LOC.
- **`scripts/`** — every Python smoke + helper. ~154 files.
- **Top-level `DESIGN_*.md`** — 8 files including this one once committed.
- **`docs/`** — including `M4_GROUND_TRUTH.md`, `SESSION_JOURNAL.md`, `PLAN_SCHEDULER_BLOCK_ON.md`, `CAPTURES_AUDIT.md`, the obsidian-vault sync output.
- **Concept notes from the vault** — the 10 hand-written editorial notes (`Concepts/M4 Boot Path.md`, etc.). These are the highest-density per-token training signal; weight them heavily.
- **Commit messages** — `git log --pretty=format:"%s%n%b"` over `main`. Captures the project's prose voice, V-incident vocabulary, and rationale style.
- **Audit-comment extracts** — `rg -n 'V\d+-' src/` produces every audit-marker comment with context. The model learns to recognize and explain markers.

Training pairs are constructed as `{instruction, output}`:

- Input: a function signature → Output: docstring + body (next-token-prediction over the actual file)
- Input: a question like "What does V8-ROOT-1 fix?" → Output: the surrounding comment + linked code (synthetic from the audit-marker extract)
- Input: a Concept-note title → Output: the concept-note body
- Input: a commit message subject → Output: the commit message body + diff stats

**Hyperparameters (initial guess; tune after first run):**

- LoRA rank 16, alpha 32
- Sequence length 4096
- Batch size 4 (with gradient accumulation = 8 for effective 32)
- Learning rate 2e-4
- Epochs 3
- Compute: RTX 5070 (12 GB VRAM is plenty for 7B-LoRA at these settings)
- Estimated wall-time: ~6-12 hours

**Output artifact:** the LoRA adapter weights (a few hundred MB), then merged into a Q4_K_M quantized GGUF (~4.5 GB) usable by ollama.

**Retraining cadence:** triggered by significant code or doc changes. Cheap; can be automated to run weekly via a cron on the 5070.

### Inference host

The RTX 5070 runs:

- Linux (Ubuntu 24.04 or similar — no constraint)
- `ollama serve` listening on `:11434` on the LAN
- The fine-tuned model registered as `bat-os-coder:latest`

The host has a stable LAN address — IP or DNS name — that the operator configures at deploy time. The cave-policy allowlist for the agent's connection encodes that exact endpoint and nothing else; everything else stays default-deny. The example `10.0.2.42:11434` used elsewhere in this doc is a placeholder for diagrams; the real value lives in the operator's deploy config and the pinned TLS cert.

ollama provides:

- HTTPS endpoint (with self-signed cert, pinned by Bat_OS — see "TLS pinning" below)
- OpenAI-compatible chat-completion API
- Streaming responses via SSE-like chunked transfer
- Built-in support for tool/function calling (Qwen 2.5 has native tool-use formatting)

### Bat_OS agent layer (`src/ai/`)

New top-level kernel module. Layout:

```
src/ai/
  mod.rs              -- public interface, AI session lifecycle
  prompt.rs           -- system prompt + user-prompt assembly
  client.rs           -- HTTPS client wrapping bat_https_open
  protocol.rs         -- request/response shapes (chat completions API)
  stream.rs           -- streaming token parser, line-buffered
  tools.rs            -- tool dispatch table + per-tool implementations
  rag.rs              -- retrieval-augmented generation: pre-prompt context selection
  audit.rs            -- audit-ring integration
  policy.rs           -- the cave-policy entry the agent's connection runs under
```

**Public API** (consumed by `src/ui/shell.rs` and the agent panel):

```rust
pub struct AgentSession { /* ... */ }

impl AgentSession {
    pub fn new() -> Result<Self, AgentError>;
    pub fn ask(&mut self, question: &str) -> StreamingResponse;
    pub fn interrupt(&mut self);
    pub fn close(self);
}

pub struct StreamingResponse {
    // Iterator-like: returns chunks (text or tool-call events) until done.
}
```

**No `async`.** The agent uses cooperative blocking — the streaming response is consumed by a yield-friendly loop similar to how `epoll_pwait` works after `DESIGN_SCHEDULER_BLOCK_ON.md` lands. Until that lands, the agent's HTTP client falls back to the existing synchronous TLS path.

### Tool catalog

The model is allowed to call exactly these tools. Each has a strict input/output schema. Operator can review the schema in source.

| Tool | Input | Output | Side effects |
|---|---|---|---|
| `read_file` | `{ path: String }` | `{ content: String, path: String, lines: u32 }` | None (read-only over `src/`, `docs/`, top-level `.md`) |
| `grep_source` | `{ pattern: String, path_glob: Option<String> }` | `{ matches: Vec<{ path, line, content }> }` | None |
| `query_audit_ring` | `{ filter: Option<AuditFilter>, limit: u32 }` | `{ entries: Vec<AuditEntry> }` | Reads audit ring under master key |
| `suggest_command` | `{ context: String }` | `{ command: String, explanation: String }` | None — surfaces a suggestion to the operator, who must confirm before exec |
| `read_concept_note` | `{ name: String }` | `{ content: String, name: String }` | Reads from the vault's `Concepts/` directory |
| `list_caves` | `{}` | `{ caves: Vec<{ pid, name, capabilities, policy }> }` | None |

**Out of scope (deliberately) for tool catalog v1:**

- Writing files. The agent suggests; the operator edits.
- Executing shell commands. Same.
- Network calls beyond the inference host.
- Modifying cave policies.

The tool catalog is small and additive — adding tools later is a function of "we trust the model with this primitive, and the operator confirmed."

### UI surface

**Shell command** (`src/ui/shell.rs`):

```
bat_os > ai how does the cave-switch TLS wipe work
```

Spawns an `AgentSession`, streams response inline, falls back to the prompt when done.

Flags:

- `ai --interactive` — opens a multi-turn conversation
- `ai --explain <file>` — pre-fills the question with "explain `<file>`"
- `ai --audit <N>` — pre-fills with "explain audit entry #N"

**Desktop panel** (`src/ui/desktop.rs`):

A new "AI" application alongside the existing apps (Files, Editor, Comms, Dashboard). Layout:

- Top: model + version banner (`bat-os-coder:latest · last trained 2026-05-08`)
- Middle: scrolling conversation history
- Bottom: input field with cyan caret (matching the lock-screen style)
- Tool calls render as expandable strips between user and assistant messages
- Citations render as clickable hairlines (clicking opens the cited file in the Editor app)

Visual style matches the existing dark + cyan-accent palette; shares the `BatMark` component for the panel icon.

## Data flow

A single `ai how does …` invocation, end to end:

1. **Operator types question.** `src/ui/shell.rs` parses the `ai` command and calls `ai::AgentSession::ask(question)`.
2. **Agent assembles the prompt.** `prompt.rs`:
   - System prompt (technical persona, citation requirement, tool-use mandate)
   - RAG context (top-K Concept notes + design-doc snippets matching the question, retrieved via `rag.rs`)
   - Tool definitions (the catalog above, formatted per the OpenAI tool-call schema)
   - User question
3. **Agent opens HTTPS connection** via `bat_https_open(host="10.0.2.42", port=11434)`. Cave-policy check happens in the kernel; this connection's policy entry is the only one that grants egress to the inference host.
4. **TLS handshake.** The kernel's hardened TLS path (PRs #21-#24) does the work. Pinned cert match against the inference host's self-signed cert (operator-installed pin in `src/net/tls_pinning.rs` at deploy time).
5. **Agent sends chat-completion request.** OpenAI-compatible JSON body, `stream: true`.
6. **ollama responds with SSE-like chunks.** Each chunk is either:
   - a text token (appended to the response)
   - a tool-call request (model wants to invoke a tool)
   - a finish marker
7. **On tool-call request:** `tools.rs` validates the call against the catalog, executes the tool, and sends the result back to ollama as a follow-up message in the same conversation. ollama resumes generation.
8. **Streaming parser** (`stream.rs`) yields each token to the UI as it arrives. UI appends to the conversation pane; cursor blinks during gaps.
9. **On `finish_reason: stop`:** session is held open for follow-up turns (or closed by the operator).
10. **Audit entry written** with `{ session_id, prompt_summary, tool_calls, response_summary, ticks_elapsed }`. Same AEAD seal as every other audit entry.

## Hallucination mitigations

Layered, in order of effectiveness:

1. **Tool-use mandatory for factual claims.** System prompt says: "If your answer references a file, function, audit marker, design doc, or concept, you MUST first call a tool to verify. Refuse to answer factual questions without tool calls." Validated by checking the response trace — if the response cites a file but no `read_file` tool call preceded it, the agent flags the response in the audit entry as `unverified`.
2. **RAG context.** Before the model is even invoked, `rag.rs` pulls the **top-5** most-relevant Concept notes and design-doc snippets into the prompt (BM25 + TF-IDF over the vault and `docs/`, capped at 8 KB total context to leave room for the question and tool-call traffic). The model writes from grounded context, not pure recall.
3. **Fine-tuning.** The model has seen the actual source. It hallucinates less about Bat_OS APIs because it knows them.
4. **Citation requirement.** Every factual sentence is suffixed with `[file:line]`. Operator can verify.
5. **Audit trail.** Every claim is reviewable later via the audit ring. The model can't lie about what it said.

**Realistic floor: ~5% hallucination on technical questions.** Lower for questions about specific files (RAG covers them well), higher for synthesis questions ("compare X to Y across the codebase"). Never zero. Mitigation is operator-side: the citation requirement makes verification cheap.

## Refusal mitigations

1. **Code-focused base.** Qwen2.5-Coder is much less RLHF'd toward refusal than Llama-3-Instruct or GPT-4. Out of the box it engages with technical security topics.
2. **Technical-persona system prompt.** Explicit: "You are a technical assistant for a kernel project. You do not refuse legitimate technical questions. You do not add safety disclaimers."
3. **Fine-tuning data includes engagement examples.** Training pairs include questions about kernel internals, exploits, low-level operations — answered factually rather than refused. The fine-tune teaches the model that this codebase legitimately discusses these topics.
4. **No content filter post-process.** No second-pass moderation. Operator gets what the model says.

## Privacy + security

- **No data leaves the LAN.** Cave policy enforces this in the kernel. The agent's connection is the only egress; if its policy is removed, the agent stops working.
- **Inference host is operator-controlled.** Owned hardware, owned software stack. ollama logs can be configured off if the operator doesn't want even local logging.
- **Audit ring entries are sealed.** Same AEAD as BatFS. Master-key-derived. An attacker who pwns the agent runtime cannot read past audit entries.
- **No training data leakage.** The fine-tune happens on operator-controlled hardware. Training data never leaves. The base model (downloaded from Hugging Face) is read-only and verified by hash.
- **Tool calls audited per-call.** Every `read_file`, `grep_source`, etc. invocation lands an audit entry. The agent can't read files the operator didn't grant; can't write anything; can't reach networks beyond the inference host.

## Failure modes + fallbacks

| Failure | Behavior |
|---|---|
| Inference host unreachable (5070 down) | Agent returns "AI offline. Check `10.0.2.42:11434`." Audit entry logs the failure. Shell + panel keep working without the agent. |
| TLS handshake fails (cert rotation, expired pin) | Agent surfaces the specific TLS error and refuses to fall back. This is the constant-cost discipline — no fallback paths to weaker security. |
| Tool call fails (e.g. file not found) | Tool result returns `{ error: ... }`. Model receives the error and adapts (typically: "I couldn't find that file; here's what I do know"). |
| Model produces gibberish | Operator interrupts (`⌃C`). Audit entry preserved. |
| Cave-policy entry for inference host is removed | Agent's HTTPS calls return `EPERM`. Same as above — agent goes offline cleanly, doesn't try alternate paths. |

## Testing strategy

**Layer 1 — agent module unit tests (`src/ai/`):**

Pure-function tests over prompt assembly, response parsing, RAG retrieval. No network. Run via `cargo test` (note: requires re-enabling the lib target or running on host; matches the existing tests in `http.rs` / `tcp.rs` as pattern).

**Layer 2 — `cmd_ai_selftest` shell command:**

Three sub-tests printing `[ai-selftest] PASS: <case>` / `FAIL: <case>`:

- `prompt-assembly` — construct a prompt with known inputs, assert structure
- `tool-dispatch` — invoke each tool with synthetic input, assert output shape
- `streaming-parser` — feed pre-recorded SSE chunks into the parser, assert tokens come out in order

**Layer 3 — `qemu_ai_smoke.py`:**

Boots Bat_OS in QEMU with a stub inference server (a tiny Python responder pretending to be ollama, returning canned replies). Verifies the full path: `ai <question>` → kernel HTTPS → stub responds → tokens stream → audit entry written. Pass criteria:

- `[ai-smoke] PASS handshake-ok`
- `[ai-smoke] PASS tool-call-roundtrip`
- `[ai-smoke] PASS streaming-token-order`
- `[ai-smoke] PASS audit-entry-sealed`
- 0 FAIL lines
- 0 panics

**Layer 4 — manual on-real-hardware test:**

Boot Bat_OS on the M4 via chainload. Run `ai how does the cave-switch TLS wipe work`. Verify response cites `src/net/tls.rs` and includes the V5-XLAYER-001 marker. Verify audit entry recorded.

**Layer 5 — eval suite:**

A pinned set of ~50 questions about Bat_OS internals with known-correct answers. Run after every fine-tune. Track regression: any question that was answered correctly in v1 must still be answered correctly in v2.

## Out of scope (this PR)

- **In-kernel inference.** Porting llama.cpp / candle to `no_std` is months of work and produces something objectively worse than calling out to ollama on the 5070. Revisit when M4 NPU/AMX drivers exist.
- **Tools that mutate state.** No `write_file`, no `run_command`, no `set_policy`. Agent is read-only on the system. Adding mutating tools is a deliberate future scope — each one needs a confirmation flow.
- **Multi-model orchestration.** One model, one host. Pluggable models is a future scope.
- **Voice input.** No microphone story.
- **Cloud fallback.** Inference NEVER falls back to a cloud API. Agent goes offline cleanly if the LAN host is down.
- **Multi-operator / multi-tenant.** Single-user assumption. Sessions don't cross caves; a session belongs to the cave that started it.
- **Model versioning UI.** Operator manages model versions on the 5070 via ollama directly.

## Reversibility

The agent is additive — `src/ai/` is a new module, the shell command is opt-in (`ai <q>`), the desktop panel is one new entry. Removing it: delete `src/ai/`, remove the shell command branch, remove the panel entry. No other subsystem depends on it.

The cave policy entry for the 5070 host is the one piece that touches the existing security perimeter — but it's an *additive* allowlist entry; removing it returns the system to default-deny.

Tag: `pre-ai-agent-2026-05-08` will be applied to `main` before any code lands.

## Implementation plan

A separate plan doc (drafted via `superpowers:writing-plans` after this design is approved) handles the actual phasing — model training first, agent module second, UI last, smoke last. This design doc is the *why*; the plan is the *how*.

🦇

# Wave 8 — AGENT App — Design

Replace the 13-line `paint_agent` placeholder in `src/ui/apps_registry.rs`
with a real WM-app AGENT that wires the existing `src/ai/` Phase-2
scaffold into a calm Wave-4-register Q&A panel. Same pattern Wave 6
used to integrate the existing 11k-line shell into the WM — no logic
changes to the underlying subsystem, just UI plumbing.

The `src/ai/` scaffold's `AgentSession::ask()` currently returns a
`StreamingResponse` whose first `poll()` reports `Done` immediately
(Phase-2 stub). AGENT detects this and shows a "stub mode" placeholder
response. When Phase-5 (`src/ai/client.rs`) lands real HTTPS-to-
inference-host wiring, the same UI renders live streaming responses
without further change.

## Scope

In:

- New `src/ui/apps/agent.rs` — full app: paint, handle_key,
  handle_click, module statics for composer + conversation ring +
  active session.
- `src/ui/apps_registry.rs` — wire `AppId::Agent` from
  `default_handle_key` / `default_handle_click` to the new
  `agent::handle_key` / `agent::handle_click`; replace `paint_agent`
  body to call `agent::paint(rect)`.
- `src/ui/apps/mod.rs` — register `pub mod agent;`.

Out:

- Tool-call dispatch UI surfacing (the agent layer handles tool
  calls internally; we don't visualize them this wave).
- Audit-ring viewer integration (already lives in SECURITY app).
- Cave-policy management for the inference host (separate ops surface).
- Phase-5 inference client wire-up (`src/ai/client.rs` stays stubbed).
- Multi-line composer / Ctrl+V paste (single-line MVP, 256 chars).
- Per-question token budget UI (the agent layer enforces; UI doesn't
  surface yet).
- Conversation persistence (history is in-RAM only, wiped on cave
  switch via `reset_for_cave_switch` like other apps).
- Action strip (composer is the only affordance).

## Visual system

Inherits Wave-3/4/5/6/7. No new constants.

- `p::BG = #0D0D10`, `p::PANEL = #18181C`, `p::HAIRLINE = #2A2A30`
- `p::INK = #E5E7EB`, `p::MID = #6B7280`, `p::FAINT = #4A4D55`
- 8×16 bitmap font (`src/ui/font.rs`)
- Pure-mono discipline preserved (no green/red/amber)

## Layout

Full-window column matching COMMS shape:

```
┌──────────────────────────────────────────────────────────────────┐
│ AGENT  READY                       session 0 · 0 tokens · stub   │  header (32 px)
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  [14:32:01] you                                                  │  conversation
│   how does cave isolation work in sphragis?                      │  history
│                                                                  │  (scrollable,
│  [14:32:01] agent                                                │  newest-at-bottom)
│   (stub mode — wire src/ai/client.rs for live inference)         │
│                                                                  │
├──────────────────────────────────────────────────────────────────┤
│ > █                                                      0 / 256 │  composer (28 px)
└──────────────────────────────────────────────────────────────────┘
```

### Header (32 px)

Drawn via `crate::ui::widgets::draw_strip` (Wave-1 widget, used by
COMMS). Left: "AGENT" wordmark in INK + state label
(`READY` / `THINKING` / `ERROR: <reason>`) in INK. Right: MID text
"session N · M tokens" + optional "stub" tag in FAINT when the
agent session reports it's running against the Phase-2 stub.

### Conversation history (middle)

A scrollable column of Q&A turns. Each turn renders as two
mini-blocks:

```
[HH:MM:SS] you
 <question text wrapped at column width>

[HH:MM:SS] agent
 <response text wrapped at column width>
```

- Timestamp in MID using `[HH:MM:SS]` format (same as COMMS / SECURITY's
  audit panel).
- Role label (`you` / `agent`) in INK.
- Body text in INK for questions, INK for normal responses, FAINT
  for stub-mode placeholders.
- 18-px row pitch matching `paint_activity_log`.

Conversation ring sized at 32 turns; older turns evicted FIFO.
Viewport scrolls via `↑` / `↓` keys.

### Composer (28 px)

Single-line, 256-char cap (matches COMMS's `MAX_MSG_LEN`).

- `> ` prompt in INK when `Idle`, FAINT when `Streaming`.
- Typed text in INK.
- 1-cell INK block cursor (matches EDITOR / COMMS pattern).
- Right side: `N / 256` counter in MID; turns FAINT when `Streaming`.

When `Streaming`: composer placeholder shows "(querying — Esc to
interrupt)" in FAINT.

## State machine

| State | Meaning | Composer | Header label |
|-------|---------|----------|--------------|
| `Idle` | No active query. Composer accepts input. | enabled | `READY` |
| `Streaming` | Query sent. `poll()` driven each desktop tick. | disabled | `THINKING` |
| `Error` | Last query hit `AgentError`. | re-enabled | `ERROR: <reason>` |

State transitions:
- `Idle → Streaming`: Enter pressed with non-empty composer.
  Append turn (question only), call `session.ask(question)`, store
  the `StreamingResponse` in module static.
- `Streaming → Idle`: `poll()` returned `Done`. Trim accumulated
  response into current turn, reset composer.
- `Streaming → Error`: `poll()` returned `Error(_)`. Surface reason
  in header. Trim partial response into current turn.
- `Streaming → Idle` via `interrupt()`: Esc pressed during streaming.
  `session.interrupt()` → next `poll()` returns
  `Error(AgentError::Interrupted)` → as above.

## Module statics

```rust
const MAX_TURNS:    usize = 32;
const MAX_QUESTION: usize = 256;
const MAX_RESPONSE: usize = 1024;

#[derive(Copy, Clone)]
struct Turn {
    active: bool,
    timestamp: u64,                  // monotonic seconds since boot
    question: [u8; MAX_QUESTION],
    question_len: u16,
    response: [u8; MAX_RESPONSE],
    response_len: u16,
    is_stub: bool,
}

static mut TURNS: [Turn; MAX_TURNS] = [Turn::empty(); MAX_TURNS];
static mut TURN_COUNT: usize = 0;
static mut COMPOSE_BUF: [u8; MAX_QUESTION] = [0u8; MAX_QUESTION];
static mut COMPOSE_LEN: usize = 0;
static mut VIEWPORT_START: usize = 0;
static mut APP_STATE: AppState = AppState::Idle;
static mut SESSION: Option<AgentSession> = None;
static mut LAST_ERROR: [u8; 64] = [0u8; 64];
static mut LAST_ERROR_LEN: usize = 0;
static mut SESSION_ID: u64 = 0;
static mut SESSION_TOKENS: u32 = 0;
```

`LAST_ERROR` is a byte buffer rather than `Option<&'static str>` because
the error labels are produced from `&AgentError` (e.g. `error_label(&e)`)
and `'static` lifetime would require interning. The buffer + length pair
gives the same effect with `no_std`-friendly memory.

`AppState` is the enum from the state-machine table above. The
`AgentSession` is constructed lazily on the first Enter so we don't
pay any session-init cost before the operator actually asks.

## Input handling

| Byte | State | Effect | Return |
|------|-------|--------|--------|
| `0x0D` Enter (`\r`) | `Idle` + non-empty composer | Append turn with question, call `session.ask`, transition to `Streaming`. | `Repaint` |
| `0x0D` Enter | `Streaming` | Ignored (composer disabled). | `Repaint` |
| `0x08` / `0x7F` Backspace | `Idle` | Delete from composer if non-empty. | `Repaint` |
| `0x03` Ctrl+C | any | If `Streaming`: call `session.interrupt()`. Else: clear composer. | `Repaint` |
| `0x1B` Esc | any | Same as Ctrl+C. | `Repaint` |
| `0x90` Up | any | Scroll viewport up 1 row. | `Repaint` |
| `0x91` Down | any | Scroll viewport down 1 row. | `Repaint` |
| `0x92` / `0x93` Left/Right | any | Ignored (composer has no in-line edit). | `Repaint` |
| `0x20`–`0x7E` printable | `Idle` | Append to composer if room. | `Repaint` |
| `0x20`–`0x7E` printable | `Streaming` | Ignored. | `Consumed` |
| Other | any | `Unhandled` (lets desktop's Tab/Ctrl+D/Ctrl+L through). | `Unhandled` |

`handle_click` returns `Consumed` for Wave 8 — no click-driven
actions. Composer focus is implicit (typing always goes to composer
when `Idle`).

## Tick driver

**Phase-2 (current — stub mode).** `send_question` drives the entire
poll loop synchronously: after appending the turn and flipping
`APP_STATE = Streaming`, it scopes a `&mut` borrow on `SESSION`,
calls `ask(q)`, and loops on `poll()` until it sees `Done` or
`Error`. Because the stub returns `Done` on the first call, the
whole Q→A cycle completes within a single `handle_key` invocation
and the painter never sees the `Streaming` state on screen. This
sidesteps the borrow-checker friction of stashing
`StreamingResponse<'_>` (which borrows `SESSION`) across paint
cycles in a `static mut`.

```rust
loop {
    match stream.poll() {
        StreamEvent::Text(s)         => append s to current turn's response buffer
        StreamEvent::ToolCall { .. } => no-op for Wave 8 UI
        StreamEvent::Done            => result_state = Idle; break
        StreamEvent::Error(e)        => if Interrupted: result_state = Idle;
                                        else: store_error; result_state = Error; break
    }
}
```

**Phase-5 (future — real HTTPS streaming).** When `src/ai/client.rs`
lands the real inference client, polls will return `Text` deltas
across multiple ticks before `Done`. At that point the driver
restructures: `paint(rect)` polls `StreamingResponse::poll()` once
per paint cycle (same drive point COMMS uses), with the response
handle stashed via a self-referential pattern or interior mutability
on `SESSION`. The synchronous Phase-2 path is a `Done`-on-first-poll
fast path that the per-tick driver subsumes.

In stub mode, the response stays empty; the renderer paints a FAINT
"(stub mode -- wire src/ai/client.rs for live inference)" placeholder.

## Stub-mode detection

After `Streaming → Idle` transition: if the response buffer is
empty AND no `Text` deltas were observed, mark the turn's `is_stub`
flag true. The renderer paints the FAINT placeholder for stub turns
and INK text for real responses.

The header also shows the `stub` tag while the most-recent turn was
stub mode. Once Phase-5 lands and the first real response streams
in, the tag disappears.

## Cross-app handoff

None for Wave 8. AGENT is a standalone Q&A surface. Future waves
might wire FILES → AGENT ("explain this file") via a similar
`set_pending_file`-style hint slot, but that's out-of-scope here.

## Cave-switch reset

Required for the same hygiene reason as COMMS's
`reset_for_cave_switch`: a logged-out cave must not leak its
conversation to the next tenant. Implementation: wipe `TURNS`,
`TURN_COUNT`, `COMPOSE_BUF`, `COMPOSE_LEN`, `VIEWPORT_START`, and
`SESSION` (close + drop). Wire into the existing cave-switch
callback path used by `console`, `comms`, etc.

## Open API gaps

Pre-flight investigation (Task 0c) resolves:

1. **`crate::ui::apps_registry::AppEvent`** + `AppId::Agent` —
   confirmed exist (Wave 2). `paint_agent` shim at `apps_registry.rs:84`.
2. **`crate::ui::widgets::draw_strip`** — Wave-1 widget, used by
   COMMS. Confirm signature `(x, y, w, h, dark: bool, bottom: bool)`.
3. **`crate::ui::widgets::paint_state_dot`** — for the optional
   "stub" tag indicator. Already pub.
4. **`crate::ui::console::redraw_in_rect`** — not used (we render
   conversation directly, not via the console).
5. **`src/ai/mod.rs::AgentSession::{new, ask, interrupt, close}`** +
   `StreamingResponse::poll` + `StreamEvent` — confirmed pub. The
   stub returns `Done` immediately on first poll.
6. **Time source** — `crate::kernel::time::monotonic_secs()` for the
   per-turn timestamp (matches COMMS).

## Failure modes

- **`AgentSession::new()` returns `Err`**: surface in header as
  `ERROR: <reason>`. Composer disabled. Operator can clear via Esc.
- **`AgentError::Network` / `Protocol`**: header `ERROR`, partial
  response retained as a stub turn for inspection.
- **`AgentError::Interrupted`**: header returns to `READY` (operator
  intent), partial response retained without the stub flag.
- **`AgentError::TokenBudget`**: header `ERROR: token budget`,
  composer re-enabled.
- **Long question (>256 chars)**: composer silently stops accepting
  (matches COMMS overflow behavior).
- **Conversation overflow (>32 turns)**: oldest turn evicted FIFO;
  the eviction is silent.

## Testing

- Build clean: `cargo build --release --target aarch64-unknown-none --features gicv3`
- Clippy clean: `cargo clippy --release --target aarch64-unknown-none --features gicv3 -- -D warnings`
- QEMU walk-through:
  - Boot, unlock with `sphragis-dev`
  - Press `8` → AGENT opens with empty state + composer focused
  - Type a question + Enter → question appears in history, response
    immediately fills with stub placeholder, composer clears
  - Type and press Esc → composer clears without sending
  - Run through 5–10 questions → conversation scrolls; header
    shows the session id incrementing
- No automated tests (kernel has no `lib.rs` / test harness).

## Reuse from prior waves

- `crate::ui::widgets::draw_strip` — Wave-1 widget for header strip.
- `crate::ui::widgets::paint_state_dot` — for the optional indicator.
- `crate::ui::font::draw_str` — bitmap-font rendering.
- `crate::ui::gpu::{fill_rect, framebuffer, width}` — framebuffer
  primitives.
- `crate::ui::palette as p` — Wave 3+ palette.
- `crate::ui::apps_registry::AppEvent` and `AppId::Agent`.
- `crate::ai::{AgentSession, StreamingResponse, StreamEvent, AgentError}` —
  the existing Phase-2 scaffold.
- `crate::kernel::time::monotonic_secs` — turn timestamps.

## Out-of-scope reminders (do not implement in Wave 8)

- Phase-5 inference client wire-up.
- Tool-call surfacing in the UI.
- Audit-ring panel integration (lives in SECURITY).
- Cave-policy editor for the inference host.
- Multi-line composer / paste / clipboard integration.
- Per-token streaming animation polish (the UI repaints whole turns
  per paint cycle, which is sufficient at 60 fps).
- Conversation export to BatFS (could route through FILES app later).
- Action strip (composer is the only affordance).

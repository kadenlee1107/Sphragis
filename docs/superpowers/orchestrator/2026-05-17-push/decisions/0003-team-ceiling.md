# ADR-0003: TeamCreate ceiling is 1 — pivot to parallel subagents

Date: 2026-05-17 23:55
Decider: leader
Status: accepted
Supersedes ADR-0001 in part (execution-model choice)

## Context

The plan §3 anticipates a `TeamCreate` ceiling of "4 spawned + 1
leader = 5 total" and prescribes merging Funding + Outreach into a
Bizdev team if the 5th `TeamCreate` fails. The actual ceiling is
tighter than that: only **ONE team alive at a time per leader**.

Observed at 2026-05-17 23:55: created `eng-1-tls` successfully; the
next four `TeamCreate` calls (eng-2-sealfs, eng-3-caves, funding,
outreach) all returned the same error:

```
Already leading team "eng-1-tls". A leader can only manage one team
at a time. Use TeamDelete to end the current team before creating a
new one.
```

This rules out the "5 long-running TeamCreate teams in parallel"
architecture in §3 / ADR-0001.

## Decision

Fall back to the **subagent-per-task pattern** described in §5, with
one adaptation for parallelism. Instead of:

> "Leader spawns a fresh subagent via the Agent tool with the team's
>  full charter + a directive: 'do one TDD cycle (test, implement,
>  commit) and then exit, leaving a status entry in your log file
>  describing what you did and what's next.'"

We use a **single long-turn subagent per team, spawned in parallel**.
Each team's subagent:

1. Is spawned via `Agent{subagent_type=general-purpose,
   run_in_background=true, prompt=<full charter>}` — NO `team_name`,
   so each subagent is a standalone non-team agent.
2. Receives the full charter (mission, reading list, files to modify,
   TDD scenarios, DoD, out-of-scope) inline in its prompt.
3. Runs to completion in ONE turn: reads §4/§5/§7 of the plan, does
   the full DoD, commits each TDD cycle, polls its inbox between
   cycles, writes `STATUS: COMPLETE` to its log as its last act.
4. Returns to the leader via the Agent tool's completion notification
   (since `run_in_background=true`, the leader gets a notification
   when the subagent finishes its single turn).
5. Vault-mediated coordination is unchanged: log/, inbox/, status.md,
   decisions/ — same as the original design.

The five subagents are spawned in a **single message block** so they
all start concurrently per the parallel-agents convention.

The TeamCreate-based long-running model in ADR-0001 is **superseded
by this ADR for execution-model purposes only**. The vault-mediated
coordination protocol is unchanged.

## Trade-offs

- **Pro**: Five teams can actually run in parallel. The plan's parallel
  push model is preserved.
- **Pro**: No SendMessage needed for any reason — coordination AND
  shutdown are non-issues since each subagent terminates naturally on
  Return.
- **Con**: A subagent that goes idle mid-task can't be re-woken — but
  this is identical to ADR-0001's risk.
- **Con**: The leader's `inbox/to-<team>.md` files become harder to
  use to send mid-flight directives to a running subagent, because
  subagents don't loop their inbox polling after completing their
  full DoD (they just exit). Mitigation: the prompt explicitly tells
  the subagent to poll its inbox between TDD cycles / draft sections,
  AND to re-read its inbox once before writing `STATUS: COMPLETE`.

## Consequences

- Leader spawns all 5 subagents in one parallel message block.
- Each subagent's notification on completion lets the leader read its
  log and confirm DoD.
- If a subagent dies mid-DoD (e.g. quality-gate failure escalated to
  URGENT), the leader can either spawn a follow-on subagent to pick
  up where it left off (subagent-per-task fallback) or escalate to
  Kaden per §7.
- The merger of Funding + Outreach into Bizdev is **NOT triggered** —
  the parallelism ceiling is the wrong ceiling; this fallback
  preserves all 5 teams.

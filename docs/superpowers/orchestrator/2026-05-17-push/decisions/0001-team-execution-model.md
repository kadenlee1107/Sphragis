# ADR-0001: Team execution model — long-running TeamCreate agents

Date: 2026-05-17 23:45
Decider: leader
Status: accepted

## Context

`multi-team-push.md` §5 calls for vault-mediated coordination across 5
teams running in parallel. The plan offers two execution models:

1. **Long-running agent in a `TeamCreate` team** — each team is one
   persistent agent that does TDD work in one long turn, polls its
   vault-mediated inbox at natural break points, commits each step,
   and exits only when DoD is met (writing `STATUS: COMPLETE` as its
   last act).
2. **Subagent-per-task fallback** — if `TeamCreate` is unavailable or
   its agents are one-shot, leader spawns a fresh subagent for each
   TDD cycle, reads the log to decide what's next, and respawns.

§9.6 of the plan asks the leader to probe `TeamCreate` and the Agent
tool with `team_name` set, observe whether the agent stays alive or
returns immediately, and record the chosen model here.

## Probe results

- `TeamCreate{team_name=probe-team, agent_type=orchestrator}` returned
  `{team_file_path: ~/.claude/teams/probe-team/config.json,
   lead_agent_id: team-lead@probe-team}` — works.
- `Agent{team_name=probe-team, subagent_type=general-purpose,
   name=probe-agent, prompt=…}` returned immediately with
  `Spawned successfully. agent_id: probe-agent@probe-team. The agent
   is now running and will receive instructions via mailbox.` — i.e.
  the spawn is asynchronous; the leader does NOT block on the
  teammate.
- The probe agent executed its one-shot prompt (append a message to
  `inbox/to-leader.md`), the edit landed in the inbox file (verified
  by reading the file after spawn), and the agent then went idle —
  i.e. the agent stays alive, available to receive further mailbox
  messages, rather than terminating.

## Decision

Use **long-running TeamCreate agents** (model 1). Each team is one
`TeamCreate` + one `Agent` spawn with `team_name`. The agent's prompt
is the full charter from §3 plus the directive to run TDD cycles
until DoD is met, committing each cycle and polling its inbox at
natural break points.

Each `Agent` call uses `run_in_background: true` so the leader stays
free to coordinate the other 4 teams in parallel and poll its own
inbox.

Coordination is **strictly vault-mediated**: the leader and the teams
write to `log/*.md`, `inbox/*.md`, `status.md`, `decisions/*.md` and
commit. **No `SendMessage`** between leader and teams — per §5.
(Note: `SendMessage` is the documented shutdown mechanism for
`TeamCreate` teams; that restriction is for *coordination*, not for
shutdown, so the leader may still send a shutdown_request at session
end if needed.)

## Consequences

- The leader can spawn all 5 teams in parallel from a single message
  block (each `Agent` call returns immediately when used with
  `run_in_background: true`).
- Each team agent must do its full DoD inside ONE long turn — read
  the spec, set up TDD, run cycles, commit each one, poll inbox
  between cycles, write `STATUS: COMPLETE` to the log when done.
- If a team agent goes idle mid-task (e.g. it pauses thinking it
  finished a sub-task), the plan's §5 forbids `SendMessage` to wake
  it up. The leader's only lever is writing to `inbox/to-<team>.md`
  and waiting for the agent to poll it on its own — but a sleeping
  agent will not wake itself. **Mitigation**: the spawn prompt
  explicitly states the agent must not idle before DoD is met; the
  inbox-polling cadence is for *receiving messages* during active
  work, not for waking from idle.
- TeamCreate ceiling not yet probed at scale. If `TeamCreate` rejects
  a 4th or 5th team, fall back per §3 (merge Funding + Outreach into
  Bizdev) and record in ADR-0002.

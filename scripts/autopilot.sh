#!/usr/bin/env bash
# Bat_OS — Ladybird port autopilot.
#
# Drives `claude --print` in a loop. Each fire:
#   1. Reads docs/LADYBIRD_AUTOPILOT.md (single source of truth).
#   2. Executes the next concrete step described there.
#   3. Commits + pushes if the step succeeds.
#   4. Updates the doc with the outcome.
#   5. Sleeps a few minutes between iterations.
#
# Stops when:
#   - Doc has "NEEDS HUMAN:" line (human escalation flagged).
#   - 5 consecutive failed iters (caught by the inner prompt).
#   - You Ctrl-C this script.
#
# Run it inside tmux so it survives terminal close:
#   tmux new -s ladybird-auto
#   ./scripts/autopilot.sh
#   (Ctrl-B D to detach; tmux attach -t ladybird-auto to come back)

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

DOC="docs/LADYBIRD_AUTOPILOT.md"
LOG_DIR="logs/autopilot"
mkdir -p "$LOG_DIR"

ITER_DELAY_SECS="${AUTOPILOT_DELAY:-300}"   # 5 min default between fires
MAX_ITERS="${AUTOPILOT_MAX_ITERS:-50}"      # safety cap; bump if needed

# The meta-prompt. This is what each headless Claude fire receives.
# It establishes the rules, points at the state file, and demands silent
# execution.
read -r -d '' META_PROMPT <<'PROMPT' || true
You are continuing autonomous work on the Ladybird browser port to Bat_OS.

CRITICAL OPERATING RULES:
1. Read docs/LADYBIRD_AUTOPILOT.md fully. The "Current iter" section tells
   you exactly what step to do.
2. Execute that step. Build, test, commit, push.
3. When you'd normally ask the user a question, ask GPT instead via the
   mcp__gpt__ask-gpt tool. Log the question + GPT's answer to the
   "GPT consultations" section of the doc. Act on the answer.
4. Update docs/LADYBIRD_AUTOPILOT.md after each iter:
   - Bump "Current iter" to the next concrete step.
   - Add an entry to "Last 5 outcomes" (drop the oldest if needed).
5. Commit your changes (including the doc update). Push to origin.
6. Then EXIT. Don't loop on your own — the wrapper script does that.

STRICT RULES (do not violate):
- Do NOT editorialize. No "great work today", "want to continue", "want a
  break". Just report facts.
- Do NOT ask the user any questions. Use GPT instead, or NEEDS HUMAN flag.
- Do NOT push broken code. If build/smoke fails, revert and document why.
- Do NOT git push --force, rm -rf outside /tmp, or rewrite history.
- Do NOT skip the commit. Even if the iter only added a doc note, commit it.

NEEDS HUMAN flag (rare, only if):
- Hardware Kaden physically controls is needed (M4 boot, USB, signing keys)
- Secrets / credentials needed
- A truly destructive operation seems necessary (push --force, etc.)
- 5+ consecutive failed iters with no apparent path forward

If you flag NEEDS HUMAN, commit + push the doc update with the flag in the
"Current iter" section, then exit cleanly.

START NOW. Don't acknowledge this prompt — just read the doc and act.
PROMPT

# Sanity checks before we start.
if ! command -v claude >/dev/null 2>&1; then
    echo "[autopilot] FATAL: claude CLI not on PATH" >&2
    exit 1
fi
if [[ ! -f "$DOC" ]]; then
    echo "[autopilot] FATAL: $DOC not found" >&2
    exit 1
fi
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "[autopilot] FATAL: not in a git repo" >&2
    exit 1
fi

start_branch="$(git rev-parse --abbrev-ref HEAD)"
echo "[autopilot] starting on branch: $start_branch"
echo "[autopilot] state file: $DOC"
echo "[autopilot] iter delay: ${ITER_DELAY_SECS}s"
echo "[autopilot] max iters: $MAX_ITERS"
echo "[autopilot] log dir: $LOG_DIR"
echo

iter=0
while [[ $iter -lt $MAX_ITERS ]]; do
    iter=$((iter + 1))
    timestamp="$(date +%Y%m%d-%H%M%S)"
    log="$LOG_DIR/iter-${timestamp}.log"
    echo "[autopilot] === iter $iter at $timestamp — log: $log"

    # Bail if the doc has NEEDS HUMAN.
    if grep -q "NEEDS HUMAN:" "$DOC"; then
        echo "[autopilot] NEEDS HUMAN flag found in $DOC — stopping."
        grep "NEEDS HUMAN:" "$DOC" | head -3
        break
    fi

    # Snapshot the current commit so we can compare after.
    pre_sha="$(git rev-parse HEAD)"

    # Drive claude headlessly. --print exits after one response. We pass the
    # meta-prompt; claude reads the doc itself and decides what to do.
    if ! claude --print \
        --dangerously-skip-permissions \
        --output-format text \
        "$META_PROMPT" >"$log" 2>&1; then
        echo "[autopilot] iter $iter: claude exited non-zero. tail of log:"
        tail -20 "$log"
        echo "[autopilot] sleeping ${ITER_DELAY_SECS}s before retry..."
        sleep "$ITER_DELAY_SECS"
        continue
    fi

    post_sha="$(git rev-parse HEAD)"
    if [[ "$pre_sha" == "$post_sha" ]]; then
        echo "[autopilot] iter $iter: no commit made. tail of log:"
        tail -10 "$log"
        echo "[autopilot] sleeping ${ITER_DELAY_SECS}s..."
    else
        commit_msg="$(git log -1 --format='%s' "$post_sha")"
        echo "[autopilot] iter $iter: committed: $commit_msg"
    fi

    sleep "$ITER_DELAY_SECS"
done

echo "[autopilot] reached max iters ($MAX_ITERS) or NEEDS HUMAN — stopping."

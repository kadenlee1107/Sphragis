#!/usr/bin/env bash
# Bat_OS — Ladybird port autopilot.
#
# Drives `claude --print --session-id <uuid>` in a loop. Every fire pins
# to the same session UUID so context accumulates across iters — Claude
# remembers prior fixes, build errors, GPT consultations within the same
# Claude Code session. When context fills, Claude auto-compacts.
#
# State file: docs/LADYBIRD_AUTOPILOT.md. Single source of truth for
# "what's the current iter, what's the next concrete step." Every fire
# reads it fresh.
#
# Stops when:
#   - Doc has "NEEDS HUMAN:" line.
#   - 5 consecutive failed iters (caught by inner prompt).
#   - You Ctrl-C this script.
#
# Run inside tmux so it survives terminal close:
#   tmux new -s ladybird-auto
#   ./scripts/autopilot.sh
#   Ctrl-B D to detach; tmux attach -t ladybird-auto to come back.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

DOC="docs/LADYBIRD_AUTOPILOT.md"
LOG_DIR="logs/autopilot"
SESSION_ID_FILE=".autopilot-session-id"
SESSION_STARTED_FILE=".autopilot-session-started"
mkdir -p "$LOG_DIR"

ITER_DELAY_SECS="${AUTOPILOT_DELAY:-300}"   # 5 min default between fires
MAX_ITERS="${AUTOPILOT_MAX_ITERS:-50}"      # safety cap; bump if needed

# Generate or load the session UUID. First run creates it; later runs
# reuse it so the same Claude session continues across multiple
# autopilot.sh invocations (you can stop and resume the loop).
if [[ -f "$SESSION_ID_FILE" ]]; then
    SESSION_ID="$(cat "$SESSION_ID_FILE")"
    if [[ -f "$SESSION_STARTED_FILE" ]]; then
        echo "[autopilot] resuming existing session: $SESSION_ID"
    else
        echo "[autopilot] session UUID exists but never started: $SESSION_ID"
    fi
else
    if command -v uuidgen >/dev/null 2>&1; then
        SESSION_ID="$(uuidgen | tr 'A-Z' 'a-z')"
    else
        SESSION_ID="$(python3 -c 'import uuid; print(uuid.uuid4())')"
    fi
    echo "$SESSION_ID" > "$SESSION_ID_FILE"
    echo "[autopilot] new session: $SESSION_ID"
    echo "[autopilot] saved to $SESSION_ID_FILE"
fi

# First-fire bootstrap prompt establishes the rules. The session-id pin
# means later fires continue this same session — they don't need the
# rules re-pasted (they're already in context). But the loop is dumb,
# can't easily distinguish "first fire" from "10th fire", so we always
# pass a SHORT prompt that points at the doc. The DOC contains the rules.
read -r -d '' ITER_PROMPT <<'PROMPT' || true
Continue the Ladybird port autopilot.

1. Read docs/LADYBIRD_AUTOPILOT.md fully (especially "The Rules" and
   "Current iter" sections).
2. Execute the next concrete step described under "Current iter".
3. When you'd ask the user a question, ask GPT instead via
   mcp__gpt__ask-gpt. Log Q+A to "GPT consultations" in the doc.
4. Commit your changes (code + doc). Push to origin.
5. Update "Current iter" + "Last 5 outcomes" sections.
6. Exit. The wrapper loops.

If you flag NEEDS HUMAN, commit the doc with the flag and exit.

No editorializing. No "great work today". No checking in. Just act.
PROMPT

# Sanity checks.
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
echo "[autopilot] branch: $start_branch"
echo "[autopilot] state: $DOC"
echo "[autopilot] session UUID: $SESSION_ID  (context accumulates across fires)"
echo "[autopilot] iter delay: ${ITER_DELAY_SECS}s"
echo "[autopilot] max iters: $MAX_ITERS"
echo "[autopilot] log dir: $LOG_DIR"
echo

iter=0
fail_streak=0
while [[ $iter -lt $MAX_ITERS ]]; do
    iter=$((iter + 1))
    timestamp="$(date +%Y%m%d-%H%M%S)"
    log="$LOG_DIR/iter-${timestamp}.log"
    echo "[autopilot] === iter $iter at $timestamp — log: $log"

    # Bail if NEEDS HUMAN was flagged. Match only lines starting with
    # blockquote `>` so we don't false-positive on the rules-explanation
    # text ("Write `> NEEDS HUMAN: ...`") that talks about the flag.
    if grep -qE "^> NEEDS HUMAN:" "$DOC"; then
        echo "[autopilot] NEEDS HUMAN flag found — stopping."
        grep -E "^> NEEDS HUMAN:" "$DOC" | head -3
        break
    fi

    pre_sha="$(git rev-parse HEAD)"

    # Use --session-id only to CREATE the session on the first fire.
    # On all subsequent fires (in this run OR in any future re-run),
    # use --resume which continues the existing session. claude refuses
    # to reuse --session-id on a session that already exists.
    if [[ -f "$SESSION_STARTED_FILE" ]]; then
        SESSION_FLAG_NAME="--resume"
    else
        SESSION_FLAG_NAME="--session-id"
    fi

    if ! claude --print \
        "$SESSION_FLAG_NAME" "$SESSION_ID" \
        --effort max \
        --dangerously-skip-permissions \
        --output-format text \
        "$ITER_PROMPT" >"$log" 2>&1; then
        fail_streak=$((fail_streak + 1))
        echo "[autopilot] iter $iter: claude exited non-zero (fail streak=$fail_streak)"
        tail -15 "$log"
        if [[ $fail_streak -ge 5 ]]; then
            echo "[autopilot] 5 consecutive failures — stopping. Investigate logs in $LOG_DIR."
            break
        fi
        sleep "$ITER_DELAY_SECS"
        continue
    fi

    # First successful fire — mark session as started so future fires
    # use --resume instead of --session-id.
    if [[ ! -f "$SESSION_STARTED_FILE" ]]; then
        touch "$SESSION_STARTED_FILE"
    fi

    post_sha="$(git rev-parse HEAD)"
    if [[ "$pre_sha" == "$post_sha" ]]; then
        fail_streak=$((fail_streak + 1))
        echo "[autopilot] iter $iter: no commit (fail streak=$fail_streak)"
        tail -10 "$log"
    else
        fail_streak=0
        commit_msg="$(git log -1 --format='%s' "$post_sha")"
        echo "[autopilot] iter $iter: ✓ $commit_msg"
    fi

    sleep "$ITER_DELAY_SECS"
done

echo "[autopilot] stopped (max iters or NEEDS HUMAN)."
echo "[autopilot] session $SESSION_ID preserved — re-running this script resumes."

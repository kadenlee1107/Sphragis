#!/usr/bin/env bash
# Bat_OS — quick git sync helper for Mac↔Ubuntu handoff.
#
# Usage:
#   ./scripts/sync.sh pull     # fetch + merge remote changes
#   ./scripts/sync.sh push     # stage all, commit with default msg, push
#   ./scripts/sync.sh status   # current repo state at a glance
#   ./scripts/sync.sh journal  # print the last 3 SESSION_JOURNAL entries

set -euo pipefail

log() { echo -e "\033[1;34m[sync]\033[0m $*"; }
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

case "${1:-}" in
    pull)
        log "fetching + merging remote"
        git fetch origin
        git pull --ff-only
        ;;
    push)
        # Default commit msg mentions which host is pushing.
        HOST=$(uname -s)
        MSG="${2:-wip from $HOST @ $(date '+%Y-%m-%d %H:%M')}"
        log "committing: $MSG"
        git add -A
        git diff --cached --quiet && log "nothing to commit" && exit 0
        git commit -m "$MSG"
        git push
        ;;
    status)
        log "branch + remote"
        git branch -vv | head -5
        log "dirty files"
        git status -s | head -20
        ;;
    journal)
        log "last 3 entries of SESSION_JOURNAL.md"
        awk '/^## / {c++} c<=3' docs/SESSION_JOURNAL.md
        ;;
    *)
        echo "Usage: $0 {pull|push [msg]|status|journal}"
        exit 1
        ;;
esac

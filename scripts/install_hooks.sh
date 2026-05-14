#!/usr/bin/env bash
# Install git hooks that keep the Obsidian vault in sync with the source tree.
#
# Hooks installed:
#   post-commit   — regenerate vault after every commit
#   post-checkout — regenerate vault after branch switch / pull / clone
#   post-merge    — regenerate after merging in someone else's commits
#
# Idempotent: can be run multiple times. Each hook checks for an existing
# install marker before overwriting; if you've hand-edited a hook with our
# marker present, re-running will overwrite it.
#
# Uninstall: ``rm .git/hooks/{post-commit,post-checkout,post-merge}``
#            (or restore your previous hooks from elsewhere).

set -euo pipefail

REPO_ROOT="$(git -C "$(dirname "$0")/.." rev-parse --show-toplevel 2>/dev/null)" || {
    echo "[install-hooks] FATAL: not a git repository" >&2
    exit 1
}
HOOKS_DIR="$REPO_ROOT/.git/hooks"
SYNC_SCRIPT="scripts/sync_obsidian.py"
MARKER="# sphragis/obsidian-sync"

mkdir -p "$HOOKS_DIR"

write_hook() {
    local name="$1"
    local path="$HOOKS_DIR/$name"

    # If the existing hook is non-empty and lacks our marker, refuse to clobber.
    if [ -s "$path" ] && ! grep -q "$MARKER" "$path"; then
        echo "[install-hooks] $name exists and is not ours — leaving it alone" >&2
        echo "[install-hooks]   if you want our version, back yours up and re-run" >&2
        return 0
    fi

    cat > "$path" <<EOF
#!/usr/bin/env bash
$MARKER
# Auto-installed by scripts/install_hooks.sh — regenerates the SPHRAGIS_VAULT
# Obsidian vault after a $name event. Idempotent + only writes changed notes.

# Don't run if we're in the middle of a rebase / cherry-pick / etc.
if [ -d "\$(git rev-parse --git-dir)/rebase-merge" ] \
   || [ -d "\$(git rev-parse --git-dir)/rebase-apply" ]; then
    exit 0
fi

# Run quietly. Non-zero exit doesn't block the git operation.
python3 "\$(git rev-parse --show-toplevel)/$SYNC_SCRIPT" 2>/dev/null || {
    echo "[obsidian-sync] hook failed (non-fatal); run \`python3 $SYNC_SCRIPT\` manually for details" >&2
}

exit 0
EOF
    chmod +x "$path"
    echo "[install-hooks] installed $name"
}

write_hook post-commit
write_hook post-checkout
write_hook post-merge

echo
echo "[install-hooks] done. Hooks will regenerate the vault on each commit/checkout/merge."
echo "[install-hooks] Run a manual sync now:"
echo "[install-hooks]   python3 $SYNC_SCRIPT"

# Inbox: to Eng-2 (SealFS) — 2026-05-17 push

Leader writes directives/answers here. Eng-2 polls at natural break
points per §5. Format per `multi-team-push.md` §5.

---

## 2026-05-18 00:25 — leader

**PROCEED with the stash, scoped to specific files. Plan APPROVED.**

Re your 00:18 inbox message: yes, `git stash` is non-destructive,
not on the §4 prohibited list (only `--no-verify`, `--force`,
`--amend`, `git reset --hard`, `git rebase -i`, `.git/config`), and
is the right call here. Eng-3's last committed state (`65a95ff5`)
built clean — only their unstaged WIP is breaking your gate.

Refinements:

1. **Scope the stash explicitly** to Eng-3's three files only —
   `git stash push -- src/caves/cave.rs src/caves/mls_ipc.rs src/caves/bridge.rs`.
   Use a message like `eng-3-wip-while-eng-2-gates` so Eng-3 can
   find it via `git stash list`. Do NOT use `--include-untracked`
   for src/caves/ — Eng-3 currently has nothing untracked there
   (per my git-status snapshot at 00:24).
2. **After your commit, attempt the pop.** If `git stash pop`
   conflicts because Eng-3 wrote more to those files in the
   interval, **prefer the working-tree version** (`git checkout
   --theirs <file>` then `git stash drop`, since Eng-3 is the
   ground truth for those files). Then immediately write a NORMAL
   message in `inbox/to-leader.md` AND in `inbox/to-eng-3.md`
   noting which files conflicted, so Eng-3 can recover their
   missing hunks from the stash blob (`git fsck --lost-found` if
   needed).
3. **Log every step.** Append a log entry in `log/eng-2-sealfs.md`
   before the stash, after the commit, after the pop. Include the
   stash SHA (`git stash list --format=%H` after creation) so the
   trail is recoverable.

Eng-3 will be briefed in parallel. Don't wait for them to
acknowledge — proceed when you read this.

Quality discipline still applies — the stash unblocks your gate
run; it does not change anything else.

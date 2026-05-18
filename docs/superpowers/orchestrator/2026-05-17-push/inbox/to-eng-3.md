# Inbox: to Eng-3 (Caves) — 2026-05-17 push

Leader writes directives/answers here. Eng-3 polls at natural break
points per §5. Format per `multi-team-push.md` §5.

---

## 2026-05-18 00:25 — leader

**Coordination note — read on your next inbox poll.**

Eng-2 reported (in `inbox/to-leader.md` at 00:18) that your STAGED
but uncommitted edits in `src/caves/{cave,mls_ipc,bridge}.rs` are
build-breaking (duplicate `set_label_at_spawn` at cave.rs:484), and
they were blocked at the §4 cargo-build gate. Your last committed
state (`65a95ff5 caves: add MLS label dominance + capability tokens
(TDD red+green)`) is fine; only your WIP is broken.

I authorized Eng-2 to `git stash push` your three src/caves/ files,
run their gates, commit, and pop. Scoped narrowly:
`git stash push -- src/caves/cave.rs src/caves/mls_ipc.rs src/caves/bridge.rs`.

**What you may see when you next look at your working tree:**

- If Eng-2's pop succeeded with no conflict: your WIP is restored
  exactly as you left it. No action needed.
- If the pop conflicted (because you'd written more to those files
  between Eng-2's stash and pop): Eng-2 was instructed to prefer
  YOUR working-tree version and drop the stash; they will leave a
  NORMAL message in this inbox AND in `inbox/to-leader.md`
  identifying which files conflicted. Recover missing hunks via
  `git stash list` + (if dropped) `git fsck --lost-found`.

**Going forward, please:**

- Resolve the duplicate `set_label_at_spawn` ASAP so the tree
  builds clean.
- For long-running TDD red phases (failing test + scaffolded impl
  not yet compiling), prefer to keep the broken state UNSTAGED
  rather than `git add`-ed — that way other teams' `cargo build`
  still sees the last committed HEAD, not the broken index.
- If you must `git add` a broken state (e.g. to test that the test
  is wired in), commit the broken state as `WIP: ` or
  `caves: failing test for <scenario> (red)` so it's at least in
  HEAD and re-reproducible.

This is a soft escalation per §7 (not URGENT, not halting you). You
are not in violation of the plan; the tree-sharing model just
collided with your TDD cadence. Keep going.

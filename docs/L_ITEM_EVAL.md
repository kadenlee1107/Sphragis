# Feasibility / scope evaluation — L-effort gap-audit items

Triage notes for the Large-effort items remaining in
`docs/OS_FEATURE_GAP_AUDIT.md`. Each entry: what the gap is, what
the *minimum-viable* shape looks like for Sphragis's threat model, an
honest implementation cost, and a recommended go/no-go.

## 030 — Per-cave CPU / memory / IO quotas (cgroups v2 equivalent)

**What's missing.** Sphragis has cave isolation for capabilities,
filesystem keys, and (now) PID + mount namespaces — but **no resource
limits**. A misbehaving cave can spin on the CPU, allocate every
remaining page, or saturate the NIC's TX ring. Nothing reins it in.

**Threat model.** A compromised cave is the primary concern. The
secondary concern is honest-but-buggy code (an infinite loop in a
shell script). Both are bounded today only by the watchdog timer
and our fixed-size kernel allocations.

**Minimum viable shape.**

- **CPU**: per-cave timeslice quota. The scheduler already tracks
  per-task priority; extend it to track per-cave cumulative ticks
  consumed during a window. When a cave exceeds its share, lower
  every task's effective priority by N for the next window. ~150 LOC
  in `scheduler.rs` + a `cpu_quota` field on `BatCave`.
- **Memory**: per-cave page count. `mm::frame::alloc_frame` currently
  has no idea who's asking. Need to thread the active cave id through
  the allocation path (or look it up via `cave::get_active()` at
  alloc time). Track per-cave page count; reject allocations past
  the cave's limit with `Err("cave: memory quota exceeded")`. ~200
  LOC across mm + cave struct.
- **IO**: per-cave TX byte counter on the NIC path. The flow shaper
  already exists (`net::flow_shaper`) — extend it to scope flows by
  cave id rather than just by 5-tuple. ~80 LOC.

**Total estimated effort:** ~450 LOC + a thoughtful "what's the
default quota" decision per resource type. **Real work**, not
shallow plumbing — every allocator and every scheduler tick gets a
new check.

**Recommended:** **Yes — but as a focused mini-project, not a
one-shot batch.** Three sub-PRs (cpu, mem, io), each independently
shippable. Memory quota first — it's the highest-impact one (an OOM
in a cave shouldn't kill everything else).

## 036 — /proc-equivalent (procfs)

**What's missing.** Linux has `/proc/<pid>/{cmdline,status,maps,...}`
exposing per-process kernel state through the filesystem. We don't.
Operators can't introspect what tasks are doing without a custom
shell command per piece of state.

**Threat model.** /proc is a *defensive* feature — needed when you're
debugging a misbehaving cave or auditing what's running. Not adding
new attack surface; existing audit/inspection primitives just become
harder to compose.

**Minimum viable shape.** A real procfs would mean BatFS gets a
*pseudo-file* type whose contents are computed at read time from
kernel state. That's a meaningful BatFS extension (~300 LOC to add a
PseudoFile variant alongside the existing encrypted-page-backed
File). Then population: `/proc/<tid>/name`, `/.../state`,
`/.../caps`, `/.../fds`, `/.../cave` — call it 50 LOC per virtual
file, 6 files per task = 300 LOC. Plus a directory-listing walker.

**Total estimated effort:** ~600-800 LOC for a credible MVP.

**Alternative — what we built today already partially closes this.**
The `procs` shell command (committed in this batch) reads the same
state /proc would expose. `caps` and `fds` shell commands could give
the rest without a filesystem layer. Trades the "compose via cat" UX
for ~80 LOC of shell commands.

**Recommended:** **Defer the filesystem version.** Add `caps`, `fds`,
`task <tid>` shell commands first (~150 LOC total) — they cover 90%
of the actual use case. Real procfs is worth doing once we have
*pseudo-file* infrastructure for some other reason (a `/sys`
equivalent for hardware introspection would also want it).

## 037 — libc / libc-compat shim (XL, listed here for context)

Beyond the L scope but worth a note: we already run hand-built
no_std C-compatible code through the cave loader (the `hello`,
`hello_libc`, `cxx` test binaries exist). A real libc-compat shim
(musl-class) is XL and probably not what Sphragis wants long-term —
the security model favors purpose-built no-libc workloads.

Recommended posture: keep the test-binary path for ground-truth ABI
work, but resist adding a real shim. Document non-goal in
`DESIGN.md`.

## Summary

| Item | Recommend | Effort | Rationale |
|------|-----------|--------|-----------|
| 030  | Yes, in 3 PRs | ~450 LOC | Real security gap; cave isolation is incomplete without it |
| 036  | Partial (shell cmds first) | ~150 LOC now, ~600 later | Real procfs needs pseudo-file infra worth building once |
| 037  | No, document as non-goal | XL | Security model doesn't want it |

# Handoff prompt — paste this when starting a fresh session

```
You're picking up Bat_OS M4 work. Your job: lift the per-cycle
HV session ceiling from ~60-96s to multi-minute by either
(a) reverse-engineering M4 CPU chicken bits from an iPad M4
IPSW kernelcache, or (b) finding the PMGR cluster-wake sequence
that lets us safely write the APSC bit on PCPU MMIO.

Read these files in this order before you do anything else:

  1. docs/M4_CHICKEN_HUNT.md  — both paths laid out with first
     commands. Read this end-to-end. It tells you exactly what
     to install, what binaries to fetch, what registers to look
     for. There is no "next session" — you finish whichever path
     you start.

  2. docs/SESSION_JOURNAL.md  — newest entry is 2026-04-20 17:15
     "XNU H16 diff + live-HW probe answer WHY M4 chickens fail".
     Skim earlier entries from 2026-04-20 only — anything older
     is stale context.

  3. docs/m4_re/H15_vs_H16.diff.txt  — definitive M3 vs M4 CPU
     feature delta from Apple's XNU drop. This is the source of
     truth for what changed at the CPU-generation level.

  4. CLAUDE.md  — project conventions (build_apple.sh, no Windows
     proxy, terse responses, etc.).

Stop reading after those four. Do not re-survey what's been
tried — the journal entries from this afternoon already prove
SMC, AOP, AIC drain, smc-pet, TX ring, naive APSC, and M3
chicken reuse all dead ends, with reasons. Don't redo any of
those.

Pick path B (PMGR cluster wake) FIRST. It's the cheaper experiment
— one PMGR power-enable write, retest PCPU MMIO accessibility,
done in 30 minutes. If it works, the APSC enable from main.c
just becomes pmgr_adt_power_enable(...) + the two set64 lines
already shown in the chicken-hunt doc, and we're shipping a
real ceiling fix. If PMGR wake doesn't make PCPU MMIO accessible,
move to Path A (IPSW disassembly) — that's a longer session
but completely tractable from this Ubuntu host.

Keep doing what works in this codebase:
  - sg dialout -c "..."  for any subprocess that touches /dev/ttyACM*
  - sudo -n with --preserve-env=M1N1DEVICE,M1N1WAIT for chainload
  - bash build_apple.sh for Bat_OS rebuilds (NEVER plain cargo
    build --release on Apple — see CLAUDE.md feedback note)
  - make -C external/m1n1 -j4 for m1n1 rebuilds
  - scripts/hv/run_hv_forever.sh for endurance testing — it's
    the supervisor that auto-recovers across resets

Tag hv-96s-baseline at d9a454f0 is the emergency rollback.
Anything in cpufreq.c / chickens.c / main.c that breaks chainload,
just `git checkout hv-96s-baseline -- <file>` and rebuild.

Mac will reset the moment you crash the chainload. Sometimes it
takes 60-180 seconds to come back to stock m1n1, sometimes it
goes into macOS — wait the full 5 minutes; don't conclude
"Mac is dead" until then. If it sticks in macOS, hold the
power button and pick m1n1 in the boot picker — Kaden has to
do that part physically, supervisor will say "Mac seems to
have booted into macOS" when it sees only /dev/ttyACM0.

Update docs/SESSION_JOURNAL.md with what you found and what
you tried, every commit. Push to origin/feat/js-engine-browser-posix.
Use the supervisor for endurance tests — it'll give you
n/min/max/p50 stats automatically so you can see if you've
moved the ceiling.

You have everything you need. Just start with Path B step 1
in docs/M4_CHICKEN_HUNT.md — dump_pmgr.py output is your first
data point.
```

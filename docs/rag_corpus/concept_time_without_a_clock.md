---
type: concept-note
topic: time · architecture
---

# Time without a clock

> Sphragis has no real-time clock. There is no battery-backed RTC chip on the M4 the kernel is wired to read. There is no NTP daemon. There is no `gettimeofday()` syscall in any meaningful sense. And yet a security workstation has to reason about time — certificate validity, sleep deadlines, audit timestamps. This note is how that contradiction is resolved.

## The two clocks the kernel does have

Both come from the ARMv8 generic timer — hardware that fires on every aarch64 core.

1. **`cntpct_el0`** — a free-running 64-bit counter, monotonically increasing. Reset to zero at power-on. Never goes backward, never wraps in any reasonable lifetime (with `cntfrq_el0` typically around 24 MHz, it takes 24,000 years to wrap a 64-bit register).

2. **`cntfrq_el0`** — a 64-bit register that tells you the rate `cntpct_el0` ticks at. On the M4 this is around 24 MHz. The kernel reads it once at boot and treats it as a constant.

That gives the kernel **monotonic time** with sub-microsecond resolution. What it doesn't give: **wall-clock time** ("what calendar date is it"). The boot moment is `cntpct_el0 = 0`, and there's no link between that and any external date.

## Where wall-clock time matters

Three places in the kernel:

1. **Certificate validity periods** ([[_generated/src/net/x509.rs]]). RFC 5280 says a cert is invalid before its `notBefore` and after its `notAfter`. Both are absolute UTC dates. Without wall-clock time, the kernel can't directly say "is now within [notBefore, notAfter]?"

2. **Audit ring timestamps** ([[_generated/src/security/audit]] / [[Concepts/Audit Ring Contract]]). Each entry should have a "when did this happen" — but without a wall clock, the best the kernel can stamp is "this happened at boot+N ticks."

3. **Sleep deadlines** ([[_generated/DESIGN_SCHEDULER_BLOCK_ON.md]] / [[Concepts/Scheduler Park-on-Deadline]]). `sys_nanosleep(30s)` needs to know when 30 seconds have passed. Monotonic time is enough for this — "wake when `cntpct_el0` ≥ start + 30s × cntfrq" — no wall clock needed.

For (3), monotonic time is the right tool and there's no problem. For (1) and (2), the kernel needs something more.

## The build-time epoch trick

For (1) — certificate validity — Sphragis uses **`SPHRAGIS_BUILD_UNIX`**: a Unix epoch timestamp baked into the binary at compile time, set by `build.rs` from the build host's `SystemTime::now()`. See [[_generated/build.rs]] and [[_generated/src/net/x509.rs]] (the `now_unix()` function).

The verifier uses this as a **lower bound**:

- A cert whose `notBefore` is after `SPHRAGIS_BUILD_UNIX` is rejected as `NotYetValid`. ("Signed in the future relative to when this binary was built" — that's a fact about the binary, not a fact about now.)
- A cert whose `notAfter` is before `SPHRAGIS_BUILD_UNIX` is rejected as `Expired`. (Same logic, other direction.)

What this catches:
- **Future-dated certs**, definitely. If a CA mistakenly issued a cert with `notBefore` two years from now, the kernel rejects it at any time during this binary's lifetime.
- **Already-expired certs**, partially. A cert that expired before the build is reliably caught. A cert that expires while the binary is running but didn't expire at build time will *not* be caught — the kernel treats it as still valid until rebuilt.

This is conservative-but-stale on the `Expired` side and strictly correct on the `NotYetValid` side. The right failure mode for an offline kernel: a binary that runs for years won't reject good certs that happen to expire mid-run, but it also won't accept obviously-future-dated bad certs.

## What this means for operations

A few practical consequences:

- **Rebuild before evaluation.** An evaluation build that's a year old has a year-old `SPHRAGIS_BUILD_UNIX` and will accept certs that expired in the past year. For best hygiene, rebuild close to deployment.
- **Certificate revocation is not handled.** Even if a cert was correctly issued at build time and is correctly time-valid, if it was revoked mid-validity-period, this kernel won't know. Revocation (OCSP / CRL) is explicitly out of scope per [[_generated/src/net/x509.rs]]'s header comment. Operators in high-security environments do their own revocation tracking.
- **`KNOWN_GOOD_TIME` operator override** is reserved for future use. The header in [[_generated/src/net/x509.rs]] mentions it as the path for an operator who's verified time from an external source (e.g. a freshly-handshaked PQ-TLS connection to a known peer) and wants to update the kernel's time floor without rebuilding. Not implemented yet.

## What this means for the audit ring

Currently: audit entries are stamped with `cntpct_el0` ticks-since-boot, not Unix epoch. An entry says "this happened 12,345,678 ticks after boot" — about 514 seconds after boot at 24 MHz.

That's enough for **ordering** within a session (entry #5 is after entry #3) and enough for **rough relative durations** (entry #5 is about 30 seconds after entry #3). It is **not** enough for "this happened at 14:22:08 on 2026-05-08."

For most operations this is fine — when you decrypt the audit ring, the dump has a known boot timestamp (SPHRAGIS_BUILD_UNIX + observed boot delay) and you can reconstruct rough wall-clock times. For forensic-grade timestamping, the operator would correlate the audit ring with external time sources at unlock time. (How that correlation works: also reserved.)

## What this means for the scheduler

For sleep deadlines, `cntpct_el0` is exactly the right thing. The plan in [[_generated/docs/PLAN_SCHEDULER_BLOCK_ON.md]] is explicit:

> All deadline math uses `cntpct_el0` absolute ticks. Not nanoseconds, not milliseconds, not timer-tick-counts. The wakeup pass compares a thread's `deadline_ticks` against `cntpct_el0()` directly, no conversion.

Threads sleeping on a deadline don't care what time it is — they care how many ticks have passed. The hardware counter is the source of truth.

## In sum

| Need | Source of truth | Limitation |
|---|---|---|
| Cert "is now after notBefore" | `SPHRAGIS_BUILD_UNIX` baked into binary | Binary-stale on `Expired` after rebuild |
| Cert "is now before notAfter" | Same | Same |
| Audit ring entry order | `cntpct_el0` ticks | No wall clock |
| Audit ring entry rough time | `cntpct_el0` + boot reconstruction | Needs external correlation for forensic use |
| Sleep deadline | `cntpct_el0` ticks (absolute) | None — this is the correct tool |

Sphragis reasons about time the way an offline cryptographer would: trust the local clock for relative ordering, trust the build-time pin for "what year is this," demand external evidence for anything sharper.

## Related

- [[_generated/src/net/x509.rs]] — `now_unix()`, validity-period checks
- [[_generated/build.rs]] — where `SPHRAGIS_BUILD_UNIX` is set
- [[_generated/DESIGN_SCHEDULER_BLOCK_ON.md]] — `cntpct_el0`-based deadlines
- [[Concepts/Audit Ring Contract]] — how ring entries get timestamped
- [[Concepts/TLS Hardening Journey]] — why `Validity` checks landed where they did (PR #21)

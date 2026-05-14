---
type: concept-note
topic: isolation · caves
---

# Cave isolation model

> A cave is what other systems call a process. The naming is deliberate — "process" implies a running thing, "cave" implies a sealed container. Sphragis thinks of caves as the unit of *trust*, not just execution.

## The contract

Every cave has:

1. A **PID** — same as any OS.
2. A **capability set** — what the cave is allowed to do (NET, RAW, DSP, FS, etc.).
3. A **policy** — what hostnames, ports, and SNI it can reach over the network.
4. An **origin handle** — the BatFS handle that gates which files it can read or write.
5. An **audit owner** — every event the cave generates is stamped with its PID; the audit ring records who did what.

A cave with no policy entry cannot reach the network at all. A cave with no FS capability cannot open a file. The default for everything is *deny*.

## Capabilities

Capabilities are typed permissions, not strings:

- **NET** — can call `bat_https_open`. Without NET, the syscall returns `-EPERM`.
- **RAW** — can construct raw IPv4 packets via the Mesh stack (rare).
- **DSP** — display compositor write. Required for any cave that draws.
- **FS** — BatFS read/write. Without FS, the cave gets a sealed view (read-only of its own writes only, no shared dir).

These are baked into the cave's handle at creation time. There is no syscall to escalate; a cave cannot ask for more than it was given.

## Default-deny egress

This is **Hush** (see [[_generated/src/net/cave_policy.rs]] and [[_generated/src/net/policy.rs]] if it exists). Every `connect()` is checked against the calling cave's policy:

- Allowed hostnames (exact + wildcard)
- Allowed ports
- Allowed SNI for TLS

A cave with no policy entry gets `-EPERM` on every outbound connect. Every denial is recorded in the [[Audit Ring Contract|audit ring]] with PID + dest + SNI.

There is no "block all by default but admin can flip a switch." There's no admin. The policy is set at cave creation and is part of the cave's identity.

## Why this is in the kernel

A user-space firewall is a process with privileges, which means a process with bugs. Hush evaluates every `connect()` *below* the syscall — there is no userspace code path to bypass. A cave that wants to reach the network has exactly one route: the kernel evaluates its policy and decides.

The kernel-mediated HTTPS syscall ([[_generated/DESIGN_HTTPS_SYSCALL.md]]) is the next layer up: even *with* network access, a cave never sees TLS bytes. It hands the kernel a hostname and gets back a plaintext file descriptor; the kernel does the handshake, chain validation, and encryption.

## How a cave is born

The shell command (or the kernel itself) creates a cave with:

```
cave-run sandbox/my-app           # creates a cave named my-app with sandbox-policy
```

Behind the syscall:

1. Kernel allocates a PID
2. Kernel reads the named policy (e.g., `sandbox/my-app.toml`) from BatFS
3. Kernel constructs the cave handle: PID + capabilities + policy + audit owner
4. Kernel hands the cave's binary an entry-point and lets it run

A cave that crashes is removed; its audit entries persist. A cave that panics propagates to the kernel as an uncaught fault, which itself logs to the ring.

## Cave switch / TLS state

When the kernel switches caves, **every** TLS state pertaining to the previous cave is wiped. See V5-XLAYER-001 in [[_generated/src/net/tls.rs]]: `reset_all_sessions` is called from `cave::enter` on every switch, wiping session keys, SPKI, expected hostname, and cert-pinning state inherited from a prior tenant. V8-ROOT-1 wraps the loop in an IRQ critical section so a timer fire mid-loop can't leave half the sessions wiped.

This is the kind of thing that's invisible until an audit asks "what happens to TLS state when caves switch?" and the answer matters.

## Related

- [[_generated/src/cave]] — the implementation (cave creation, switch, teardown)
- [[_generated/DESIGN_CAVES.md]] — the original design doc
- [[_generated/src/net/cave_policy.rs]] — Hush
- [[_generated/DESIGN_HTTPS_SYSCALL.md]] — kernel-mediated HTTPS, layered on cave isolation
- [[Concepts/Audit Ring Contract]] — every cave event lands here

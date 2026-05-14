---
type: concept-note
topic: security · audit
---

# Audit ring contract

> Echo. The append-only kernel-only-writer ring buffer that records every security-relevant event the OS sees. Most "audit logs" are files; this one is not. This note is the contract.

## What goes in the ring

- Denied egress (Hush DENY events)
- Successful and failed authentication
- Master-key derivations (Argon2id rounds, KDF parameter changes)
- Signature verification failures
- Lock-screen transitions (lock, unlock, dead-man's-switch fires)
- TLS handshake outcomes (PQ negotiation, chain anchor)
- Filesystem seal/unseal events on BatFS
- Driver init failures

What does *not* go in: timing data fine enough to use as a side channel, plaintext of payloads, anything that would reverse-engineer a secret.

## Why it lives in the kernel

A file-based audit log is just a file — modifiable by anything that can write that file. Echo is a kernel ring; the kernel is the only writer. Userspace cannot append to the ring directly; subsystems publish via a kernel-internal interface that stamps each entry with a monotonically-increasing `#N`.

This means: an attacker who pwns a cave cannot forge an entry, suppress an entry, or reorder entries. The ring fails closed — if the kernel can't append (e.g., out of space), the operation that would have generated the entry fails.

## Storage layout

- 1024 entries (current default). Spillover semantics are still being decided — overwrite vs. seal-and-rotate is an [open question in [[_generated/src/security/audit/mod.rs]]].
- Each entry is fixed-size and sealed under the same master key as [[Cryptography Stack|BatFS]]. AEAD: ChaCha20-Poly1305.
- The ring lives in BatFS as a special file but is mapped read-write by the kernel only. User caves cannot stat or read it directly.

## How callers use it

The pattern, from any security-relevant code path:

```rust
// (Notional — actual API may have different shape)
audit::record(AuditEvent::EgressDeny {
    pid: caller.pid(),
    dst: ip_port,
    sni: sni_str,
});
```

The function returns nothing — there is no error path. If recording fails, the kernel panics. (This is a deliberate choice: silent failure to record a security event is worse than a halt.)

## Why 1024 entries

Enough for a session of work, not enough for a deployment. The number is tuned so an operator can scroll through a session's worth of events; an attacker generating noise to push real events out of the ring would have to generate 1000+ events, which is itself an audit signal.

For long-running systems, the ring will rotate. The pre-rotate slice is sealed and persisted to BatFS as a numbered file (`audit.0`, `audit.1`, …). The unsealed-in-memory window is always the latest 1024.

## What the operator sees

The shell has a `audit` command (in [[_generated/src/ui/shell.rs]]) that decrypts the ring and prints recent entries. Sample output (from [[Concepts/M4 Boot Path]]):

```
#247  DENY  out 8.8.8.8:53 · cave my-app
#246  AUTH  passphrase · session #4
#245  DERIV argon2id 8 MiB / 3p
#244  SEAL  batfs / boot.log
#243  TLS   ml-kem-768 · ISRG X1
#242  VRFY  firmware sig · ok
```

Each entry has a category, a one-line description, and the originating cave. The shell's audit pane mirrors the lock-screen's bottom-left boot-log strip in style.

## Related

- [[_generated/src/security/audit]] — the implementation
- [[Concepts/Cave Isolation Model]] — caves are who generates events
- [[Concepts/Cryptography Stack]] — same AEAD as BatFS
- [[_generated/DESIGN_CAVES.md]] — design doc that calls out the ring's role

## Open

- Spillover: overwrite vs. seal-and-rotate. Operations sees both arguments.
- Entry size: currently fixed; some events would benefit from variable-length context.
- Off-device export: explicitly *not* supported. The ring stays on the device; if you need it elsewhere, the device shows it to you and you transcribe.

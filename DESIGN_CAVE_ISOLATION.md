# DESIGN — Cave Isolation Primitives

> **Status:** Implementation arcs landed 2026-05-12. This document
> consolidates the design that emerged from the security-depth run
> the cave-MMU work + WireGuard relocation produced. New arcs that
> isolate kernel state (sys-net buffers, future sys-tor circuits,
> etc.) should reuse these primitives rather than build parallel
> ones.
>
> See also: `DESIGN_SYS_CAVES.md` (the architectural premise this
> design realises), `DESIGN_BATCAVES.md` (the cave runtime
> proper), `docs/SESSION_JOURNAL.md` (2026-05-12 entries — the
> day-by-day evolution).

## What this design solves

Before this work landed, "isolation between caves" meant:

  * Each cave has a `cave_l1_phys` page table (built lazily).
  * The scheduler MMU hook swaps `TTBR0_EL1` to that L1 when a
    task with non-zero `cave_id` is picked.

That was already running, but the boundary was hollow:

  1. Every cave's L1 mapped the same kernel-identity range
     `[0x40000000, 0x140000000)`. Code running with cave A's L1
     active could read every byte of cave B's kernel state via
     identity mapping.
  2. Sensitive state (e.g. sys-wg's WireGuard static keypair) lived
     in `.bss`, inside the kernel-identity range — reachable from
     any context.
  3. The MMU swap was a register write to a register the hardware
     might not even be using: the kernel boots with MMU off in the
     serial-shell path, so `TTBR0_EL1` writes had no translation
     effect.
  4. Even if the boundary were enforced, no way existed to *test*
     it — a kernel-ns access to a cave-private VA either hung the
     walker or hit the demand-pager.

This design fixes all four.

## The five layers

Sensitive cave state is now protected by five concentric layers,
strongest first:

```
┌──────────────────────────────────────────────────────────────┐
│  1. Module privacy (compile-time)                            │
│     no pub getter returns SecretKey / TransportKeys bytes    │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  2. VA-level MMU isolation (runtime)                 │   │
│  │     cave-private VA unmapped in PRIMARY_L1 + every   │   │
│  │     other cave's L1; pte_lookup proves this          │   │
│  │                                                      │   │
│  │  ┌────────────────────────────────────────────────┐ │   │
│  │  │  3. PA-level isolation (carve-out)             │ │   │
│  │  │     cave-pool PA range carved out of every     │ │   │
│  │  │     kernel L1's identity map; pte_lookup       │ │   │
│  │  │     proves PA also unreachable via identity    │ │   │
│  │  │                                                │ │   │
│  │  │  ┌──────────────────────────────────────────┐  │ │   │
│  │  │  │  4. Defence-in-depth (demand-page guard) │  │ │   │
│  │  │  │     demand_page refuses to install       │  │ │   │
│  │  │  │     mappings in the carve-out range —    │  │ │   │
│  │  │  │     stray kernel-ns store cannot shadow  │  │ │   │
│  │  │  │     the carve-out by accident            │  │ │   │
│  │  │  │                                          │  │ │   │
│  │  │  │  ┌────────────────────────────────────┐  │  │ │   │
│  │  │  │  │  5. Runtime fault visibility       │  │  │ │   │
│  │  │  │  │     mmu::probe_read_u64 turns a    │  │  │ │   │
│  │  │  │  │     wrong-context dereference     │  │  │ │   │
│  │  │  │  │     into a clean None instead of  │  │  │ │   │
│  │  │  │  │     a hang — selftests can       │  │  │ │   │
│  │  │  │  │     attempt the access and        │  │  │ │   │
│  │  │  │  │     observe the fault             │  │  │ │   │
│  │  │  │  └────────────────────────────────────┘  │  │ │   │
│  │  │  └──────────────────────────────────────────┘  │ │   │
│  │  └────────────────────────────────────────────────┘ │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

Each layer is independently selftest-verified — see "Selftest
contracts" below.

## Primitives

### `cave::with_cave_active(cave_id, f) -> R`

`src/batcave/cave.rs` exports a generic trampoline that wraps a
closure with cave context:

  1. Save the caller's `task.cave_id` + `TTBR0_EL1`.
  2. Set `task.cave_id = cave_id` and load the cave's L1 into
     TTBR0_EL1 (via `mmu::switch_to_cave`).
  3. Run the closure.
  4. Restore the saved `cave_id` + `TTBR0_EL1`.

The scheduler MMU hook (Arc 1 of the sys-caves design) ensures the
cave's L1 stays installed across any yields that happen INSIDE the
closure, so this works correctly under cooperative scheduling.

All public entry points of `sys_wg_service` go through this — it's
the canonical way to "step into" a cave to do sensitive work.

### `cave_private::ensure_page(cave_id) -> Option<usize>`

`src/batcave/cave_private.rs` allocates one 4 KiB page per cave at
a deterministic VA:

```
CAVE_PRIVATE_VA_BASE = 0x140000000
cave_private_va(cave_id) = CAVE_PRIVATE_VA_BASE + cave_id * 0x1000
```

That base sits one L1 entry (1 GiB) above the kernel-identity
range, so PRIMARY_L1 and every other cave's L1 have no mapping
covering it.

The page is allocated from the cave-pool (see below) — its PA also
lives outside every kernel L1's identity range. Combined: the page
is reachable only from inside `with_cave_active(owning_cave_id, ...)`.

Idempotent — second call for the same cave_id returns the same VA
without re-allocating.

### `cave_pool` — dedicated PA pool

`src/kernel/mm/cave_pool.rs` reserves PA range
`0xB000_0000..0xC000_0000` (256 MiB at the top of the kernel pool).
The pool's `init()` calls `frame::reserve_range(...)` so the
general `alloc_frame` / `alloc_kernel_frame` paths skip it.

`mmu::setup_and_enable` and `mmu::setup_native_cave_l1` skip blocks
384..512 of their L2_xhi tables (covering this PA range). No
kernel L1 has an identity mapping into the cave pool.

`cave_pool::alloc_page` does NOT zero the page — the carve-out
makes kernel-ns stores to that VA data-abort. `cave_private::
ensure_page` zeros via the just-installed cave-private VA under
`with_cave_active` (see "Subtle bug we hit" below).

### `mmu::map_4k_in_l1(l1_phys, va, pa, flags)`

`src/batcave/linux/mmu.rs` installs a 4 KiB page descriptor in an
arbitrary L1, allocating intermediate L2/L3 tables on demand from
the kernel pool. Cache-cleans every page-table page it writes to
PoC; the caller is responsible for TLB invalidation.

Used by `cave_private` to install per-page cave-private mappings.

### `mmu::pte_lookup(l1_phys, va) -> Option<u64>`

Walks a 3-level page-table from `l1_phys` and returns the leaf
PTE if `va` is mapped, or `None` if any descriptor along the walk
is invalid. Recognises both 2 MiB block leaves (kernel identity)
and 4 KiB page leaves (cave-private + demand-paged user pages).

Selftests use this to *prove* mapped-in-A / unmapped-in-B
properties at the page-table level.

### `mmu::probe_read_u64(va) -> Option<u64>`

Attempts to read a u64 at `va`. Returns `Some(value)` on success
or `None` if the access faults. Implementation:

  * Sets `PROBE_ACTIVE`.
  * Issues `ldr`.
  * If the load faults at EL1, the sync-exception handler
    (`kernel::arch::handle_sync_exception_inner`) sees
    `PROBE_ACTIVE`, sets `PROBE_FAULTED`, advances `ELR_EL1` by 4
    to skip past the faulting `ldr`, and returns.
  * Probe code reads `PROBE_FAULTED` on resume; returns `None`.

Single-threaded contract (no nesting; cooperative single-CPU).
Suitable for selftests that attempt out-of-cave reads.

### `demand_page` cave-pool guard

`src/batcave/linux/demand_page.rs` `try_handle` early-exits with a
loud log if `FAR_EL1` falls inside `0xB000_0000..0xC000_0000`.
Without this guard, a stray kernel-ns store to the carve-out VA
would be caught by the demand-pager, which would silently install
a fresh mapping in PRIMARY_L1's L2_xhi — defeating the carve-out.

We hit this exact bug during bring-up (see "Subtle bug we hit").

## Sys-wg as the reference user

`src/batcave/sys_wg_service.rs` is the canonical client of these
primitives. Its WireGuard `KEYPAIR` and `PEERS` table both live
inside a `PrivateState` struct placed at `cave_private_va(
sys_wg_id)` (= `0x140001000`):

```rust
#[repr(C)]
struct PrivateState {
    initialized: u32,
    _pad0: [u8; 4],
    static_sk_seed: [u8; 32],
    static_pk: [u8; 32],
    // ... per-peer SoA arrays (see source)
}
```

A `STATE_VA: AtomicUsize` is the only kernel-ns-visible handle —
it holds the VA itself (which kernel-ns can't dereference), not a
pointer to dereferenceable memory.

Every public entry point (`service_pubkey`, `register_peer`,
`close_peer`, `complete_handshake_as_responder`, `wrap`, `unwrap`,
etc.) goes through `cave::with_cave_active(sys_wg_id, ...)` before
touching state.

## Selftest contracts

Each primitive has a dedicated selftest that proves its property
end-to-end on every boot:

| Selftest | Asserts |
|------|------|
| `sys-caves-selftest` | Arc-2 cross-cave MMU swap forward + return |
| `cave-private-selftest` | `cave_private` allocates; `pte_lookup` confirms VA mapped in cave / unmapped in PRIMARY; `pte_lookup` confirms PA unmapped in PRIMARY; in-cave write/read round-trip; idempotency; `probe_read_u64` faults on out-of-cave VA + PA reads; sanity on a known-mapped VA |
| `sys-wg-selftest` | sys-wg's `KEYPAIR + PEERS` live at `0x140001000`; pte walk confirms; trampoline restores caller cave_id; handshake round trip works; peer registration + duplicate rejection; wrap/unwrap round trips |
| `wg-wire-selftest` | Phase 2 wire framing: Init/Response/Transport encoders + parsers + mac1 verification + mac1 tamper detection |
| `wg-replay-selftest` | Phase 2.6 sliding-window replay protection: 6 scenarios per WG whitepaper §5.4.6 |
| `wg-dispatch-selftest` | Phase 2.5 end-to-end dispatch: synthetic Init wire bytes → `dispatch_wire` → Response reply; encrypt + dispatch → plaintext |

All run headless via `scripts/qemu_*_selftest.py`.

## Subtle bug we hit

First implementation of the cave-pool carve-out had
`cave_pool::alloc_page` zero the page through its identity VA
before returning. With the carve-out in place, that VA was
unmapped in PRIMARY_L1; the `str` data-aborted; the kernel's
`demand_page` handler caught the abort and **installed a fresh
mapping in PRIMARY_L1's L2_xhi[384]** to "fix" the fault. The
mapping pointed at the cave's L2_xxxhi page (whatever
`frame::alloc_kernel_frame` happened to return next), and the
walker then resolved the cave-private PA through PRIMARY identity
— defeating the carve-out the next instruction after it was put
in place.

The fix has two parts:

  1. `cave_pool::alloc_page` no longer zeroes. The work moves to
     `cave_private::ensure_page`, which zeros *after* installing
     the mapping in the cave's L1, under `with_cave_active`.
  2. `demand_page::try_handle` got a carve-out guard: any FAR_EL1
     inside `0xB000_0000..0xC000_0000` is rejected outright. This
     is defence-in-depth — any future stray store will surface
     cleanly instead of silently shadowing the carve-out.

Recorded here because a future reader who tries the "zero in
`alloc_page`" obvious-looking simplification will hit the same
bug.

## What this design DOESN'T cover

  * **`with_cave_active` is a trampoline, not a service-task
    model.** The closure runs in the caller's task with the
    cave's L1 installed. It does NOT run as a separate scheduling
    principal. The architectural cleanliness ("sys-wg is its own
    process") would come from Arc 3 slice 3 (real EL0 service
    task + IPC mailbox), which is a future arc. For the security
    property — "the keypair is unreachable from outside" — the
    trampoline + carve-out is sufficient.
  * **No `mmu::probe_write_u64`**. Only reads. Adding writes is
    straightforward but we haven't needed it.
  * **No SMP.** `PROBE_ACTIVE` and `cave_pool::BITMAP` are global
    statics with IrqGuard discipline. An SMP arc replaces both
    with per-CPU state.
  * **No fault telemetry**. The probe handler currently swallows
    all data/inst-abort faults while `PROBE_ACTIVE`. A genuine
    kernel bug inside the probe call would be hidden. Probes are
    tiny (one `ldr`) so this is acceptable; future arc could
    capture FAR + log if `PROBE_ACTIVE` accumulates faults beyond
    expected.

## References in tree

  * `src/batcave/cave.rs` — `with_cave_active` + cave runtime.
  * `src/batcave/cave_private.rs` — per-cave page allocator.
  * `src/batcave/sys_wg_service.rs` — canonical client of all
    these primitives; sys-wg's WG state lives here.
  * `src/batcave/linux/mmu.rs` — `pte_lookup`, `map_4k_in_l1`,
    `probe_read_u64`, carve-out skip in `setup_and_enable` +
    `setup_native_cave_l1`.
  * `src/batcave/linux/demand_page.rs` — carve-out guard.
  * `src/kernel/mm/cave_pool.rs` — dedicated PA pool.
  * `src/kernel/arch/mod.rs` — `handle_sync_exception_inner` with
    probe-mode hook.
  * `src/net/wg_dispatch.rs` — Phase-2.5 dispatcher.
  * `src/net/wireguard.rs` — protocol code, wire codec, replay
    window.
  * `docs/SESSION_JOURNAL.md` — 2026-05-12 entries with the
    day-by-day evolution and the subtle-bug walkthrough.

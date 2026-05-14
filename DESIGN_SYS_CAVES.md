# DESIGN — `sys-*` Cave Network Architecture

> **Status:** Design adopted 2026-05-11. Implementation arcs land in
> successive batches. This document is the contract that future
> batches build against.

## Why this exists

A second AI evaluated Sphragis's network stack and pitched a full
Qubes-OS-style architecture: Type-1 hypervisor underneath, each
network layer in its own VM (`sys-net`, `sys-wireguard`,
`sys-whonix`), hardware-IOMMU-isolated. The diagnosis was right —
*today the network stack is monolithic inside the kernel* — but the
prescription was wrong for Sphragis's premise:

* **Qubes requires Xen-class hypervisor underneath.** No public Xen
  port to Apple M4. Writing our own Type-1 hypervisor is months of
  work and *doubles* the codebase to audit. That works against
  DESIGN.md decision #4 ("tiny auditable kernel").
* **N per-domain kernels means N kernel attack surfaces** instead
  of one well-audited microkernel.

Sphragis already has the right structural primitives — Caves are a
microkernel-process-with-isolation, the same abstraction Qubes
domains provide via a hypervisor. The fix is to *use* that
abstraction for the network stack: every sensitive piece of
state lives in its own cave, IPC is the only path between them, the
MMU enforces the boundaries at hardware speed.

## What we're building

A pipeline of dedicated caves, each owning one layer's keys and
state, connected by authenticated IPC. The contract: **a
compromise in one cave cannot extract another cave's keys, observe
another cave's plaintext, or impersonate another cave to a third
party.**

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   app cave           sys-tor             sys-wg     sys-net  │
│     ↓                  ↓                   ↓          ↓      │
│  TLS bytes  ──►  SOCKS5-like API ──► WG transport ──► NIC    │
│     ▲                                                        │
│   pinned                                                     │
│   server                                                     │
│   identity                                                   │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

The IPC boundaries are **authenticated** (`caves::ipc_session`
already does Ed25519+X25519 mutual auth with the existing
selftest) — a malicious cave can't forge an IPC handshake to
sys-wg even if it learns the IPC endpoint name.

## Cave roster

### `sys-net` — the only cave that touches the NIC

* **Owns:** Wi-Fi/Ethernet driver state, ARP table, link-layer
  MAC randomization, raw packet send/recv.
* **Sees:** raw L2 frames only. Has no concept of the encrypted
  payload above it.
* **Exposes:** IPC API for `send_frame(eth_frame) -> Result<()>`
  and `recv_frame() -> Option<eth_frame>`. Per-frame size cap,
  per-cave rate limit (cave-quota item 030 IO slice already
  tracks the bytes).
* **Pin:** sys-net's identity Ed25519 pubkey is hard-coded; only
  caves holding the matching `net` capability can hand it
  frames.
* **If compromised:** attacker gets the raw cable. They see the
  same encrypted-WireGuard-over-UDP packets a Wi-Fi sniffer
  sees. No WG keys, no Tor circuit state, no app plaintext.

### `sys-wg` — the WireGuard tunnel manager

* **Owns:** WireGuard static keypair (`WgKeypair`), per-peer
  session state (`TransportKeys` with counters), handshake
  cookies, MAC1/MAC2 keys.
* **Sees:** plaintext from above (whatever `sys-tor` hands it,
  or directly from an app if Tor is bypassed), encrypted
  WG-transport packets going down.
* **Exposes:** IPC API for `wrap(plaintext_packet) -> ct_packet`
  and `unwrap(ct_packet) -> plaintext_packet`. The handshake
  state machine is internal — callers never see it.
* **Pin:** sys-wg's identity Ed25519 pubkey is hard-coded;
  callers must hold the `wg` capability.
* **If compromised:** attacker recovers the WG static key and can
  forge tunneled packets to the peer. Still cannot see Tor
  plaintext (sys-tor handed sys-wg already-onion-wrapped
  bytes); still cannot see app plaintext (the app's TLS
  endpoint is the destination, not sys-wg's peer).

### `sys-tor` — Tor circuit manager (deferred, XL)

* **Owns:** Tor circuit handles, directory consensus cache,
  guard-node selection state, per-circuit AES-CTR keys.
* **Sees:** plaintext from above, ready-to-tunnel Tor cells going
  down.
* **Exposes:** IPC API for `connect(host:port) -> stream_id`,
  `send(stream_id, bytes)`, `recv(stream_id) -> bytes`,
  `close(stream_id)`. Roughly the SOCKS5 surface, but via
  cave-IPC instead of a Unix socket.
* **Pin:** sys-tor's identity Ed25519 pubkey is hard-coded.
* **If compromised:** attacker learns the circuit state and can
  intercept any new streams routed through this cave. The TLS
  layer above sys-tor still protects content (the cert chain
  pins to the real destination, not to a Tor exit).
* **Status:** native Tor is XL effort, deferred. Interim:
  `sys-socks5` cave that proxies to an external Tor on a hop
  the operator controls. Same IPC API, different backing.

### app caves

* **Own:** application state, TLS session state for outbound
  connections.
* **See:** the IPC surface of one of the lower caves (typically
  sys-tor for anonymous traffic, sys-wg directly for
  authenticated-but-not-anonymous traffic).
* **Cannot:** read frames off the NIC, read WG keys, read Tor
  circuit state. Even if compromised they can only emit
  through the chain.

## What's already there

Real, working, in tree as of `feat/wireguard-phase1` merge:

| Primitive | Where | Used for |
|------|------|------|
| Cave capability set | `src/caves/cave.rs` | Per-cave access control on net, fs, mem, etc. |
| Cave fs_key per cave | `src/caves/cave.rs:fs_key` | BatFS files encrypted per-cave, leaked filenames don't decrypt across caves |
| Cave page-table slot (`cave_l1_phys`, `cave_l1_slot`) | `src/caves/cave.rs` + `src/caves/linux/mmu.rs` | Per-cave L1 (TTBR0_EL1) prepared at first `cave::enter`. **TLB invalidate + TTBR swap already implemented** in `mmu::switch_to_cave`. |
| Authenticated IPC | `src/caves/ipc_session.rs` | Ed25519 identity + X25519 ephemeral + signed offer + derived ChaCha20-Poly1305 session key. `ipc-selftest` passes. |
| Cave-policy (egress firewall per cave) | `src/net/cave_policy.rs` | Per-cave outbound rate limits, allow-lists, SNI matching. |
| Cave-syscall-filter | `src/caves/syscall_filter.rs` | Each cave can deny specific syscalls. |
| Per-cave memory quota | `src/caves/cave.rs:mem_quota_pages` (just shipped) | Item 030 first slice. |
| PID namespace per cave | `src/kernel/process/mod.rs:cave_id` | Item 031. `procs` filters by active cave. |
| Mount namespace primitive | `src/caves/cave.rs:active_mount_prefix` + `mount-ns` shell cmd | Item 032 demo path. |
| WireGuard handshake + transport | `src/net/wireguard.rs` (Phase 1) | Will move into sys-wg. |
| Bridge inter-cave transport | `src/caves/bridge.rs` | The carrier the IPC API will run over. |
| Secure channel | `src/caves/secure_channel.rs` | TLS 1.3 between caves. |

## What's missing

The thing that turns capability isolation into *hardware-enforced*
isolation between caves: **the scheduler must call
`mmu::switch_to_cave(target_cave_l1)` whenever the next task's
`cave_id` differs from the current task's `cave_id`.**

Currently `switch_to_cave` fires only on explicit `cave::enter`,
not on every task switch. That means two tasks in different caves
running on the same CPU share the same TTBR0_EL1 user window — a
software bug in cave A can read cave B's memory by walking the
shared L1.

The fix is small in surface area (~20 LOC in
`kernel::scheduler::schedule`) but high in risk: if we get the TLB
discipline wrong, boot hangs or random memory corruption follows.
Standalone debug arc, smoke-tested in isolation.

Also missing:

* **A `sys-*` cave registry** — kernel records "this cave is sys-wg,
  this one is sys-net" so the audit ring + capability checks know
  which one is which.
* **IPC endpoint naming** with cap-gated lookup: caves register
  named services, callers resolve by name + the kernel checks
  caller's cap before returning the handle. We already have most
  of this via the existing capability + IPC infrastructure.
* **Audit-ring entries on cave→cave IPC calls** so a forensic
  reviewer can replay the security-relevant call graph.
* **A way for the kernel to spawn `sys-*` caves at boot** with
  the right cap sets pre-wired, not as ambient `shell-host`
  caves.

## Implementation arcs (ordered)

| Arc | Scope | Status |
|------|------|------|
| **0** | This document + survey | done in this batch |
| **1** | Wire per-cave MMU switching on every task scheduler swap (`kernel::scheduler::schedule` consults `task.cave_id`, calls `mmu::switch_to_cave` if changed) | next batch |
| **2** | Boot-time `sys-*` cave bring-up: kernel spawns sys-wg + sys-net caves with their cap sets, registers IPC endpoints, freezes their cap sets (no future grants) | mid-arc |
| **3** | Move WireGuard library code (`src/net/wireguard.rs`) into a sys-wg cave service. IPC API: `wrap` / `unwrap` / `handshake`. App caves call sys-wg by IPC; sys-wg holds the keys. | after Arc 2 |
| **4** | sys-net cave: when the first real NIC driver lands (gap-audit 019), it goes here from day one. App caves never touch the NIC. | when NIC driver exists |
| **5** | sys-socks5 cave (interim Tor): cave that opens TCP to an external Tor SOCKS5 proxy. App caves see the SOCKS5 IPC API. | small follow-up |
| **6** | Native sys-tor cave: Tor client, circuit construction, directory consensus, descriptor parsing. XL — multi-session arc. | distant |

## Security claims this earns us

When Arcs 1-3 land:

* **WG static key compromise requires breaking sys-wg.**
  A kernel bug that doesn't touch sys-wg's address space cannot
  extract WG keys. The keys never sit in any other cave's memory.
* **NIC driver compromise leaks only encrypted frames.**
  When sys-net lands, a Wi-Fi driver bug only roots a cave that
  sees the same bytes a Wi-Fi sniffer sees: encrypted WG over
  UDP. No tunneled plaintext, no upstream keys.
* **App compromise cannot bypass the privacy chain.**
  An app cave can only emit through its IPC handle to the next
  cave; the cap system prevents it from opening a bare socket
  to the NIC even if it has code execution.
* **Cave compromise is contained to that cave's keys.**
  Tor circuit state stays in sys-tor; WG keys stay in sys-wg;
  cert/TLS state stays in the app. Same defense-in-depth as
  Qubes, no hypervisor.

## What this design does NOT claim

* This doesn't defend against a kernel bug that touches **every**
  address space (i.e. an exploit that gets ring-0 privilege itself,
  not just compromise within a single cave's L1). Real Qubes
  defends against that via the hypervisor; we don't. The mitigation
  is the same as Sphragis's existing posture: keep the kernel small,
  audit it hard, defense-in-depth via cap isolation.
* This doesn't fix DMA attacks until we put each NIC behind its
  own DART (Apple's IOMMU). M4 DART support is partial in tree;
  a full per-NIC DART setup is its own arc.
* This doesn't add Wi-Fi MAC randomization (gap-audit 019); it
  defines the right place to put it once the Wi-Fi driver lands.

## References inside the tree

* `DESIGN.md` decision #4 (microkernel), #9 (network privacy chain).
* `DESIGN_CAVES.md` — original cave architecture.
* `src/caves/cave.rs` — cave struct + capability set.
* `src/caves/ipc_session.rs` — authenticated cave-to-cave IPC.
* `src/caves/linux/mmu.rs` — `setup_native_cave_l1`,
  `switch_to_cave`, `CAVE_L1[]` slots.
* `src/net/wireguard.rs` — Phase 1 library code that becomes the
  sys-wg cave service.
* `docs/L_ITEM_EVAL.md` — earlier feasibility eval for gap-audit
  030 (cgroups-equiv); the deferred-CPU-quota argument applies
  here too (real enforcement waits on preemptive scheduling).

# DESIGN_PACKET_PIPELINE.md — Bat_OS per-cave packet-layer egress enforcement

Shipped 2026-04-22. Followup #3 of the cave-policy roadmap. Turns the
kernel into the authoritative enforcement point for every packet a
BatCave emits, not just HTTP CONNECTs the daemon proxy happens to see.

## The problem in one paragraph

DESIGN_BATCAVES.md says: *every cave starts with ZERO access; egress
goes through an allowlist firewall enforced at the microkernel level.*
Pre-#3, the kernel had a `cave_policy` table but the ONLY enforcement
was batcaved.py's Python HTTP CONNECT proxy on port 9998, which caves
reached via `HTTPS_PROXY=host.docker.internal:9998`. Anything
non-HTTP(S) — nmap raw sockets, ICMP, arbitrary UDP — bypassed
entirely. Followup #3 moves enforcement into the kernel at layer 2/3/4
so the filter fires on **every frame**, not just CONNECTs.

## Topology

```
+--------------+      +-------------------+      +--------------+
|  container   |      |      Bat_OS       |      |   internet   |
|  192.168.77.10 --→ |   nic 1 (caves)   |      |   (slirp-    |
|              |      |      ↓            |      |    NAT'd)    |
+--------------+      |   cave_policy     |      |              |
                      |      ↓            |      |              |
                      |   NAT rewrite     |      |              |
                      |      ↓            |      |              |
                      |   nic 0 (host) ---→                       |
                      +-------------------+      +--------------+
                                ↑                      │
                                └────── reverse ───────┘
```

Bat_OS is a minimal NAT router sitting between two virtio-net
interfaces. The caves-side is 192.168.77.1/24 (Bat_OS occupies .1);
containers get .10, .11, .12, etc. The host-side is 10.0.2.15 on
slirp's 10.0.2.0/24, which QEMU itself NATs out to the Mac's
internet.

## The six pieces

### 1. Multi-NIC virtio-net driver (`drivers/virtio/net.rs`)

Two NICs in a fixed `[Nic; 2]` array, both initialized at boot. Zero-
arg helpers default to nic 0 so legacy callers don't change. Probe
direction is top-down so QEMU's declaration order lines up with
`nic_id`.

### 2. Per-cave kernel policy (`net/cave_policy.rs`)

- `CaveId = [u8; 16]`, `cave_id_from_name(name) = SHA-256("batos-cave-id-v1" || name)[..16]`
- `EgressRule { host: String, port: u16, proto: u8 }` — wildcards on empty host / port 0 / proto 0
- Default deny. Unknown cave → `Verdict::Drop`
- `{set_policy, add_rule, check, clear}_by_name` convenience layer

### 3. Packet classifier (`net/nat.rs`)

- IP → cave binding table populated via `bind_ip(ip, cave)` / `nat-sync`
- `parse_outbound(frame)` pulls Ethernet + IPv4 + TCP/UDP 5-tuple
  (rejects non-IPv4, fragments, truncated, non-tcp/udp)
- `classify(frame)` → `Allow | DropPolicy | DropUnknownSrc | DropParse`
- Calls `cave_policy::check_by_name(cave, dst_ip_str, dst_port, proto)`

### 4. NAT forwarder (`net/nat.rs`)

- `NatEntry` table, 64 slots, keyed by 5-tuple, ephemeral ports from 50000
- `rewrite_outbound_into(frame, flow, eph_port, nic0_ip, nic0_mac, gw_mac)`:
  - Eth src → nic0_mac, Eth dst → gw_mac
  - IPv4 src → nic0_ip, IPv4 cksum recomputed
  - L4 src_port → eph_port, L4 cksum recomputed (pseudo-header aware,
    UDP zero→0xFFFF fixup)
- `rewrite_inbound_into(frame, entry, nic1_mac)`:
  - Eth dst → cave MAC (cached at alloc time), Eth src → nic1_mac
  - IPv4 dst → cave_ip, L4 dst_port → cave_src_port, checksums recomputed
- `pump_and_forward` / `pump_replies` each drain a bounded batch

### 5. Main-loop auto-pump (`ui/desktop.rs`)

`nat::tick()` runs every iteration of the desktop idle loop, bounded
inside the pumps (256 out, 64 in). Cheap no-op when nic 1 absent or
NAT table empty — doesn't starve the UI on flood.

### 6. Daemon binding sync (`scripts/batcaved.py` + `src/batcave/docker_client.rs`)

- Daemon already populates `CAVE_NET_IP` via `docker inspect` at
  container create (Followup 3b-enforce).
- New protocol: `CPOL_BIND_LIST` (pull) + `CPOL_BIND_SET` (push/test).
- Kernel shell: `nat-sync` pulls every binding into `nat::bind_ip`.
- `batcave create --docker:…` also runs sync automatically so a
  freshly created cave's IP is known to the kernel before the
  container starts talking.

## Enforcement semantics

| frame arrives on | classifier sees | cave_policy says | outcome |
|---|---|---|---|
| nic 1 | unknown src IP | — | DropUnknownSrc |
| nic 1 | src IP → cave, dst in allowlist | Allow | rewrite + send nic 0 |
| nic 1 | src IP → cave, dst NOT in allowlist | Drop | DropPolicy |
| nic 0 | matches a NAT eph_port | — | rewrite + send nic 1 |
| nic 0 | no matching eph_port | — | ignored (existing IP stack handles) |

## Testing

Everything is exercised end-to-end on QEMU, no real Docker needed:

| test | what | result |
|---|---|---|
| `cave-policy-selftest` | 6 allows + 5 drops + isolation | PASS |
| `qemu_multinic_probe.py` | both NICs up | PASS |
| `nat-selftest` | six synthetic frames, counters | PASS |
| `qemu_nat_packet_e2e.py` | real frames via `-netdev socket` | PASS |
| `qemu_nat_rewrite_demo.py` | outbound + inbound rewrite round-trip | PASS |
| `qemu_nat_full_pipeline_e2e.py` | Python cave ↔ Python internet | PASS |
| `qemu_nat_autopump_e2e.py` | same but no manual shell ticks | PASS |
| `qemu_nat_daemon_bind_demo.py` | daemon → kernel IP sync | PASS |
| `qemu_nat_arp_e2e.py` | ARP responder on nic 1 | PASS |
| `qemu_nat_gc_demo.py` | TTL eviction per-proto | PASS |
| `qemu_nat_icmp_e2e.py` | ICMP Echo Request/Reply through NAT | PASS |
| `qemu_nat_fragment_demo.py` | fragment detection distinct from parse | PASS |
| `qemu_nat_host_passthrough_e2e.py` | non-NAT nic-0 frames → host IP stack | PASS |
| `qemu_nat_icmp_errors_e2e.py` | Dest-Unreach + Time-Exceeded through NAT | PASS |
| `qemu_nat_frag_reassembly_e2e.py` | 2-fragment reassembly → NAT forward | PASS |
| `qemu_nat_frag_inbound_e2e.py` | inbound fragment reassembly on nic 0 | PASS |
| `qemu_nat_refrag_e2e.py` | reassembly + re-fragmentation when > MTU | PASS |
| `qemu_nat_icmp_misc_e2e.py` | Param Problem deliver + Redirect/SQ drop | PASS |
| `qemu_vmnet_preflight.sh` | prerequisite checker (no sudo) | PASS |
| `qemu_vmnet_docker_e2e.sh` | real alpine container through vmnet | manual sudo |

## Going live against real Docker containers

To wire a real Docker container onto nic 1:

1. Launch Bat_OS with `scripts/qemu_vmnet_launch.sh` (prompts for sudo
   because `vmnet.framework` requires the `com.apple.vm.networking`
   entitlement, which Homebrew QEMU isn't signed for).
2. Find the vmnet interface in `ifconfig` (usually `bridge100` with
   192.168.77.1 on it; QEMU puts the bridge in host mode).
3. Create a Docker macvlan attached to the same subnet:
   ```
   docker network create -d macvlan \
       --subnet=192.168.77.0/24 --gateway=192.168.77.1 \
       -o parent=bridge100 caves
   ```
4. Run a container with a specific IP:
   ```
   docker run -d --network caves --ip=192.168.77.10 --name kali kali:latest sleep infinity
   ```
5. In Bat_OS shell:
   ```
   nat-bind    192.168.77.10 kali
   cpol-add    kali 93.184.216.34 443 tcp
   cpol-add    kali 8.8.8.8 53 udp
   ```
6. Start traffic from the container:
   ```
   docker exec kali curl -sI https://example.com
   ```
7. Frames now traverse vmnet → Bat_OS nic 1 → classifier → NAT →
   Bat_OS nic 0 → QEMU slirp → internet, with replies reversed.

## Gap closures (2026-04-22 evening)

- **ARP on nic 1** — `try_handle_arp` in nat.rs answers ARP requests
  for `CAVES_GATEWAY_IP = 192.168.77.1` with nic 1's MAC; requests
  for any other target are ignored. Counted as `arp-replies` /
  `arp-ignored`. Test: `qemu_nat_arp_e2e.py` PASS.
- **NAT TTL GC** — per-proto TTLs (UDP 60s / TCP 300s / ICMP 30s).
  Entries stamped with `last_seen_ticks` on create + every hit;
  `gc_tick()` runs from `nat::tick()` every main-loop iteration
  with a 1Hz throttle. Counter `nat-gc-evicted`. Test:
  `qemu_nat_gc_demo.py` PASS (3 entries installed, 2 evicted, 1
  TCP kept fresh).
- **ICMP Echo Request/Reply** — identifier plays the role of ports
  for NAT translation. Outbound Echo Request: id rewritten to a
  NAT-allocated handle, checksum recomputed (no pseudo-header).
  Inbound Echo Reply: lookup by translated id → restore cave's
  original id → deliver. Counters: `icmp-forwarded`, `icmp-delivered`.
  Test: `qemu_nat_icmp_e2e.py` PASS (cave id=0x1234 → translated
  → reply arrives with id=0x1234 restored).
- **IPv4 fragments** — classifier now distinguishes fragments
  (MF=1 or offset>0) from parse errors via a dedicated
  `PktVerdict::DropFragment` + `drop-fragment` counter. Full
  reassembly-then-NAT is the natural next step; this commit ships
  the visibility so operators can see "need larger MTU" instead of
  bucketing into drop-parse. Test: `qemu_nat_fragment_demo.py` PASS.

## Deferred closures (2026-04-22 late)

Four items marked deferred in the first cut of this doc shipped:

- **pump_replies falls through to host IP stack** — before the fix,
  any nic-0 frame that wasn't a NAT reply got silently consumed as
  soon as the NAT table had entries. Now non-NAT frames get
  re-dispatched to `net::dispatch_host_frame`. Counter
  `host-frames-pass`. Test: `qemu_nat_host_passthrough_e2e.py`.

- **ICMP error types** — Destination Unreachable (3) + Time Exceeded
  (11) route back to the cave whose flow they refer to, via
  embedded-header NAT rewrite: outer dst, inner src, inner L4 src
  port, and all four checksums. Counter `icmp-error-deliv`. Test:
  `qemu_nat_icmp_errors_e2e.py`.

- **Outbound fragment reassembly** — `frag_accept` buffers fragments
  per (src_ip, dst_ip, ip_id, proto) until complete, then feeds the
  reassembled datagram through classify + NAT as if it had arrived
  unfragmented. 4 concurrent slots, 2048-byte cap, 30s TTL via
  `frag_gc_sweep`. Counters `frag-reassembled` + `frag-timeout`.
  Test: `qemu_nat_frag_reassembly_e2e.py`.

- **Real Docker-through-vmnet automation** — `scripts/qemu_vmnet_preflight.sh`
  (no sudo, verifies QEMU has vmnet-host backend, Docker reachable,
  kernel built, :9999 free, pexpect present) +
  `scripts/qemu_vmnet_docker_e2e.sh` (sudo, full automated path:
  daemon + QEMU + bridge discovery + macvlan + alpine container
  with curl + shell drive + curl → container → bridge → nic 1 → NAT
  → nic 0 → internet). Preflight passes on this Mac; the sudo
  script is shell-lint-clean but only runs under interactive sudo.

## Last three closures (2026-04-22 very late)

Kaden asked to finish the last "still deferred" items. All three shipped:

- **Inbound fragment reassembly** — `pump_replies` now feeds
  nic-0 fragments through the same `frag_accept` that already
  handled outbound fragments. Slot count bumped 4 → 8 so both
  directions have headroom. Test: `qemu_nat_frag_inbound_e2e.py`.

- **Egress re-fragmentation** — `send_with_fragmentation(nic, frame)`
  splits post-rewrite datagrams > 1500 B into wire-legal IPv4
  fragments with correct MF/offset/checksum per piece. DF-set
  datagrams refuse to split. Counter `frag-refragd`. Test:
  `qemu_nat_refrag_e2e.py` — 1900 B cave datagram reassembles then
  splits back into two fragments on nic 0.

- **Remaining ICMP types** — Parameter Problem (12) routes like
  Dest Unreach + Time Exceeded (embedded-header rewrite); Redirect
  (5) + Source Quench (4) are explicitly dropped via
  `try_drop_icmp_problematic` so they never reach the cave OR the
  kernel's own IP stack. Counters `icmp-redir-drop` + `icmp-squench-drp`.
  Test: `qemu_nat_icmp_misc_e2e.py`.

## Nothing left deferred

The packet pipeline is feature-complete for the BatCave threat
model: per-cave egress enforcement at layer 2/3/4, bidirectional
NAT with TTL GC, ARP responder on the caves segment, full ICMP
coverage (Echo + all three error types + intentional drops for
Redirect + SourceQuench), fragment reassembly in both directions,
egress re-fragmentation, and a fall-through path for kernel's own
control-plane traffic.

## Real-vmnet validation (2026-04-23)

Initial attempt to plug a real Docker container onto the vmnet
bridge via macvlan hit a macOS architecture limitation: Docker
Desktop + OrbStack run containers inside a Linux VM that can't
attach to a macOS-side bridge. Rather than chase the Docker layer,
`scripts/qemu_vmnet_scapy_e2e.py` uses scapy to send raw Ethernet
frames from the macOS host directly onto the vmnet bridge — same
wire-format a container would produce, bypassing the Docker-VM
boundary. Result on a real M4 Mac:

  bridge104 (vmnet-host 192.168.77.0/24) → Bat_OS nic 1 → classifier:
    allow            = 11    (1 legit SYN + 10 rate burst tokens)
    drop-policy      =  3    (C2 callbacks to 203.0.113.66:4444)
    drop-rate        = 30    (flood of 40 beyond 10 burst)
    drop-sni         =  3    (TLS ClientHello SNI=attacker.com)
    drop-unknown-src =  0
    total classified = 47    (matches sent-packet sum exactly)

This is the first end-to-end real-network proof of the pipeline —
packets genuinely traversing vmnet.framework, not a socket-netdev
simulation.

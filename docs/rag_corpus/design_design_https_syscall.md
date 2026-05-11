# DESIGN: kernel-mediated HTTPS syscall

## Goal

Give caves the ability to make HTTPS requests through a single
Bat_OS-private syscall, with the kernel running TLS itself so caves
never see network bytes in plaintext and can never ship broken /
old / cert-validation-skipping TLS code.

## Status

Planned. Branch `feat/https-syscall`. Tag
`pre-https-syscall-2026-05-08` marks the pre-feature HEAD.

## Why

After PR #1 (no-browser pivot) deleted the in-tree browser, our
`fetch_https` wrapper became orphan code and PR #7 deleted it. The
TLS protocol implementation is correct (PR #6 proved it end-to-end
against `pq.cloudflareresearch.com`), but no caller exists. This PR
makes HTTPS a usable feature for caves — the way the user actually
intends to use it.

The `cave_policy` already implements default-deny network egress
(`check_with_sni` returns `Drop` for any cave with no rules). HTTPS
syscall consumes that as-is — no new policy infrastructure.

## Why kernel-mediated, not Linux-style userspace TLS

Two choices were on the table:

**A.** Caves use `socket(AF_INET, SOCK_STREAM)` + `connect()` and
bundle their own TLS library (OpenSSL/BoringSSL/etc.) in userspace.

**B.** Caves call a Bat_OS-private syscall; kernel runs TLS;
caves see plaintext over an fd.

We chose **B**. The reasoning:

- **Single audited TLS implementation.** Every cave gets the same
  hybrid PQ + chain-only-strict + GTS/ISRG/Amazon trust store. A
  cave can't ship broken TLS by accident.
- **Uniform `cave_policy` enforcement.** Same kernel chokepoint that
  gates raw sockets gates HTTPS. Per-host SNI rules apply uniformly.
- **Smaller cave binaries.** No bundled TLS library (~1MB+ saved per
  cave).
- **Matches the existing design.** Same reason there's a kernel-level
  firewall (`cpol`) instead of trusting each cave to filter traffic.

Cost: caves can't be drop-in Linux binaries that depend on
OpenSSL — they need to know about the Bat_OS syscall. Acceptable
for our threat model (security-first, OS-specific tooling).

## Wire layout

### New syscall

```
syscall_no = 0x4001                                         # 16385

bat_https_open(host_ptr: *const u8,
               host_len: usize,
               port: u16,
               flags: u32) -> i64

# Returns:
#   fd >= 0  on success
#   -EACCES  cave_policy denied
#   -EFAULT  host_ptr/host_len out of cave's user range
#   -EINVAL  bad host (CRLF/control chars), bad port (0)
#   -ENOMEM  no free TLS PCB slot (all 64 in use)
#   -ECONNREFUSED  TCP connect failed
#   -EHOSTUNREACH  DNS resolution failed
#   -EIO     TLS handshake / cert chain validation failed
```

`flags` is reserved (must be 0) for future O_NONBLOCK-style options.

### Returned fd

A standard Linux fd that supports:
- `read(fd, buf, n)` — returns `n` bytes of TLS-decrypted application data
- `write(fd, buf, n)` — sends `n` bytes through TLS to the server
- `close(fd)` — sends close_notify, tears down TLS, closes TCP, frees fd
- `epoll_ctl(epfd, ADD, fd)` — selectable like other sockets (later)

Cave writes raw HTTP bytes (`GET / HTTP/1.1\r\nHost: ...\r\n\r\n`)
and reads raw HTTP bytes back. The `Host:` header is the cave's job;
the SNI to send in the TLS ClientHello is the `host_ptr` argument.

### fd internals

The fd carries a new `FdKind::TlsSocket(pcb_id: u16)` variant. Since
`tls::TLS_MAX_PCBS == tcp::MAX_PCBS` and the two are 1:1 paired, a
single u16 identifies both the TLS slot and the underlying TCP PCB.

The vfs node behind the fd reuses `NodeType::Socket` (already exists);
the differentiation is in `FdKind`.

## Default-deny semantics

`cave_policy::check_with_sni(cave, host, port=443, proto=6, sni=host)`
already returns `Verdict::Drop` for any cave with no policy entry.
The HTTPS syscall just consults that. No code path can bypass it.

To grant a cave HTTPS access to a specific host, the operator runs
the existing `cpol-add-sni` shell command:

```
cpol-add-sni mycave example.com 443 example.com
```

This adds an `EgressRule { host:"example.com", port:443, proto:6,
sni:Some("example.com") }` to `mycave`. Any other host attempted
from `mycave` returns `-EACCES` and audit-logs the attempt.

## Per-cave concurrency

`TLS_MAX_PCBS == 64`. A cave can have multiple HTTPS fds open at
once (up to the per-cave fd limit, currently 1024) — each consumes
one TLS slot. If all 64 slots are busy, `-ENOMEM`.

No reference counting / sharing — each `bat_https_open` allocates a
fresh slot, each `close(fd)` frees it.

## Verification

### Boot-time smoke (this PR)

New Cargo feature `https-smoke-test`. Boot hook (after net init,
before auth gate) calls the kernel-side `https_open_kernel` (which
bypasses cave_policy because there's no cave context yet), writes
`GET / HTTP/1.1\r\nHost: pq.cloudflareresearch.com\r\nConnection:
close\r\n\r\n`, drains the response, asserts it starts with
`HTTP/1.1 ` and contains `Content-Length:` or `Transfer-Encoding:
chunked`. Closes loop on "HTTPS works as a feature."

`scripts/qemu_https_smoke.py` builds with the feature, boots in
QEMU virt with virtio-net, scans serial for the PASS line.

### Cave-side ABI smoke (follow-up)

A test cave that calls the actual syscall (instead of the kernel
function directly) lands in a follow-up PR after we have a
miniature cave that can invoke syscalls. Out of scope for this PR.

### Default-deny test (this PR)

Boot hook also (with feature `https-deny-smoke-test`?) drives a
fake cave context that has no policy and asserts the syscall
returns -EACCES. Validates the policy gate. *Tentative — may collapse
into the main smoke if simpler.*

## What this PR does NOT do

- **No SOCK_STREAM emulation.** Caves cannot use plain
  `socket()/connect()` and get auto-TLS. The HTTPS syscall is the
  only way to TLS. Plain TCP sockets stay plaintext-only.
- **No HTTP/2.** The kernel TLS layer only does TLS records; HTTP
  framing (header parsing, request building, response parsing) is
  the cave's job. HTTP/2 framing is a userspace concern.
- **No TLS server side.** Caves can be HTTPS clients only.
- **No mid-handshake epoll.** The syscall blocks until handshake
  completes (typical 50–200 ms). Async-handshake support is a
  follow-up.

## Implementation phases

1. **TLS slot-parameterized API.** Add `tls::handshake_pcb`,
   `send_app_data_pcb`, `recv_app_data_pcb`, `close_pcb` taking an
   explicit slot id. Existing `tls::handshake / send_app_data /
   recv_app_data / close` become thin wrappers around `_pcb(0)`.
2. **TLS slot allocator.** Add `tcp::alloc_pcb` / `tcp::free_pcb`
   (returning the index of the first free PCB). TLS slot id = TCP
   PCB id, so one allocator suffices.
3. **fd kind.** Add `FdKind::TlsSocket(u16)` and `fd::alloc_fd_tls`.
4. **Kernel function.** `https::open_kernel(host, port) ->
   Result<u16 /* tcp/tls pcb */, &'static str>`. Called by syscall
   and by boot hook.
5. **Syscall.** `sys_batos_https_open` at syscall_no 0x4001:
   user-pointer validation, cave_policy gate, calls
   `https::open_kernel`, allocates fd via `fd::alloc_fd_tls`.
6. **Read/write/close routing.** `sys_read` / `sys_write` /
   `sys_close` add a `FdKind::TlsSocket(pcb)` arm that routes through
   the `_pcb` TLS API.
7. **Boot smoke.** Cargo feature `https-smoke-test` + boot hook +
   `scripts/qemu_https_smoke.py`.
8. **Spec/journal/PR.**

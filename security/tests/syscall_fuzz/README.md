# Sphragis syscall fuzzer

Static-linked ARM64 Linux ELF that runs as a BatCave guest, issues every
wired syscall with a curated set of hostile argument tuples, and logs each
call to stdout. Because Sphragis shares one VA space between kernel and
guest, a crash is terminal — the last "TRY sys=..." line before UART
silence identifies the faulting input.

## Build

On a host with an aarch64 musl cross-toolchain:

    make                       # default: aarch64-linux-musl-gcc

or

    CC=aarch64-linux-gnu-gcc make

Emits `syscall_fuzz` (fully static, no dynamic linker needed).

## Run inside Sphragis

Copy the binary into the Sphragis image (same path as `chromium`) and
launch it the same way:

    batcave run /bin/syscall_fuzz

It will print lines like:

    TRY sys=63  args=[ -1, 0x00000000, 0xffffffffffffffff, 0, 0, 0 ] -> ret=-14
    TRY sys=222 args=[ 0x0, 0x0, 0x7, 0x20, -1, 0 ]                  -> ret=-22

If the kernel panics, note the last TRY line.

## What it covers

- Every syscall number listed in `syscall.rs::handle()`'s match arms
  (covers ~110 entries including the custom 500 framebuffer blit).
- At least 10 seeded argument tuples per syscall, drawn from the
  categories called out in the audit:
  NULL, -1, SIZE_MAX, kernel-like addresses (0xFFFF_...), unaligned
  pointers, misused fd types (epoll fd where a socket is expected,
  stdin where a file is expected), huge counts, TOCTOU-relevant
  iovec shapes.
- A cheap deterministic LCG extends each seed with randomised
  derivatives so repeated runs can widen coverage.

The fuzzer does **not** try to oracle specific kernel addresses —
that's the auditor's job. This harness only exists to reach every
attack surface mechanically.

## Extending

- Add seeds: edit `seeds.h` — each block is `{ nr, { a0..a5 } }`.
- Add post-call side-effect checks: wrap `do_syscall()` and compare
  before/after (e.g. dump `/proc/self/maps` to confirm no new
  mapping appeared).

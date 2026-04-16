# Chromium content_shell — ARM64 Syscall Research for Bat_OS

**Audience:** Bat_OS kernel engineer (Kaden) implementing Phase 4 of the Chromium port.
**Scope:** Every Linux syscall that Chromium `content_shell --single-process --headless --no-sandbox --disable-gpu` is expected to invoke on Linux ARM64 (AArch64), categorized against Bat_OS's current coverage in `src/batcave/linux/syscall.rs`.
**Target Chromium version:** M132 (pinned in `CHROMIUM_PORT_PLAN.md`).
**Methodology:** Enumerate Chromium's own seccomp-bpf allow-lists (these are the ground truth of "syscalls Chromium may invoke") and cross-reference each with the ARM64 (`asm-generic/unistd.h`) syscall table.

---

## Why seccomp policies are the right source

Chromium's sandbox whitelists every syscall that any renderer/GPU/network process is permitted to make. If a syscall isn't in the allow-list, calling it hard-kills the process with `SIGSYS`. Therefore the union of:

- `sandbox/linux/seccomp-bpf-helpers/baseline_policy.cc`  (baseline, applies to every sandboxed child)
- `sandbox/linux/seccomp-bpf-helpers/syscall_sets.cc`    (reusable syscall sets: epoll, futex, signals, IO, gettime, etc.)
- `sandbox/policy/linux/bpf_base_policy_linux.cc`        (base Chromium policy)
- `sandbox/policy/linux/bpf_renderer_policy_linux.cc`    (renderer delta)
- `sandbox/policy/linux/bpf_gpu_policy_linux.cc`         (GPU delta)
- `sandbox/policy/linux/bpf_network_policy_linux.cc`     (network service delta)

…is a strict upper bound on what the renderer half of `content_shell` invokes. In `--single-process` mode, all three roles (browser + renderer + GPU-as-SwiftShader) run in the same process, so the effective allow-list is the union of all four policies minus fork/exec/namespace stuff (we pass `--no-sandbox` and `--no-zygote`).

This gives us ~180-220 distinct syscalls, matching the Phase 3 estimate of "150-200."

---

## Task 1 — Current Bat_OS Coverage (from `src/batcave/linux/syscall.rs`)

Extracted by reading the `match` in `handle()` at lines 101-212. 89 arms; several collapse onto shared handlers (e.g. `sys_stub_zero`).

### ARM64 numbers currently wired

| # (AArch64) | Name                  | Bat_OS handler         | Depth  |
|-------------|-----------------------|------------------------|--------|
| 17          | `getcwd`              | `sys_getcwd`           | partial |
| 20          | `epoll_create1`       | `sys_epoll_create1`    | stub    |
| 21          | `epoll_ctl`           | `sys_epoll_ctl`        | stub    |
| 22          | `epoll_pwait`         | `sys_epoll_pwait`      | stub    |
| 23          | `dup`                 | `sys_dup`              | partial |
| 24          | `dup3`                | `sys_dup3`             | partial |
| 25          | `fcntl`               | `sys_fcntl`            | partial (F_GETFD/F_SETFD/F_GETFL/F_SETFL return 0) |
| 29          | `ioctl`               | `sys_ioctl`            | partial |
| 34          | `mkdirat`             | `sys_mkdirat`          | partial |
| 35          | `unlinkat`            | `sys_stub_zero`        | stub    |
| 46          | `ftruncate`           | `sys_stub_zero`        | stub    |
| 48          | `faccessat`           | `sys_faccessat`        | partial |
| 49          | `chdir`               | `sys_chdir`            | partial |
| 56          | `openat`              | `sys_openat`           | partial |
| 57          | `close`               | `sys_close`            | partial |
| 59          | `pipe2`               | `sys_pipe2`            | partial |
| 61          | `getdents64`          | `sys_getdents64`       | partial |
| 62          | `lseek`               | `sys_stub_zero`        | stub    |
| 63          | `read`                | `sys_read`             | partial |
| 64          | `write`               | `sys_write`            | partial |
| 66          | `writev`              | `sys_writev`           | partial |
| 71          | `sendfile`            | `sys_sendfile`         | partial |
| 73          | `ppoll`               | `sys_ppoll`            | partial |
| 78          | `readlinkat`          | `sys_readlinkat`       | partial |
| 79          | `newfstatat` (fstatat)| `sys_newfstatat`       | partial |
| 80          | `fstat`               | `sys_fstat`            | partial |
| 93          | `exit`                | `sys_exit`             | full    |
| 94          | `exit_group`          | `sys_exit_group`       | full    |
| 96          | `set_tid_address`     | `sys_set_tid_address`  | full    |
| 98          | `futex`               | `sys_futex`            | **partial (critical — see gap analysis)** |
| 99          | `set_robust_list`     | `sys_stub_zero`        | stub    |
| 100         | `get_robust_list`     | `sys_stub_zero`        | stub    |
| 101         | `nanosleep`           | `sys_nanosleep`        | full    |
| 102         | `getitimer`           | `sys_stub_zero`        | stub    |
| 103         | `setitimer`           | `sys_stub_zero`        | stub    |
| 113         | `clock_gettime`       | `sys_clock_gettime`    | full    |
| 131         | `tgkill`              | `sys_tgkill`           | partial |
| 132         | `sigaltstack`         | `sys_sigaltstack`      | partial |
| 134         | `rt_sigaction`        | `sys_rt_sigaction`     | partial |
| 135         | `rt_sigprocmask`      | `sys_rt_sigprocmask`   | partial |
| 136         | `rt_sigpending`       | `sys_stub_zero`        | stub    |
| 137         | `rt_sigtimedwait`     | `sys_stub_zero`        | stub    |
| 139         | `rt_sigreturn`        | `sys_rt_sigreturn`     | partial |
| 144         | `setgid`              | `sys_stub_zero`        | stub    |
| 146         | `setuid`              | `sys_stub_zero`        | stub    |
| 153         | `times`               | `sys_stub_zero`        | stub    |
| 154         | `setpgid`             | `sys_stub_zero`        | stub    |
| 155         | `getpgid`             | `sys_stub_zero`        | stub    |
| 157         | `sched_getscheduler`  | `sys_stub_zero`        | stub    |
| 158         | `sched_getparam`      | `sys_stub_zero`        | stub    |
| 160         | `uname`               | `sys_uname`            | full    |
| 166         | `umask`               | `sys_stub_zero`        | stub    |
| 167         | `sysinfo` (compat)    | `sys_stub_zero`        | stub    |
| 169         | `gettimeofday`        | `sys_stub_zero`        | stub    |
| 170         | `getpgrp`             | `sys_stub_zero`        | stub    |
| 171         | `sigaltstack` (alt)   | `sys_sigaltstack`      | partial |
| 172         | `getpid`              | `sys_getpid`           | returns 1 |
| 173         | `getppid`             | `sys_getppid`          | returns 0 |
| 174         | `getuid`              | `sys_getuid`           | returns 0 |
| 175         | `geteuid`             | `sys_getuid`           | returns 0 |
| 176         | `getgid`              | `sys_getgid`           | returns 0 |
| 177         | `getegid`             | `sys_getgid`           | returns 0 |
| 178         | `gettid`              | `sys_gettid`           | partial |
| 179         | `sysinfo`             | `sys_sysinfo`          | full    |
| 198         | `socket`              | `sys_socket`           | partial |
| 200         | `bind`                | `sys_stub_zero`        | stub    |
| 201         | `listen`              | `sys_stub_zero`        | stub    |
| 202         | `accept`              | `sys_stub_zero`        | stub    |
| 203         | `connect`             | `sys_connect`          | partial |
| 204         | `getsockname` (confusingly mapped to sched_getaffinity in file) | `sys_stub_zero` | **BUG: number 204 is `getsockname` on AArch64, not `sched_getaffinity` (which is 123). Code comment is wrong.** |
| 206         | `sendto`              | `sys_sendto`           | partial |
| 207         | `recvfrom`            | `sys_recvfrom`         | partial |
| 208         | `setsockopt`          | `sys_stub_zero`        | stub    |
| 209         | `getsockopt`          | `sys_stub_zero`        | stub    |
| 210         | `shutdown`            | `sys_stub_zero`        | stub    |
| 214         | `brk`                 | `sys_brk`              | partial |
| 215         | `munmap`              | `sys_munmap`           | partial |
| 220         | `clone`               | `sys_clone_thread`     | partial |
| 221         | `execve`              | `sys_execve`           | partial |
| 222         | `mmap` (**note conflict — see below**) | `sys_mmap` / also mapped as shmget | **BUG: arm64 has no shmget; number 222 is `mmap`. The `sys_shmget` arm at line 144 is dead code.** |
| 223         | `shmctl` (also dead on arm64) | `sys_stub_zero` | dead code |
| 226         | `mprotect`            | `sys_mprotect`         | partial |
| 233         | `madvise`             | `sys_stub_zero`        | stub    |
| 260         | `wait4`               | `sys_wait_stub`        | stub    |
| 261         | `prlimit64`           | `sys_prlimit64`        | partial |
| 262         | `getrlimit` (compat)  | `sys_stub_zero`        | stub    |
| 276         | `renameat2`           | `sys_stub_zero`        | stub    |
| 278         | `getrandom`           | `sys_getrandom`        | partial |
| 279         | `memfd_create`        | `sys_memfd_create`     | partial |
| 500         | `blit_framebuffer` (Bat_OS custom) | `sys_blit_framebuffer` | full |

**Issues flagged (report only, do NOT auto-fix per malware-analysis policy):**

1. Line 142 maps **number 204** to `sched_getaffinity` — on AArch64 (asm-generic) the `sched_getaffinity` number is **123**, and 204 is `getsockname`. This is a pre-existing inconsistency.
2. Lines 144-145 map **222/223** to `shmget`/`shmctl` — AArch64 does not expose SysV shm syscalls at those numbers. Number **222 is `mmap`**, which is *also* correctly mapped at line 182 via `nr::MMAP`. The `shmget`/`shmctl` arms at 144-145 are unreachable (the `nr::MMAP` arm wins via enum ordering, but actually `match` arm order rules apply — the second arm is likely dead or shadowed depending on Rust match semantics; worth auditing).
3. `getpid` returns a literal `1`; `getuid`/`getgid` return `0`. Chromium's V8 logging writes files named `v8-<pid>.log` so a constant `1` will cause log-clobbering across threads — low priority but worth knowing.

**Summary:** ~89 arms, mapping to roughly 75 distinct AArch64 syscall numbers once bugs/duplicates are removed. Of those, ~20 are full implementations, ~25 partial, ~30 stubs (return 0 or -ENOSYS).

---

## Task 2 — Chromium's Expected Syscall Usage (content_shell single-process headless)

Compiled from the seccomp policy sources:

- [baseline_policy.cc](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/sandbox/linux/seccomp-bpf-helpers/baseline_policy.cc)
- [syscall_sets.cc](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/sandbox/linux/seccomp-bpf-helpers/syscall_sets.cc)
- [bpf_base_policy_linux.cc](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/sandbox/policy/linux/bpf_base_policy_linux.cc)
- [bpf_renderer_policy_linux.cc](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/sandbox/policy/linux/bpf_renderer_policy_linux.cc)
- [bpf_gpu_policy_linux.cc](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/sandbox/policy/linux/bpf_gpu_policy_linux.cc)
- [bpf_network_policy_linux.cc](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/sandbox/policy/linux/bpf_network_policy_linux.cc)

AArch64 syscall numbers below come from `arch/arm64/include/uapi/asm/unistd.h` (generic table in `include/uapi/asm-generic/unistd.h`).

### Union of allowed syscalls (AArch64 numbers)

#### Address space / memory (`SyscallSets::IsAllowedAddressSpaceAccess`)
| #   | Name              | Chromium uses it for                                   |
|-----|-------------------|--------------------------------------------------------|
| 213 | `madvise` (wait, 233) → actually **233** `madvise`    | TCMalloc/PartitionAlloc release pressure; `MADV_DONTNEED`, `MADV_FREE` |
| 214 | `brk`             | glibc heap (musl uses mmap mostly)                     |
| 215 | `munmap`          | every free of an mmap region                           |
| 216 | `mremap`          | PartitionAlloc, V8 heap grow/shrink                    |
| 222 | `mmap`            | **hottest path** — PartitionAlloc, V8 code pages, Skia |
| 226 | `mprotect`        | V8 JIT (RX↔RW), guard pages                            |
| 227 | `msync`           | rare (only when file-backed mappings are flushed)      |
| 228 | `mlock`           | V8 code cache lock (optional)                          |
| 229 | `munlock`         | paired with above                                      |
| 232 | `mincore`         | PartitionAlloc committed-page check                    |
| 233 | `madvise`         | (correct here)                                         |
| 462 | `mseal`           | V8/PartitionAlloc sealing on 6.10+ kernels (stub OK)   |

#### Basic scheduler (`IsAllowedBasicScheduler`)
| #   | Name              | Uses                              |
|-----|-------------------|-----------------------------------|
| 101 | `nanosleep`       | `base::PlatformThread::Sleep`     |
| 115 | `clock_nanosleep` | same, precise                     |
| 124 | `sched_yield`     | cpu relaxation in spin-wait tails |
| 140 | `setpriority`    | thread priority                    |
| 141 | `getpriority`    | thread priority                    |

#### Epoll (`IsAllowedEpoll`)
| #   | Name              | Uses                                            |
|-----|-------------------|-------------------------------------------------|
| 20  | `epoll_create1`   | every `base::MessagePumpEpoll`                  |
| 21  | `epoll_ctl`       | add/mod/del fds from watchers                   |
| 22  | `epoll_pwait`     | the MessagePump's main wait                     |
| 441 | `epoll_pwait2`    | newer glibc/musl; Chromium falls back to pwait  |

#### Event/signal/timer fds
| #   | Name              | Uses                                     |
|-----|-------------------|------------------------------------------|
| 19  | `eventfd2`        | `base::WaitableEvent`, cross-thread wake |
| 74  | `signalfd4`       | `base::FileDescriptorWatcher` signals    |
| 85  | `timerfd_create`  | `base::MessagePumpEpoll::ScheduleWork`   |
| 86  | `timerfd_settime` | re-arm animation frame timer             |
| 87  | `timerfd_gettime` | occasional read                          |

#### File system via fd
| #   | Name              | Uses                                |
|-----|-------------------|-------------------------------------|
| 44  | `fstatfs`         | `base::SysInfo::AmountOfFreeDiskSpace` |
| 46  | `ftruncate`       | memfd / cache sizing                |
| 82  | `fsync`           | rare — disk cache flushes           |
| 83  | `fdatasync`       | same                                |
| 80  | `fstat`           | every stat on an open fd            |
| 32  | `flock`           | file cache lock                     |
| 47  | `fallocate`       | disk cache pre-allocation           |

#### Futex / robust list (`IsAllowedFutex`)
| #   | Name              | Uses                                     |
|-----|-------------------|------------------------------------------|
| 98  | `futex`           | **every mutex, cv, barrier in Chromium** |
| 99  | `set_robust_list` | libpthread init per-thread               |
| 100 | `get_robust_list` | rarely called                            |
| 422 | `futex_time64`    | 32-bit ABI shim, not emitted by 64-bit   |
| 449 | `futex_waitv`     | kernel 5.16+, new API, unused by M132    |

#### General IO (`IsAllowedGeneralIo`)
| #   | Name              | Uses                             |
|-----|-------------------|----------------------------------|
| 29  | `ioctl`           | TCGETS on stdout (`isatty()`), sockets |
| 62  | `lseek`           | file reads                       |
| 63  | `read`            | pervasive                        |
| 64  | `write`           | logging, IPC                     |
| 65  | `readv`           | IPC boundaries                   |
| 66  | `writev`          | Mojo/IPC and logging             |
| 67  | `pread64`         | BlobDB, cache                    |
| 68  | `pwrite64`        | cache                            |
| 69  | `preadv`          | rare                             |
| 70  | `pwritev`         | rare                             |
| 72  | `pselect6`        | some MessagePump paths           |
| 73  | `ppoll`           | legacy MessagePump               |
| 75  | `splice`          | network→disk fast path           |
| 76  | `tee`             | not usually                      |
| 77  | `vmsplice`        | not usually                      |
| 203 | `connect`         | sockets (also under network)     |
| 206 | `sendto`          | UDP + TCP                        |
| 207 | `recvfrom`        | UDP + TCP                        |
| 211 | `sendmsg`         | Mojo UNIX socket IPC             |
| 212 | `recvmsg`         | same                             |
| 269 | `sendmmsg`        | network stack batching           |
| 243 | `recvmmsg`        | same                             |

#### Time (`IsAllowedGettime`)
| #   | Name              | Uses                           |
|-----|-------------------|--------------------------------|
| 113 | `clock_gettime`   | **every tick** (TimeTicks::Now) |
| 114 | `clock_settime`   | blocked in sandbox; won't be called |
| 115 | `clock_nanosleep` | precise sleeps                 |
| 116 | `clock_adjtime`   | NTP — won't be called          |
| 117 | `clock_getres`    | V8 DateTime precision probe    |
| 169 | `gettimeofday`    | legacy code paths              |
| 171 | `setitimer`       | rare                           |

#### Process lifecycle (`IsAllowedProcessStartOrDeath`)
| #   | Name              | Uses                                |
|-----|-------------------|-------------------------------------|
| 93  | `exit`            | thread exit                         |
| 94  | `exit_group`      | process exit                        |
| 95  | `waitid`          | (no subprocs in --single-process)   |
| 96  | `set_tid_address` | pthread init                        |
| 220 | `clone`           | **every new thread** (~30 at startup) |
| 260 | `wait4`           | zygote only; --no-zygote skips      |

#### Signal handling
| #   | Name              | Uses                        |
|-----|-------------------|-----------------------------|
| 132 | `sigaltstack`     | pthread alt stack           |
| 133 | `rt_sigsuspend`   | rare                        |
| 134 | `rt_sigaction`    | init + ASan/signal handlers |
| 135 | `rt_sigprocmask`  | block/unblock per thread    |
| 136 | `rt_sigpending`   | rare                        |
| 137 | `rt_sigtimedwait` | optional                    |
| 138 | `rt_sigqueueinfo` | rare                        |
| 139 | `rt_sigreturn`    | every signal return         |
| 240 | `rt_tgsigqueueinfo`| thread kill                |

#### FD operations
| #   | Name              | Uses                          |
|-----|-------------------|-------------------------------|
| 57  | `close`           | pervasive                     |
| 23  | `dup`             | IPC pipes                     |
| 24  | `dup3`            | same                          |
| 25  | `fcntl`           | F_GETFD, F_SETFL (O_NONBLOCK), F_SETFD (FD_CLOEXEC), F_SETLK |
| 210 | `shutdown`        | graceful socket teardown      |

#### Kernel internal
| #   | Name              | Uses                |
|-----|-------------------|---------------------|
| 128 | `restart_syscall` | auto by kernel after signal |

#### Baseline extras (from `baseline_policy.cc`)
| #   | Name                | Uses                                           |
|-----|---------------------|------------------------------------------------|
| 160 | `uname`             | glibc/musl startup                             |
| 167 | `prctl`             | **PR_SET_NAME**, **PR_SET_DUMPABLE**, PR_CAP_AMBIENT, PR_SET_SECCOMP, PR_SET_NO_NEW_PRIVS, PR_SET_VMA |
| 278 | `getrandom`         | BoringSSL seeding, hashmap seeds, V8          |
| 279 | `memfd_create`      | shared memory for renderer↔GPU / renderer↔browser |
| 261 | `prlimit64`         | discover RLIMIT_STACK, RLIMIT_NOFILE          |
| 123 | `sched_getaffinity` | `base::SysInfo::NumberOfProcessors`           |
| 122 | `sched_setaffinity` | **blocked in renderer**; only called in GPU-ish paths |
| 129 | `kill` (tgkill 131) | crash handler                                 |
| 131 | `tgkill`            | ASan, crash handler                           |
| 293 | `pkey_alloc`        | V8 code page protection keys (optional)       |
| 294 | `pkey_free`         | paired                                        |
| 288 | `pkey_mprotect`     | paired                                        |
| 293 | `rseq`              | glibc/musl restartable sequences startup call |

#### File system paths (baseline + broker-handled; called directly in `--no-sandbox`)
| #   | Name              | Uses                                 |
|-----|-------------------|--------------------------------------|
| 17  | `getcwd`          | absolute path resolution             |
| 34  | `mkdirat`         | disk cache dir create                |
| 35  | `unlinkat`        | cache eviction                       |
| 36  | `symlinkat`       | rare                                 |
| 37  | `readlinkat`      | `/proc/self/exe`                     |
| 38  | `linkat`          | rare                                 |
| 48  | `faccessat`       | existence check before open          |
| 49  | `chdir`           | process launch                       |
| 50  | `fchdir`          | rare                                 |
| 51  | `chroot`          | not called in --no-sandbox           |
| 52  | `fchmod`          | cache perms                          |
| 53  | `fchmodat`        | cache perms                          |
| 54  | `fchownat`        | rare                                 |
| 55  | `fchown`          | rare                                 |
| 56  | `openat`          | **every file open**                  |
| 59  | `pipe2`           | IPC channels                         |
| 61  | `getdents64`      | directory enumeration                |
| 78  | `readlinkat`      | symlinks                             |
| 79  | `newfstatat`      | `stat(path)` variant                 |
| 269 | `sendmmsg`        | net stack                            |
| 276 | `renameat2`       | cache atomic rename                  |
| 285 | `copy_file_range` | faster large copies                  |

#### Network-service policy delta (used by Chromium's in-process net stack)
| #   | Name              | Uses                                                      |
|-----|-------------------|-----------------------------------------------------------|
| 198 | `socket`          | create TCP / UNIX socket                                  |
| 199 | `socketpair`      | Mojo UNIX socket pair                                     |
| 200 | `bind`            | ephemeral UDP                                             |
| 201 | `listen`          | devtools server (--remote-debugging-port) only            |
| 202 | `accept`          | same                                                      |
| 203 | `connect`         | TCP open                                                  |
| 204 | `getsockname`     | learn local addr after bind                               |
| 205 | `getpeername`     | remote addr on accepted conn                              |
| 206 | `sendto`          | UDP                                                       |
| 207 | `recvfrom`        | UDP, TCP                                                  |
| 208 | `setsockopt`      | SO_REUSEADDR, SO_KEEPALIVE, TCP_NODELAY, IP_TOS           |
| 209 | `getsockopt`      | SO_ERROR, SO_RCVBUF                                       |
| 210 | `shutdown`        | half-close                                                |
| 211 | `sendmsg`         | scatter-gather TCP                                        |
| 212 | `recvmsg`         | scatter-gather TCP                                        |
| 242 | `accept4`         | modern accept with flags                                  |
| 243 | `recvmmsg`        | UDP batch (QUIC)                                          |
| 269 | `sendmmsg`        | UDP batch (QUIC)                                          |

#### GPU path (SwiftShader; called even with `--disable-gpu` by some code paths)
| #   | Name              | Uses                                       |
|-----|-------------------|--------------------------------------------|
| 47  | `fallocate`       | shm buffer backing                         |
| 285 | `copy_file_range` | buffer copies                              |

#### ARM64-specific kernel helpers
| #   | Name / helper            | Uses                                        |
|-----|--------------------------|---------------------------------------------|
| N/A | `__ARM_NR_cmpxchg` (not on AArch64, legacy ARM) | n/a               |
| vDSO | `__kernel_clock_gettime`, `__kernel_gettimeofday`, `__kernel_rt_sigreturn` | V8, TimeTicks — resolved via auxv AT_SYSINFO_EHDR; if we don't expose a vDSO, glibc/musl falls back to syscall 113, 169, 139 which we already handle |

---

### Total unique syscalls across union (AArch64)

Counting distinct AArch64 numbers from the tables above: **~185 syscalls**. This matches the Phase 3 plan's "150-200" budget.

---

## Task 3 — Gap Analysis

### Legend
- ✅ = already wired in `syscall.rs`
- 🟡 = can stub (-ENOSYS or 0) without breaking content_shell
- 🔴 = must implement (Chromium hard-crashes or silently deadlocks without)

---

### ✅ Already have (full or partial) — 47 syscalls

| #   | Name              | Bat_OS status | Enough for Chromium? |
|-----|-------------------|---------------|----------------------|
| 17  | `getcwd`          | partial       | Yes (returns `/`)    |
| 20  | `epoll_create1`   | stub          | No — see 🔴          |
| 21  | `epoll_ctl`       | stub          | No — see 🔴          |
| 22  | `epoll_pwait`     | stub          | No — see 🔴          |
| 23  | `dup`             | partial       | Yes if real fd table |
| 24  | `dup3`            | partial       | Yes                  |
| 25  | `fcntl`           | partial (ignores F_SETFL O_NONBLOCK) | **No** — Chromium REQUIRES O_NONBLOCK to work for sockets+pipes; current impl returns 0 for F_SETFL which is lossy |
| 29  | `ioctl`           | partial       | Yes for TCGETS/FIONREAD |
| 34  | `mkdirat`         | partial       | Yes                  |
| 35  | `unlinkat`        | stub          | Yes (cache eviction skipped) |
| 46  | `ftruncate`       | stub          | **No if used with memfd** — see 🔴 |
| 48  | `faccessat`       | partial       | Yes                  |
| 49  | `chdir`           | partial       | Yes                  |
| 56  | `openat`          | partial       | Yes (given VFS)      |
| 57  | `close`           | partial       | Yes                  |
| 59  | `pipe2`           | partial       | Yes                  |
| 61  | `getdents64`      | partial       | Yes                  |
| 62  | `lseek`           | stub(0)       | **Partial** — any file read after seek returns wrong data; must return real offset |
| 63  | `read`            | partial       | Yes                  |
| 64  | `write`           | partial       | Yes                  |
| 66  | `writev`          | partial       | Yes                  |
| 71  | `sendfile`        | partial       | Usually fine         |
| 73  | `ppoll`           | partial       | **Depends on impl** — Chromium uses epoll mostly but `base::files` sometimes ppolls |
| 78  | `readlinkat`      | partial       | Yes if `/proc/self/exe` is handled |
| 79  | `newfstatat`      | partial       | Yes                  |
| 80  | `fstat`           | partial       | Yes                  |
| 93  | `exit`            | full          | Yes                  |
| 94  | `exit_group`      | full          | Yes                  |
| 96  | `set_tid_address` | full          | Yes                  |
| 98  | `futex`           | partial       | **See 🔴 — foundational** |
| 99  | `set_robust_list` | stub          | Yes                  |
| 100 | `get_robust_list` | stub          | Yes                  |
| 101 | `nanosleep`       | full          | Yes                  |
| 113 | `clock_gettime`   | full          | Yes                  |
| 131 | `tgkill`          | partial       | Yes                  |
| 132 | `sigaltstack`     | partial       | Yes                  |
| 134 | `rt_sigaction`    | partial       | **Depends** — if ASan/crash handlers fire, must deliver correctly |
| 135 | `rt_sigprocmask`  | partial       | Yes                  |
| 139 | `rt_sigreturn`    | partial       | Yes (only called from signal handler) |
| 160 | `uname`           | full          | Yes                  |
| 172 | `getpid`          | returns 1     | **Partial** — see note; low impact |
| 174-177 | getuid/geteuid/getgid/getegid | returns 0 | Yes |
| 178 | `gettid`          | partial       | **No** — must return per-thread unique ID once threads exist; currently returns caller pid equivalent |
| 179 | `sysinfo`         | full          | Yes                  |
| 198 | `socket`          | partial       | Yes if net stack wired |
| 203 | `connect`         | partial       | Yes                  |
| 206 | `sendto`          | partial       | Yes                  |
| 207 | `recvfrom`        | partial       | Yes                  |
| 214 | `brk`             | partial       | Yes (musl mostly uses mmap) |
| 215 | `munmap`          | partial       | Yes                  |
| 220 | `clone`           | partial       | **See 🔴 — must support CLONE_VM|CLONE_FS|CLONE_FILES|CLONE_SIGHAND|CLONE_THREAD|CLONE_SYSVSEM|CLONE_SETTLS|CLONE_PARENT_SETTID|CLONE_CHILD_CLEARTID** |
| 222 | `mmap`            | partial       | **See 🔴 — MAP_ANONYMOUS|MAP_PRIVATE|MAP_FIXED, MAP_STACK, MAP_NORESERVE, PROT_EXEC toggling at runtime** |
| 226 | `mprotect`        | partial       | **See 🔴 — V8 flips RX↔RW per JIT page many times/sec** |
| 233 | `madvise`         | stub          | Yes (hints only)     |
| 260 | `wait4`           | stub          | Yes (no children)    |
| 261 | `prlimit64`       | partial       | Yes                  |
| 278 | `getrandom`       | partial       | **Must be real randomness** — BoringSSL will fail TLS otherwise |
| 279 | `memfd_create`    | partial       | Yes for anonymous memfds |

---

### 🟡 Can stub safely — 26 syscalls

These are called by Chromium or its libc but either (a) don't affect correctness of rendering or (b) Chromium has a "return value 0/-ENOSYS is OK" codepath.

| #   | Name               | What Chromium does with it                           | Why stubbing is safe |
|-----|--------------------|------------------------------------------------------|----------------------|
| 19  | `eventfd2`         | Cross-thread wake. | **NOT safe to stub — see 🔴.** (placed here only to note: the `eventfd` *variant* is rarely used; `eventfd2` is the real one) |
| 32  | `flock`            | Disk cache file lock.                                 | Single-process, nobody else is racing. Return 0.      |
| 36  | `symlinkat`        | Rare, cache compaction.                               | Return -EPERM.       |
| 38  | `linkat`           | Same.                                                 | Return -EPERM.       |
| 44  | `fstatfs`          | Disk free space query.                                | Return 0 with large free-space struct.                |
| 47  | `fallocate`        | Pre-allocate cache file.                              | Return 0; cache will just not be pre-sized.           |
| 50  | `fchdir`           | Rare.                                                 | Return 0.            |
| 52  | `fchmod`           | Cache perms.                                          | Return 0.            |
| 53  | `fchmodat`         | Same.                                                 | Return 0.            |
| 54/55 | `fchownat`/`fchown` | Rare.                                              | Return 0.            |
| 67  | `pread64`          | Could be real or stubbed                              | Real-ish — implement as lseek+read. Low effort.       |
| 68  | `pwrite64`         | Same pattern.                                         | Same.                |
| 74  | `signalfd4`        | `base::FileDescriptorWatcher` SIGCHLD.                 | `--single-process` has no children; return EINVAL, Chromium falls back to no-op. |
| 75  | `splice`           | Network→disk fast path.                                | Return -ENOSYS; Chromium falls back to read+write loop. |
| 76  | `tee`              | Same pattern.                                         | Return -ENOSYS.      |
| 77  | `vmsplice`         | Same.                                                 | Return -ENOSYS.      |
| 82  | `fsync`            | Disk cache flush.                                      | Return 0 (fake success). |
| 83  | `fdatasync`        | Same.                                                 | Return 0.            |
| 115 | `clock_nanosleep`  | Precise sleep.                                        | Redirect to `nanosleep` (trivial).                    |
| 116 | `clock_adjtime`    | NTP.                                                  | -EPERM.              |
| 117 | `clock_getres`    | V8 probes clock precision.                             | Return `{0, 1}` ns. Safe.                             |
| 122 | `sched_setaffinity`| Pin threads to cores.                                  | Ignore (return 0); we're single-CPU anyway.            |
| 123 | `sched_getaffinity`| **Called by `base::SysInfo::NumberOfProcessors`**.    | Return cpuset with 1 CPU set; Chromium adapts thread pool.|
| 128 | `restart_syscall`  | Kernel-generated.                                     | -EINTR is fine; we don't preempt across syscalls.     |
| 133 | `rt_sigsuspend`    | Rare.                                                 | Block until signal — if no signals delivered, block forever (OK for V8 watchdogs). |
| 136 | `rt_sigpending`    | Rare.                                                 | Return 0 (empty set).                                 |
| 137 | `rt_sigtimedwait`  | Rare.                                                 | Return -EAGAIN.      |
| 140 | `setpriority`      | Thread niceness.                                      | Return 0 (ignore).   |
| 141 | `getpriority`      | Same.                                                 | Return 0 (nice=0).   |
| 142 | `setpgid`, 155 `getpgid`, 154 `setsid`, 170 `getpgrp` | Process group mgmt.          | All already stubbed — safe.                           |
| 153 | `times`            | CPU time accounting.                                   | Return 0.            |
| 166 | `umask`            | File-create perms.                                     | Return 022.          |
| 167 | `prctl`            | PR_SET_NAME (thread naming), PR_SET_DUMPABLE, PR_SET_SECCOMP, PR_SET_VMA | Stub all subops to return 0 except unknown → -EINVAL. PR_SET_NAME is nice-to-have for debug but non-essential. |
| 171 | `sigaltstack`      | pthread alt stack (already partial).                   | Keep as stub 0.      |
| 201 | `listen`, 202 `accept`, 242 `accept4` | Chromium content_shell does NOT listen unless `--remote-debugging-port` | Stub return 0 / -ENOSYS. |
| 208/209 | `setsockopt`/`getsockopt` | TCP_NODELAY, SO_REUSEADDR, SO_ERROR          | Stub return 0 for setsockopt; for getsockopt(SO_ERROR) must return 0 to indicate no error.  |
| 228/229 | `mlock`/`munlock` | V8 code lock.                                         | Return 0 (no-op).    |
| 232 | `mincore`         | PartitionAlloc commit check.                           | Return all-committed.                                 |
| 240 | `rt_tgsigqueueinfo`| ASan.                                                 | -EPERM.              |
| 276 | `renameat2`       | Cache atomic rename.                                   | Already stubbed.     |
| 285 | `copy_file_range` | Fast file copy.                                        | -ENOSYS; Chromium uses read+write fallback.           |
| 288 | `pkey_mprotect`   | V8 code protection keys (Intel MPK on ARM: unsupported)| -ENOSYS; V8 falls back to mprotect. |
| 293 | `pkey_alloc`      | Same.                                                 | -ENOSYS.             |
| 294 | `pkey_free`       | Same.                                                 | -ENOSYS.             |
| 441 | `epoll_pwait2`    | New API.                                              | -ENOSYS; glibc/musl falls back to `epoll_pwait`.     |
| 449 | `futex_waitv`     | New API (6.0+).                                        | -ENOSYS; libpthread falls back to futex.              |
| 462 | `mseal`           | New API (6.10+).                                       | -ENOSYS; Chromium silently skips sealing.             |
| 293 | `rseq`            | glibc/musl registers rseq at startup.                  | -ENOSYS; musl+glibc tolerate this (check musl-1.2.4+ behavior). |

---

### 🔴 Must implement (or seriously extend) — 21 syscalls

Ordered from most foundational to least.

| #   | Name             | What Chromium does with it                                                                                                                                                   | Effort   |
|-----|------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------|
| 98  | **`futex`** (full FUTEX_WAIT, FUTEX_WAKE, FUTEX_WAIT_BITSET, FUTEX_WAKE_BITSET, FUTEX_REQUEUE, FUTEX_CMP_REQUEUE, FUTEX_WAIT_PRIVATE, FUTEX_WAKE_PRIVATE) | Every mutex, condvar, `base::WaitableEvent`, V8 isolate lock, Skia image cache lock. Without a real futex, every thread-safe data structure deadlocks after the first contended op. `partial` impl must be upgraded to: wait queues keyed by `(addr, bitset)`, timeout via timerfd or direct timer integration, proper wakeup ordering.  | **Hard**  |
| 220 | **`clone`** (CLONE_VM|CLONE_FS|CLONE_FILES|CLONE_SIGHAND|CLONE_THREAD|CLONE_SYSVSEM|CLONE_SETTLS|CLONE_PARENT_SETTID|CLONE_CHILD_CLEARTID)                      | Every thread Chromium spawns. M132 content_shell --single-process spawns ~30 threads: Compositor, IO, GpuIOThread, ServiceWorkerContextManager, V8 Background, TaskScheduler workers, etc. The child needs to share the VM but get its own stack, its own TLS (via the SETTLS flag), its own signal mask, and on exit must write 0 + futex-wake at the CHILD_CLEARTID address. | **Hard**  |
| 222 | **`mmap`** (full flag support: MAP_ANONYMOUS, MAP_PRIVATE, MAP_SHARED, MAP_FIXED, MAP_FIXED_NOREPLACE, MAP_STACK, MAP_NORESERVE, MAP_GROWSDOWN, MAP_32BIT; PROT_READ/WRITE/EXEC mixing including PROT_NONE guard pages) | PartitionAlloc reserves huge regions (~16 GB address space) with MAP_NORESERVE + PROT_NONE and then mprotects chunks RW on demand. V8 code pages need MAP_ANONYMOUS+PROT_READ|PROT_EXEC (flipped at runtime). Thread stacks are MAP_PRIVATE|MAP_ANONYMOUS|MAP_STACK. Shared memory comes via mmap of a memfd. | **Hard**  |
| 226 | **`mprotect`**   | V8 W^X: during codegen the page is RW; before executing, flipped to RX; before patching, back to RW. Happens thousands of times/second. Must be cheap and must not trap. Also used to install guard pages.         | **Medium** |
| 20  | **`epoll_create1`** | `base::MessagePumpEpoll` — every thread with a pump. One epoll fd per message pump.                                                                                   | **Medium** |
| 21  | **`epoll_ctl`**  | Same pump — every AddFdWatcher / RemoveFdWatcher.                                                                                                                        | **Medium** |
| 22  | **`epoll_pwait`** | The actual blocking wait. Must block and wake on: fd readiness (from read/write ops on registered fds), eventfd wake, timerfd expiry, signal delivery.                  | **Hard**   |
| 19  | **`eventfd2`**   | `base::WaitableEvent` — cross-thread notification. Every time a thread posts a task to another thread, it's an eventfd-write. Must integrate with epoll readiness.       | **Medium** |
| 85  | **`timerfd_create`** | `base::MessagePumpEpoll::ScheduleDelayedWork`. Every animation frame, every setTimeout in JS, every fetch timeout. Must integrate with epoll.                         | **Medium** |
| 86  | **`timerfd_settime`** | Arm/disarm the timerfd.                                                                                                                                              | **Easy**   |
| 87  | **`timerfd_gettime`** | Rarely called but must work.                                                                                                                                         | **Easy**   |
| 278 | **`getrandom`** (with real entropy) | BoringSSL seeds its DRBG on first TLS op via getrandom. Must return cryptographically strong bytes — a predictable stream will break TLS.                         | **Medium** (needs a kernel CSPRNG) |
| 211 | **`sendmsg`**    | Mojo IPC over UNIX domain sockets (even single-process touches it). TCP scatter-gather in net stack.                                                                 | **Medium** |
| 212 | **`recvmsg`**    | Paired with sendmsg. Must support SCM_RIGHTS (fd passing) for Mojo.                                                                                                 | **Medium** |
| 199 | **`socketpair`** | Mojo IPC. Even in single-process, Mojo uses socketpair internally.                                                                                                  | **Easy**   |
| 216 | **`mremap`**     | V8 heap grow/shrink; PartitionAlloc.                                                                                                                                 | **Medium** |
| 204 | **`getsockname`**| After `connect`, Chromium retrieves local endpoint for NEL / QUIC.                                                                                                  | **Easy**   |
| 205 | **`getpeername`**| On accepted sockets.                                                                                                                                                 | **Easy**   |
| 115 | **`clock_nanosleep`** | Most precise sleep; Chromium uses this preferentially over nanosleep for pump timers.                                                                            | **Easy** (redirect to nanosleep with absolute-time math) |
| 117 | **`clock_getres`** | V8 probes.                                                                                                                                                          | **Trivial** |
| 25  | **`fcntl`** (real F_SETFL for O_NONBLOCK, F_SETFD for FD_CLOEXEC, F_GETFL that returns current flags, F_DUPFD) | Chromium sets O_NONBLOCK on every socket. Current stub silently ignores F_SETFL → reads return 0 bytes instead of EAGAIN → Chromium's `SocketPosix::Read` returns EOF → stream closes. Must store real flag state per-fd. | **Easy** |

---

## Task 4 — Implementation Priority (ordered)

Ordered by "unblocks the most other code when done":

### Tier 1 — Foundational (without these, content_shell cannot even initialize its thread pool)

1. **`futex` (98) — full impl.** *Rationale:* every single mutex and condvar in Chromium, V8, Skia, pthreads funnels through futex. If futex is wrong, every thread-safe data-structure access deadlocks. This is THE keystone. Must support FUTEX_WAIT + FUTEX_WAKE at minimum; FUTEX_WAIT_BITSET + FUTEX_WAKE_BITSET come next; requeue is needed for condvar broadcasts. Without this, nothing past main() works.

2. **`clone` (220) — thread clone with CLONE_VM|CLONE_THREAD|CLONE_SETTLS|CLONE_CHILD_CLEARTID at minimum.** *Rationale:* Chromium spawns its 30-ish threads during startup *before* running a single line of blink code. Without clone, content_shell never progresses past `ThreadGroupImpl::Start`. Paired with futex: clone creates the thread; CHILD_CLEARTID triggers a futex-wake at thread exit, which the joiner is FUTEX_WAITing on.

3. **`mmap` (222) — full MAP_ANONYMOUS|MAP_PRIVATE|MAP_FIXED|MAP_NORESERVE + PROT_NONE guard pages.** *Rationale:* PartitionAlloc's first act is to reserve a multi-GB region with MAP_NORESERVE. V8 reserves its isolate region similarly. Without correct flag handling, the first PartitionAlloc reservation fails and content_shell aborts.

4. **`mprotect` (226) — RW↔RX flipping.** *Rationale:* V8's JIT protection. Chromium will not execute any JavaScript without working mprotect. Flips happen at high frequency; implementation must be fast (don't walk page tables per call if we can avoid it).

5. **`getrandom` (278) — real CSPRNG.** *Rationale:* BoringSSL refuses to operate without entropy. No HTTPS → can't load google.com. We have `src/crypto/` — route it there.

### Tier 2 — Message pump (without these, nothing times out, nothing wakes up, no IPC)

6. **`epoll_create1` (20), `epoll_ctl` (21), `epoll_pwait` (22) — real impl.** *Rationale:* `base::MessagePumpEpoll` IS the heart of every thread's event loop. Chromium builds on the assumption that you can epoll-watch an fd and be woken when it's readable/writable. Must integrate with our fd table, with eventfd readiness, and with timerfd expiry.

7. **`eventfd2` (19) — real.** *Rationale:* Cross-thread task posting uses eventfd. Without it, one thread cannot wake another. The UI thread posts a paint task to the Compositor thread by writing to a compositor-owned eventfd; the Compositor's epoll_pwait returns.

8. **`timerfd_create` (85), `timerfd_settime` (86), `timerfd_gettime` (87).** *Rationale:* `setTimeout`, animation frames, network timeouts all flow through timerfd registered in the message pump's epoll. Without these, nothing times out and animations never fire.

### Tier 3 — fcntl correctness (cheap, critical)

9. **`fcntl` (25) — real F_SETFL(O_NONBLOCK) + F_SETFD(FD_CLOEXEC) + F_GETFL + F_DUPFD.** *Rationale:* Chromium calls `fcntl(fd, F_SETFL, O_NONBLOCK)` on every socket and pipe. Current impl silently ignores. Result: blocking reads stall the IO thread forever. Very low effort to fix since we already have an fd table.

### Tier 4 — IPC / Mojo foundation (needed for any multi-thread composition)

10. **`sendmsg` (211) + `recvmsg` (212) + `socketpair` (199).** *Rationale:* Mojo IPC over UNIX sockets uses socketpair + sendmsg (with SCM_RIGHTS for fd passing between browser and renderer halves — happens even in single-process when the browser creates a renderer-backed pipe). Chromium's `mojo::core::Channel::PosixChannel` calls sendmsg/recvmsg on every message.

### Tier 5 — Network (gates https://example.com)

11. **`connect` (203), `sendto` (206), `recvfrom` (207), `getsockopt` (209), `setsockopt` (208), `getsockname` (204), `getpeername` (205), `shutdown` (210), `close` (57)** — must be real for the TCP path and must integrate with epoll (`base::SocketPosix` sets O_NONBLOCK, registers in epoll for EPOLLIN/EPOLLOUT, does write, etc.). We already have stubs; tighten them to use `src/net/tcp.rs`.

### Tier 6 — File system correctness (gates reading ICU data tables, fonts, etc.)

12. **`openat` (56), `read` (63), `write` (64), `fstat` (80), `newfstatat` (79), `close` (57), `lseek` (62) — fix correctness.** *Rationale:* Chromium loads ICU tables (`icudtl.dat`), V8 context snapshot (`v8_context_snapshot.bin`), Skia font files, etc. from disk at startup. `lseek` returning 0 (stub) means every read-after-seek returns the wrong data and ICU init fails. Most of these are already "partial" — close the gaps.

### Tier 7 — V8/PartitionAlloc extras

13. **`mremap` (216).** *Rationale:* V8 heap expansion calls mremap. A fallback of "fail with -ENOSYS" causes V8 to unmap+remap which works but fragments; still ok for v1.

### Tier 8 — Nice-to-have / fine-tuning

14. **`clock_nanosleep` (115)** — redirect to `nanosleep`.
15. **`clock_getres` (117)** — return `{0, 1}`.
16. **`sched_getaffinity` (123)** — return single-CPU mask.
17. **`prctl` (167)** — handle PR_SET_NAME, PR_SET_DUMPABLE, PR_SET_VMA. Others → 0.

---

## Counts Summary

| Category                              | Count (approx) |
|---------------------------------------|----------------|
| ✅ Already have (full or partial)     | **47**         |
| 🟡 Can stub safely                    | **~43**        |
| 🔴 Must implement or seriously extend | **21**         |
| **Total distinct syscalls content_shell may invoke** | **~185**  |

Note: the 🟡 bucket is big because the seccomp policy is an upper bound; many syscalls it allows are called only in rare code paths (e.g., `clock_adjtime`, `fallocate`, `flock`) and returning -ENOSYS is benign. The 🔴 bucket is the real workload.

---

## Top-10 Must-Implement in Priority Order (for Phase 4)

1. **`futex` (98)** — full — keystone of all threading. **Hard.**
2. **`clone` (220)** — thread-clone flag set. **Hard.** (pairs with futex via CHILD_CLEARTID)
3. **`mmap` (222)** — all flags, especially MAP_NORESERVE + PROT_NONE. **Hard.**
4. **`mprotect` (226)** — V8 W^X flipping. **Medium.**
5. **`getrandom` (278)** — real CSPRNG. **Medium.**
6. **`epoll_create1` / `epoll_ctl` / `epoll_pwait` (20/21/22)** — real event loop. **Hard.**
7. **`eventfd2` (19)** — cross-thread wake, integrated into epoll readiness. **Medium.**
8. **`timerfd_create` / `timerfd_settime` (85/86)** — timers, integrated into epoll. **Medium.**
9. **`fcntl` (25)** — real F_SETFL O_NONBLOCK, F_SETFD FD_CLOEXEC, F_GETFL, F_DUPFD. **Easy.**
10. **`sendmsg` / `recvmsg` / `socketpair` (211/212/199)** — Mojo IPC, SCM_RIGHTS fd passing. **Medium.**

---

## Sources

- [Chromium baseline_policy.cc](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/sandbox/linux/seccomp-bpf-helpers/baseline_policy.cc)
- [Chromium syscall_sets.cc](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/sandbox/linux/seccomp-bpf-helpers/syscall_sets.cc)
- [Chromium bpf_base_policy_linux.cc](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/sandbox/policy/linux/bpf_base_policy_linux.cc)
- [Chromium bpf_renderer_policy_linux.cc](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/sandbox/policy/linux/bpf_renderer_policy_linux.cc)
- [Chromium bpf_gpu_policy_linux.cc](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/sandbox/policy/linux/bpf_gpu_policy_linux.cc)
- [Chromium bpf_network_policy_linux.cc](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/sandbox/policy/linux/bpf_network_policy_linux.cc)
- [ChromiumOS Linux system call table (NR name ↔ number across arches)](https://chromium.googlesource.com/chromiumos/docs/+/master/constants/syscalls.md)
- ARM64 / AArch64 numbers: `arch/arm64/include/uapi/asm/unistd.h` (Linux kernel) — the generic table in `include/uapi/asm-generic/unistd.h`.

---

End of report.

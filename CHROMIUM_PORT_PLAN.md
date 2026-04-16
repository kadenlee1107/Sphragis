# Chromium Port Plan — Option 1: Run Chromium as a Linux Guest in Bat_OS

**Goal:** Render `https://www.google.com` pixel-correctly in Bat_OS by running real
Chromium content_shell as a Linux ARM64 ELF inside the BatCave runtime.

**Strategy:** Build Chromium's `content_shell` (Chrome's rendering engine without
Chrome UI) for Linux ARM64. Statically link everything we can. Run it inside our
existing BatCave Linux runner by extending the runner's syscall coverage and
adding a framebuffer-backed Ozone display backend.

**Why this path:** It's the only option short of months of bare-metal porting
that gets us *real Chrome rendering* — the same code that ships in Chrome,
including LayoutNG, Blink, V8, Skia, the cascade, fonts, everything. We do the
work in our OS layer (syscalls, display) instead of forking Chromium itself.

---

## Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                          Bat_OS Kernel                         │
│  ┌──────────────┐  ┌──────────────────────────────────────┐    │
│  │ Display drv  │  │ BatCave Linux Runner (EL0)           │    │
│  │ (framebuffer)│  │ ┌──────────────────────────────────┐ │    │
│  │     ▲        │  │ │  Chromium content_shell ELF      │ │    │
│  │     │ blit   │  │ │   (real Chrome: Blink+V8+Skia)   │ │    │
│  │     │        │  │ │                                  │ │    │
│  │  Shared ─────┼──┼─┤  Ozone-batos backend             │ │    │
│  │  framebuffer │  │ │  Software GL (SwiftShader)       │ │    │
│  │              │  │ │  Single-process mode             │ │    │
│  │              │  │ └──────────────────────────────────┘ │    │
│  │              │  │   ▲                                  │    │
│  │              │  │   │ syscalls (futex, mmap, epoll...) │    │
│  │              │  │ ┌─┴────────────────────────────────┐ │    │
│  │              │  │ │ Syscall emulation layer (~200)   │ │    │
│  │              │  │ └──────────────────────────────────┘ │    │
│  │              │  └──────────────────────────────────────┘    │
│  └──────────────┘                                              │
└────────────────────────────────────────────────────────────────┘
```

---

## Success Criteria (in order — each gates the next)

| Stage | Verifiable Result                                                  |
|-------|--------------------------------------------------------------------|
| **1** | Chromium content_shell builds for Linux ARM64 in Docker            |
| **2** | `content_shell --screenshot=out.png URL` produces a valid PNG      |
| **3** | content_shell binary is static-PIE (or near-static) ARM64 ELF      |
| **4** | content_shell loads + runs in Bat_OS, exits cleanly on `exit_group`|
| **5** | content_shell renders a blank page to Bat_OS framebuffer           |
| **6** | content_shell renders `data:text/html,<h1>hello</h1>` to fb        |
| **7** | content_shell renders `https://example.com` to fb                  |
| **8** | content_shell renders `https://www.google.com` to fb               |

Stage 8 = mission complete.

---

## Phase Plan

### PHASE 0 — Build Environment (1-2 days)

**Deliverable:** A reproducible Linux ARM64 build environment on the M4 Mac.

**Steps:**
1. Set up a Docker Linux ARM64 container (Docker Desktop on Apple Silicon
   runs ARM64 Linux natively — no emulation overhead).
2. Use **Debian Bookworm ARM64** (matches Chromium's tested sysroot).
3. Install depot_tools: `git clone https://chromium.googlesource.com/chromium/tools/depot_tools.git`
4. Persistent volume for the source tree (~30 GB) and build output (~50 GB).
5. `gclient config https://chromium.googlesource.com/chromium/src.git`
6. `gclient sync` (~30 GB, multiple hours on first run)
7. **Pin the Chromium version**: target `M132` (stable as of writing) for
   reproducibility. `git checkout 132.0.6834.83 && gclient sync -D`

**Files to create:**
- `ports/chromium_port/Dockerfile` — the build container definition
- `ports/chromium_port/build.sh` — drives the build inside the container
- `ports/chromium_port/.gn-args` — pinned GN arguments

**Verification:** Inside the container:
```
out/Default/chrome --version    # prints "Chromium 132.0.x"
```

**Risks:**
- gclient sync might pull MacOS-specific deps; suppress with `target_os` in `.gclient`
- Disk space: the source + build output is ~80 GB. Fail loudly if the volume isn't big enough.

---

### PHASE 1 — content_shell Build (2-3 days)

**Deliverable:** A working Linux ARM64 `content_shell` binary in the container.

**GN args (pinned in `ports/chromium_port/.gn-args`):**
```
target_os                      = "linux"
target_cpu                     = "arm64"
is_official_build              = true
is_debug                       = false
symbol_level                   = 0
is_component_build             = false
treat_warnings_as_errors       = false

# Disable everything we don't need
enable_nacl                    = false
enable_widevine                = false
enable_pdf                     = false
enable_print_preview           = false
enable_remoting                = false
enable_extensions              = false
enable_plugins                 = false
proprietary_codecs             = false
safe_browsing_mode             = 0

# No host integrations
use_alsa                       = false
use_pulseaudio                 = false
use_cups                       = false
use_dbus                       = false
use_kerberos                   = false
use_gnome_keyring              = false
use_glib                       = false
use_gio                        = false
use_udev                       = false

# Use Chromium's bundled toolchain + sysroot for reproducibility
use_sysroot                    = true
use_lld                        = true
clang_use_chrome_plugins       = false

# Ozone with a custom platform we'll add
use_ozone                      = true
ozone_auto_platforms           = false
ozone_platform_headless        = true
ozone_platform                 = "headless"
# Later phase: ozone_platform = "batos"

# Headless single-process for sanity
headless_mode                  = true
use_v8_context_snapshot        = true
```

**Build:**
```
gn gen out/BatOs --args="$(cat .gn-args)"
autoninja -C out/BatOs content_shell
```

**Verification:** Inside the container:
```
out/BatOs/content_shell \
  --headless \
  --no-sandbox \
  --disable-gpu \
  --screenshot=/tmp/google.png \
  https://www.google.com
```
Should produce a valid PNG of Google's homepage.

**Risks:**
- Build will fail at first; iterate on GN args until it succeeds
- Some flags conflict; `gn args out/BatOs --list` shows available knobs

---

### PHASE 2 — Static Linking (3-5 days)

**Deliverable:** A content_shell binary with **zero or minimal** dynamic library deps.

**The problem:** Chromium normally dynamically links to libc, libstdc++, libdl,
libpthread, libm, librt, etc. Our BatCave loader supports static-PIE only.

**Approach:**
1. Force `-static-libstdc++ -static-libgcc` (already done with `is_official_build=true`)
2. Switch from glibc to **musl** for the build:
   - Use Alpine ARM64 sysroot instead of Debian
   - Build with `--target=aarch64-linux-musl`
3. Modify Chromium's BUILD.gn for content_shell to add `-static-pie` to ldflags
4. Verify with `ldd ./content_shell` — should print "not a dynamic executable"

**If full-static fails (likely for some libs):**
- Identify the few .so files we MUST load
- Implement a minimal dynamic loader in BatCave that handles them
- Bundle them alongside the binary

**Files to modify:**
- `out/BatOs/args.gn` (add `extra_ldflags = "-static-pie"`)
- Potentially: `content/shell/BUILD.gn` to add static-PIE link flags

**Verification:**
```
file out/BatOs/content_shell
# Want: "ELF 64-bit LSB pie executable, ARM aarch64, ..., static-pie linked"

ldd out/BatOs/content_shell
# Want: "not a dynamic executable"
```

**Risks:**
- musl is API-compatible but not ABI-compatible with glibc; some Chromium code uses glibc-specific calls (e.g., `gnu_get_libc_version`). Patch them out.
- libc++ vs libstdc++: Chromium uses libc++ from its bundled clang. Should be fine.
- TLS (thread-local storage): static-PIE TLS works but is finicky. Might need to disable some optimizations.

---

### PHASE 3 — Syscall Coverage Analysis (2-3 days)

**Deliverable:** A complete list of every syscall content_shell uses, categorized.

**Steps:**
1. Inside the container, run:
   ```
   strace -c -f -e trace=all out/BatOs/content_shell \
     --headless --no-sandbox --disable-gpu \
     --screenshot=/tmp/x.png https://example.com 2> strace.log
   ```
2. Parse strace output to extract the unique syscall set.
3. Cross-reference with `src/batcave/linux/syscalls.rs` — what we have vs need.
4. Categorize each missing syscall:
   - **Trivial:** Just need to wire to existing kernel facility (clock_gettime, getpid)
   - **Stub:** Can return -ENOSYS or success without breaking Chromium (prctl with unknown args)
   - **Hard:** Needs real implementation (futex, clone, epoll, mmap variants)

**Expected count:** ~150-200 distinct syscalls.

**Files to create:**
- `ports/chromium_port/SYSCALLS.md` — the full categorized list
- `ports/chromium_port/strace.log.gz` — the raw trace for reference

**Verification:** SYSCALLS.md exists with every syscall categorized.

---

### PHASE 4 — BatCave Syscall Implementation (1-2 weeks, the longest phase)

**Deliverable:** BatCave syscall coverage sufficient to run content_shell.

**Critical syscalls that MUST work (Chromium will hard-crash without them):**

| Syscall              | Status        | Notes                                            |
|----------------------|---------------|--------------------------------------------------|
| `mmap` (all flags)   | partial       | Need MAP_ANONYMOUS, MAP_PRIVATE, MAP_FIXED, etc. |
| `mprotect`           | partial       | Page permission changes                          |
| `madvise`            | stub OK       | Most advice is hints                             |
| `futex`              | **MUST IMPL** | Used by every Chromium thread synchronization    |
| `clone`              | **MUST IMPL** | Used to spawn threads                            |
| `set_tid_address`    | trivial       | Just store a pointer                             |
| `set_robust_list`    | stub OK       | Robust mutex cleanup                             |
| `epoll_create1`      | **MUST IMPL** | Mojo IPC, network                                |
| `epoll_ctl`          | **MUST IMPL** |                                                  |
| `epoll_pwait`        | **MUST IMPL** |                                                  |
| `eventfd2`           | **MUST IMPL** | Cross-thread wakeups                             |
| `signalfd4`          | stub OK       | Signal delivery                                  |
| `timerfd_create`     | **MUST IMPL** | Animation timers                                 |
| `timerfd_settime`    | **MUST IMPL** |                                                  |
| `clock_gettime`      | trivial       | We have a clock                                  |
| `clock_nanosleep`    | trivial       | Blocking sleep                                   |
| `nanosleep`          | trivial       |                                                  |
| `sched_yield`        | trivial       | Cooperative yield                                |
| `sched_getaffinity`  | stub OK       | Return all CPUs                                  |
| `sched_setaffinity`  | stub OK       | Ignore                                           |
| `getrandom`          | **MUST IMPL** | Crypto, hash seeds                               |
| `prlimit64`          | stub OK       | Return high limits                               |
| `getrlimit/setrlimit`| stub OK       |                                                  |
| `prctl`              | partial stub  | Many subops; stub unknown ones                   |
| `rt_sigaction`       | partial       | Signal handlers                                  |
| `rt_sigprocmask`     | partial       |                                                  |
| `rt_sigreturn`       | trivial       | Return from signal handler                       |
| `tgkill`             | partial       | Send signal to thread                            |
| `socket/connect/...` | partial       | Network — see Phase 6                            |
| `getaddrinfo`        | userspace     | Implemented in libc, not a syscall               |
| `openat/readlinkat`  | partial       | File system access                               |
| `fstatat`            | partial       |                                                  |
| `getdents64`         | **MUST IMPL** | Directory enumeration                            |
| `pipe2`              | **MUST IMPL** | Process IPC                                      |
| `dup2/dup3`          | partial       | FD duplication                                   |

**Hardest item: real threading.**

Chromium runs ~30 threads even in single-process mode (Compositor, IO, Worker
pool, V8 GC, etc.). Our BatCave currently runs binaries single-threaded.

Two options:
- **(a) Real threads via clone():** Implement `clone(CLONE_VM | CLONE_FS | ...)` to spawn a new BatCave task that shares the parent's address space. Wire up futex. This is the right answer but is several days of kernel work.
- **(b) Fake it with `--single-process --no-zygote`** plus Chromium flags to disable as many threads as possible. We'd still need a couple of threads minimum.

**Recommendation:** Go with (a). It's the foundation we'd need anyway for any
future Linux app that uses threads. Implement enough of clone+futex to get a
working pthreads.

**Files to create/modify:**
- `src/batcave/linux/syscalls.rs` — add ~50 new syscall handlers
- `src/batcave/linux/futex.rs` — futex implementation (wait queues per address)
- `src/batcave/linux/threads.rs` — clone() + thread management
- `src/batcave/linux/epoll.rs` — epoll implementation
- `src/batcave/linux/eventfd.rs` — eventfd
- `src/batcave/linux/timerfd.rs` — timerfd

**Verification:**
After each batch of syscalls, run content_shell and observe how far it gets:
- `--version` flag (touches very few syscalls) — first milestone
- `--headless --screenshot --disable-gpu` of `data:text/html,<h1>hi</h1>` — second milestone
- The same with `https://example.com` — third milestone

---

### PHASE 5 — Display Backend (Ozone-batos) (1 week)

**Deliverable:** Chromium renders pixels into Bat_OS's framebuffer.

**Chromium's display abstraction is Ozone** — it has backends for X11, Wayland,
DRM, headless, etc. We'll add a "batos" backend that:
1. Allocates a shared memory region for the rendered pixels
2. Tells Chromium "your window is XxY pixels"
3. After each frame, Chromium writes pixels to the shared region
4. Bat_OS's display driver blits the shared region to the real framebuffer

**Two implementation paths:**

**Path A (faster): Use the existing `headless` Ozone backend**
- The `--headless` mode renders into an off-screen surface
- We hook the surface's pixel buffer (it's just a `SkBitmap`)
- After each frame, blit to fb
- Pros: less work, uses upstream code
- Cons: not a "real" backend, may have rough edges with full rendering

**Path B (correct): Custom Ozone platform**
- `ui/ozone/platform/batos/` — new directory in Chromium tree
- Implement: `OzonePlatformBatos`, `BatosWindow`, `BatosSurfaceFactory`
- ~1500 lines of code following the headless backend as template
- Pros: clean, supports real input later
- Cons: ~3-5 days of work, requires patching Chromium

**Recommendation:** Start with Path A to get pixels on screen FAST. Migrate to
Path B once everything else works.

**For GPU:**
- Use `--disable-gpu --disable-gpu-compositing --use-gl=swiftshader-webgl`
  to force software rendering
- Or `--disable-gpu --disable-software-rasterizer` to use the simplest 2D path
- SwiftShader is statically linkable (~200 KB)

**Files to create:**
- `src/drivers/display/chromium_blit.rs` — receives Chromium's pixel buffer
  and blits to framebuffer
- `ports/chromium_port/patches/ozone-blit.patch` — small Chromium patch to
  expose the headless surface's pixels via shared memory

**Verification:**
- `chromium data:text/html,<h1 style='color:red'>hello</h1>` shows red "hello" on Bat_OS framebuffer
- Resolution matches what we ask for
- No tearing, no garbage pixels

---

### PHASE 6 — Network Bridge (3-5 days)

**Deliverable:** Chromium can fetch URLs through Bat_OS's network stack.

**Chromium has its own complete network stack** (`//net`). What we need to
provide is the underlying socket I/O.

**The good news:** `socket()`, `connect()`, `send()`, `recv()`, `getaddrinfo()`
are syscalls (or libc functions backed by syscalls). If our BatCave can route
those to our existing TLS 1.3 stack and TCP socket implementation, Chromium's
entire network stack just works.

**Steps:**
1. Identify which network syscalls Chromium calls: `socket`, `connect`, `bind`,
   `accept4`, `recvmsg`, `sendmsg`, `getsockopt`, `setsockopt`, `getpeername`,
   `getsockname`, `shutdown`, `close`.
2. Wire each to our existing `src/net/` stack:
   - `socket(AF_INET, SOCK_STREAM, ...)` → allocate a TCP socket struct
   - `connect()` → drive the TCP handshake using our TCP impl
   - `recv/send` → read/write to the TCP socket buffer
   - For HTTPS, **Chromium does TLS itself in userspace** (BoringSSL is in the binary). It just wants raw TCP from us.
3. DNS: Chromium uses its own DNS resolver (DoH or system stub). Provide
   `getaddrinfo()` via our DNS or just hardcode common IPs for testing.

**Critical detail:** Chromium uses non-blocking sockets + epoll. Our BatCave
TCP must support `O_NONBLOCK` and integrate with epoll's readiness signaling.

**Files to modify:**
- `src/batcave/linux/syscalls.rs` — add socket family
- `src/batcave/linux/sockets.rs` — new file, glue layer
- `src/net/tcp.rs` — add non-blocking mode + epoll integration

**Verification:**
- `chromium https://example.com` shows the example.com page
- `chromium https://www.google.com` shows Google
- Network panel of `--enable-logging=stderr --vmodule=*net*=2` shows real traffic

---

### PHASE 7 — Boot Integration (3-5 days)

**Deliverable:** A `chromium <url>` shell command in Bat_OS launches Chromium and renders the URL.

**Steps:**
1. Embed `content_shell` as a resource:
   - Option A: `static CONTENT_SHELL: &[u8] = include_bytes!(...)` (kernel grows by ~150 MB — too much)
   - Option B: Load from a virtual ROM disk (BatFS-mounted blob)
   - Option C: Initrd-style: append after kernel image, locate via boot params
   - **Recommend Option C** for size efficiency
2. Add `chromium` shell command in `src/ui/shell.rs`:
   ```
   chromium <url>
       Launch Chromium content_shell rendering the URL.
   ```
3. The command:
   - Locates content_shell in the appended blob
   - Loads via `batcave::linux::loader::load_elf`
   - Sets up argv: `["content_shell", "--headless", "--no-sandbox",
     "--disable-gpu", "--ozone-platform=batos", url]`
   - Allocates ~512 MB of address space for it
   - Starts it; routes stdout/stderr to Bat_OS UART
   - On display callback, blits to framebuffer

**Files to modify:**
- `src/ui/shell.rs` — add `chromium` command
- `src/batcave/linux/loader.rs` — support large mmap regions
- `linker.ld` / `linker_apple.ld` — add a section for the appended content_shell blob
- `boot_chain/` — bootloader needs to copy the blob into RAM

**Verification:**
- `chromium https://example.com` from the Bat_OS shell renders example.com
- Process exits cleanly when content_shell exits
- Memory is freed

---

### PHASE 8 — Polish & Hit the Goal (ongoing)

**Final mile:**
1. Tune memory: Chromium needs ~500 MB minimum. Bat_OS currently has 116 MB free at boot. Need to bump VM size and verify our frame allocator scales.
2. Performance: First load will be slow (cold caches, no JIT warm-up). Acceptable for v1.
3. Input: Wire keyboard events from Bat_OS PS/2 driver to Chromium key events (Phase 8 stretch).
4. Mouse: Same for mouse (stretch).
5. Multiple URLs: Implement reload, navigation, history (stretch).

**Final verification:** `chromium https://www.google.com` from Bat_OS shell
shows the Google homepage on the framebuffer. Pixels should be byte-identical
to what Chrome renders on Linux.

---

## Risk Register

| Risk                                                         | Probability | Impact | Mitigation                                                |
|--------------------------------------------------------------|-------------|--------|-----------------------------------------------------------|
| Chromium build fails on first attempt                        | High        | Low    | Iterate on GN args; community has done this many times    |
| Static linking impossible for some lib                       | Medium      | Medium | Bundle .so files; implement minimal dynamic loader        |
| futex implementation has races                               | Medium      | High   | Test thoroughly; reference Linux kernel impl              |
| Threading exposes scheduler bugs                             | High        | High   | Phase 4 may surface kernel issues; budget extra time      |
| Chromium uses something we forgot (vDSO, AT_RANDOM auxv)     | High        | Medium | strace will show us; fix iteratively                      |
| Memory consumption exceeds VM size                           | Medium      | High   | Pre-bump VM in QEMU args; allocate more in BatCave       |
| SwiftShader doesn't statically link                          | Low         | Medium | Bundle as .so or use `--disable-gpu` only                 |
| Network: TCP edge cases with epoll non-blocking              | High        | Medium | Add non-blocking + epoll readiness piece by piece         |
| First page renders garbage (font, layout)                    | High        | Low    | Iterate; usually missing fonts or display config          |

---

## Decisions Made (Don't Re-litigate)

1. **Single-process mode.** No fork/exec, no Mojo IPC complexity. Use
   `--single-process --no-zygote`. We lose sandboxing; that's fine for v1.
2. **content_shell, not chrome.** No Chrome UI; we don't need omnibox, tabs.
   Just a window that renders a URL.
3. **Software rendering via SwiftShader.** No GPU drivers in Bat_OS.
4. **musl libc** in the build sysroot, not glibc. Better static linking story.
5. **Pin to M132.** Don't chase head; reproducibility matters.
6. **Path A first for Ozone (headless backend hack), Path B later.**
7. **Blob-appended-to-kernel** for content_shell binary distribution.
8. **Real threading via clone()+futex.** No hacks; we need this anyway.

---

## Working Directory Layout

```
ports/chromium_port/
├── Dockerfile              # Linux ARM64 build container
├── build.sh                # Drives the build
├── .gn-args                # Pinned GN arguments (Phase 1)
├── SYSCALLS.md             # Categorized syscall list (Phase 3)
├── strace.log.gz           # Raw trace for reference
├── patches/                # Chromium source patches we apply
│   ├── 001-static-pie.patch
│   ├── 002-ozone-blit.patch
│   └── 003-musl-compat.patch
└── README.md               # How to reproduce the build
```

---

## ETA (calendar time, not effort time)

| Phase                              | Duration  | Cumulative |
|------------------------------------|-----------|-----------|
| 0 — Build environment              | 1-2 days  | day 2     |
| 1 — content_shell builds in Docker | 2-3 days  | day 5     |
| 2 — Static linking                 | 3-5 days  | day 10    |
| 3 — Syscall analysis               | 2-3 days  | day 13    |
| 4 — Syscall implementation         | 7-14 days | day 27    |
| 5 — Display backend                | 5-7 days  | day 34    |
| 6 — Network bridge                 | 3-5 days  | day 39    |
| 7 — Boot integration               | 3-5 days  | day 44    |
| 8 — Polish & hit goal              | open      | day 50+   |

**Realistic target: 6-8 weeks to first Google render.** Faster if no surprises.

---

## What's Out of Scope (For Now)

- Multi-process Chromium (sandboxing)
- GPU acceleration (Vulkan, OpenGL drivers)
- Audio (no <audio>/<video> playback)
- Video decoders (proprietary codecs disabled)
- Geolocation, USB, Bluetooth, mDNS
- Save/print
- File downloads (need filesystem write path beyond what we have)
- Service workers' background sync (hard without background processes)

These are all addable later. None block "render google.com."

---

## Mantras for the Build

1. **No surprises.** If something is hard, document it. If it's blocked,
   write down WHY.
2. **Verify each stage before moving on.** Don't stack debugging.
3. **Real, not stubs, for the runtime layer.** Stub Chromium-internal code
   ruined the previous attempt; this time the OS extends, Chromium stays whole.
4. **Boring is good.** Use upstream defaults wherever possible.

---

End of plan.

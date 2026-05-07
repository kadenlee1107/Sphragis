# DESIGN: Real Browser on Bat_OS (Project Gotham)

> # ⚠️ SUPERSEDED (2026-05-07): Bat_OS no longer ships a browser.
>
> The Chromium-port effort hit a wall on 2026-05-05 (Mojo IPC
> livelock + V8 sandbox cage). We briefly pivoted to **Ladybird**
> (path 2 in the prior `DESIGN_BROWSER.md`) and added a
> **stream-client** thin-client (path 3). On 2026-05-07 we made
> the harder call: Bat_OS doesn't ship a browser at all.
>
> See **`DESIGN_NO_BROWSER.md`** for the current strategy and
> rationale, and tag `pre-no-browser-2026-05-07` for the last
> commit that contained any browser code.
>
> The historical content below is preserved as reference for
> the syscall coverage / libc gap analysis that informed the
> Linux ELF runtime — those primitives still apply to running
> non-browser ELFs (busybox, posix_test, cxx_test).
>
> Original PARKED notice (kept for trail):
>
> **Why parked.** Two compounding blockers, neither cheap to break:
>
> 1. **Mojo IPC livelock.** Chromium's process model (browser ↔ renderer
>    ↔ GPU ↔ utility) talks over Mojo, which assumes Linux-specific
>    primitives (sealed memfd, kcmp, pidfd, futex semantics) we only
>    partially provide. After landing iters 1–8 of fixes (Landlock
>    stubs, demand_page perm-fault, real renameat, brk-skip same-fn
>    redirect, futex-wake all-buckets, MAP_FIXED L3 overwrite), Chromium
>    reaches Mojo IPC setup and livelocks waiting on cross-process
>    handles that don't transit our model.
> 2. **V8 sandbox cage.** V8 wants a 1 TB virtual reservation for the
>    pointer-compression cage. Disable-able with build flags, but every
>    Mac-side `content_shell` we have was built with cage-on; rebuilding
>    cage-off requires another full Chromium build (~6h on M-series in
>    Docker) and does not address the Mojo issue.
>
> **What pivoted to.**
> - **Ladybird (path 2)** — independent BSD-licensed browser, uses
>   LibIPC over Unix sockets (not Mojo), uses LibJS (not V8, no
>   sandbox cage). Built alongside SerenityOS so its authors already
>   solved "make a browser portable to a brand-new OS." `port/ladybird`
>   branch, iter 28 renders pixels.
> - **stream-client (path 3)** — `cmd_web` shell command. Chromium
>   stays on Mac (headless, via Playwright), Bat_OS displays BGRA
>   pixels. Chromebook/VNC pattern.
>
> **What's still useful in this doc.** The kernel-side syscall coverage
> notes, libc/pthreads gap analysis, and the layer-cake architecture
> still apply to running *any* dynamic Linux ELF — Ladybird (Lagom)
> and busybox both ride on the same primitives. iters 1–8 fixes from
> the Chromium effort all carried over and apply to Ladybird.
>
> **What to read instead.**
> - `DESIGN_BROWSER.md` — the active native engine + the three-path overview.
> - `docs/SESSION_JOURNAL.md` 2026-05-05 entries — Chromium-wall debugging
>   trail and pivot rationale.
> - `ports/chromium_port/` — the parked artifacts; `content_shell` binary
>   and bake scripts still exist if anyone tries this again.
>
> ---

## Vision (as originally written)
Run a full graphical web browser with proper CSS rendering on Bat_OS — 
a bare-metal ARM64 OS with zero external dependencies. Start with NetSurf 
(milestone 1), build toward Chromium (final milestone).

## Current State (What We Have)

### Kernel
- ARM64 microkernel: MMU, priority scheduler, IPC, exception handling
- HVF atomic instruction emulation (LDAXR/STLXR/CAS)
- Frame allocator: 125,168 KB free, 4KB pages
- VirtIO drivers: GPU (framebuffer), keyboard, network

### Linux Compatibility (BatCave)
- ELF loader: loads aarch64 Linux binaries
- 50+ Linux syscall stubs: read, write, open, close, mmap, brk, 
  ioctl, socket, connect, send, recv, fork, exec, wait, etc.
- Per-container VFS (8 instances, 512 nodes each)
- FD table (64 entries per container)
- Successfully runs busybox (ash shell, coreutils)

### Network
- Full TCP/IP stack (DNS, TCP 3-way handshake, UDP)
- TLS 1.3 (X25519 + AES-128-GCM, ALPN, multi-record)
- HTTP/1.1 with chunked encoding
- Gzip decompression (DEFLATE)

### Browser (BatBrowser)
- HTML parser, CSS engine, layout engine, paint engine
- Bytecode JavaScript engine (8K lines, NaN-boxing, closures)
- Renders Wikipedia over HTTPS
- Text-mode and DOM-based rendering

### What's Missing for Real C/C++ Programs
- No libc (C standard library)
- No pthreads (threading)
- No mmap for userspace (only kernel)
- No signal handling
- No pipe/socketpair IPC
- No dynamic linker
- No C++ standard library
- No pkg-config / build system integration

---

## Architecture

### Layer Cake

```
+--------------------------------------------------+
|              Chromium / NetSurf                    |  <-- Goal
+--------------------------------------------------+
|              Third-party libs                     |
|    (Skia, ICU, zlib, libpng, freetype, etc.)     |
+--------------------------------------------------+
|              C++ Standard Library                 |
|              (libstdc++ or libc++)                |
+--------------------------------------------------+
|              C Standard Library (musl libc)       |
|    malloc, stdio, string, math, pthread, etc.     |
+--------------------------------------------------+
|              Bat_OS Syscall Interface              |
|    (Linux-compatible syscall numbers, ABI)        |
+--------------------------------------------------+
|              Bat_OS Kernel                         |
|    ARM64 microkernel, MMU, scheduler, VirtIO      |
+--------------------------------------------------+
```

### Key Insight
We DON'T need to port Chromium's source code. We need to make 
Bat_OS look enough like Linux that pre-compiled aarch64 Linux 
binaries (or cross-compiled ones) can run on it.

Our BatCave already emulates Linux syscalls. We just need to 
expand it to cover everything a browser needs.

---

## Phase 1: Minimal libc (musl port)
**Goal**: C programs with printf, malloc, file I/O compile and run.
**Estimated Lines**: ~5,000 (subset of musl)
**Duration**: 1-2 sessions

### What to Build
1. **Memory allocator** — dlmalloc or simple bump allocator
   - malloc(), free(), calloc(), realloc()
   - mmap() syscall for large allocations
   - brk() for heap growth (already have this)

2. **String functions** — memcpy, memset, strlen, strcmp, strcpy, 
   strstr, snprintf, strtol, strtod, etc.
   - These are pure computation, no syscalls needed

3. **stdio** — FILE*, fopen, fclose, fread, fwrite, fprintf, printf
   - Backed by our FD table + VFS
   - stdout/stderr → UART serial

4. **stdlib** — exit, abort, atexit, getenv, rand, qsort, bsearch

5. **math** — sin, cos, sqrt, pow, exp, log, floor, ceil, round
   - We already have some in the JS engine
   - Use soft-float implementations for no_std

6. **errno** — thread-local errno (or global for now)

### Approach
Option A: Port musl libc (MIT license, ~80K lines total, we need ~5K)
Option B: Write our own minimal libc from scratch
Recommendation: **Option B** — our own, because:
- We control everything
- No build system complexity
- Can integrate directly with our syscalls
- Only implement what we actually need

### Syscalls to Add/Fix
- SYS_mmap (9): proper anonymous mmap for heap/stack
- SYS_munmap (11): free mmap'd regions
- SYS_mprotect (10): change page permissions
- SYS_mremap (25): resize mappings
- SYS_getpid (172): return container ID
- SYS_gettid (178): return thread ID (same for now)
- SYS_clock_gettime (113): real timestamps
- SYS_nanosleep (101): sleep implementation

### Test Milestone
```c
#include <stdio.h>
#include <stdlib.h>
int main() {
    char *buf = malloc(1024);
    snprintf(buf, 1024, "Hello from libc on Bat_OS!\n");
    printf("%s", buf);
    free(buf);
    return 0;
}
```
Cross-compile with `aarch64-linux-gnu-gcc -static`, load as ELF, runs.

---

## Phase 2: pthreads + Synchronization
**Goal**: Multi-threaded C programs run correctly.
**Estimated Lines**: ~3,000
**Duration**: 1-2 sessions

### What to Build
1. **Thread creation** — pthread_create, pthread_join, pthread_exit
   - Map to kernel thread creation (need to add to scheduler)
   - Each thread gets its own stack (mmap'd)

2. **Mutexes** — pthread_mutex_init/lock/unlock/destroy
   - Use ARM64 atomics (LDAXR/STLXR — already emulated!)
   - Spinlock-based initially, futex later

3. **Condition variables** — pthread_cond_init/wait/signal/broadcast
   - Wait queue per condvar

4. **Thread-local storage** — __thread, pthread_key_create/setspecific
   - Use TPIDR_EL0 register for TLS base

5. **Once** — pthread_once (run initialization exactly once)

6. **Read-write locks** — pthread_rwlock_*

### Kernel Changes Required
- Add thread creation syscall (SYS_clone = 220)
- Add futex syscall (SYS_futex = 98) for efficient waiting
- Scheduler: support multiple threads per container
- Stack allocation: mmap new stack per thread

### Test Milestone
```c
#include <pthread.h>
#include <stdio.h>
void *worker(void *arg) {
    printf("Thread %d running\n", *(int*)arg);
    return NULL;
}
int main() {
    pthread_t t[4];
    int ids[4] = {1,2,3,4};
    for (int i = 0; i < 4; i++)
        pthread_create(&t[i], NULL, worker, &ids[i]);
    for (int i = 0; i < 4; i++)
        pthread_join(t[i], NULL);
    printf("All threads done\n");
}
```

---

## Phase 3: Port Core Libraries
**Goal**: Build the dependency tree that browsers need.
**Estimated Lines**: ~10,000 (our code) + cross-compiled libs
**Duration**: 2-3 sessions

### Libraries to Port (in dependency order)

1. **zlib** (11K lines C) — compression
   - Already have DEFLATE, but zlib API needed
   - Or: compile zlib with our libc

2. **libpng** (30K lines C) — PNG image decoding
   - Already have PNG decoder, but libpng API needed
   - Depends on zlib

3. **libjpeg-turbo** (50K lines C) — JPEG decoding
   - Already have JPEG decoder

4. **FreeType** (100K lines C) — Font rendering
   - CRITICAL for real browsers
   - Renders TrueType/OpenType fonts to bitmaps
   - No current equivalent in Bat_OS

5. **HarfBuzz** (80K lines C++) — Text shaping
   - Complex text layout (ligatures, RTL, etc.)
   - Depends on FreeType

6. **Pixman** (30K lines C) — Pixel manipulation
   - Used by Cairo for rendering

7. **Cairo** (100K lines C) — 2D graphics
   - Vector graphics, anti-aliasing
   - Depends on Pixman, FreeType

8. **cURL** (150K lines C) — HTTP client
   - Or use our TCP/TLS stack with a cURL-compatible API

### Approach
Cross-compile each library with `aarch64-linux-gnu-gcc -static` 
targeting our libc. Test each one individually.

### Test Milestone
Load a TrueType font file, render "Hello Bat_OS" with FreeType,
display anti-aliased text on the framebuffer.

---

## Phase 4: NetSurf Browser (MILESTONE 1)
**Goal**: Run NetSurf — a real graphical browser with CSS support.
**Estimated Lines**: ~100K (NetSurf) + our glue code
**Duration**: 3-5 sessions

### Why NetSurf
- Designed for embedded/alternative OS (runs on RISC OS, Haiku, Amiga)
- Has a clean platform abstraction layer (easy to port)
- Full CSS2.1 support, partial CSS3
- ~100K lines of C (manageable)
- MIT licensed
- Has its own rendering engine (no Blink/Gecko dependency)
- Already runs on framebuffer Linux (fbdev)

### NetSurf Architecture
```
NetSurf
├── content/ — Content handling (HTML, CSS, images)
├── css/ — LibCSS (CSS parser + selector matching)
├── render/ — Layout engine (box model, tables, floats)
├── desktop/ — Platform-independent browser chrome
├── frontends/
│   ├── gtk/     — GTK frontend
│   ├── framebuffer/ — Direct framebuffer (THIS IS US)
│   └── ...
└── utils/ — Utilities
```

### What We Need to Provide (Platform Layer)
1. **Framebuffer access** — map our GPU framebuffer
2. **Input events** — keyboard from our VirtIO keyboard
3. **Network** — HTTP fetching via our TCP/TLS stack
4. **Font rendering** — via FreeType (Phase 3)
5. **File system** — for config files, cache (our VFS)
6. **Timer** — for animations, timeouts

### Steps
1. Cross-compile NetSurf's framebuffer frontend for aarch64
2. Implement the platform abstraction layer (nsfb backend)
3. Wire our network stack to NetSurf's fetch API
4. Wire our keyboard to NetSurf's event system
5. Wire our framebuffer to NetSurf's display output
6. Test with simple pages, then Wikipedia

### Test Milestone
NetSurf renders https://example.com with proper CSS:
- Correct fonts (not monospace!)
- Proper margins, padding, borders
- Text reflow / word wrap
- Images displayed

---

## Phase 5: Expanded OS Features (for Chromium)
**Goal**: Full enough Linux compatibility for complex C++ programs.
**Estimated Lines**: ~8,000
**Duration**: 2-3 sessions

### What to Add
1. **Signals** — SIGTERM, SIGCHLD, SIGPIPE, SIGSEGV handlers
2. **Pipes** — pipe(), pipe2() for IPC
3. **Unix sockets** — socketpair(), for Chromium IPC
4. **epoll** — epoll_create, epoll_ctl, epoll_wait (event loop)
5. **Shared memory** — shmget or mmap(MAP_SHARED)
6. **Process management** — proper fork with COW pages
7. **/proc filesystem** — /proc/self/maps, /proc/cpuinfo
8. **inotify** — file system notifications
9. **Dynamic linker** — load .so files (or use static linking)

---

## Phase 6: Skia + V8 (Chromium Foundation)
**Goal**: Chromium's 2D graphics and JS engine running.
**Duration**: 3-5 sessions

### Skia (Chromium's Graphics)
- 1M lines of C++
- Renders all 2D content in Chromium
- Needs: FreeType, zlib, libpng, libjpeg
- Output: our framebuffer (software rendering path)
- Key: use Skia's software rasterizer, not GPU path

### V8 (Chromium's JS Engine)
- 10M lines of C++
- JIT compiler for ARM64 (we're on ARM64!)
- Alternative: use our bytecode JS engine for initial testing
- V8 needs: pthreads, mmap, signals, lots of memory

---

## Phase 7: Blink + Chromium Shell
**Goal**: Chromium rendering web pages.
**Duration**: Many sessions

### Approach
Don't build the full Chrome browser. Build a minimal "content shell":
- Blink rendering engine only
- Single-process mode (no sandbox)
- Our framebuffer for display
- Our network stack for HTTP/TLS
- Skia for 2D rendering
- V8 for JavaScript

This is similar to Chromium's `content_shell` target, which is 
a minimal browser for testing Blink.

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| libc incompatibility | High | Test each function thoroughly |
| Memory usage (256MB RAM) | High | Use swap or increase QEMU RAM |
| FreeType port complexity | Medium | Start with bitmap fonts as fallback |
| Thread bugs (race conditions) | High | Start single-threaded |
| Chromium build complexity | Very High | Use content_shell, not full Chrome |
| Performance (emulated CPU) | Medium | Focus on correctness first |

## Priority Order

1. **libc** (Foundation — everything depends on this)
2. **FreeType** (Biggest visual upgrade — real fonts)
3. **NetSurf** (First real browser milestone)
4. **pthreads** (Needed for V8/Chromium)
5. **Expanded syscalls** (Needed for Chromium)
6. **Skia + V8** (Chromium foundation)
7. **Blink** (Final milestone)

## Success Criteria

### Milestone 1 (NetSurf): "Real Browser"
- Renders Wikipedia with proper CSS
- Anti-aliased TrueType fonts
- Clickable links with navigation
- Images displayed inline
- Smooth scrolling

### Milestone 2 (Chromium content_shell): "Modern Browser"
- Renders Google.com with full interactivity
- JavaScript-driven pages work (React, Vue, etc.)
- CSS3 animations
- Web fonts (Google Fonts)
- HTML5 video (stretch goal)

---

## File Structure (New)

```
src/
  compat/
    libc/
      mod.rs        — libc module root
      string.rs     — memcpy, strlen, strcmp, etc.
      stdio.rs      — FILE, printf, fopen, etc.
      stdlib.rs     — malloc, free, exit, etc.
      math.rs       — sin, cos, sqrt, etc.
      pthread.rs    — thread creation, mutexes
      errno.rs      — error numbers
      syscall.rs    — raw syscall wrappers
    posix/
      mmap.rs       — virtual memory management
      signal.rs     — signal handling
      pipe.rs       — pipe IPC
      epoll.rs      — event polling
      socket.rs     — expanded socket support
  ports/
    freetype/       — FreeType integration
    netsurf/        — NetSurf platform layer
    skia/           — Skia backend (Phase 6)
```

---

## Current Stats (Starting Point)

- Source files: 111
- Lines of Rust: ~30,000
- Dependencies: 0
- Sites rendering: example.com, Google, Wikipedia
- Encryption: TLS 1.3 (X25519 + AES-128-GCM)
- JS Engine: Bytecode VM (8,294 lines)

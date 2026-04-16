# Phase 7 — Boot Integration

**Goal:** Make `content_shell` (the Chromium rendering engine ELF, ~150 MB after
static linking) bootable inside Bat_OS via the shell command
`chromium <url>`.

**Prerequisites:** Phases 0-6 complete. We have a static-PIE aarch64 ELF
at `ports/chromium_port/out/BatOs/content_shell` and the BatCave Linux runner
has enough syscall coverage + an Ozone backend to render into our framebuffer.

**Scope of Phase 7:**
1. Decide how the 150 MB blob is stored/shipped with the kernel image.
2. Wire the kernel so that at runtime it can find, relocate, and load the blob.
3. Add a shell command that routes to the BatCave runner.
4. Adjust QEMU memory, frame allocator, and VM ranges.
5. stdout/stderr, cleanup, and debugging.
6. Verifiable milestone list.

This doc is a **spec**. Nothing in it has been merged yet. Line numbers cited
are from `HEAD` of `feat/js-engine-browser-posix` at the time of writing.

---

## 1. The binary size problem

Current kernel image: ~2 MB (`target/aarch64-unknown-none/release/bat_os`).
Static-linked `content_shell`: projected ~150 MB (includes Blink, V8 without
ICU external data, Skia, BoringSSL, SwiftShader if statically bundled).

Naive `include_bytes!()` (the pattern used today in
`src/batcave/linux/runner.rs:9-38` for busybox and the test binaries) would
produce a 152 MB kernel ELF. That has three concrete failure modes:

- **Image load time.** QEMU's `-kernel` loads the image into DRAM before
  jumping to `_start`. A 150 MB image on HVF is ~1-2 seconds; on a real Apple
  Silicon M1 via m1n1 (the target in `linker_apple.ld:12`, load address
  `0x810000000`), it is a real transfer over USB/ADT.
- **Kernel `.rodata` bloat.** `include_bytes!()` emits into `.rodata`.
  `linker.ld:16-18` and `linker_apple.ld:20-22` place `.rodata` right after
  `.text`, so the blob ends up inside the kernel's PC-relative reach. Fine
  today (busybox at 1.2 MB is well inside `adrp` range), but 150 MB breaks
  `.bss` placement — `__bss_start` ends up far past the kernel's natural code
  addressing range and we'd need to rethink relocations.
- **Rust/LLD link time.** LLD ingests the blob as a byte array; the entire
  150 MB goes through `ld.lld` on every `cargo build`. Incremental dev loops
  die.

We evaluated three alternatives.

### Option A — Initrd-style appended blob

Concatenate the blob onto the kernel image after linking, so the physical
byte sequence in DRAM after QEMU/m1n1 load is:

```
[kernel ELF (2 MB)] [8-byte magic] [header] [content_shell ELF (150 MB)] [CRC32]
```

The kernel discovers the blob at runtime by reading a linker-provided
symbol `__kernel_end` (already exposed — see `linker.ld:38` and
`linker_apple.ld:42`). QEMU's `-kernel` loader copies the whole file into
RAM starting at `0x40080000` (per `linker.ld:8`), so the appended bytes land
contiguously after the kernel in physical memory.

**Pros:**
- No filesystem, no network dep, no change to the QEMU command line beyond
  `-m`.
- Symmetric with how Linux kernels handle initramfs — well-understood.
- Cargo builds stay fast: blob is concatenated by a *post-link* script, not
  by the Rust compiler.
- Works identically on QEMU virt and on m1n1-booted Apple Silicon (m1n1
  payload blobs are already appended in the same fashion).

**Cons:**
- The bootloader (or, for QEMU, the `-kernel` loader) must copy the full
  152 MB into RAM even if we don't run Chromium in that session.
- Symbol `__kernel_end` gives us the start of the blob, but we still need a
  length header or a trailer we can walk backward from `__kernel_end + size`.

### Option B — Virtual ROM disk mounted by BatFS

Bake the blob as an encrypted BatFS file at boot, accessed through VFS
(`src/batcave/linux/vfs.rs`). Already used for `batfs::read` in
`src/ui/shell.rs:328`.

**Pros:**
- Integrates with the vault/crypto story: the blob ends up AES-256-CTR
  encrypted at rest (`cmd_write` in shell.rs:305-312).
- Lets us exercise the VFS path Chromium will already use for
  `/opt/chromium/content_shell`.

**Cons:**
- BatFS today is sized for small config files, not 150 MB blobs. `cmd_read`
  allocates a 4096-byte buffer on the stack (`shell.rs:327`); that path is
  not hot enough for a 150 MB binary without a redesign.
- Still needs the 150 MB to live *somewhere* before BatFS mounts it. Either
  a second appended blob (reinventing Option A) or a separate QEMU
  `-drive` file (new bootloader path).
- Encryption/decryption cost on every `chromium` invocation is pure
  overhead for content that is *already* on our kernel image.

### Option C — Fetch from network at boot

Use `net::tls` (already working — see `src/net/tls.rs`) plus the existing
`cmd_fetch` scaffolding (`shell.rs:503-577`) to pull content_shell from a
pinned URL on first run.

**Pros:**
- Kernel image stays tiny.
- Updates are out-of-band; no rebuild to push a new Chromium.

**Cons:**
- Hard dependency on network for the "render google.com" mission. If the
  network doesn't come up, the killer demo is dead.
- Adds a trust boundary: we'd need certificate pinning + signature
  verification of the fetched blob. That's real crypto work on top of
  Phase 7's already-full scope.
- First-boot latency: 150 MB over typical dev networks is 30-120 seconds
  before the shell is usable.
- Conflicts with the project's "zero external dependencies" stance
  (see project vision).

### Recommendation: **Option A**

Initrd-style appended blob. It matches how the rest of Bat_OS already
operates (busybox embedded, test ELFs embedded), avoids the network-trust
rabbit hole, and keeps the dev loop fast because the blob is concatenated
by a shell script after Rust linking — NOT fed through `rustc`/`lld`.

The developer-loop-pain (2 MB kernel vs 152 MB kernel on `cargo build`) is
mitigated by making the bake step opt-in: plain `cargo build --release`
produces the 2 MB kernel; `./tools/bake_chromium.sh` produces the 152 MB
bootable image in a separate output file. See §3.

---

## 2. Recommended loading mechanism — deep dive

### 2.1 On-disk / in-image layout

After `tools/bake_chromium.sh` runs, the final kernel image file is:

```
offset           contents                                  size
--------         --------------------------------------    ----------
0x0              kernel ELF (stripped flat binary)         K bytes
K                MAGIC          ("BATCHROM")               8
K + 8            header version (u32 LE)                   4
K + 12           blob length in bytes (u64 LE)             8
K + 20           blob CRC32 (u32 LE, over blob bytes)      4
K + 24           blob flags (u32 LE; bit0 = compressed)    4
K + 28           reserved                                  4
K + 32           content_shell ELF bytes                   N bytes
K + 32 + N       trailer MAGIC ("CHROMEND")                8
K + 40 + N       total appended length (u64 LE)            8
```

Having both a header (forward-reachable from `__kernel_end`) and a trailer
(reachable by seeking from the end of the loaded image) gives us two
independent ways to validate and locate the blob. If either check fails the
kernel refuses to advertise the `chromium` command.

Choice of `K`: strictly the value of `__kernel_end` at link time, rounded up
to a 4 KB boundary. `linker.ld:33` already aligns `__stack_start` to 4 KB
and `__kernel_end` immediately follows the kernel stack, so in practice
the append offset is aligned.

### 2.2 How the kernel finds the blob

Introduce two new linker-exported symbols in `linker.ld` and
`linker_apple.ld`, next to `__kernel_end`:

```
__chromium_blob_start = __kernel_end;   /* may contain the MAGIC */
__chromium_blob_maxlen = 0xC000000;     /* sanity cap: 192 MB */
```

At runtime, a new module `src/kernel/mm/initrd.rs` exposes:

```rust
pub struct BlobInfo {
    pub base:   usize,  // physical address of content_shell ELF bytes
    pub length: usize,  // N
    pub crc32:  u32,
}

pub fn locate_chromium_blob() -> Option<BlobInfo>;
```

Implementation:
1. Read the 8-byte magic starting at `__chromium_blob_start`. If it is not
   `BATCHROM`, return `None` (kernel was built without bake).
2. Read header fields. Bounds-check `length <= __chromium_blob_maxlen`.
3. Walk to the declared trailer; verify `CHROMEND` magic and the length
   field matches.
4. Compute CRC32 over the blob bytes (can use
   `src/crypto/` existing primitives — CRC is cheap).
5. Return `Some(BlobInfo { base: header_end, length, crc32 })`.

The kernel calls `initrd::locate_chromium_blob()` exactly once during
`main::kernel_main`, stashes the result in a `static OnceCell`-equivalent,
and uses it when the shell command runs.

### 2.3 Runtime memory layout

At kernel start time on QEMU virt, physical layout is:

```
0x40000000  DRAM base (QEMU)
0x40080000  kernel image load address (per linker.ld:8)
0x40080000 + K                                    ┐
            ... content_shell blob ...            │  loaded by QEMU -kernel
0x40080000 + K + 32 + N                           ┘
0x40080000 + K + 40 + N  first byte the frame allocator can hand out
```

The frame allocator (`src/kernel/mm/frame.rs:20-27`) takes `start, end` and
ignores whatever exists inside that range — so step 1 is to have
`kernel_main` call `frame::init(chromium_blob_end_page_aligned, dram_end)`.
That way the frame allocator never stomps the blob.

**Blob stays at its load address.** We do NOT copy the blob to a new buffer
— that would require allocating 150 MB of frames to park a copy we already
have in RAM. The BatCave loader (`src/batcave/linux/loader.rs:119-258`)
accepts `&[u8]`, which is just a fat pointer over the in-place bytes.

**The ELF itself still gets loaded into fresh frames.** `loader::load_elf`
parses PT_LOAD headers and copies segment bytes to freshly allocated,
2 MB-aligned physical frames (`loader.rs:147-155`, `loader.rs:172-186`). The
150 MB blob in DRAM is only the *source* the loader reads from; the
running process's code/data lives in fresh frames.

So during a Chromium session we need roughly:
- 150 MB for the source blob (one-time, DRAM-resident)
- 150 MB for the loaded segments (from the frame allocator)
- 500+ MB working heap (see §5)

Total peak ≈ 800 MB. This is the number that drives the `-m` bump.

### 2.4 Boot sequence

1. QEMU loads `bat_os_chromium.bin` at `0x40080000` in one shot.
2. Kernel `_start` → `.text.boot` → Rust `kernel_main`.
3. `kernel_main` determines `__kernel_end` and:
   - calls `initrd::locate_chromium_blob()`
   - calls `frame::init(blob_end_aligned, 0x40000000 + dram_size)`
4. Shell loop starts. When the user types `chromium <url>`:
   - `shell::execute` dispatches (see §4).
   - The handler calls `loader::load_elf(blob_bytes)`.
   - `loader::execute_with_args(entry, argv)` runs content_shell at EL0.
5. Content_shell's Ozone-headless backend writes pixels into the shared
   framebuffer region (Phase 5 work).
6. Content_shell calls `exit_group`; BatCave's syscall handler tears down
   the MMU mapping and returns to the shell (`loader.rs:376-389`).

### 2.5 On Apple Silicon

`linker_apple.ld:12` puts the kernel at `0x810000000`. m1n1's payload
mechanism already supports appended blobs — the payload is the kernel+blob
concatenation, and m1n1 copies the whole thing to DRAM before jumping.

The only difference from QEMU: instead of `0x40000000` DRAM base, we use
`0x800000000` unified memory (see `linker_apple.ld:11`). The BlobInfo
discovery code is identical because it's driven by `__kernel_end`, which
is computed per-linker-script.

---

## 3. Build system changes

### 3.1 What stays the same

`cargo build --release` continues to produce `target/aarch64-unknown-none/release/bat_os`
as a 2 MB kernel ELF. No Rust code changes for `include_bytes!()` of the
Chromium blob — we specifically do NOT embed it via the compiler.

`build.rs` stays minimal. Today it is two lines (see `build.rs:1-4`). We
add **one** cargo directive so that the linker scripts get a fresh look
whenever they change:

```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rerun-if-changed=linker_apple.ld");
}
```

Rust `build.rs` does NOT run Docker and does NOT produce the blob. Keeping
those concerns separate is critical: a Rust build that requires a 10-hour
Chromium compile is an unshippable developer loop.

### 3.2 The bake step

New file: `tools/bake_chromium.sh` (new directory `tools/` — it does not
exist yet). This is the *only* script responsible for producing the
bootable image.

Behavior:

1. Verify `target/aarch64-unknown-none/release/bat_os` exists (or
   `cargo build --release` first).
2. Extract the raw binary from the ELF:
   `aarch64-none-elf-objcopy -O binary bat_os bat_os.bin`.
3. Verify `ports/chromium_port/out/BatOs/content_shell` exists (error
   clearly if Phase 2 has not been run).
4. Strip `content_shell`: `aarch64-linux-gnu-strip -s` (saves ~30-50 MB
   of debug sections).
5. Optionally run it through `xz -9 --format=raw` (controlled by
   `--compress` flag; only if we wire decompression into the kernel —
   Phase 7 recommends NOT compressing initially to keep the kernel code
   path simple).
6. Write the header struct described in §2.1 to a temp file.
7. Concatenate: `cat bat_os.bin header content_shell trailer > bat_os_chromium.bin`.
8. Verify the resulting file matches the declared layout by re-reading
   header + trailer.
9. Print final size and offsets.

Shell run is roughly:

```sh
#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KERNEL="$ROOT/target/aarch64-unknown-none/release/bat_os"
BLOB="$ROOT/ports/chromium_port/out/BatOs/content_shell"
OUT="$ROOT/target/aarch64-unknown-none/release/bat_os_chromium.bin"

[ -f "$KERNEL" ] || { echo "missing $KERNEL — run cargo build --release"; exit 1; }
[ -f "$BLOB" ]   || { echo "missing $BLOB — run ports/chromium_port/build.sh"; exit 1; }

KBIN="$(mktemp)"
trap 'rm -f "$KBIN" "$KBIN.hdr" "$KBIN.tail"' EXIT

aarch64-linux-gnu-objcopy -O binary "$KERNEL" "$KBIN"
aarch64-linux-gnu-strip -s -o "$KBIN.blob" "$BLOB"

LEN=$(stat -f%z "$KBIN.blob" 2>/dev/null || stat -c%s "$KBIN.blob")
CRC=$(python3 -c "import binascii,sys; print(binascii.crc32(open(sys.argv[1],'rb').read()))" "$KBIN.blob")

python3 tools/bake_header.py "$KBIN.hdr" "$LEN" "$CRC"
python3 tools/bake_trailer.py "$KBIN.tail" "$((LEN + 32))"

cat "$KBIN" "$KBIN.hdr" "$KBIN.blob" "$KBIN.tail" > "$OUT"

echo "baked: $OUT  ($(stat -f%z "$OUT" 2>/dev/null || stat -c%s "$OUT") bytes)"
```

Two small Python helpers (`tools/bake_header.py`, `tools/bake_trailer.py`)
emit the fixed binary structs. Keeping them in Python (not shell) avoids
endianness bugs with `printf`/`xxd` hand-rolled headers.

### 3.3 `run.sh` split

Today `run.sh` always runs `cargo build --release` and boots the plain
kernel. We split into three scripts:

- `run.sh` — existing behavior (2 MB kernel, no Chromium command). Good
  for day-to-day OS work where you don't touch Chromium.
- `run_chromium.sh` — runs `cargo build --release`, then
  `tools/bake_chromium.sh`, then QEMU with the bumped `-m` and the baked
  image as `-kernel`.
- `run.sh` with an env var `BAT_OS_CHROMIUM=1` can also trigger the
  chromium path for CI.

This preserves the inner dev loop: 5-second rebuilds for OS work,
60-second bakes only when a new `content_shell` binary ships.

### 3.4 Developer loop

- Changing Bat_OS code that doesn't touch the Chromium blob:
  `run.sh` (5-10s).
- Changing Bat_OS code that DOES interact with the blob format or the
  BatCave runner, without a new Chromium build:
  `run_chromium.sh` but skip `build.sh` inside — just relink + rebake.
  Rebake is ~2-5 seconds (plain concatenation + CRC).
- Changing Chromium itself:
  Phase 1's `ports/chromium_port/build.sh` inside the Docker container,
  ~10-60 minutes for a full rebuild, minutes for incremental.

---

## 4. Shell command `chromium <url>`

### 4.1 Syntax

```
chromium <url>
chromium --screenshot=<path> <url>
chromium --headless <url>
chromium --size=<WxH> <url>
```

Minimum viable: `chromium <url>` alone. The extended flags map one-to-one
to content_shell flags.

### 4.2 argv routed to content_shell

Default:

```
["content_shell",
 "--headless",
 "--no-sandbox",
 "--disable-gpu",
 "--single-process",
 "--ozone-platform=headless",
 "--enable-logging=stderr",
 "--log-level=0",
 url]
```

All of these are documented content_shell flags and are the combination
used in Phase 1 verification (`CHROMIUM_PORT_PLAN.md:156-162`). Once the
Ozone "batos" backend from Phase 5 lands, swap
`--ozone-platform=headless` → `--ozone-platform=batos`.

### 4.3 Expected behavior

1. `chromium` parses its own argv and constructs the content_shell argv
   above.
2. It resolves the blob (`initrd::chromium_blob()`) and calls
   `loader::load_elf(bytes)`.
3. It calls `loader::execute_with_args(entry, &final_argv)`.
4. content_shell runs until it calls `exit_group` (either via the
   `--screenshot` path auto-exiting once the PNG is written, or via a
   timeout/kill in interactive mode).
5. The shell prints a summary (bytes rendered, pixel dimensions, elapsed
   time) and returns to the prompt.

### 4.4 Replacing the existing alias

`src/ui/shell.rs:123` currently contains:

```rust
"blink" | "chromium" | "chrome" => cmd_run_elf("blink"),
```

This aliases `chromium` to the blink tokenizer test. Phase 7 retires that
alias. Rename the existing alias back to just `"blink"` and add `chromium`
as its own command that takes a URL argument.

### 4.5 Concrete code sketch

New function in `src/ui/shell.rs` (fits alongside `cmd_fetch` at line 503):

```rust
/// `chromium <url>` — launch Chromium content_shell on the given URL.
fn cmd_chromium(url: &str) {
    use crate::batcave::linux::{loader, runner};
    use crate::kernel::mm::initrd;

    if url.is_empty() {
        console::puts("  usage: chromium <url>\n");
        console::puts("         chromium --screenshot=<path> <url>\n");
        return;
    }

    // 1. Confirm the baked blob is present.
    let blob = match initrd::chromium_blob() {
        Some(b) => b,
        None => {
            console::puts("  chromium: image was built without content_shell.\n");
            console::puts("  run tools/bake_chromium.sh and boot the baked image.\n");
            return;
        }
    };

    console::puts("  content_shell: ");
    print_num(blob.length / (1024 * 1024));
    console::puts(" MB, CRC 0x");
    print_hex_u32(blob.crc32);
    console::puts("\n");

    // 2. Boot the default BatCave so caps (net, fs, display) are live.
    ensure_default_cave();

    // Grant display explicitly: Chromium needs the framebuffer region.
    use crate::batcave::cave;
    cave::grant_cap("default", "display").ok();
    cave::grant_cap("default", "net").ok();

    // 3. Wrap blob bytes as a slice (no copy).
    let elf_bytes: &[u8] = unsafe {
        core::slice::from_raw_parts(blob.base as *const u8, blob.length)
    };

    // 4. Load ELF into fresh 2MB-aligned frames.
    let entry = match loader::load_elf(elf_bytes) {
        Ok(e) => e,
        Err(e) => {
            console::puts("  load_elf failed: ");
            console::puts(e);
            console::puts("\n");
            return;
        }
    };

    // 5. Build argv. The first element must be argv[0] ("content_shell"),
    //    not the user's `url`. URL is LAST (content_shell convention).
    let argv: [&str; 9] = [
        "content_shell",
        "--headless",
        "--no-sandbox",
        "--disable-gpu",
        "--single-process",
        "--ozone-platform=headless",
        "--enable-logging=stderr",
        "--log-level=0",
        url,
    ];

    console::puts("  launching content_shell on ");
    console::puts(url);
    console::puts("...\n");
    uart::puts("[chromium] >>> execute_with_args\n");

    let t0 = now_ticks();
    match loader::execute_with_args(entry, &argv) {
        Ok(()) => {
            let dt = now_ticks() - t0;
            console::puts("  chromium exited OK (");
            print_num((dt_to_ms(dt)) as usize);
            console::puts(" ms)\n");
        }
        Err(e) => {
            console::puts("  chromium: ");
            console::puts(e);
            console::puts("\n");
        }
    }

    // 6. Free the loaded segments.
    loader::unload_current(); // new helper — see §8.
}

fn now_ticks() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {}, cntpct_el0", out(reg) v); }
    v
}

fn dt_to_ms(ticks: u64) -> u64 {
    let f: u64;
    unsafe { core::arch::asm!("mrs {}, cntfrq_el0", out(reg) f); }
    (ticks * 1000) / f
}

fn print_hex_u32(v: u32) {
    let hex = b"0123456789abcdef";
    for i in (0..8).rev() {
        console::putc(hex[((v >> (i * 4)) & 0xF) as usize]);
    }
}
```

Dispatcher change in `shell.rs` at the `match command` block
(currently around line 94-146):

```rust
"chromium" | "chrome" => cmd_chromium(parts[1]),
"blink"              => cmd_run_elf("blink"),
```

Note that today's `split_cmd` (`shell.rs:900-923`) supports 4 whitespace-
separated parts — enough for `chromium --screenshot=foo.png <url>` if we
teach `cmd_chromium` to recognize `parts[1].starts_with("--")` and fall
through to `parts[2]` for the URL.

---

## 5. Memory requirements

### 5.1 Current state

`run.sh:20` passes `-m 256M`. On boot, the frame allocator (`frame.rs:8`)
is hard-coded to `MAX_FRAMES = 32768`, i.e. 128 MB manageable.
`cmd_memory` (`shell.rs:187-204`) reports the same.

Two numbers disagree:
- QEMU gives us 256 MB of DRAM from `0x40000000` to `0x50000000`.
- Our frame allocator only tracks the first 128 MB of it.

That's fine today because everything (busybox + all embedded tests) fits in
128 MB. It is NOT fine for Chromium.

### 5.2 What Chromium actually needs

From the Chromium docs and the content_shell `--single-process` profile:

| Component                                       | Steady-state RSS |
|-------------------------------------------------|-----------------|
| V8 heap (isolate + spaces)                      | 100-150 MB      |
| Blink DOM + style + layout trees                | 30-60 MB        |
| Skia paint buffers (1280×1024×4 × 3 surfaces)    | ~15 MB          |
| BoringSSL pools                                 | 5-10 MB         |
| Mojo shared buffers                             | 20-40 MB        |
| Thread stacks (~30 threads × 1 MB)              | ~30 MB          |
| JIT code + icache                               | 40-80 MB        |
| Network buffers + resource cache                | 20-100 MB       |
| Font cache (Skia + HarfBuzz)                    | 20-50 MB        |

Peak ~500 MB on google.com cold load. Add the 150 MB source blob sitting in
DRAM, the 150 MB of loaded PT_LOAD segments, and slack for page tables,
and we need **800 MB-1 GB of physical memory** budgeted, **2 GB of DRAM**
exposed to the guest to have headroom.

### 5.3 QEMU bump

`run_chromium.sh` uses `-m 2G`. `run.sh` unchanged at 256 MB.

### 5.4 Frame allocator scaling

`frame.rs:8` constant `MAX_FRAMES = 32768` is the one line to change. At
4 KB pages, 2 GB needs 524288 frames, meaning the bitmap grows from
512 × `u64` (4 KB) to 8192 × `u64` (64 KB). That's still comfortable in
`.bss`.

Concrete change:

```rust
// src/kernel/mm/frame.rs
const MAX_FRAMES: usize = 524288;           // 2 GB / 4 KB
const BITMAP_SIZE: usize = MAX_FRAMES / 64; // 8192 u64s
```

Two gotchas:
- `alloc_frame` (`frame.rs:29-68`) linearly scans the bitmap. A worst-case
  scan of 8192 × u64 is ~65k memory reads — still microseconds, fine.
- `free_frame` (`frame.rs:70-85`) is O(1), unchanged.

The bitmap lives in `.bss` (statics), which is zeroed by `_start`. Kernel
image size grows by 60 KB of zero-filled BSS — a rounding error.

### 5.5 Fragmentation risk

Content_shell wants contiguous 2 MB-aligned regions for the loader's
`load_elf` (`loader.rs:148-155`). After a long session with many
allocations and frees, fragmentation could prevent a second
`chromium` invocation from finding a 2 MB-aligned run. Mitigation:
make `loader::unload_current` zero *and* `free_frame` every page
(see §8), and have `cmd_chromium` optionally trigger a
"defragment" (currently a no-op because we don't have one).

Phase 7 does not solve fragmentation. We accept that after 5-10 Chromium
runs, a reboot may be required. File a follow-up.

---

## 6. Process isolation

### 6.1 What Chromium does even in single-process mode

`--single-process --no-zygote` means no fork/exec and no Mojo IPC between
renderer/browser. It does NOT mean single-threaded. Content_shell still
spins up:

- Compositor thread
- IO thread
- ThreadPool workers (4-16)
- V8 GC thread, concurrent marker
- Audio thread (disabled with our flags, but allocated in startup code)
- Font service thread
- DNS resolver thread

Roughly 20-30 threads.

### 6.2 BatCave's current MMU model

`src/batcave/linux/mmu.rs:53-95` sets up **one** page table per BatCave,
mapping:
- 10 × 2 MB blocks for the cave's busybox (line 72-76, up to ~20 MB)
- MMIO regions (line 79-81)
- 128 × 2 MB of kernel RAM at `0x40000000` (line 84-87)

Every thread in a cave today **shares the same page table** because
`switch_to_cave` (line 99+) just changes `TTBR0_EL1`. Chromium's threads
all living in one address space is actually the natural fit — they're
just kernel tasks pointing at the same L1.

### 6.3 What needs to change

Three concrete items:

1. **Grow the cave's mapped virtual range.** 10 blocks × 2 MB = 20 MB of
   code/data. content_shell text+data+bss after relocation is ~150 MB.
   Raise the loop at `mmu.rs:72-76` from `0..10` to something like
   `0..100` (200 MB of 2 MB blocks), gated by a new argument to
   `setup_cave_pagetable`.

2. **Allow heap growth.** Chromium calls `brk` and `mmap(MAP_ANONYMOUS)`.
   Today the BatCave loader doesn't grow the heap beyond what's mapped
   upfront. Phase 4 should have added demand-paging or at least an
   `mmap`/`brk` backed by fresh frames — confirm this is wired to use
   the larger bitmap from §5.

3. **Stack-per-thread.** `loader::execute_with_args` allocates one 1 MB
   stack (`loader.rs:268-270`) for the initial thread. Every subsequent
   thread (via Phase 4's `clone()` implementation) needs its own stack.
   The cave's L2 must map each thread stack. Recommend a contiguous
   "thread stack pool" region, e.g. 64 MB at virtual
   `0x40000000` + 200 MB, split into 64 × 1 MB slots.

### 6.4 TLB pressure

A page table with 200 MB of 2 MB blocks plus kernel identity plus MMIO is
still comfortably within the L2 table. No need to promote to L3 4 KB
granularity. TLB flushes happen on cave switch (`mmu.rs:100`
`tlbi vmalle1`) — fine.

---

## 7. stdout/stderr routing

### 7.1 Current chain

`uart::puts` (`src/drivers/uart.rs`) is the kernel's UART writer. The
BatCave `write` syscall is wired into this path (see
`src/batcave/linux/syscalls.rs` — write to fd 1 or 2 → uart::puts).
QEMU is launched with `-serial stdio` (`run.sh:25`), so host terminal
receives everything.

### 7.2 What Chromium emits

`--enable-logging=stderr --log-level=0` produces:
- Mojo startup (~50 lines)
- V8 bootstrap (~30 lines)
- Network requests (1-5 lines per request; google.com cold-load is ~40 requests)
- Compositor frame logs
- Blink style/layout/paint traces

Order of magnitude: **10-50 MB of stderr text for a single page load**.

### 7.3 Problem

`uart::puts` writes one byte at a time to the UART MMIO. The pl011 (QEMU
virt) runs at 115200 baud = ~11 KB/s sustained. 10 MB of logs = 15
minutes. Worse, the UART write is *synchronous* in BatCave's write path —
content_shell stalls waiting on UART drain. This will make the page
feel broken.

### 7.4 Mitigation

Two layers:

1. **Raise the default log level.** `--log-level=2` (WARN) trims stderr
   from ~50 MB to ~200 KB per page load. Make `--log-level=0` opt-in via
   a `--verbose` flag to `chromium`.

2. **Add a ring buffer between the syscall write and the UART.** New
   file `src/batcave/linux/stdio_ring.rs`:
   - 256 KB circular buffer in `.bss`.
   - `write(fd=1|2, buf, len)` appends to the ring, returns immediately.
   - A kernel "stdio drain" hook runs on every shell idle tick (the
     `core::hint::spin_loop()` in `shell.rs:76`) and drains bytes to
     `uart::puts` until either the ring is empty or the UART would
     block.
   - On ring overflow, drop bytes and bump a counter; print "[stdio: N
     bytes dropped]" on drain. Chromium is NOT silenced — just
     back-pressured by kernel scheduling.

3. **A separate ring for a "capture" file** when `--screenshot` is used
   so post-mortem debugging doesn't require scrolling hundreds of MB
   of serial output.

### 7.5 Volume sanity check

At 2 GB of DRAM and 256 KB ring buffer, overflow is cheap. The ring is
not the durability layer — the UART is.

---

## 8. Exit + cleanup

### 8.1 How content_shell exits

Two normal paths:
- `--screenshot=<path>` — renders, writes PNG, calls `exit_group(0)`.
- Interactive/`--headless` with no screenshot — runs until the
  JavaScript on the page calls `window.close()` or the process is
  signaled. For our first target (`chromium <url>`), we rely on
  `--virtual-time-budget=5000` (one of the flags not yet in the argv
  above but we should add) to force exit after 5s of virtual time, OR
  follow Stage 1's approach and always pass `--screenshot=/tmp/out.png`
  under the hood.

### 8.2 BatCave reap path

`loader::execute_with_args` (`loader.rs:264-393`) returns to the shell
after `exit_group` via the hardcoded SP restore at `loader.rs:378-386`.
At that point:

- TTBR0_EL1 has been flipped back to the kernel table.
- The cave's page table still exists in physical memory.
- The loaded content_shell segments still occupy their 2 MB-aligned
  frames.
- Thread stacks (1 per thread) still occupy their frames.
- `TLS` page still mapped.
- Any `mmap` regions created via Phase 4 syscalls still occupy frames.

None of these are automatically freed today. For busybox's 1.2 MB
working set, nobody notices. For Chromium's 500+ MB, leaking a session
means the next `chromium` invocation OOMs.

### 8.3 Required work

New function `loader::unload_current()`:

```rust
pub fn unload_current() -> Result<(), &'static str> {
    // 1. Walk L2 block entries in the cave's page table. For each
    //    mapped-to-cave block (NOT kernel identity, NOT MMIO), free
    //    the backing frame(s).
    // 2. Free thread stacks (track in a per-cave stack list).
    // 3. Free any mmap regions (track in a per-cave mmap list).
    // 4. Zero + free TLS pages.
    // 5. Free the L1/L2 page-table pages themselves.
    // 6. Clear LOADED_ENTRY / LOADED_PHYS_BASE atomics.
    Ok(())
}
```

For this to be correct, Phase 4's `mmap` and `clone` implementations
must record their allocations in a per-cave list that `unload_current`
can walk.

### 8.4 Cleanup correctness test

After `chromium https://example.com` completes, run `mem` in the shell.
Before-vs-after `used` frame count should differ by < 50 pages
(accounting for shell state). If it differs by thousands, we are
leaking.

### 8.5 Crash path

If content_shell panics (hits a not-yet-implemented syscall, SIGSEGVs,
etc.), the BatCave exception handler should still call
`loader::unload_current` before returning control to the shell. Today
the exception path at `loader.rs:378` does nothing but restore SP —
extend it.

---

## 9. Debugging aids

### 9.1 Logging chain

End-to-end, a log line from Blink reaches the human as:
1. `LOG(INFO) << ...` in Chromium C++.
2. Chromium's logging writes to fd 2 via `writev`.
3. BatCave write syscall appends to `stdio_ring` (§7.4).
4. Shell's idle tick drains the ring to `uart::puts`.
5. pl011 MMIO → QEMU `-serial stdio` → host terminal.

Every hop gets its own prefix so we can trace failures:
- Chromium's native prefix (`[123:456:0416/...]`)
- BatCave write handler adds `[cs/out]` or `[cs/err]`
- The shell adds nothing (raw pass-through)

### 9.2 Screenshot capture

Always pass `--screenshot=/batfs/chromium_last.png` under the hood.
After exit, the shell reads back the PNG via `batfs::read` and:
- Prints SHA-256 + size + pixel dimensions.
- Blits the image to the framebuffer region if running inside the WM.
- Leaves the file in BatFS so `cat chromium_last.png` (hex dump) or a
  host-side `qemu-img` export can grab it for pixel diffing.

**Pixel diff is the acceptance criterion.** We compare our rendered PNG
to a reference rendered by the same content_shell build on plain Linux;
byte-identical is the goal (same build, same font cache, same
SwiftShader → deterministic).

### 9.3 Boot-time diagnostics

`kernel_main`, after blob discovery:

```
[bat_os] kernel end: 0x40280000
[initrd] chromium blob: 0x40280000 .. 0x4A280000 (150 MB, crc 0xDEADBEEF)
[frame] allocator: 0x4A280000 .. 0xC0000000 (1.6 GB usable)
```

On `chromium <url>`:

```
[chromium] content_shell 150 MB, CRC 0xdeadbeef
[chromium] launching content_shell on https://www.google.com...
[cs/out] Starting content_shell
[cs/err] [123:0001] Ozone: headless backend up
[cs/err] [123:0002] Network: resolving www.google.com
[cs/err] [123:0003] Network: TLS handshake OK
[cs/err] [123:0004] Loaded 142 KB of HTML
[cs/err] [123:0005] First paint: 1280x1024
[chromium] exited OK (4217 ms)
[chromium] captured chromium_last.png — 1280x1024, 384 KB
```

These lines are what we grep in CI.

### 9.4 Post-mortem on failure

If content_shell doesn't reach "First paint":
1. Check the last `[cs/err]` — it almost always names a missing
   syscall number or a syscall with an unsupported flag.
2. Check `mem` output — did we OOM during startup?
3. Check `fw` stats (`shell.rs:489-501`) — did the firewall drop
   something?

---

## 10. Milestone checklist

Each step is verifiable and must pass before the next begins.

1. **`tools/bake_chromium.sh` exists and runs** on a machine with both
   the built kernel and a dummy 1 MB stand-in file. Output file is
   correctly laid out per §2.1. Verify by `xxd | head` that the
   `BATCHROM` magic is at `__kernel_end` and `CHROMEND` at the tail.

2. **Kernel locates the blob.** `src/kernel/mm/initrd.rs` exists and
   `kernel_main` prints `[initrd] chromium blob: ...` with valid
   base/length/CRC when booted on the baked image. CRC must match the
   bake-time value.

3. **Kernel refuses corrupt blobs.** Flip a bit in the baked image, boot
   — kernel prints `[initrd] CRC mismatch, chromium disabled` and the
   `chromium` command reports "not available".

4. **Frame allocator scales.** With `-m 2G` and `MAX_FRAMES = 524288`,
   `mem` reports ~524000 free frames after boot. Allocate and free
   100000 frames in a stress test — no corruption.

5. **`chromium` command is wired.** `shell.rs` dispatch hits
   `cmd_chromium`, which prints usage when called with no URL. Old
   `"chromium" → cmd_run_elf("blink")` alias is removed.

6. **content_shell loads.** `cmd_chromium` calls `loader::load_elf` on
   the blob and gets back a non-zero entry. Boot log shows the loader's
   `[loader] Applied N R_RELATIVE` with N ≥ 10000 (real relocations).

7. **content_shell starts executing.** After `execute_with_args`, we see
   at least one `[cs/out]` or `[cs/err]` line indicating the Chromium
   binary reached user code. (If it dies in `_start` before printing,
   static-PIE TLS setup is broken — back to Phase 2.)

8. **exit_group returns to shell.** content_shell with `--version` flag
   (touches minimal syscalls) exits cleanly and `chromium` command
   returns to the prompt.

9. **`unload_current` frees ~everything.** Before/after `mem` diff after
   `chromium --version` is < 50 pages. Run 10 iterations — free count
   is stable.

10. **Headless renders `data:text/html,<h1>hi</h1>`.**
    `chromium --screenshot=/batfs/hi.png "data:text/html,<h1>hi</h1>"`
    produces a PNG. `cat hi.png | wc -c` ≥ 500 bytes (PNG header +
    non-empty content).

11. **PNG is a valid render.** Extract `hi.png` via `batfs` export,
    open on host, visually confirm "hi" is rendered.

12. **Stderr ring buffer holds under load.** With `--log-level=0` on a
    non-trivial data URI, shell stays responsive (prompt returns within
    a few seconds post-exit). No ring overflow on pages under 1 MB of
    logs.

13. **HTTPS loads render.** `chromium https://example.com` writes a
    PNG that, when diffed against reference Chrome on host, has < 1%
    pixel difference.

14. **Memory leaks bounded over 5 back-to-back runs.**
    `chromium https://example.com` run 5 times. `mem` delta < 200
    pages total.

15. **Final mission: `chromium https://www.google.com`.** PNG written,
    pixel-diff vs reference < 2%, no kernel panics, shell returns to
    prompt. Mark Phase 7 complete.

---

## 11. Risks + mitigations

| # | Risk                                                                                             | Likelihood | Impact | Mitigation                                                                                                                       |
|---|--------------------------------------------------------------------------------------------------|-----------|-------|---------------------------------------------------------------------------------------------------------------------------------|
| 1 | QEMU `-kernel` refuses the 152 MB image or truncates it silently.                                | Medium    | High  | Verify with `ls -la` on the baked image and a boot-time re-CRC of the blob. If QEMU caps, switch to `-device loader,file=` or `-initrd`. |
| 2 | `loader::load_elf` fails on a real-world 150 MB ELF (our ELF parser only handles small binaries). | High      | High  | The parser in `loader.rs:119-258` has no hardcoded size limits, but the relocation loop (`loader.rs:214-232`) touches each byte — walk the full `RELA` table on busybox first as a smoke test. Phase 4 should have stretched it. |
| 3 | Fragmentation after multiple runs prevents 2 MB-aligned allocations (see `loader.rs:148-155`).   | High      | Medium| `unload_current` must return frames in LIFO order; reject allocations only after a compaction pass. File a follow-up for a real buddy allocator. |
| 4 | `uart::puts` stalls the kernel when Chromium floods stderr (§7.3).                               | High      | Medium| Ring buffer (§7.4) non-optional before step 10 of §10.                                                                           |
| 5 | Blob append causes the kernel's PC-relative addressing to break (see §1).                        | Low       | Critical | Bake the blob AFTER linking; never let LLD see it. Verify `readelf -l bat_os.bin` before bake — sizes must match the kernel-only build. |
| 6 | MMU mapping the blob area as executable accidentally lets content_shell jump into the raw source bytes (not the relocated copy). | Medium    | High  | Map the blob region R-only in the cave page table. The L2 block flag `PTE_AP_RW` at `mmu.rs:28` needs a new `PTE_AP_RO` variant. |
| 7 | Single-process Chromium still spawns threads that expect `futex`/`clone` we don't fully support.  | High      | High  | Phase 4 must be green before Phase 7 milestone 7; if it isn't, spawn a follow-up instead of papering over with stubs.           |
| 8 | Page tables run out of space with 200 MB of 2 MB blocks + thread stacks.                         | Low       | Medium| 200 MB / 2 MB = 100 entries in one L2 table (512 entries max). Fine. If thread stacks push this past 512, promote to two L2s.   |
| 9 | PNG capture path writes through BatFS and BatFS is size-limited (§1 Option B concerns).           | Medium    | Low   | Add a BatFS configuration knob to allow one large file, or bypass BatFS for the screenshot (write directly to a reserved DRAM region and expose via QEMU `pflash`). |
| 10| The bake step on macOS hosts fails because `aarch64-linux-gnu-strip` isn't installed.             | High      | Low   | Document the brew/apt package. Fall back to `llvm-strip --strip-all` which ships with Apple's Xcode toolchain.                  |
| 11| `cargo build` times out in CI because someone accidentally added `include_bytes!("content_shell")`. | Low       | Low   | CI asserts kernel image size < 10 MB before bake. Large include gets caught in the build step.                                  |
| 12| Apple Silicon path diverges: m1n1 loads the payload to a different address than our `linker_apple.ld` assumes (see `linker_apple.ld:12`). | Medium | High  | Test on m1n1 after QEMU works. m1n1 can be told the load address; document it alongside Phase 8.                                |
| 13| `--single-process` disables a sandbox that content_shell assumes is there; side effect is security-irrelevant for v1 but may enable code paths that crash. | Medium | Medium| Live with the crashes in v1, fix in Phase 8. File any hit.                                                                      |
| 14| Pixel-diff target (< 2% for google.com) is unachievable because Google serves personalized content. | Medium    | Low   | Use `https://www.google.com/?hl=en&pws=0&gws_rd=cr&gl=us` and pin a reference capture. Accept that this is a best-effort target.|
| 15| CRC32 collision makes corruption undetected.                                                     | Very low  | Medium| Fine for v1. If it ever bites, upgrade to SHA-256 — we already have it (`batfs::verify`).                                       |

---

## Appendix A — Files created / modified in Phase 7

New:
- `tools/bake_chromium.sh`
- `tools/bake_header.py`
- `tools/bake_trailer.py`
- `src/kernel/mm/initrd.rs`
- `src/batcave/linux/stdio_ring.rs`
- `run_chromium.sh`
- `ports/chromium_port/PHASE7_BOOT_INTEGRATION.md` (this file)

Modified:
- `src/ui/shell.rs` — add `cmd_chromium`, drop `"chromium" → blink` alias.
- `src/batcave/linux/loader.rs` — add `unload_current`.
- `src/batcave/linux/mmu.rs` — widen cave mapping from 20 MB to 200 MB;
  add R-only block flags for the source blob region.
- `src/kernel/mm/frame.rs` — `MAX_FRAMES` 32768 → 524288.
- `linker.ld` — add `__chromium_blob_start` = `__kernel_end`.
- `linker_apple.ld` — same.
- `build.rs` — add `rerun-if-changed` for linker scripts.
- `run.sh` — unchanged. (New `run_chromium.sh` handles the bake path.)

---

## Appendix B — Memory budget table

| Region                                     | Size     | Residency                                          |
|--------------------------------------------|----------|----------------------------------------------------|
| Kernel image (+ stack + bss)               | ~4 MB    | Permanent                                          |
| Appended content_shell blob (source bytes) | 150 MB   | Permanent while image is booted                    |
| content_shell loaded segments              | 150 MB   | During a `chromium` session only                   |
| V8 + Blink + Skia working heap             | 400 MB   | During a `chromium` session only                   |
| Thread stacks (~30 × 1 MB)                 | 30 MB    | During a `chromium` session only                   |
| Page tables (cave L1 + L2s)                | ~64 KB   | During a `chromium` session only                   |
| stdio_ring buffer                          | 256 KB   | Permanent                                          |
| Frame allocator bitmap                     | 64 KB    | Permanent                                          |
| **Peak**                                   | ~735 MB  | During a session                                   |
| **Headroom on `-m 2G`**                    | ~1.3 GB  | Sufficient for GC spikes, cache, defrag            |

---

## Appendix C — Byte-level verification of a baked image

```
$ ls -la target/aarch64-unknown-none/release/bat_os_chromium.bin
-rwxr-xr-x  152334912

$ python3 -c "
import struct
data = open('target/aarch64-unknown-none/release/bat_os_chromium.bin','rb').read()
kernel_end = 0x200000  # from readelf -s bat_os | grep __kernel_end minus 0x40080000
assert data[kernel_end:kernel_end+8] == b'BATCHROM'
ver, length, crc, flags = struct.unpack_from('<IQII', data, kernel_end+8)
print(f'version={ver} length={length} crc={crc:#x} flags={flags:#x}')
trailer_off = kernel_end + 32 + length
assert data[trailer_off:trailer_off+8] == b'CHROMEND'
print('layout OK')
"
version=1 length=157286400 crc=0xdeadbeef flags=0x0
layout OK
```

---

End of Phase 7 design.

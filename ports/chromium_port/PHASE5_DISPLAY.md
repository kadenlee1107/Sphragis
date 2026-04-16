# Phase 5 — Display Backend: Detailed Engineering Spec

**Scope.** How pixels produced by Chromium `content_shell` reach the Bat_OS
framebuffer. Covers the shared-memory bridge, the Chromium-side patch, the
Bat_OS-side blit loop, GPU flags, debugging plan, and fallbacks.

**Input to this doc:**
- `CHROMIUM_PORT_PLAN.md` § Phase 5 (two-path sketch, one-paragraph each)
- `src/drivers/virtio/gpu.rs` (our 1280x1024 BGRA8888 virtio-gpu framebuffer)
- `src/browser/paint/mod.rs` (how our Rust browser currently blits — same
  model works here)

**Output:** an implementable spec. Every file path, Chromium class, and Ozone
API call in this doc is load-bearing. If you change one, check the cross-refs.

---

## Table of Contents

1. Architecture overview
2. Path A vs Path B — decision & rationale
3. Path A implementation — Chromium-side patch
4. Shared memory mechanism
5. Framebuffer blit loop (Bat_OS side)
6. GPU strategy (SwiftShader vs disable)
7. Debugging plan & first-visible milestone
8. Risks & fallbacks
9. Concrete file list (every file, one line each)
10. ETA and parallelism

---

## 1. Architecture Overview

### 1.1 How content_shell produces pixels today (software path)

With `--disable-gpu` and Ozone `headless`, the pixel path looks like this
(class names are all from upstream Chromium M132; paths are `//src/...`):

```
Blink paints layers
  └─ cc::LayerTreeHostImpl::DrawLayers
       └─ viz::SoftwareRenderer::DrawQuad   (software compositor)
            └─ draws into an SkCanvas backed by an SkBitmap
                 └─ The SkBitmap lives in a shared-memory segment
                    allocated by viz::SoftwareOutputDevice
                       └─ SoftwareOutputDeviceHeadless::EndPaint
                          copies / presents the frame
```

Key classes/files (upstream Chromium):

| Role                           | File                                                                              | Class                                |
|--------------------------------|-----------------------------------------------------------------------------------|--------------------------------------|
| Ozone platform registration    | `ui/ozone/platform/headless/ozone_platform_headless.cc`                            | `OzonePlatformHeadless`              |
| Headless surface factory       | `ui/ozone/platform/headless/headless_surface_factory.cc`                           | `HeadlessSurfaceFactory`             |
| Software output device (hl)    | `ui/ozone/platform/headless/headless_surface_factory.cc` (nested class)            | `SoftwareOutputDeviceHeadless`       |
| Base software output device    | `components/viz/service/display_embedder/software_output_device.h`                 | `viz::SoftwareOutputDevice`          |
| Software output surface        | `components/viz/service/display_embedder/software_output_surface.cc`               | `viz::SoftwareOutputSurface`         |
| Software renderer              | `components/viz/service/display/software_renderer.cc`                              | `viz::SoftwareRenderer`              |
| Headless window                | `ui/ozone/platform/headless/headless_window.cc`                                    | `HeadlessWindow`                     |

The **SkBitmap** in `SoftwareOutputDeviceHeadless` is the pixel buffer we care
about. It is `kBGRA_8888_SkColorType`, premultiplied alpha, row-major,
stride = width*4, top-left origin. This is — fortuitously — the exact format
our virtio-gpu scanout expects (`FORMAT_B8G8R8A8`, see `gpu.rs:23`).

### 1.2 How our framebuffer accepts pixels

`src/drivers/virtio/gpu.rs`:
- Allocates `1280 * 1024 * 4 = 5_242_880` bytes of contiguous frames
- Registers the region with the host via `VIRTIO_GPU_CMD_RESOURCE_ATTACH_BACKING`
- `SET_SCANOUT` binds it to scanout 0 (the display)
- Per-frame: `flush(x, y, w, h)` issues `TRANSFER_TO_HOST_2D` + `RESOURCE_FLUSH`

The raw buffer pointer is `gpu::framebuffer() -> *mut u32`, 32-bit pixels in
`0xAARRGGBB` little-endian (i.e. memory order B,G,R,A — same as Chromium's
BGRA8888).

### 1.3 The bridge

```
  ┌──────────────────────────┐       ┌──────────────────────────┐
  │ content_shell (EL0 ELF)  │       │ Bat_OS kernel            │
  │                          │       │                          │
  │ SoftwareOutputDevice ─┐  │       │                          │
  │   (patched)           │  │       │  chromium_blit kthread   │
  │                       ▼  │       │    │                     │
  │ SkBitmap pixels ─► SHM ──┼──────►│──► Copy/convert to fb ───┼──► virtio-gpu scanout
  │                          │       │    │                     │
  │ Post-EndPaint: set       │       │    │ Poll shm.seq for    │
  │   shm->seq++ (release)   │       │    │ new frames, then    │
  │                          │       │    │ gpu::flush()        │
  └──────────────────────────┘       └──────────────────────────┘
```

The shared memory segment is a single fixed region containing:
- A small header (w, h, stride, seq, pts, damage rect)
- The pixel bytes

Single writer (content_shell compositor thread), single reader (Bat_OS blit
kthread). Sequence-number handshake — no locks needed.

---

## 2. Path A vs Path B — Decision

**Path A (hack headless):** monkey-patch `SoftwareOutputDeviceHeadless` to
write pixels into our shared region instead of keeping them in-process.

**Path B (real platform):** add `ui/ozone/platform/batos/` — `OzonePlatformBatos`,
`BatosWindow`, `BatosSurfaceFactory`, `BatosSoftwareOutputDevice`,
`BatosGpuPlatformSupportHost` stubs. Register via
`ui/ozone/platform_object.h` and `ui/ozone/BUILD.gn`.

### Decision: **Start with Path A. Migrate to Path B in a later milestone.**

**Justification:**

1. Path A is ~1 patched file + ~50 lines of C++. Path B is 6-8 files and
   ~1500 lines of C++ + GN wiring.
2. Path A is enough for the Stage 6-8 success criteria in the overall plan
   (render HTML, render example.com, render google.com). Input is not
   required for Stage 8.
3. Path B's main upside is clean input dispatch (`PlatformEventDispatcher`,
   `KeyEvent`, `MouseEvent` wired through `ui/events/ozone/`). We don't have
   keyboard/mouse routed into BatCave yet (Phase 8 stretch), so Path B
   earns us nothing Stage-8-critical.
4. Once everything else is working, the switch to Path B is additive, not
   destructive — the shared-memory contract defined in §4 is reusable
   verbatim; only the *source* of pixels in the Chromium process changes.

**Cost of the switchover later (Path A → Path B):** Estimated **10-14 days**.

| Task                                                      | Days |
|-----------------------------------------------------------|------|
| Scaffold `ui/ozone/platform/batos/` with 6 classes        | 3    |
| Register platform in `ozone.gni` + `BUILD.gn`, add tests  | 1    |
| Port the SHM writer from Path A patch into `Batos` device | 1    |
| Wire `BatosWindow` geometry, `AcceleratedWidget` ids      | 2    |
| Input: `PlatformEventSource` + dispatcher for keys/mouse  | 3    |
| Event route Bat_OS ← PS/2/virtio-keyboard ← IPC ← Chromium| 2    |
| Rebuild, re-verify every stage                            | 2    |

The input half is the expensive bit; pure display migration is ~5 days.

---

## 3. Path A Implementation

### 3.1 Where to patch

**One file, one class:**

- `ui/ozone/platform/headless/headless_surface_factory.cc`
- `ui/ozone/platform/headless/headless_surface_factory.h` (tiny addition)

Specifically the `SoftwareOutputDeviceHeadless` class inside
`headless_surface_factory.cc`. In M132 it looks approximately like:

```cpp
class SoftwareOutputDeviceHeadless : public viz::SoftwareOutputDevice {
 public:
  explicit SoftwareOutputDeviceHeadless() = default;
  ~SoftwareOutputDeviceHeadless() override = default;

  void Resize(const gfx::Size& viewport_pixel_size, float) override {
    viewport_pixel_size_ = viewport_pixel_size;
    SkImageInfo info = SkImageInfo::MakeN32Premul(
        viewport_pixel_size_.width(), viewport_pixel_size_.height());
    surface_ = SkSurfaces::Raster(info);
  }

  SkCanvas* BeginPaint(const gfx::Rect& damage_rect) override {
    damage_rect_ = damage_rect;
    return surface_ ? surface_->getCanvas() : nullptr;
  }

  void EndPaint() override {
    // Default headless: drop the pixels on the floor.
  }
 private:
  gfx::Size viewport_pixel_size_;
  gfx::Rect damage_rect_;
  sk_sp<SkSurface> surface_;
};
```

### 3.2 What to add

Two changes:

1. **At `Resize`**, instead of a heap-backed `SkSurface`, create one whose
   pixels live in our shared memory region.
2. **At `EndPaint`**, publish: write the damage rect + sequence number into
   the shm header so the reader knows a new frame is ready.

### 3.3 Diff-level specificity

File: `ui/ozone/platform/headless/headless_surface_factory.cc`

```diff
 #include "ui/ozone/platform/headless/headless_surface_factory.h"

+#include <fcntl.h>
+#include <sys/mman.h>
+#include <sys/stat.h>
+#include <unistd.h>
+#include <atomic>
+#include <cstdio>
+#include <cstdlib>
+#include <cstring>
+
 #include "base/files/file_util.h"
 ...

+namespace {
+
+// Bat_OS ↔ content_shell shared-memory display contract. Keep in sync with
+// src/drivers/display/chromium_blit.rs.
+struct BatosFbHeader {
+  uint32_t magic;        // 'BFB1' = 0x42464231
+  uint32_t version;      // 1
+  uint32_t width;
+  uint32_t height;
+  uint32_t stride;       // bytes per row
+  uint32_t format;       // 1 = BGRA8888 premultiplied
+  std::atomic<uint32_t> seq;         // incremented after each frame
+  std::atomic<uint32_t> damage_x;
+  std::atomic<uint32_t> damage_y;
+  std::atomic<uint32_t> damage_w;
+  std::atomic<uint32_t> damage_h;
+  uint64_t pts_ns;       // monotonic ns from content_shell clock
+  uint32_t reserved[8];
+};
+static_assert(sizeof(BatosFbHeader) <= 128, "fb header drift");
+
+constexpr size_t kBatosFbHeaderSize = 128;
+constexpr size_t kBatosFbMaxPixelBytes = 1280 * 1024 * 4;  // 5 MiB
+constexpr size_t kBatosFbRegionSize = kBatosFbHeaderSize + kBatosFbMaxPixelBytes;
+constexpr const char* kBatosFbPath = "/batos/fb0";  // BatCave well-known path
+
+struct BatosFbMapping {
+  void* base = nullptr;
+  BatosFbHeader* hdr = nullptr;
+  uint8_t* pixels = nullptr;
+  bool valid = false;
+};
+
+BatosFbMapping MapBatosFb(int width, int height) {
+  BatosFbMapping m;
+  int fd = ::open(kBatosFbPath, O_RDWR | O_CREAT, 0600);
+  if (fd < 0) {
+    std::fprintf(stderr, "[batos-fb] open failed: errno=%d\n", errno);
+    return m;
+  }
+  if (::ftruncate(fd, kBatosFbRegionSize) != 0) {
+    std::fprintf(stderr, "[batos-fb] ftruncate failed\n");
+    ::close(fd);
+    return m;
+  }
+  void* p = ::mmap(nullptr, kBatosFbRegionSize, PROT_READ | PROT_WRITE,
+                   MAP_SHARED, fd, 0);
+  ::close(fd);
+  if (p == MAP_FAILED) {
+    std::fprintf(stderr, "[batos-fb] mmap failed\n");
+    return m;
+  }
+  m.base = p;
+  m.hdr = reinterpret_cast<BatosFbHeader*>(p);
+  m.pixels = reinterpret_cast<uint8_t*>(p) + kBatosFbHeaderSize;
+
+  if (m.hdr->magic != 0x42464231u) {
+    std::memset(m.hdr, 0, sizeof(BatosFbHeader));
+    m.hdr->magic = 0x42464231u;
+    m.hdr->version = 1;
+    m.hdr->format = 1;
+  }
+  m.hdr->width = width;
+  m.hdr->height = height;
+  m.hdr->stride = width * 4;
+  m.valid = true;
+  return m;
+}
+
+}  // namespace
+
 class SoftwareOutputDeviceHeadless : public viz::SoftwareOutputDevice {
  public:
-  explicit SoftwareOutputDeviceHeadless() = default;
+  explicit SoftwareOutputDeviceHeadless() = default;
   ~SoftwareOutputDeviceHeadless() override = default;

   void Resize(const gfx::Size& viewport_pixel_size, float) override {
     viewport_pixel_size_ = viewport_pixel_size;
-    SkImageInfo info = SkImageInfo::MakeN32Premul(
-        viewport_pixel_size_.width(), viewport_pixel_size_.height());
-    surface_ = SkSurfaces::Raster(info);
+    const int w = viewport_pixel_size_.width();
+    const int h = viewport_pixel_size_.height();
+    fb_ = MapBatosFb(w, h);
+    if (!fb_.valid) {
+      // Fall back to old behavior so --screenshot still works if SHM fails.
+      SkImageInfo info = SkImageInfo::MakeN32Premul(w, h);
+      surface_ = SkSurfaces::Raster(info);
+      return;
+    }
+    SkImageInfo info = SkImageInfo::Make(
+        w, h, kBGRA_8888_SkColorType, kPremul_SkAlphaType);
+    surface_ = SkSurfaces::WrapPixels(info, fb_.pixels, fb_.hdr->stride);
   }

   SkCanvas* BeginPaint(const gfx::Rect& damage_rect) override {
     damage_rect_ = damage_rect;
     return surface_ ? surface_->getCanvas() : nullptr;
   }

   void EndPaint() override {
-    // Default headless: drop the pixels on the floor.
+    if (!fb_.valid || !fb_.hdr) return;
+    fb_.hdr->damage_x.store(damage_rect_.x(), std::memory_order_relaxed);
+    fb_.hdr->damage_y.store(damage_rect_.y(), std::memory_order_relaxed);
+    fb_.hdr->damage_w.store(damage_rect_.width(), std::memory_order_relaxed);
+    fb_.hdr->damage_h.store(damage_rect_.height(), std::memory_order_relaxed);
+    // release ordering: pixel stores must be visible before seq bump.
+    fb_.hdr->seq.fetch_add(1, std::memory_order_release);
   }

  private:
   gfx::Size viewport_pixel_size_;
   gfx::Rect damage_rect_;
   sk_sp<SkSurface> surface_;
+  BatosFbMapping fb_;
 };
```

### 3.4 Why this is the right hook

- `SoftwareOutputDevice::EndPaint` is the contract point at which Chromium
  guarantees the canvas is flushed (see `SoftwareOutputSurface::SwapBuffers`
  in `components/viz/service/display_embedder/software_output_surface.cc`).
- `SkSurfaces::WrapPixels` binds the `SkSurface` directly to our memory —
  no copy happens inside Chromium. Blink paints, layers composite, and the
  final pixel stores land directly in the shared region.
- Damage rect is already computed by `viz::SoftwareRenderer`; we just
  forward it. This lets the blit loop only push the dirty region over
  virtio-gpu, which matters on slow emulated transports.

### 3.5 What the patch does NOT change

- No change to Mojo, no change to gpu/ process boundaries (single-process).
- No change to SwiftShader — this is the 2D software path, not the GL one.
  GL path (SwiftShader ICD) is covered in §6.
- No change to `HeadlessWindow` geometry; we accept whatever size Chromium
  decides. We set it from the command line (`--window-size=1280,1024`).

---

## 4. Shared Memory Mechanism

### 4.1 Options considered

| Option                     | Pros                                         | Cons                                             |
|----------------------------|----------------------------------------------|--------------------------------------------------|
| A. SYSV `shmget`/`shmat`   | Simple, persistent IDs                       | Requires `IPC_PRIVATE` key server; 6 new syscalls|
| B. POSIX `shm_open`        | Standard, used by Chromium internals         | Requires `/dev/shm`, more FS scaffolding         |
| C. `mmap` of known file    | Zero new syscalls (we have open/mmap)        | Needs a file node in BatFS                       |
| D. Fixed physical address  | Zero-copy, zero syscalls                     | Breaks abstraction, fights VM subsystem          |

### 4.2 Recommendation: **Option C — `mmap` of a known path `/batos/fb0`**

**Why:**

1. **Syscall surface is already planned.** Phase 4 adds `openat`, `ftruncate`,
   `mmap`. No new syscall categories.
2. **Chromium does this pattern natively.** `base::File` + `base::MemoryMappedFile`
   expect exactly this. If we ever extend to multi-process, `shm_open` +
   `mmap` is a one-line switch.
3. **Debuggable from the shell.** A `hexdump /batos/fb0 | head` from the
   Bat_OS shell tells us if pixels are being written, with no new tooling.
4. **Bootstrap-friendly.** We can pre-create `/batos/fb0` at mount time with
   the kernel already knowing the backing frames.

**Why not D (fixed physaddr):** Chromium will `mmap` its own mappings during
startup and may land on whatever address we pick. We'd have to statically
reserve a guaranteed-free VA range in the loader, which fights the static-PIE
ASLR we get for free. Option C composes cleanly with the VM.

### 4.3 Semantics

- Region size: **5 MiB + 128 bytes** (header). Fixed.
- Magic: `'BFB1'` = `0x42464231`. Header `version = 1`.
- Writer (content_shell): fills pixels, then **release-store** `seq`.
- Reader (Bat_OS kthread): **acquire-load** `seq`; if changed, blit damage
  rect; remember last seen seq.
- No locks. Single producer / single consumer. If the reader misses a frame
  because it's still blitting the previous one, that's fine — it just skips
  to the latest.

### 4.4 Code sketch — Chromium side

(Already in the patch above. The critical contract is this function:)

```cpp
// Writes: pixels must be flushed to memory before seq bump.
void PublishFrame(BatosFbHeader* hdr, const gfx::Rect& damage) {
  hdr->damage_x.store(damage.x(),      std::memory_order_relaxed);
  hdr->damage_y.store(damage.y(),      std::memory_order_relaxed);
  hdr->damage_w.store(damage.width(),  std::memory_order_relaxed);
  hdr->damage_h.store(damage.height(), std::memory_order_relaxed);
  hdr->seq.fetch_add(1, std::memory_order_release);
}
```

### 4.5 Code sketch — Bat_OS side (Rust)

`src/drivers/display/chromium_blit.rs` (new file):

```rust
// Bat_OS — Chromium → virtio-gpu blit bridge.
//
// Polls a shared-memory region filled by content_shell's patched
// SoftwareOutputDeviceHeadless and blits damaged regions to the scanout.
//
// Contract:  see ports/chromium_port/PHASE5_DISPLAY.md §4.

use core::sync::atomic::{AtomicU32, Ordering};
use crate::drivers::virtio::gpu;

const MAGIC: u32 = 0x4246_4231; // 'BFB1'
const HDR_SIZE: usize = 128;
const MAX_W: u32 = 1280;
const MAX_H: u32 = 1024;

#[repr(C)]
struct FbHeader {
    magic: u32,
    version: u32,
    width: u32,
    height: u32,
    stride: u32,
    format: u32,
    seq: AtomicU32,
    damage_x: AtomicU32,
    damage_y: AtomicU32,
    damage_w: AtomicU32,
    damage_h: AtomicU32,
    pts_ns: u64,
    _reserved: [u32; 8],
}

pub struct Bridge {
    base: *mut u8,
    last_seq: u32,
}

impl Bridge {
    /// Bind to a region backed by the BatFS node /batos/fb0.
    /// `base` is the kernel VA pointing at the header.
    pub unsafe fn new(base: *mut u8) -> Self {
        let hdr = &*(base as *const FbHeader);
        assert_eq!(hdr.magic, MAGIC, "chromium_blit: magic mismatch");
        assert_eq!(hdr.version, 1);
        assert!(hdr.width  <= MAX_W);
        assert!(hdr.height <= MAX_H);
        Bridge { base, last_seq: 0 }
    }

    fn hdr(&self) -> &FbHeader {
        unsafe { &*(self.base as *const FbHeader) }
    }

    fn pixels(&self) -> *const u32 {
        unsafe { self.base.add(HDR_SIZE) as *const u32 }
    }

    /// Check for a new frame; if present, blit & flush. Returns true if blit.
    pub fn tick(&mut self) -> bool {
        let h = self.hdr();
        let cur = h.seq.load(Ordering::Acquire);
        if cur == self.last_seq { return false; }
        self.last_seq = cur;

        let w  = h.width;
        let ht = h.height;
        let stride_px = h.stride / 4;

        // Damage rect; clamped to virtio-gpu bounds.
        let dx = h.damage_x.load(Ordering::Relaxed).min(w);
        let dy = h.damage_y.load(Ordering::Relaxed).min(ht);
        let dw = h.damage_w.load(Ordering::Relaxed).min(w - dx);
        let dh = h.damage_h.load(Ordering::Relaxed).min(ht - dy);
        if dw == 0 || dh == 0 { return false; }

        let fb = gpu::framebuffer();
        let fb_w = gpu::width();
        let src = self.pixels();

        // Straight copy — formats match (BGRA8888 premul).
        // No conversion, no resize (content_shell is set to our resolution).
        for row in 0..dh {
            let src_row = (dy + row) * stride_px;
            let dst_row = (dy + row) * fb_w;
            for col in 0..dw {
                unsafe {
                    let p = core::ptr::read_volatile(src.add((src_row + dx + col) as usize));
                    core::ptr::write_volatile(fb.add((dst_row + dx + col) as usize), p);
                }
            }
        }

        // Tell virtio-gpu about the damage.
        gpu::flush(dx, dy, dw, dh);
        true
    }
}
```

Notes on this sketch:

- Read and write are `volatile` because the region is shared across a trust
  boundary (the sandbox-free EL0 process). The compiler must not assume
  pixels are stable.
- The copy is the simple version. If we later care about per-row memcpy
  performance, the inner loop becomes a `core::intrinsics::copy_nonoverlapping`
  call — typical copy is ~5 MiB at 60 Hz = 300 MiB/s, well within cached
  memcpy budget on M4.
- `gpu::flush` issues `TRANSFER_TO_HOST_2D` + `RESOURCE_FLUSH` (see
  `src/drivers/virtio/gpu.rs:159`). For full-screen frames that's two
  virtqueue round-trips per frame.

---

## 5. Framebuffer Blit Loop

### 5.1 Where to install it

Options:

| Where                              | Verdict                                                |
|------------------------------------|--------------------------------------------------------|
| Per-frame timer IRQ handler        | **No** — we'd hold timer IRQ for milliseconds copying  |
| Kernel thread (dedicated)          | **Yes** — clean, isolated, preemptible                 |
| Inside the Chromium task loop       | No — we can't run Rust inside the guest cleanly        |
| Polled from idle loop              | No — latency is tied to idle, too bursty               |

**Recommendation: a dedicated kernel thread `chromium_blit_kthread`.**

- Created when the `chromium` shell command launches (see Phase 7).
- Priority just above idle, below interactive tasks (we have no "interactive"
  yet, so default priority is fine).
- Torn down when the Chromium process exits.

### 5.2 How often

Two valid strategies:

**(a) Pull-driven, 60 Hz timer:**
```
loop {
    bridge.tick();
    timerfd_wait(1_000_000_000 / 60);
}
```
Simplest. Worst case: 16 ms latency from paint to display. Fine for v1.

**(b) Push-driven, futex-on-seq:**
Chromium's `PublishFrame` does `futex_wake(&seq, 1)` after bumping seq.
The kthread does `futex_wait(&seq, last_seq)`.

**Recommendation: start with (a). Add (b) if vsync matters.**

Why not (b) immediately: futex across the process-boundary hasn't been
exercised yet in Phase 4. Pull-driven is stress-free.

### 5.3 Pixel format

| Side            | Format                   | Byte order         |
|-----------------|--------------------------|--------------------|
| Chromium        | BGRA8888, premul alpha   | B, G, R, A         |
| virtio-gpu      | `FORMAT_B8G8R8A8`        | B, G, R, A         |

**These match exactly.** Zero conversion. This is a gift — the N32 color
type in Skia happens to be BGRA on little-endian ARM64, and it lines up
with what we told virtio-gpu at init time in `gpu.rs:23`.

If we later want to support a different virtio-gpu format (e.g. the host
gives us `R8G8B8A8`), the blit inner loop gains a byte-swap; cost is one
`swap_bytes()` per pixel. Still well under memory bandwidth.

### 5.4 Clipping, resolution mismatch, scaling

- **Clipping:** the blit respects `gpu::width()/height()`. Anything outside
  is truncated by the bounds in `tick()`.
- **Resolution mismatch:** we launch content_shell with
  `--window-size=1280,1024` (or whatever `gpu::width/height()` returns),
  so Chromium's viewport equals our scanout. No scaling.
- **Scaling (deferred):** If we ever run content_shell at a smaller size
  (e.g. 800x600 to save memory), `tick()` gains a nearest-neighbor upscale.
  Don't do this in v1 — render at native res.
- **DPI:** pass `--force-device-scale-factor=1.0`. We don't have HiDPI
  awareness yet.

### 5.5 Tearing

Full-screen blit at 60 Hz from a non-vsynced thread will tear at
scroll-heavy moments. For v1 this is acceptable (no scroll yet). When
we add input and scrolling:
- Double-buffer the pixel region (header carries `active_buffer_index: u32`)
- Chromium writes to one buffer; blit reads from the other
- Swap on seq bump

Not in Phase 5 scope.

---

## 6. GPU Strategy

### 6.1 Chromium's rendering paths

| Path                           | Flag                                          | What you get                                |
|--------------------------------|-----------------------------------------------|---------------------------------------------|
| Hardware GPU                   | (default)                                     | Real GL/Vulkan — we have no driver. NO.     |
| SwiftShader (software GL)      | `--use-gl=angle --use-angle=swiftshader`      | Software OpenGL ES via a statically-linkable library |
| SwiftShader WebGL only         | `--use-gl=swiftshader-webgl`                  | WebGL uses SwiftShader, compositor still tries GL |
| Software compositor            | `--disable-gpu --disable-gpu-compositing`     | `viz::SoftwareRenderer` → `SkCanvas`        |
| Software + no rasterizer       | `--disable-gpu --disable-software-rasterizer` | 2D path only, simplest                      |

### 6.2 Decision matrix for Phase 5

We need **the path with the fewest moving parts that still renders real
pages correctly**. WebGL is out of scope for Stage 8 (Google homepage is
2D). That narrows it to software.

| Flag combo                                             | Render quality | Complexity                    |
|--------------------------------------------------------|----------------|-------------------------------|
| `--disable-gpu --disable-gpu-compositing`              | Full 2D        | SoftwareOutputDevice (§3)     |
| `--disable-gpu --disable-software-rasterizer`          | 2D w/o tiles   | Lower code path, less code    |
| `--use-gl=angle --use-angle=swiftshader`               | Full, slow     | SwiftShader startup cost      |

**Recommendation for Phase 5 launch:**
```
--single-process --no-zygote --no-sandbox \
--headless \
--disable-gpu \
--disable-gpu-compositing \
--disable-software-rasterizer \
--use-gl=disabled \
--ozone-platform=headless \
--window-size=1280,1024 \
--force-device-scale-factor=1.0
```

Rationale:
- `--disable-gpu` + `--disable-gpu-compositing` → forces `viz::SoftwareRenderer`
- `--disable-software-rasterizer` → skip cc's software rasterizer
  (Blink paints directly into the compositor surface). Saves ~15 MB RAM.
- `--use-gl=disabled` → do not even try to load a GL backend, avoiding
  SwiftShader mmap (~10 MB of codegen heap we don't need).
- `--ozone-platform=headless` → use the backend we patched.
- `--no-sandbox --single-process --no-zygote` → already decided in
  CHROMIUM_PORT_PLAN.md. No fork/exec needed.

### 6.3 WebGL / Canvas2D fallback

With the recommended flags, `<canvas>` 2D works (it's CPU). WebGL pages
will render black / show fallback. That's acceptable for Stage 8. If/when
we want WebGL:
- Add `--use-gl=angle --use-angle=swiftshader`
- SwiftShader ships as a `.so` by default (`libvk_swiftshader.so`). We
  need either:
  - static-link it (Chromium has `swiftshader_for_tests` static targets
    but they're not meant for production),
  - or support loading one `.so` in BatCave (minimal dynamic loader).
- Budget: 3-5 extra days. Post-Phase 5.

### 6.4 Chromium references (for double-check during implementation)

- Flag definitions: `gpu/config/gpu_switches.cc`, `ui/gl/gl_switches.cc`
- Ozone platform switch: `ui/ozone/public/ozone_switches.cc`
  (`switches::kOzonePlatform = "ozone-platform"`)
- Software path entry: `components/viz/service/display/display.cc` →
  `Display::InitializeRenderer`
- The SwiftShader ICD is loaded via
  `third_party/swiftshader/src/Vulkan/libvk_swiftshader.so.json`

---

## 7. Debugging Plan

### 7.1 How we'll know pixels are flowing (early signal, before full page)

Add a **heartbeat mode** to the patch, gated by an env var:

```cpp
// In SoftwareOutputDeviceHeadless::Resize, after mapping:
if (getenv("BATOS_FB_HEARTBEAT")) {
  // Fill with solid magenta so we see something before the compositor runs.
  uint32_t* p = reinterpret_cast<uint32_t*>(fb_.pixels);
  size_t n = (fb_.hdr->width * fb_.hdr->height);
  for (size_t i = 0; i < n; ++i) p[i] = 0xFFFF00FFu; // BGRA: B=FF G=00 R=FF
  fb_.hdr->damage_x.store(0);
  fb_.hdr->damage_y.store(0);
  fb_.hdr->damage_w.store(fb_.hdr->width);
  fb_.hdr->damage_h.store(fb_.hdr->height);
  fb_.hdr->seq.fetch_add(1, std::memory_order_release);
}
```

If we see a magenta screen in Bat_OS, the pipe is alive; the problem is
upstream Chromium. If not, the pipe is broken; look at shm mapping.

### 7.2 Layered test fixtures (in order — each gates the next)

| # | Command                                                           | Expected output                               |
|---|-------------------------------------------------------------------|-----------------------------------------------|
| 1 | `BATOS_FB_HEARTBEAT=1 content_shell --headless about:blank`       | Solid magenta in Bat_OS fb                    |
| 2 | `content_shell --headless about:blank`                            | Solid white in Bat_OS fb                      |
| 3 | `content_shell --headless 'data:text/html,<body bgcolor=red>'`    | Solid red                                     |
| 4 | `content_shell --headless 'data:text/html,<h1>hello</h1>'`        | White bg + black "hello" top-left             |
| 5 | `content_shell --headless 'data:text/html,<h1 style=color:red>hi</h1>'` | Red "hi" — confirms CSS + color path    |
| 6 | `content_shell --headless https://example.com`                    | example.com homepage                          |
| 7 | `content_shell --headless https://www.google.com`                 | Google homepage — **mission done**            |

**First visible milestone = test 1 (heartbeat magenta).** Target: end of
first week of Phase 5.

### 7.3 Diagnostics

- **Chromium side:** `--enable-logging=stderr --v=1
  --vmodule=software_output*=2,headless_*=2` dumps one line per `EndPaint`.
  We'll wire Chromium stderr to Bat_OS UART.
- **Bat_OS side:** in `chromium_blit.rs`, behind `#[cfg(debug_assertions)]`,
  print `seq` transitions to UART with `uart::puts`. One line per frame.
- **Shared region inspection:** a `fbdump` shell builtin that prints the
  header fields. Three-line Rust function.
- **virtio-gpu:** if frames arrive but nothing displays, suspect the
  `flush` path — the existing code polls `used_ring` synchronously
  (`gpu.rs:70-80`). Confirm `queue.poll_used()` returns.

### 7.4 Performance sanity

Rough expectations:
- Full-frame copy: 5 MiB at ~1 GB/s L1 bandwidth = 5 ms
- virtio-gpu flush at QEMU latency: 1-3 ms
- `EndPaint` → `tick()` latency (60 Hz poll): avg 8 ms, worst 16 ms
- Total input-to-display: 20-30 ms. Good enough.

If worse: first suspect is the per-pixel volatile loop. Replace with
`copy_nonoverlapping` on a row-by-row basis — expect 3-5x.

---

## 8. Risks & Fallbacks

| # | Risk                                                                 | Probability | Impact | Fallback                                                                    |
|---|----------------------------------------------------------------------|-------------|--------|-----------------------------------------------------------------------------|
| 1 | `SkSurfaces::WrapPixels` in M132 is gated by a flag or removed       | Low         | Med    | Use `SkImage::MakeRasterData` + blit from `SoftwareOutputDevice::OnSwapBuffers` instead |
| 2 | Premul alpha mismatch — Chromium outputs unpremul in some code path  | Low         | Med    | Force `kPremul_SkAlphaType` explicitly in `SkImageInfo::Make`, confirm     |
| 3 | BatFS lacks `ftruncate`                                              | Med         | Low    | Pre-size the `/batos/fb0` node at VFS-mount time                            |
| 4 | `mmap(MAP_SHARED)` of a FS-backed file not yet supported             | Med         | High   | Implement `MAP_SHARED` for anonymous-mmap first, back `/batos/fb0` with a special fs object that returns the known frames |
| 5 | Headless backend bypasses `SoftwareOutputDevice` for screenshot mode | Med         | High   | Don't use `--screenshot=`; use our EndPaint hook. This is the entire point of the patch. |
| 6 | Chromium resizes the viewport mid-run (window config)                | Med         | Med    | `Resize` remaps. Our kthread reads `hdr.width/height` each tick; handle it |
| 7 | Damage rect wrong/stale for first frame                              | High        | Low    | On first frame (`last_seq == 0`), force full-screen damage                  |
| 8 | Blit kthread starves on a high-paint page (animated GIF, video)      | Med         | Med    | Cap to ≤60 Hz; drop frames if behind (already the design)                   |
| 9 | Tearing visible in animated content                                  | Med         | Low    | Double-buffer (see §5.5). Post-Phase 5                                      |
| 10| We picked the wrong Ozone hook and paint happens *elsewhere*         | Med         | High   | Add a second hook in `viz::SoftwareOutputSurface::SwapBuffers` as belt-and-braces |
| 11| Chromium writes after our seq bump (races)                           | Low         | Med    | Release fence before seq bump guarantees ordering — already in patch         |
| 12| Page is rendered fine but fonts look wrong                           | High        | Low    | Separate issue (font subsystem); not a Phase 5 risk                         |

**Top 3 by likely-damage:** #4 (mmap shared), #5 (headless bypass), #10 (wrong hook).

Mitigations for all three:
- Build a tiny `hello_shm` Linux test (no Chromium) that does `mmap
  /batos/fb0`, fills it, bumps seq. If Bat_OS can't see that, Chromium
  won't either. Solves #4.
- Run `content_shell --headless about:blank --enable-logging=stderr` under
  strace first to confirm `EndPaint` is invoked. Solves #5.
- If we get all the way to integration and see no pixels, instrument
  *every* `SoftwareOutputDevice` subclass EndPaint via
  `base::debug::StackTrace()` to find which one actually runs. Solves #10.

---

## 9. Concrete File List

### 9.1 Chromium files to patch (Path A)

| File                                                                              | Change                                                                             |
|-----------------------------------------------------------------------------------|------------------------------------------------------------------------------------|
| `ui/ozone/platform/headless/headless_surface_factory.cc`                           | Patch `SoftwareOutputDeviceHeadless` to map `/batos/fb0` and publish frames (§3.3) |
| `ui/ozone/platform/headless/headless_surface_factory.h`                            | Fwd-declare the shm header struct if we split it into a helper                     |
| `ui/ozone/platform/headless/BUILD.gn`                                              | Add no deps — all stdlib/Skia already present                                      |

Saved as `ports/chromium_port/patches/005-ozone-batos-shm.patch`.

### 9.2 Bat_OS files to create

| File                                                 | Role                                                                                      |
|------------------------------------------------------|-------------------------------------------------------------------------------------------|
| `src/drivers/display/mod.rs`                         | New module root. Re-exports `chromium_blit`.                                              |
| `src/drivers/display/chromium_blit.rs`               | The `Bridge` struct and poll loop (§4.5)                                                  |
| `src/batcave/linux/shm_fb.rs`                        | VFS shim for `/batos/fb0`: returns the fixed frames to `mmap`                             |
| `ports/chromium_port/patches/005-ozone-batos-shm.patch` | Chromium patch produced from §3.3                                                      |
| `ports/chromium_port/PHASE5_DISPLAY.md`              | This document                                                                             |

### 9.3 Bat_OS files to modify

| File                                    | Change                                                                                  |
|-----------------------------------------|-----------------------------------------------------------------------------------------|
| `src/drivers/mod.rs`                    | Add `pub mod display;`                                                                  |
| `src/drivers/virtio/gpu.rs`             | Expose `framebuffer_ptr_mut() -> *mut u32` publicly (already have `framebuffer()`)      |
| `src/batcave/linux/syscalls.rs`         | Route `openat("/batos/fb0")` and `mmap` MAP_SHARED on that fd into `shm_fb::map`        |
| `src/batcave/linux/vfs.rs` (or similar) | Register `/batos/` namespace with the one node `fb0` backed by `shm_fb`                 |
| `src/ui/shell.rs`                       | `chromium <url>` command: pre-create /batos/fb0, spawn blit kthread, launch content_shell |
| `src/kernel/task.rs` (or similar)       | Add `spawn_kthread` if we don't already have one (we may; this is a sanity check)        |

### 9.4 Ports dir deliverables

| File                                                 | Role                                                                        |
|------------------------------------------------------|-----------------------------------------------------------------------------|
| `ports/chromium_port/patches/005-ozone-batos-shm.patch` | The patch                                                                |
| `ports/chromium_port/tests/hello_shm.c`              | Minimal C test: mmap `/batos/fb0`, fill with magenta, bump seq              |
| `ports/chromium_port/tests/hello_shm.sh`             | Build + run the C test through BatCave                                      |

---

## 10. ETA and Parallelism

### 10.1 Work breakdown

| #  | Task                                                                  | Days | Prereq      |
|----|-----------------------------------------------------------------------|------|-------------|
| T1 | Design review + sign-off on this doc                                  | 0.5  | —           |
| T2 | Implement `shm_fb` VFS node + wire into `openat`/`mmap`               | 1.5  | Phase 4 mmap |
| T3 | Implement `chromium_blit::Bridge` + kthread plumbing                  | 1    | T2          |
| T4 | Write `hello_shm.c` test. Run in BatCave. Confirm Bat_OS sees magenta | 1    | T2, T3      |
| T5 | Write the Chromium patch per §3.3                                     | 0.5  | T1          |
| T6 | Rebuild content_shell with patch. Run `--headless about:blank`        | 0.5  | T5, Phase 1 complete |
| T7 | End-to-end: test fixtures 1-5 in §7.2                                 | 1    | T4, T6      |
| T8 | End-to-end: test fixtures 6-7 (network needed) — **Phase 6 gate**     | —    | Phase 6     |
| T9 | Damage-rect correctness + edge cases (resize, overflow)               | 0.5  | T7          |
| T10| Perf pass: row-memcpy, drop pathological logs                         | 0.5  | T7          |

**Total Phase 5 effort (excluding T8 which lives under Phase 6):** ~6 days.
Matches the "5-7 days" line item in CHROMIUM_PORT_PLAN.md.

### 10.2 What can be parallelized

- **T2 + T5** are independent. One person on the Bat_OS VFS work, one
  writing the Chromium patch. Saves ~2 days on the critical path.
- **T4** (hello_shm test) can be written before T2 is done — it's a C
  program that just exercises `openat+mmap`. Pre-writing it means T2's
  "done" criterion is "T4 passes". Tightens the feedback loop.
- **T10** (perf pass) is parallel with Phase 6 network work. Can slip.

### 10.3 Critical path

```
T1 (0.5) → T2 (1.5) → T3 (1) → T4 (1) → T7 (1) → T9 (0.5)  ≈ 5.5 days
                     ↘        ↗
                       T5 (0.5) → T6 (0.5) ────────────┘
```

T5+T6 run parallel with T3+T4. Neither side blocks the other until T7.

### 10.4 Definition of Done for Phase 5

1. `chromium data:text/html,<h1 style='color:red'>hello</h1>` from Bat_OS
   shell renders red "hello" on the virtio-gpu scanout.
2. No magenta / garbage pixels.
3. Shell can Ctrl-C out; kthread exits cleanly; Chromium cleans up.
4. Second run succeeds (shm region is reused, not leaked).
5. `fbdump` shows sane header values (magic, non-zero seq).

Stages 6-8 (example.com, google.com) are gated on Phase 6 (network).

---

## Appendix A — Why we're confident BGRA8888 matches

Verified by cross-referencing:

- Chromium: `SkImageInfo::MakeN32Premul` returns `kN32_SkColorType` which
  on little-endian ARM64 is `kBGRA_8888_SkColorType`. See
  `third_party/skia/include/core/SkImageInfo.h` — the
  `#if SK_PMCOLOR_BYTE_ORDER(B,G,R,A)` branch.
- Bat_OS: `FORMAT_B8G8R8A8 = 1` is set at virtio-gpu init
  (`src/drivers/virtio/gpu.rs:23`). Per VirtIO 1.2 spec §5.7.6.1 this
  means "each pixel is 32-bit, memory order B, G, R, A".

Both sides agree: a raw `memcpy` (or equivalent) is correct.

## Appendix B — Why not Wayland/DRM Ozone as templates

`ozone/platform/wayland` and `ozone/platform/drm` are tempting as Path B
templates but both assume a compositor/kernel partner that speaks their
protocol. We have neither. `headless` is the closest match to our
"write pixels to a buffer, someone else does the rest" story. Use it.

## Appendix C — Checklist for the Path B migration

When we revisit Path B after Phase 5 lands:

- [ ] Create `ui/ozone/platform/batos/` with files:
  - [ ] `ozone_platform_batos.{cc,h}`
  - [ ] `batos_window.{cc,h}`
  - [ ] `batos_surface_factory.{cc,h}`
  - [ ] `batos_software_output_device.{cc,h}` (port the patch into here)
  - [ ] `client_native_pixmap_factory_batos.{cc,h}` (can be a stub)
  - [ ] `BUILD.gn`
- [ ] Register in `ui/ozone/ozone.gni` under `ozone_platforms`
- [ ] Register in `ui/ozone/BUILD.gn` dep graph
- [ ] Add entry in `ui/ozone/platform/platform_object_internal.h`
- [ ] Command line switches to `--ozone-platform=batos`
- [ ] Remove the patch from §3 (or mark it dead)
- [ ] Port input: `PlatformEventSource` pulling from BatCave keyboard/mouse fd
- [ ] Regression test: re-run test fixtures 1-7 from §7.2

---

End of Phase 5 spec.

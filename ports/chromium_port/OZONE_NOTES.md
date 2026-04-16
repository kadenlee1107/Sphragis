# Ozone Headless Rendering Notes

Research report for **Phase 5** of the Bat_OS Chromium port — understanding
how Chromium's `--ozone-platform=headless --disable-gpu` path produces pixels,
so we can wire those pixels into Bat_OS's framebuffer.

**Runtime configuration we are targeting:**
```
content_shell \
    --single-process \
    --no-zygote \
    --no-sandbox \
    --disable-gpu \
    --disable-gpu-compositing \
    --ozone-platform=headless \
    --ozone-dump-file=<path>          (the escape hatch we exploit)
    --screenshot=<path>               (alternative: DevTools-style capture)
    <URL>
```

All file paths below are relative to the Chromium source tree
(`chromium/src/…`) pinned at **M132** (commit range `132.0.6834.x`).
Line numbers are approximate; Chromium moves code around frequently — grep by
symbol name if the exact line drifts.

Upstream references used:

- `ui/ozone/platform/headless/` — the headless Ozone backend
- `ui/ozone/public/surface_ozone_canvas.h` — the paint interface
- `components/viz/service/display_embedder/software_output_device_ozone.*`
  — the viz-side consumer of `SurfaceOzoneCanvas`
- `components/viz/service/display_embedder/software_output_surface.cc`
  — wraps `SoftwareOutputDevice` for the display compositor
- `headless/lib/browser/headless_web_contents_impl.cc` — `--screenshot`
  (DevTools `CopyFromSurface` path — different from `--ozone-dump-file`)

---

## 1. Ozone Headless Backend (ui/ozone/platform/headless/)

### 1.1 File inventory

| File                                        | Purpose                                                   |
|---------------------------------------------|-----------------------------------------------------------|
| `BUILD.gn`, `DEPS`                          | Build plumbing                                            |
| `ozone_platform_headless.{cc,h}`            | `OzonePlatformHeadless` — the entry point class           |
| `headless_window.{cc,h}`                    | `HeadlessWindow` — a `ui::PlatformWindow` that has no surface of its own |
| `headless_window_manager.{cc,h}`            | `HeadlessWindowManager` — IDMap<HeadlessWindow*> by `gfx::AcceleratedWidget` |
| `headless_surface_factory.{cc,h}`           | `HeadlessSurfaceFactory`, `FileSurface`, `FileGLSurface`, `GLOzoneEGLHeadless` |
| `headless_screen.{cc,h}`                    | Fake `display::Screen` reporting one display              |
| `client_native_pixmap_factory_headless.{cc,h}` | Stub pixmap factory                                    |
| `vulkan_*`                                  | Vulkan path (irrelevant — we `--disable-gpu`)             |

### 1.2 OzonePlatformHeadless

`ui/ozone/platform/headless/ozone_platform_headless.cc`

```cpp
OzonePlatform* CreateOzonePlatformHeadless() {
  base::CommandLine* cmd = base::CommandLine::ForCurrentProcess();
  base::FilePath location;
  if (cmd->HasSwitch(switches::kOzoneDumpFile))
    location = cmd->GetSwitchValuePath(switches::kOzoneDumpFile);
  return new OzonePlatformHeadlessImpl(location);
}
```

- `switches::kOzoneDumpFile == "ozone-dump-file"` (defined in
  `ui/ozone/public/ozone_switches.cc`). The doc string is literally
  *"Specify location for image dumps."*
- `InitializeUI()` constructs:
  - `HeadlessWindowManager`
  - `HeadlessSurfaceFactory(location)` — carries `--ozone-dump-file` down
  - `HeadlessPlatformEventSource` (only if one doesn't already exist)
  - stubs for overlay manager, cursor factory, input controller
- `InitializeGPU()` lazily re-creates `HeadlessSurfaceFactory` if
  `InitializeUI` wasn't run (e.g., GPU process). In single-process mode this
  branch is usually skipped.

### 1.3 HeadlessWindow (no pixel buffer here)

`ui/ozone/platform/headless/headless_window.{cc,h}`

Contrary to what the name might suggest, **`HeadlessWindow` does NOT own any
pixel buffer**. It is a thin `ui::PlatformWindow` that:

- stores `bounds_`, `visible_`, `window_state_`, `activation_state_`
- holds `widget_` (a `gfx::AcceleratedWidget`, basically an integer)
- forwards state changes via `delegate_->OnBoundsChanged(...)`,
  `OnWindowStateChanged(...)`, `OnActivationChanged(...)`

The widget ID is the handle Chromium uses on the viz/cc side when it asks for
a surface. The pixel buffer is allocated lazily **inside the surface factory**
when the software compositor calls `CreateCanvasForWidget(widget)`.

### 1.4 HeadlessSurfaceFactory — the real pixel-producing class

`ui/ozone/platform/headless/headless_surface_factory.h`:

```cpp
class HeadlessSurfaceFactory : public SurfaceFactoryOzone {
 public:
  explicit HeadlessSurfaceFactory(base::FilePath base_path);
  ~HeadlessSurfaceFactory() override;

  std::vector<gl::GLImplementationParts> GetAllowedGLImplementations() override;
  GLOzone* GetGLOzone(const gl::GLImplementationParts&) override;

  std::unique_ptr<SurfaceOzoneCanvas> CreateCanvasForWidget(
      gfx::AcceleratedWidget widget) override;

  scoped_refptr<gfx::NativePixmap> CreateNativePixmap(...) override;

 private:
  void CheckBasePath() const;

  base::FilePath base_path_;                    // from --ozone-dump-file
  std::unique_ptr<GLOzone> swiftshader_implementation_;
};
```

`CreateCanvasForWidget()` is the factory we care about:

```cpp
std::unique_ptr<SurfaceOzoneCanvas>
HeadlessSurfaceFactory::CreateCanvasForWidget(gfx::AcceleratedWidget widget) {
  return std::make_unique<FileSurface>(GetPathForWidget(base_path_, widget));
}
```

where

```cpp
base::FilePath GetPathForWidget(const base::FilePath& base_path,
                                gfx::AcceleratedWidget widget) {
  if (base_path.empty() || base_path == base::FilePath(kDevNull))
    return base_path;
  return base_path.Append(base::NumberToString(widget) + ".png");
}
```

So:

- If `--ozone-dump-file` is not set → `base_path_` is empty → every
  `PresentCanvas()` call early-returns (pixels get rendered but discarded).
- If set → every presented frame is encoded to PNG and dumped to
  `<base_path>/<widget_id>.png`, overwriting the previous frame each time.

### 1.5 FileSurface — the SkSurface-backed software canvas

`ui/ozone/platform/headless/headless_surface_factory.cc` (approx. lines 85–130):

```cpp
class FileSurface : public SurfaceOzoneCanvas {
 public:
  explicit FileSurface(const base::FilePath& location) : base_path_(location) {}

  // SurfaceOzoneCanvas:
  void ResizeCanvas(const gfx::Size& viewport_size, float scale) override {
    SkSurfaceProps props = skia::LegacyDisplayGlobals::GetSkSurfaceProps();
    surface_ = SkSurfaces::Raster(
        SkImageInfo::MakeN32Premul(viewport_size.width(),
                                   viewport_size.height()),
        &props);
  }

  SkCanvas* GetCanvas() override { return surface_->getCanvas(); }

  void PresentCanvas(const gfx::Rect& damage) override {
    if (base_path_.empty())
      return;
    SkBitmap bitmap;
    bitmap.allocPixels(surface_->getCanvas()->imageInfo());
    if (surface_->getCanvas()->readPixels(bitmap, 0, 0)) {
      base::ThreadPool::PostTask(
          FROM_HERE,
          {base::MayBlock(), base::TaskShutdownBehavior::CONTINUE_ON_SHUTDOWN},
          base::BindOnce(&WriteDataToFile, base_path_, bitmap));
    }
  }

  std::unique_ptr<gfx::VSyncProvider> CreateVSyncProvider() override {
    return nullptr;
  }

 private:
  base::FilePath base_path_;
  sk_sp<SkSurface> surface_;
};
```

Key properties of the buffer:

- **Pure in-memory raster surface.** `SkSurfaces::Raster(...)` allocates a
  heap-backed Skia pixel buffer via Skia's default pixel allocator
  (`malloc`/`operator new` underneath — eventually hitting our `mmap`
  syscall). **Not file-backed, not shared memory.**
- **Format: `kN32_SkColorType`, `kPremul_SkAlphaType`.**
  On little-endian ARM64 (our Bat_OS target), `kN32_SkColorType` is
  `kBGRA_8888_SkColorType`: 32-bit LE word is `[A:31..24 R:23..16 G:15..8 B:7..0]`.
  In memory order (byte 0 → byte 3): `B G R A`. Premultiplied alpha.
- **Stride:** Skia's raster surface uses tight packing (`rowBytes = width*4`)
  unless alignment forces padding. Read `bitmap.rowBytes()` / `imageInfo()` if
  you care — don't assume.

### 1.6 WriteDataToFile — the PNG sink

Approx. lines 71–86:

```cpp
void WriteDataToFile(const base::FilePath& location, const SkBitmap& bitmap) {
  DCHECK(!location.empty());
  std::optional<std::vector<uint8_t>> png_data =
      gfx::PNGCodec::FastEncodeBGRASkBitmap(bitmap, /*discard_transparency=*/true);
  if (!png_data || !base::WriteFile(location, png_data.value())) {
    static bool logged_once = false;
    LOG_IF(ERROR, !logged_once)
        << "Failed to write frame to file. "
           "If running with the GPU process try --no-sandbox.";
    logged_once = true;
  }
}
```

- Runs on the **ThreadPool** (arbitrary worker thread), `MayBlock`,
  `CONTINUE_ON_SHUTDOWN`.
- Does PNG encoding via `gfx::PNGCodec::FastEncodeBGRASkBitmap`.
- Writes via `base::WriteFile` → `open(...O_CREAT|O_TRUNC) / write / close`.

**This is the one chokepoint we want to hijack.** Everything a headless Chromium
frame goes through lands here with a ready-to-use BGRA8888 premultiplied
`SkBitmap`.

### 1.7 FileGLSurface / GLOzoneEGLHeadless

Only reached if someone forces `--use-gl=swiftshader-webgl` or similar. In the
`--disable-gpu` world we are targeting this is **not** on the hot path —
mentioned here only so future reader doesn't confuse it with `FileSurface`:

- `FileGLSurface : public GLSurfaceEglReadback` — overrides `HandlePixels()`,
  receives RGBA pixels via EGL readback (SwiftShader), constructs an
  `SkBitmap` and posts to the same `WriteDataToFile` helper.
- `GLOzoneEGLHeadless` is the `GLOzone` that creates `FileGLSurface`
  instances.

We can ignore both classes for the framebuffer wire-up.

---

## 2. Software Compositor Path

How a Blink paint turns into a `FileSurface::PresentCanvas()` call.

### 2.1 High-level flow (single-process content_shell, --disable-gpu)

```
Blink (main thread)
   │  (paint ops, cc::Layer tree)
   ▼
cc::LayerTreeHostImpl (compositor thread)
   │  Commit + Draw
   ▼
viz::DisplayCompositor (viz thread, in-process in single-process mode)
   │  aggregates CompositorFrame → DrawFrame
   ▼
viz::SoftwareRenderer   (DirectRenderer subclass, software path)
   │  issues draw calls against an SkCanvas obtained from
   │  SoftwareOutputDevice::BeginPaint()
   ▼
viz::SoftwareOutputSurface  (wraps a SoftwareOutputDevice)
   │  SwapBuffers(OutputSurfaceFrame)
   ▼
viz::SoftwareOutputDeviceOzone  (components/viz/.../software_output_device_ozone.cc)
   │  owns: ui::PlatformWindowSurface + ui::SurfaceOzoneCanvas
   │  BeginPaint → surface_ozone_->GetCanvas()
   │  EndPaint   → surface_ozone_->PresentCanvas(damage)
   │  OnSwapBuffers → surface_ozone_->OnSwapBuffers(...)  (if supported)
   ▼
FileSurface (our ui/ozone/platform/headless/headless_surface_factory.cc)
   │  readPixels → SkBitmap → ThreadPool::PostTask(WriteDataToFile)
   ▼
WriteDataToFile → PNGCodec::FastEncodeBGRASkBitmap → base::WriteFile
```

### 2.2 SurfaceOzoneCanvas interface contract

`ui/ozone/public/surface_ozone_canvas.h`:

| Method                         | Purpose                                                            |
|--------------------------------|--------------------------------------------------------------------|
| `GetCanvas()`                  | Returns `SkCanvas*` valid until next `PresentCanvas`/`ResizeCanvas` |
| `ResizeCanvas(size, scale)`    | Allocates / reallocates the surface at viewport size                |
| `PresentCanvas(damage)`        | Pixels in damage region assumed fresh; outside assumed unchanged    |
| `CreateVSyncProvider()`        | May return nullptr (headless does)                                 |
| `SupportsAsyncBufferSwap()`    | Whether the ozone impl supports async swap callbacks                |
| `OnSwapBuffers(cb, FrameData)` | Completion callback when swap is done                              |
| `MaxFramesPending()`           | Pipelining depth                                                    |
| `SupportsOverridePlatformSize()` | Whether surface can be a different size than the platform window  |

### 2.3 SoftwareOutputDeviceOzone glue

`components/viz/service/display_embedder/software_output_device_ozone.cc`:

```cpp
void SoftwareOutputDeviceOzone::Resize(const gfx::Size& viewport_pixel_size,
                                       float scale_factor) {
  viewport_pixel_size_ = viewport_pixel_size;
  surface_ozone_->ResizeCanvas(viewport_pixel_size_, scale_factor);
}

SkCanvas* SoftwareOutputDeviceOzone::BeginPaint(const gfx::Rect& damage_rect) {
  damage_rect_ = damage_rect;
  return surface_ozone_->GetCanvas();
}

void SoftwareOutputDeviceOzone::EndPaint() {
  SoftwareOutputDevice::EndPaint();          // base class bookkeeping
  surface_ozone_->PresentCanvas(damage_rect_);
}

void SoftwareOutputDeviceOzone::OnSwapBuffers(
    SwapBuffersCallback swap_ack_callback,
    gfx::FrameData data) {
  if (surface_ozone_->SupportsAsyncBufferSwap()) {
    surface_ozone_->OnSwapBuffers(std::move(swap_ack_callback), data);
  } else {
    SoftwareOutputDevice::OnSwapBuffers(std::move(swap_ack_callback), data);
  }
}
```

Notes for our wire-up:

- `PresentCanvas` (and therefore our PNG write) is **triggered by `EndPaint()`,
  not by `OnSwapBuffers()`.** `EndPaint` always runs after the software
  renderer finishes the frame. `OnSwapBuffers` is a separate callback loop for
  presentation feedback.
- Every frame follows the strict order `Resize?` → `BeginPaint` →
  (Skia draw ops) → `EndPaint` → `OnSwapBuffers`.

### 2.4 Thread affinity — where the blit actually happens

Mapping Chromium's threads for a single-process `--disable-gpu` run:

| Work                                                | Thread                           |
|-----------------------------------------------------|----------------------------------|
| Blink layout, paint recording                       | Renderer main thread             |
| cc compositor commit + draw tiles (raster pool)     | Compositor thread + worker pool  |
| viz aggregation + `SoftwareRenderer::DrawFrame`     | **viz display thread** (in-proc) |
| `SoftwareOutputDeviceOzone::{BeginPaint,EndPaint}`  | **viz display thread**           |
| `FileSurface::PresentCanvas` (readPixels + PostTask) | **viz display thread**           |
| `WriteDataToFile` (PNG encode + write)              | base::ThreadPool worker (arbitrary) |

So the SkSurface "is owned by" the viz display thread — and the final CPU-side
blit (if we replace the PNG path with a framebuffer copy) should happen on
that thread. It's an in-process `base::Thread` in single-process mode; no IPC.

### 2.5 What "a frame is done" means

Two distinct signals:

1. **Paint complete for that swap:** `SoftwareOutputDeviceOzone::EndPaint()`
   returned. Equivalent to `FileSurface::PresentCanvas` returning.
2. **Presentation acked back to viz:** the callback passed to
   `SoftwareOutputSurface::SwapBuffers()` fires →
   `SwapBuffersCompleteParams` →
   `OutputSurfaceClient::DidReceiveSwapBuffersAck()` +
   `DidReceivePresentationFeedback()`.

For our framebuffer blit loop, signal (1) is what we want to hook. The viz
thread is already doing an `SkBitmap readPixels` right after that point — the
pixels are guaranteed consistent. Signal (2) is for VSync / latency
accounting; we can drive it synchronously from our blit (just ack
"presented").

---

## 3. Screenshot Mechanism — Two Distinct Paths

There are **two unrelated** ways to get a PNG out of Chromium. Don't confuse
them.

### 3.1 Path A — `--ozone-dump-file` (Ozone-level)

- Flag: `--ozone-dump-file=/path/to/dir`.
- Handled in `ui/ozone/platform/headless/ozone_platform_headless.cc`.
- Every **frame** the software compositor presents is dumped as
  `/path/to/dir/<widget>.png`, overwriting the prior frame.
- Dump code: `FileSurface::PresentCanvas → ThreadPool::PostTask →
  WriteDataToFile → PNGCodec::FastEncodeBGRASkBitmap → base::WriteFile`.
- This runs **unconditionally on the normal paint path** — it *is* the paint
  path for headless Ozone.
- **This is the path we should hijack for Bat_OS.**

### 3.2 Path B — `--screenshot` (content-level, DevTools-style)

- Flag: `--screenshot=/path/file.png` (one-shot).
- Lives in `headless/lib/browser/headless_web_contents_impl.cc` (not in
  `content_shell`'s main at all — this is the `headless_shell` binary, which
  upstream ships as `chrome-headless-shell` since M118).
- Mechanism:
  ```cpp
  // roughly, in HeadlessWebContentsImpl::BeginFrame(...)
  if (capture_screenshot) {
    content::RenderWidgetHostView* view =
        web_contents()->GetRenderWidgetHostView();
    if (view && view->IsSurfaceAvailableForCopy()) {
      view->CopyFromSurface(
          gfx::Rect(), gfx::Size(),
          base::BindOnce(&OnReadbackComplete, ...));
    }
  }
  ```
  `CopyFromSurface` is the same API the DevTools protocol's
  `Page.captureScreenshot` uses — it triggers a viz-side readback into an
  `SkBitmap`, then the callback PNG-encodes and writes.
- **Fires once per navigation**, not every frame.
- Relies on "the surface is ready"; for headless Ozone that's basically
  equivalent to "first frame presented."

### 3.3 Can we hook the same point?

Yes — but Path A is strictly better for our use case:

- Path A gives us **every frame** (animation/video/scrolling work "for free"
  once input is wired up).
- Path A's hook point (`WriteDataToFile` or `FileSurface::PresentCanvas`) is
  in the already-modified file we own as a backend.
- Path B gives us a one-shot. Great for initial bring-up (stage 2 of the
  plan's success criteria), but wrong shape for the live framebuffer loop.

---

## 4. Output Format

As produced by `FileSurface::ResizeCanvas` →
`SkImageInfo::MakeN32Premul(w, h)`:

| Property             | Value                                                        |
|----------------------|--------------------------------------------------------------|
| Color type           | `kN32_SkColorType` = `kBGRA_8888_SkColorType` on LE ARM64    |
| Alpha type           | `kPremul_SkAlphaType` (premultiplied)                        |
| Bytes per pixel      | 4                                                            |
| Byte order in memory | `B, G, R, A` (low address → high)                            |
| 32-bit LE word       | `0xAARRGGBB`                                                 |
| Color space          | sRGB (Skia default; untagged)                                |
| Stride (`rowBytes`)  | Typically `width*4`, but **read `bitmap.rowBytes()`**; don't assume contiguous |
| Pitch/padding        | No scanline padding by default; alignment comes from allocator |

**Bat_OS framebuffer assumption to confirm:** our current UI code
(`src/ui/`) paints BGRA or RGBA? Look at `src/drivers/display/…` — if the
panel expects RGBA we need an in-place byte-swap (B↔R) per pixel, or a simple
shader-less memcpy + swizzle kernel. The cost is ~1.5 ns/pixel on an M-class
core; ~12 ms for a 1080p frame — painful but acceptable for v1.

**Premultiplied alpha gotcha:** when blitting to an opaque framebuffer, we
can just ignore A (alpha is irrelevant against a solid backing). But if we
ever composite with anything else, remember Chromium gave us premultiplied
RGB; don't "multiply by alpha" again.

---

## 5. Frame-Completion Signal

Minimal chain we'd want to tap:

```
SoftwareOutputDeviceOzone::EndPaint
    └─> FileSurface::PresentCanvas(damage)
          ├─ readPixels → SkBitmap                [PIXELS ARE READY HERE]
          └─ ThreadPool::PostTask(WriteDataToFile) [or: blit to fb]
```

The **"frame is done and the pixels are consistent"** moment is inside
`FileSurface::PresentCanvas`, right after `readPixels` returns true and right
before the ThreadPool post.

For a blit loop, we have two options:

1. **Synchronous blit on the viz thread** — do the framebuffer copy
   *instead of* posting to ThreadPool. Simplest. Blocks viz for the duration
   of the blit (1–10 ms @ 1080p). Viz will happily pipeline the next frame.
2. **Async hand-off via shared memory** — `readPixels` into a shared buffer
   (shmem / our equivalent), signal Bat_OS display driver via an eventfd or
   direct function call, return fast. Display driver does the blit on its
   own thread.

Recommend #1 for Phase 5 (simpler, fewer moving parts); migrate to #2 once
input injection + smooth scrolling matter.

For back-pressure / VSync, `SurfaceOzoneCanvas::CreateVSyncProvider()`
currently returns `nullptr` in headless. If we want viz to throttle to our
display's refresh rate, we return a real `gfx::VSyncProvider` that uses
Bat_OS's vblank timer (if our framebuffer driver has one — if not, fake one
at 60 Hz with a `timerfd`).

---

## 6. Input Injection (Phase 8 stretch)

`ui/ozone/platform/headless/ozone_platform_headless.cc` creates a
`HeadlessPlatformEventSource` only if one doesn't already exist. It is a
mostly-empty `ui::PlatformEventSource` — it exists for the event infra's
sanity but does nothing on its own.

`OzonePlatformHeadlessImpl::CreateInputController()` returns a stub
controller (`CreateStubInputController()`), and `GetInputController()` returns
nullptr-ish — there is no native input pipeline in the headless backend.

To inject events, Chromium's usual headless path uses the **DevTools
protocol**: `Input.dispatchMouseEvent`, `Input.dispatchKeyEvent`,
`Input.insertText`. That's the route `puppeteer`/`playwright` take. For us:

- Short-term: open a DevTools socket and POST DevTools commands.
- Medium-term: fork the headless backend into a Bat_OS backend that turns our
  PS/2 scancodes into `ui::KeyEvent` / `ui::MouseEvent` directly and dispatches
  via `ui::PlatformEventSource::PlatformEventSourceDispatchEvent`. This is
  cleaner but requires writing ~50–100 lines of event-translation code.
- Long-term: `ui/ozone/platform/batos/` with a real
  `BatosPlatformEventSource` and a real `InputController`.

Out of scope for Phase 5. Note only that the existing headless backend
**cannot receive input** — so on Phase 5 hardware we'll see a static render
of the initial page, not an interactive browser. That matches the Phase 5
success criterion (render pixels on screen) fine.

---

## 7. Hacking Points — Three Smallest/Safest Patches

Ranked smallest/safest first.

### Hack #1 — Redirect `WriteDataToFile` to a framebuffer sink

**File:** `ui/ozone/platform/headless/headless_surface_factory.cc`
**Function:** `WriteDataToFile` (approx. lines 71–86)

**What:** Replace the `PNGCodec::FastEncodeBGRASkBitmap` + `base::WriteFile`
calls with a direct memcpy (or shared-memory write) into our framebuffer
region.

**Why safe:**
- Single function, one call site (`FileSurface::PresentCanvas`).
- Signature already has the `SkBitmap` in hand — no new plumbing needed.
- We can gate it on a new env var / switch so upstream Ozone behavior is
  untouched for non-Bat_OS builds.
- No changes to viz, cc, Blink, or any base/ infrastructure.

**Minimal patch shape (for documentation — do NOT apply blindly):**
1. Add a new command-line switch (e.g., `--bat-os-fb-shmem=<size>:<path>`
   or an env var like `BATOS_FRAMEBUFFER_FD=<int>`).
2. At `FileSurface` construction, if the switch is present, stash the fd/ptr.
3. In `PresentCanvas`, branch: if fb sink is set, do the blit on the viz
   thread (`memcpy(fb_ptr, bitmap.getPixels(), bitmap.computeByteSize())`).
   If not, fall back to the existing `ThreadPool::PostTask(WriteDataToFile)`.

**Line count:** ~30 LOC.

### Hack #2 — Bypass PNG encoding with a raw BGRA dump mode

**File:** `ui/ozone/platform/headless/headless_surface_factory.cc`
**Function:** `WriteDataToFile` same location.

**What:** When `--ozone-dump-file` points at a special path (e.g.,
`/dev/batfb` or a specific suffix like `.bgra`), write **raw BGRA pixels**
instead of a PNG. `base::WriteFile(location, bitmap.getPixels(),
bitmap.computeByteSize())`.

**Why safe:**
- Even smaller than Hack #1.
- Reuses the existing `--ozone-dump-file` plumbing; no new switch.
- On the Bat_OS side, our syscall layer can route `openat("/dev/batfb", …)`
  directly to the framebuffer region; each `write()` becomes a memcpy.
- Falls back to PNG for any other path — zero risk to upstream behavior.

**Downside:** every frame pays one extra `write(2)` syscall. Still cheap
(~microseconds on an M4), but Hack #1 avoids the detour entirely.

### Hack #3 — Teach `SoftwareOutputDeviceOzone` to expose its `SurfaceOzoneCanvas`

**File:** `components/viz/service/display_embedder/software_output_device_ozone.cc` (+ .h)

**What:** Add a `GetSurfaceOzone()` accessor, then on the Bat_OS side get a
pointer to the `FileSurface` and read its `surface_->peekPixels()` directly
after `EndPaint`. Lets us avoid the `readPixels` copy (Skia's `peekPixels`
hands back the surface's own buffer).

**Why useful:** for 1080p BGRA at 60 Hz, skipping the readPixels copy saves
~3 ms/frame + ~8 MB/s of memory bandwidth.

**Why #3 and not #1:** bigger blast radius (two files, one of them in
`components/viz/…`), and requires exposing an Ozone-private type
(`SurfaceOzoneCanvas*`) across the viz/ui boundary. Save for optimization
pass after Phase 5 works end-to-end.

---

## 8. Key Unknowns / Questions Left Open

These need answering during Phase 5 implementation, not by more reading:

1. **What does the first viewport size default to?** Nothing in
   `HeadlessWindow::Show` or `HeadlessScreen` looked definitive. Does
   content_shell default to 800×600? 1024×768? We may need `--window-size=WxH`
   on the command line. Needs empirical test with a content_shell build.
2. **Is the viz thread a real `base::Thread` in `--single-process`
   mode, or does it run on the main thread?** Affects the blit strategy. The
   `SwapBuffers` async callback pattern suggests a real thread; confirm with a
   `base::PlatformThread::CurrentId()` log.
3. **Does `SkSurfaces::Raster` go through `malloc` or call `mmap` directly
   for large allocations?** A 1080p BGRA surface is 8 MB — likely a direct
   `mmap` via `base::AllocPages`. Affects our Phase 4 `mmap` syscall coverage.
4. **How is damage communicated by the software compositor?** `PresentCanvas`
   gets a `gfx::Rect damage` — is it tight (for partial repaint) or always
   full-viewport with the software renderer? If tight, Hack #1 can do partial
   blits and save bandwidth.
5. **What happens on resize?** Does `ResizeCanvas` get called with stale
   widget ID? Do we need to reallocate our fb side too? Probably yes — the
   viz side will call it on every `Reshape`.
6. **Skia stride on ARM64:** does `SkImageInfo::MakeN32Premul(w, h)` guarantee
   `rowBytes == w*4`, or does Skia round up for SIMD? Read the bitmap's actual
   `rowBytes()` and don't hard-code.
7. **Does `base::ThreadPool` work in our BatCave runtime?** It uses futex
   heavily; that's Phase 4 territory. If the ThreadPool task never runs,
   Hack #2 would silently drop frames. Hack #1 (sync on viz thread) sidesteps
   this.
8. **Color management:** Skia treats `N32Premul` as untagged / passthrough.
   Does Blink actually paint in sRGB, or does content_shell hand off a
   tagged surface? If tagged and we want wide-gamut later, we need
   `SkImageInfo::Make` with a proper `SkColorSpace`.
9. **How does `--disable-gpu-compositing` interact with `--disable-gpu`?**
   Both seem needed to be sure Blink doesn't try to instantiate a GPU
   channel. Verify with `--enable-logging=stderr --vmodule=*gpu*=2`.
10. **Lifetime of `FileSurface` vs its `SkSurface`:** if viz decides to
    reuse a surface across reshapes, we could blit into a size-stale fb.
    `ResizeCanvas` does reallocate the `surface_` (see §1.5 code), so OK —
    but confirm with a resize test.

---

## 9. Ready-to-Use Switch Cocktail for Phase 5

```
content_shell \
    --single-process \
    --no-zygote \
    --no-sandbox \
    --disable-gpu \
    --disable-gpu-compositing \
    --disable-software-rasterizer=false \    # keep the sw rasterizer ON
    --enable-features=UseOzonePlatform \
    --ozone-platform=headless \
    --ozone-dump-file=/dev/batfb \           # our hijack point
    --window-size=1920,1080 \
    --hide-scrollbars \
    --force-device-scale-factor=1 \
    --enable-logging=stderr --v=1 \
    https://example.com
```

`/dev/batfb` would be a BatCave special file whose `write()` handler does a
BGRA memcpy into the display framebuffer. The kernel sees: one `openat`, one
`ftruncate`/`ioctl` maybe, then per-frame `write(8 MB)`. Cheapest possible
integration.

---

## 10. File Paths Relevant to Phase 5

Upstream Chromium (read-only reference):

- `ui/ozone/platform/headless/ozone_platform_headless.cc` — entry + factory wiring
- `ui/ozone/platform/headless/headless_surface_factory.cc` — `FileSurface`, `WriteDataToFile` **← patch here**
- `ui/ozone/platform/headless/headless_surface_factory.h` — `HeadlessSurfaceFactory`
- `ui/ozone/platform/headless/headless_window.{cc,h}` — window state (no pixels)
- `ui/ozone/platform/headless/headless_window_manager.{cc,h}` — widget ID allocator
- `ui/ozone/public/surface_ozone_canvas.h` — the interface contract
- `ui/ozone/public/ozone_switches.cc` — `kOzoneDumpFile`, `kOzonePlatform`
- `components/viz/service/display_embedder/software_output_device_ozone.{cc,h}` — viz ↔ ozone glue
- `components/viz/service/display_embedder/software_output_surface.cc` — the OutputSurface layer

Bat_OS side (to be written in Phase 5):

- `src/drivers/display/chromium_blit.rs` — accepts BGRA pixels, blits to fb
- `src/batcave/linux/special_files.rs` — `/dev/batfb` (or similar) handling
- `ports/chromium_port/patches/002-ozone-blit.patch` — the actual upstream patch

---

## Sources

- Chromium source tree — `ui/ozone/platform/headless/` at
  https://chromium.googlesource.com/chromium/src/+/refs/heads/main/ui/ozone/platform/headless/
- `ui/ozone/public/surface_ozone_canvas.h` — interface
- `components/viz/service/display_embedder/software_output_device_ozone.{cc,h}`
- `headless/lib/browser/headless_web_contents_impl.cc` — `CopyFromSurface` path
- `headless/README.md` — headless vs `chrome-headless-shell` binary
- Skia — `SkImageInfo`, `kN32_SkColorType` docs
  (https://api.skia.org/SkColorType_8h.html, Skia include/core/SkColorType.h)
- Chromium Compositor Thread Architecture —
  https://www.chromium.org/developers/design-documents/compositor-thread-architecture/
- Chromium Graphics and Skia —
  https://www.chromium.org/developers/design-documents/graphics-and-skia/

End of notes.

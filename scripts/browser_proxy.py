#!/usr/bin/env python3
"""Bat_OS browser proxy — Mac-side service.

Runs headless Chromium via Playwright. Accepts HTTP requests from the
Bat_OS guest cave and returns rendered frames.

API:
  POST /render     {"url": "https://...", "w": 1280, "h": 1024}
                   → 200, body = raw BGRA bytes (w*h*4) of the rendered page
  POST /event      {"type": "click"|"key"|"scroll", "x":..., "y":..., "key":...}
                   → 200, body = raw BGRA bytes (updated frame)
  POST /goto       same as /render but reuses the existing tab (faster)
  GET  /healthz    → 200 "ok"

The cave (running in QEMU) reaches the Mac via QEMU user-mode net at
10.0.2.2. So bind 0.0.0.0:9100 on Mac, cave fetches http://10.0.2.2:9100.

Network architecture:
  ┌────────────────────┐    ┌────────────────────┐
  │ Mac host           │    │ Bat_OS in QEMU     │
  │ this script        │    │ browser-client cave│
  │ + headless Chrome  │←──→│ TCP HTTP/1.1       │
  │ on :9100           │    │ to 10.0.2.2:9100   │
  └────────────────────┘    └────────────────────┘

Usage:
  python3 scripts/browser_proxy.py
  (in another terminal:)
  curl -X POST http://localhost:9100/render \\
       -d '{"url":"https://example.com","w":800,"h":600}' > /tmp/example.bgra

Then on Bat_OS, the browser-client cave does the same POST and dumps the
returned bytes into /batos/fb0.
"""
from __future__ import annotations

import json
import struct
import sys
import threading
import time
import traceback
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from queue import Queue
from typing import Optional

try:
    from playwright.sync_api import sync_playwright, Browser, Page
except ImportError:
    print("[fatal] playwright not installed. pip3 install playwright && python3 -m playwright install chromium", file=sys.stderr)
    sys.exit(1)

try:
    from playwright_stealth import Stealth
except ImportError:
    print("[fatal] playwright-stealth not installed. pip3 install playwright-stealth", file=sys.stderr)
    sys.exit(1)

try:
    import Quartz
    QUARTZ_OK = True
except ImportError:
    QUARTZ_OK = False
    print("[warn] pyobjc-framework-Quartz not installed; /click bridge disabled. "
          "pip3 install pyobjc-framework-Quartz to enable real mouse.", file=sys.stderr)


# ────────────────────────────────────────────────────────────────────────
# Browser worker — runs in its own thread, owns the Playwright instance.
# HTTP handlers post jobs to a queue and block on a reply queue.
# Playwright's sync API isn't thread-safe, so all browser ops happen here.
# ────────────────────────────────────────────────────────────────────────

class BrowserWorker:
    def __init__(self):
        self.req_q: Queue = Queue()
        self.thread: Optional[threading.Thread] = None
        self._page: Optional[Page] = None
        self._browser: Optional[Browser] = None
        # Page rect within the QEMU window — set by Bat_OS via the
        # /set-rect endpoint when the browser app activates. Used by
        # the Mac input bridge to translate cursor coords accounting
        # for WM chrome (URL bar / bookmarks / status strip), so
        # clicks land where the user actually points within the
        # browser pane. Default to "fills the QEMU window" — which
        # is what the legacy fullscreen `web` command wants.
        self.page_rect_x: int = 0
        self.page_rect_y: int = 0
        self.page_rect_w: int = 0  # 0 = use full window
        self.page_rect_h: int = 0
        # Latest rendered frame, monotonic seq, and a Condition so /poll
        # waiters wake the moment a new frame is produced. Mac-side
        # input-capture publishes here too — every click/scroll done by
        # the bridge ends up snapshotted into LATEST_FRAME.
        self._frame_lock = threading.Lock()
        self._frame_cv = threading.Condition(self._frame_lock)
        self._frame_seq: int = 0
        self._frame_bgra: bytes = b""
        self._frame_w: int = 0
        self._frame_h: int = 0

    def publish_frame(self, bgra: bytes, w: int, h: int):
        with self._frame_cv:
            self._frame_seq += 1
            self._frame_bgra = bgra
            self._frame_w = w
            self._frame_h = h
            self._frame_cv.notify_all()

    def latest_seq(self) -> int:
        with self._frame_lock:
            return self._frame_seq

    def wait_for_frame_after(self, since_seq: int, timeout_s: float):
        """Block up to timeout_s for a frame newer than since_seq.
        Returns (bgra, w, h, seq) or None if timed out with nothing new."""
        deadline = time.time() + timeout_s
        with self._frame_cv:
            while self._frame_seq <= since_seq:
                remaining = deadline - time.time()
                if remaining <= 0:
                    return None
                self._frame_cv.wait(remaining)
            return (self._frame_bgra, self._frame_w, self._frame_h, self._frame_seq)

    def start(self):
        self.thread = threading.Thread(target=self._run, daemon=True)
        self.thread.start()

    def _run(self):
        # Layered anti-detection — Playwright's defaults leak ~30
        # fingerprint signals (canvas, WebGL, plugins, navigator.*,
        # CDP, missing chrome runtime, etc.). playwright-stealth
        # patches them all at the JS level. Pair with:
        #   - new headless mode (--headless=new) → modern Chromium
        #     that's much harder to detect than the old headless
        #   - persistent user profile → cookies/history accrue, so we
        #     look like a returning user across runs
        #   - realistic Mac Chrome UA matching navigator.platform
        ua = ("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) "
              "AppleWebKit/537.36 (KHTML, like Gecko) "
              "Chrome/131.0.0.0 Safari/537.36")
        stealth = Stealth(
            navigator_user_agent_override=ua,
            navigator_platform_override="MacIntel",
            navigator_languages_override=("en-US", "en"),
            webgl_vendor_override="Intel Inc.",
            webgl_renderer_override="Intel Iris OpenGL Engine",
        )
        profile_dir = "/tmp/batos-chromium-profile"
        with stealth.use_sync(sync_playwright()) as p:
            # Persistent context = stable identity across restarts.
            ctx = p.chromium.launch_persistent_context(
                user_data_dir=profile_dir,
                headless=True,
                args=[
                    "--headless=new",  # modern headless, less detectable
                    "--disable-blink-features=AutomationControlled",
                    "--no-first-run",
                    "--no-default-browser-check",
                    "--disable-features=Translate,InterestFeedContentSuggestions",
                ],
                viewport={"width": 1920, "height": 1080},
                user_agent=ua,
                locale="en-US",
                timezone_id="America/Los_Angeles",
            )
            self._browser = ctx.browser  # may be None for persistent ctx
            # Use the existing about:blank page or open a new one.
            self._page = ctx.pages[0] if ctx.pages else ctx.new_page()
            print(f"[browser-proxy] chromium ready (stealth+persistent at {profile_dir})", flush=True)
            while True:
                job, reply_q = self.req_q.get()
                try:
                    result = self._handle(job)
                    reply_q.put(("ok", result))
                except Exception as e:
                    traceback.print_exc()
                    reply_q.put(("err", str(e)))

    def _handle(self, job: dict) -> dict:
        op = job.get("op")
        if op == "render" or op == "goto":
            url = job["url"]
            w = int(job.get("w", 1920))
            h = int(job.get("h", 1080))
            if (w, h) != (self._page.viewport_size["width"], self._page.viewport_size["height"]):
                self._page.set_viewport_size({"width": w, "height": h})
            print(f"[browser-proxy] goto {url} @ {w}x{h}", flush=True)
            self._page.goto(url, wait_until="domcontentloaded", timeout=30000)
            # Give it a moment for above-the-fold to settle.
            self._page.wait_for_timeout(400)
            bgra, w2, h2 = self._safe_screenshot()
            self.publish_frame(bgra, w2, h2)
            return {"bgra": bgra, "w": w2, "h": h2}
        elif op == "click":
            x = int(job["x"])
            y = int(job["y"])
            self._page.mouse.click(x, y)
            # Click can navigate (anchor link). Try to wait for that
            # but don't block forever if it was just a focus change.
            try:
                self._page.wait_for_load_state("domcontentloaded", timeout=3000)
            except Exception:
                pass
            self._page.wait_for_timeout(300)
            bgra, w2, h2 = self._safe_screenshot()
            self.publish_frame(bgra, w2, h2)
            return {"bgra": bgra, "w": w2, "h": h2}
        elif op == "mousedown":
            # Move cursor to point and press button — but DON'T release.
            # Used to support press-and-hold captchas (PerimeterX etc.)
            # where the verifier measures actual hold duration. We do
            # NOT screenshot here: nothing visible has changed yet, and
            # blocking the worker on a screenshot would delay the
            # subsequent mouseup. Returns minimal ack.
            x = int(job["x"])
            y = int(job["y"])
            self._page.mouse.move(x, y)
            self._page.mouse.down()
            return {"bgra": b"", "w": 0, "h": 0}
        elif op == "mouseup":
            # Release the button. We then snapshot the page MULTIPLE
            # times over the next ~700ms so UI animations (hCaptcha
            # popup expanding, dropdowns, accordion menus, etc.) get
            # captured smoothly. With one screenshot at +400ms we
            # caught hCaptcha's challenge popup half-rendered and
            # Bat_OS would show that partial frame until the next user
            # action. Each shot calls publish_frame → /poll waiters
            # wake → Bat_OS's interactive loop paints each frame.
            x = int(job["x"])
            y = int(job["y"])
            self._page.mouse.move(x, y)
            self._page.mouse.up()
            try:
                self._page.wait_for_load_state("domcontentloaded", timeout=3000)
            except Exception:
                pass
            # Single screenshot at 250ms — fast feedback. Animations
            # that finish later (hCaptcha popup expand, Discord
            # transitions, etc.) get caught by the background frame
            # ticker on its next pass.
            self._page.wait_for_timeout(250)
            try:
                bgra, w2, h2 = self._safe_screenshot()
                self.publish_frame(bgra, w2, h2)
                return {"bgra": bgra, "w": w2, "h": h2}
            except Exception:
                return {"bgra": b"", "w": 0, "h": 0}
        elif op == "key":
            # Playwright distinguishes:
            #   keyboard.type("hello")  → types each char (h,e,l,l,o)
            #   keyboard.press("Enter") → fires the named-key event
            # Bat_OS sends BOTH through this endpoint, so we route on
            # whether the value matches a known Playwright key name.
            # Without this split, "Backspace" gets typed as nine chars
            # ("B","a","c","k","s","p","a","c","e") which corrupts the
            # focused input rather than deleting from it.
            key = job.get("key", "")
            NAMED = {
                "Enter", "Backspace", "Tab", "Escape", "Delete",
                "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight",
                "Home", "End", "PageUp", "PageDown",
                "Shift", "Control", "Alt", "Meta",
                "F1","F2","F3","F4","F5","F6","F7","F8","F9","F10","F11","F12",
            }
            if key in NAMED:
                self._page.keyboard.press(key)
            else:
                self._page.keyboard.type(key)
            # Enter typically triggers navigation (form submit, link
            # activation). 150 ms is way too short — the screenshot
            # ends up being the blank intermediate state. Wait for
            # DOM-loaded with a generous timeout, then a short
            # above-the-fold settle. For non-Enter keys, the original
            # 150 ms is fine (just typing into a field).
            if key == "Enter":
                try:
                    self._page.wait_for_load_state("domcontentloaded", timeout=5000)
                except Exception:
                    pass
                self._page.wait_for_timeout(800)
            else:
                self._page.wait_for_timeout(150)
            bgra, w2, h2 = self._safe_screenshot()
            self.publish_frame(bgra, w2, h2)
            return {"bgra": bgra, "w": w2, "h": h2}
        elif op == "snapshot":
            # Background frame ticker. Captures the current page and
            # publishes only if BGRA differs from the last published
            # frame. Lets Bat_OS see things that happened *without*
            # user input — login redirect, Discord's loading→home
            # transition, lazy-loaded content, server-pushed UI
            # changes — within ~1 ticker interval.
            bgra, w, h = self._safe_screenshot()
            h_now = hash(bgra)
            if getattr(self, '_last_pub_hash', None) == h_now:
                # No-op — don't bump seq.
                return {"bgra": bgra, "w": w, "h": h}
            self._last_pub_hash = h_now
            self.publish_frame(bgra, w, h)
            return {"bgra": bgra, "w": w, "h": h}
        elif op == "scroll":
            dy = int(job.get("dy", 100))
            self._page.mouse.wheel(0, dy)
            self._page.wait_for_timeout(150)
            bgra, w2, h2 = self._safe_screenshot()
            self.publish_frame(bgra, w2, h2)
            return {"bgra": bgra, "w": w2, "h": h2}
        else:
            raise ValueError(f"unknown op: {op}")

    def _safe_screenshot(self):
        """Take a screenshot, retrying transient Chromium errors.
        If retries fail, return the previously published frame so the
        caller (and Bat_OS) at least gets *something* renderable
        instead of a 500. Returns (bgra, w, h).

        Uses JPEG (q=85): Chromium's JPEG encoder is ~3× faster than
        its PNG encoder, and Pillow decodes either the same way. We
        still emit raw BGRA on the wire because Bat_OS doesn't have
        a JPEG decoder — the speedup is in the screenshot capture
        path, not the network."""
        last_err = None
        for attempt in range(3):
            try:
                jpg = self._page.screenshot(type="jpeg", quality=85, full_page=False)
                vw = self._page.viewport_size
                bgra = self._png_to_bgra(jpg, vw["width"], vw["height"])
                return bgra, vw["width"], vw["height"]
            except Exception as e:
                last_err = e
                # Common cause: page mid-navigation. Wait a bit and try.
                self._page.wait_for_timeout(300)
        # Fall back to last good frame if we have one.
        with self._frame_lock:
            if self._frame_bgra:
                print(f"[browser-proxy] screenshot failed ({last_err}); "
                      f"returning stale frame seq={self._frame_seq}", flush=True)
                return self._frame_bgra, self._frame_w, self._frame_h
        # Truly nothing we can do.
        raise last_err  # type: ignore

    def _png_to_bgra(self, png_bytes: bytes, w: int, h: int) -> bytes:
        # Decode PNG → RGBA via Pillow, swizzle to BGRA. Bat_OS's framebuffer
        # is BGRA8888 (matches Skia's default on little-endian).
        from PIL import Image
        import io
        img = Image.open(io.BytesIO(png_bytes)).convert("RGBA")
        if img.size != (w, h):
            img = img.resize((w, h))
        rgba = img.tobytes()
        # RGBA → BGRA: swap R and B per pixel.
        ba = bytearray(rgba)
        for i in range(0, len(ba), 4):
            ba[i], ba[i + 2] = ba[i + 2], ba[i]
        return bytes(ba)

    def submit(self, job: dict) -> dict:
        reply_q: Queue = Queue()
        self.req_q.put((job, reply_q))
        status, result = reply_q.get(timeout=60)
        if status == "err":
            raise RuntimeError(result)
        return result


WORKER = BrowserWorker()


# ────────────────────────────────────────────────────────────────────────
# Mac-side input bridge.
#
# Why this exists: virtio-tablet on QEMU's macOS cocoa backend silently
# drops EV_ABS pointer events (we confirmed via tracing — clicks generate
# zero events on the guest's input ring). virtio-mouse delivers events
# but cocoa grabs the host cursor as a side effect, which is bad UX.
#
# So instead of routing mouse through QEMU's input device at all, we
# capture it on the host: a thread polls Quartz for cursor position +
# button state, finds the QEMU window's screen rect, computes a
# guest-relative coordinate, and fires `page.mouse.click()` /
# `page.mouse.wheel()` directly into Chromium. Bat_OS's interactive
# loop polls /poll for the resulting frame.
#
# No accessibility permission needed — `CGEventGetLocation` and
# `CGEventSourceButtonState` work without prompts. (Real CGEventTap
# scroll capture WOULD need accessibility; we approximate by using
# CGEventSourceButtonState for the wheel and capturing scroll deltas
# via a tiny event tap if we can — disabled by default for now since
# we already have keyboard-driven scroll.)
# ────────────────────────────────────────────────────────────────────────

class InputBridge:
    POLL_HZ = 60             # mouse poll rate
    # FB dimensions are dynamic — pulled from `worker._page.viewport_size`
    # on every tick. Bat_OS sends w/h in each /render call, the worker
    # resizes the viewport to match, and the bridge reads that back.
    # No hardcoded constants → no drift between OS and bridge.
    DEFAULT_FB_W = 1280
    DEFAULT_FB_H = 1024

    def __init__(self, worker: 'BrowserWorker'):
        self.worker = worker
        self.thread: Optional[threading.Thread] = None
        self.qemu_pid: Optional[int] = None
        self.last_btn = False
        self.last_pos = (0, 0)
        # Track whether WE sent a mousedown — separate from the
        # physical button state. If the user pressed outside the QEMU
        # window (we didn't send mousedown), then released inside, we
        # must NOT send an orphan mouseup; the page state would be
        # inconsistent.
        self.bridge_held = False

    def start(self):
        if not QUARTZ_OK:
            print("[input-bridge] disabled (Quartz unavailable)", flush=True)
            return
        self.thread = threading.Thread(target=self._run, daemon=True)
        self.thread.start()
        print("[input-bridge] started — left-click in QEMU window forwards to Chromium", flush=True)

    def _find_qemu_window(self):
        """Returns (x, y, w, h) of QEMU window's screen rect, or None.

        QEMU on cocoa creates ghost title-bar slices when sessions
        end abruptly (~33px-tall windows that linger). And cocoa
        full-screen places the real window on its own Mission Control
        space, where CGWindowList from another space sees only those
        same tiny slices. So:
          (1) Match all 'qemu' windows and pick the one with the
              LARGEST area — that's the actual visible content.
          (2) If even the largest is anomalously short (< 100px),
              we're probably looking at fullscreen-on-another-space;
              fall back to the display rect.
        """
        windows = Quartz.CGWindowListCopyWindowInfo(
            Quartz.kCGWindowListExcludeDesktopElements,
            Quartz.kCGNullWindowID,
        )
        best_area = 0
        bounds = None
        for w in windows:
            name = w.get('kCGWindowOwnerName', '')
            if 'qemu' in name.lower():
                b = w.get('kCGWindowBounds', {})
                bw, bh = b.get('Width', 0), b.get('Height', 0)
                area = bw * bh
                if area > best_area:
                    best_area = area
                    bounds = (b.get('X', 0), b.get('Y', 0), bw, bh)
        if bounds is None:
            return None
        x, y, w, h = bounds
        if h < 100:
            # Almost certainly fullscreen-on-another-space. Use the
            # display rect that contains the window's anchor point.
            err, displays, count = Quartz.CGGetActiveDisplayList(8, None, None)
            if err == 0:
                for did in list(displays)[:count]:
                    db = Quartz.CGDisplayBounds(did)
                    if (db.origin.x <= x < db.origin.x + db.size.width
                            and db.origin.y <= y < db.origin.y + db.size.height):
                        return (db.origin.x, db.origin.y,
                                db.size.width, db.size.height)
            # Fallback: main display.
            main = Quartz.CGMainDisplayID()
            db = Quartz.CGDisplayBounds(main)
            return (db.origin.x, db.origin.y, db.size.width, db.size.height)
        return bounds

    def _fb_dims(self):
        """Read the current FB size from the worker's Playwright page.
        Bat_OS calls /render with whatever gpu::width/height returned
        from virtio-gpu's DISPLAY_INFO query, the worker resizes the
        viewport to match, and we read it back. Falls back to the
        default if Playwright hasn't initialised yet."""
        try:
            v = self.worker._page.viewport_size  # type: ignore[union-attr]
            return v["width"], v["height"]
        except Exception:
            return self.DEFAULT_FB_W, self.DEFAULT_FB_H

    def _screen_to_fb(self, sx: float, sy: float, win_rect):
        """Translate global cursor pos → FB coord, or None if outside.

        Three reference frames in play:
          - host screen coords (what Quartz gives us as cursor pos)
          - QEMU-window coords (subtract win_rect origin + cocoa
            title bar = 32px in windowed, 0 in fullscreen)
          - page-rect coords (subtract page_rect origin set by
            Bat_OS — WM chrome offset when browser app is active)

        If page_rect is unset (w==0), we map directly window→FB —
        the legacy fullscreen `web` command. If set, clicks outside
        the page rect return None (don't fire) and inside-rect coords
        get mapped 1:1 to FB dims.
        """
        x, y, w, h = win_rect
        fb_w, fb_h = self._fb_dims()

        if self.worker.page_rect_w > 0 and self.worker.page_rect_h > 0:
            # Browser-app mode: the page lives in a sub-rect of the
            # virtio-gpu surface, defined in QEMU-window pixel space.
            # The QEMU window on screen is `w` host-points wide, but
            # the underlying virtio-gpu surface is `gpu_w` pixels.
            # We need to know `gpu_w` to scale sx,sy → gpu pixels
            # before subtracting page_rect_x/y. Bat_OS sends the
            # rect in gpu pixels, so gpu_w/gpu_h are the kernel's
            # gpu::width/height — the same values it uses to derive
            # the rect. We can recover gpu_w by aspect-matching
            # against page_rect: if the rect is sized for a 1512×950
            # surface, then gpu_w must be 1512.
            #
            # Practically: the kernel always sends the rect at the
            # same gpu-pixel scale that the proxy renders Chrome
            # at. Chrome's viewport_size = page_rect_w × page_rect_h
            # (because Bat_OS asked /render at that size). So
            # fb_w == page_rect_w. We need an SCALE from window
            # host-points to gpu pixels — assume 1:1 for now (most
            # cocoa configs at non-retina) and scale linearly off
            # the window aspect.
            #
            # Title bar detection: the QEMU window's rendered area
            # height should match (window_w * gpu_h / gpu_w). When
            # there's a title bar, h_window > view_h.
            prx = self.worker.page_rect_x
            pry = self.worker.page_rect_y
            prw = self.worker.page_rect_w
            prh = self.worker.page_rect_h
            # Estimate the gpu surface dims from the page rect's
            # known relationship: gpu_w == prx + prw + (right slack
            # 0). We can't know exactly, but the kernel renders the
            # surface at full gpu width so prx + prw <= gpu_w. Use
            # the QEMU window's width (in host points) → assume 1:1
            # to gpu pixels, since QEMU virtio-gpu surface is shown
            # at native size on cocoa with show-cursor=on.
            gpu_w = w  # host points → gpu pixels (1:1 on cocoa)
            gpu_h_estimated = prh + 200  # slop; not used for math
            _ = gpu_h_estimated
            # Title-bar detection by ratio: if window aspect matches
            # gpu_w/gpu_h_full, no title bar.
            view_y = y
            # Practical: cocoa always reports an extra 32px above
            # the surface for the title bar in windowed mode, 0 in
            # fullscreen. Aspect-check.
            # Approximate gpu_h from page rect + chrome offset:
            # nothing reliable here; just check 32px title bar.
            # If window height looks like it includes a title bar
            # (h > prh + chrome_estimate + 50), subtract 32.
            # Use a simpler rule: if h < gpu_w * 0.7 it's likely
            # fullscreen (very wide screen), else windowed.
            if h > w:  # tall window → likely has title bar
                view_y = y + 32
            view_h = (y + h) - view_y
            if sx < x or sx >= x + w or sy < view_y or sy >= view_y + view_h:
                return None
            # Window-pixel cursor (gpu pixel space).
            wx = sx - x
            wy = sy - view_y
            if wx < prx or wx >= prx + prw or wy < pry or wy >= pry + prh:
                return None
            fx = int(wx - prx)
            fy = int(wy - pry)
            fx = max(0, min(prw - 1, fx))
            fy = max(0, min(prh - 1, fy))
            return (fx, fy)

        # Legacy fullscreen path — fits viewport=fb_w/fb_h.
        expected_view_h = w * fb_h / max(fb_w, 1)
        title_bar = 0 if abs(h - expected_view_h) < 6 else 32
        view_y = y + title_bar
        view_h = h - title_bar
        if sx < x or sx >= x + w or sy < view_y or sy >= view_y + view_h:
            return None
        fx = int((sx - x) * fb_w / max(w, 1))
        fy = int((sy - view_y) * fb_h / max(view_h, 1))
        fx = max(0, min(fb_w - 1, fx))
        fy = max(0, min(fb_h - 1, fy))
        return (fx, fy)

    def _run(self):
        period = 1.0 / self.POLL_HZ
        while True:
            try:
                self._tick()
            except Exception:
                traceback.print_exc()
            time.sleep(period)

    def _tick(self):
        win = self._find_qemu_window()
        if win is None:
            return
        ev = Quartz.CGEventCreate(None)
        loc = Quartz.CGEventGetLocation(ev)
        sx, sy = loc.x, loc.y
        btn = bool(Quartz.CGEventSourceButtonState(
            Quartz.kCGEventSourceStateHIDSystemState,
            Quartz.kCGMouseButtonLeft))

        # Edge detection: split DOWN and UP into separate Playwright
        # actions so the held duration matches the user's actual hold.
        # This is what makes press-and-hold captchas (PerimeterX et al)
        # work — they measure the time between mousedown and mouseup
        # to filter out instant programmatic clicks.
        if not self.last_btn and btn:
            # Down edge — only send if cursor is inside QEMU window.
            fb = self._screen_to_fb(sx, sy, win)
            if fb is not None:
                fx, fy = fb
                print(f"[input-bridge] down {fx},{fy}", flush=True)
                try:
                    self.worker.submit({"op": "mousedown", "x": fx, "y": fy})
                    self.bridge_held = True
                except Exception as e:
                    print(f"[input-bridge] mousedown failed: {e}", flush=True)
        elif self.last_btn and not btn:
            # Up edge — only send if WE actually sent a corresponding
            # mousedown. Otherwise it's an orphan (user pressed outside
            # the window or before bridge started).
            if self.bridge_held:
                fb = self._screen_to_fb(sx, sy, win)
                if fb is not None:
                    fx, fy = fb
                    print(f"[input-bridge] up {fx},{fy}", flush=True)
                    try:
                        self.worker.submit({"op": "mouseup", "x": fx, "y": fy})
                    except Exception as e:
                        print(f"[input-bridge] mouseup failed: {e}", flush=True)
                else:
                    # Released outside window while we had pressed
                    # inside — release at the last in-window position
                    # to keep Playwright's state consistent.
                    print(f"[input-bridge] up (out-of-window release; using last)", flush=True)
                    try:
                        self.worker.submit({"op": "mouseup",
                                            "x": self.last_pos[0],
                                            "y": self.last_pos[1]})
                    except Exception as e:
                        print(f"[input-bridge] mouseup failed: {e}", flush=True)
                self.bridge_held = False
            else:
                print(f"[input-bridge] orphan up; skipping", flush=True)
        self.last_btn = btn
        self.last_pos = (sx, sy)


INPUT_BRIDGE = InputBridge(WORKER)


class FrameTicker:
    """Background thread that periodically snapshots the page so
    out-of-band changes (login responses, server-pushed UI updates,
    lazy-loaded content) reach Bat_OS even when the user isn't
    actively clicking. The worker hashes BGRA and skips publish if
    nothing changed, so idle pages don't burn /poll cycles."""
    INTERVAL_S = 0.4

    def __init__(self, worker: 'BrowserWorker'):
        self.worker = worker

    def start(self):
        threading.Thread(target=self._run, daemon=True).start()
        print(f"[frame-ticker] started — auto-snapshot every {self.INTERVAL_S}s",
              flush=True)

    def _run(self):
        while True:
            time.sleep(self.INTERVAL_S)
            # Skip if there's user-initiated work pending — don't
            # contend with mousedown/up/key/poll for the worker.
            if self.worker.req_q.qsize() > 0:
                continue
            try:
                self.worker.submit({"op": "snapshot"})
            except Exception:
                pass


FRAME_TICKER = FrameTicker(WORKER)


# ────────────────────────────────────────────────────────────────────────
# HTTP handler.
# ────────────────────────────────────────────────────────────────────────

class Handler(BaseHTTPRequestHandler):
    # HTTP/1.0 — one request per connection. HTTP/1.1 keep-alive is a
    # latent goal but had a subtle Bat_OS-side recv-loop hang we
    # haven't pinned down yet, so reverted to be safe.
    protocol_version = "HTTP/1.0"

    def log_message(self, fmt, *args):
        pass  # quiet; the worker prints what it does

    def _read_json(self) -> dict:
        n = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(n) if n else b""
        return json.loads(body or b"{}")

    def _ok_bgra(self, bgra: bytes, w: int, h: int):
        self.send_response(200)
        self.send_header("Content-Type", "application/octet-stream")
        self.send_header("Content-Length", str(len(bgra)))
        self.send_header("X-BatOS-Width", str(w))
        self.send_header("X-BatOS-Height", str(h))
        # Bat_OS uses this to detect "already loaded" — without it
        # REMOTE_SEQ stays 0 and on_activate re-fires on every Tab.
        self.send_header("X-BatOS-Seq", str(WORKER.latest_seq()))
        self.end_headers()
        self.wfile.write(bgra)

    def _err(self, code: int, msg: str):
        self.send_response(code)
        self.send_header("Content-Type", "text/plain")
        self.end_headers()
        self.wfile.write(msg.encode())

    def do_GET(self):
        if self.path == "/healthz":
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.end_headers()
            self.wfile.write(b"ok\n")
        else:
            self._err(404, "not found")

    def do_POST(self):
        # /set-rect is special: tells the bridge where in the QEMU
        # window the renderable page lives (so it can subtract WM
        # chrome before mapping clicks). Body: {"x":N,"y":N,"w":N,"h":N}
        # All coords are in QEMU-window space (i.e. relative to the
        # virtio-gpu surface origin). Empty/zero w/h disables the
        # offset (returns to "page fills window" mode).
        if self.path == "/set-rect":
            try:
                req = self._read_json()
            except Exception:
                req = {}
            WORKER.page_rect_x = int(req.get("x", 0))
            WORKER.page_rect_y = int(req.get("y", 0))
            WORKER.page_rect_w = int(req.get("w", 0))
            WORKER.page_rect_h = int(req.get("h", 0))
            print(f"[browser-proxy] page-rect set: "
                  f"({WORKER.page_rect_x},{WORKER.page_rect_y}) "
                  f"{WORKER.page_rect_w}x{WORKER.page_rect_h}", flush=True)
            self.send_response(200)
            self.send_header("Content-Length", "0")
            self.end_headers()
            return

        # /poll is special: it long-polls for the latest frame and
        # returns 304 if nothing's changed since the caller's `since`.
        # Bat_OS calls this in its interactive loop so frames produced
        # by Mac-side mouse capture (via InputBridge) reach the FB.
        if self.path == "/poll":
            try:
                req = self._read_json()
            except Exception:
                req = {}
            since = int(req.get("since", 0))
            timeout = float(req.get("timeout", 0.5))
            result = WORKER.wait_for_frame_after(since, timeout)
            if result is None:
                # Nothing new — let caller poll again.
                self.send_response(304)
                self.send_header("Content-Length", "0")
                self.end_headers()
                return
            bgra, w, h, seq = result
            self.send_response(200)
            self.send_header("Content-Type", "application/octet-stream")
            self.send_header("Content-Length", str(len(bgra)))
            self.send_header("X-BatOS-Width", str(w))
            self.send_header("X-BatOS-Height", str(h))
            self.send_header("X-BatOS-Seq", str(seq))
            self.end_headers()
            self.wfile.write(bgra)
            return

        try:
            req = self._read_json()
        except Exception as e:
            self._err(400, f"bad json: {e}")
            return
        op = self.path.lstrip("/")
        # Map URL paths to ops.
        op_map = {"render": "render", "goto": "goto", "click": "click",
                  "key": "key", "scroll": "scroll"}
        if op not in op_map:
            self._err(404, f"unknown op: {op}")
            return
        req["op"] = op_map[op]
        try:
            result = WORKER.submit(req)
            self._ok_bgra(result["bgra"], result["w"], result["h"])
        except Exception as e:
            traceback.print_exc()
            self._err(500, str(e))


def main():
    WORKER.start()
    INPUT_BRIDGE.start()
    FRAME_TICKER.start()
    addr = ("0.0.0.0", 9100)
    httpd = ThreadingHTTPServer(addr, Handler)
    print(f"[browser-proxy] listening on http://{addr[0]}:{addr[1]}", flush=True)
    print(f"[browser-proxy] from QEMU guest: http://10.0.2.2:{addr[1]}", flush=True)
    print(f"[browser-proxy] test: curl -X POST http://localhost:{addr[1]}/render -d '{{\"url\":\"https://example.com\"}}' -o /tmp/x.bgra", flush=True)
    httpd.serve_forever()


if __name__ == "__main__":
    main()

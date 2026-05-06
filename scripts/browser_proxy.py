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
import traceback
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from queue import Queue
from typing import Optional

try:
    from playwright.sync_api import sync_playwright, Browser, Page
except ImportError:
    print("[fatal] playwright not installed. pip3 install playwright && python3 -m playwright install chromium", file=sys.stderr)
    sys.exit(1)


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

    def start(self):
        self.thread = threading.Thread(target=self._run, daemon=True)
        self.thread.start()

    def _run(self):
        with sync_playwright() as p:
            self._browser = p.chromium.launch(headless=True)
            self._page = self._browser.new_page(viewport={"width": 1280, "height": 1024})
            print("[browser-proxy] chromium ready (headless)", flush=True)
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
            w = int(job.get("w", 1280))
            h = int(job.get("h", 1024))
            if (w, h) != (self._page.viewport_size["width"], self._page.viewport_size["height"]):
                self._page.set_viewport_size({"width": w, "height": h})
            print(f"[browser-proxy] goto {url} @ {w}x{h}", flush=True)
            self._page.goto(url, wait_until="domcontentloaded", timeout=30000)
            # Give it a moment for above-the-fold to settle.
            self._page.wait_for_timeout(400)
            png = self._page.screenshot(type="png", full_page=False)
            bgra = self._png_to_bgra(png, w, h)
            return {"bgra": bgra, "w": w, "h": h}
        elif op == "click":
            x = int(job["x"])
            y = int(job["y"])
            self._page.mouse.click(x, y)
            self._page.wait_for_timeout(200)
            png = self._page.screenshot(type="png", full_page=False)
            vw = self._page.viewport_size
            bgra = self._png_to_bgra(png, vw["width"], vw["height"])
            return {"bgra": bgra, "w": vw["width"], "h": vw["height"]}
        elif op == "key":
            self._page.keyboard.type(job.get("key", ""))
            self._page.wait_for_timeout(150)
            png = self._page.screenshot(type="png", full_page=False)
            vw = self._page.viewport_size
            bgra = self._png_to_bgra(png, vw["width"], vw["height"])
            return {"bgra": bgra, "w": vw["width"], "h": vw["height"]}
        elif op == "scroll":
            dy = int(job.get("dy", 100))
            self._page.mouse.wheel(0, dy)
            self._page.wait_for_timeout(150)
            png = self._page.screenshot(type="png", full_page=False)
            vw = self._page.viewport_size
            bgra = self._png_to_bgra(png, vw["width"], vw["height"])
            return {"bgra": bgra, "w": vw["width"], "h": vw["height"]}
        else:
            raise ValueError(f"unknown op: {op}")

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
# HTTP handler.
# ────────────────────────────────────────────────────────────────────────

class Handler(BaseHTTPRequestHandler):
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
    addr = ("0.0.0.0", 9100)
    httpd = ThreadingHTTPServer(addr, Handler)
    print(f"[browser-proxy] listening on http://{addr[0]}:{addr[1]}", flush=True)
    print(f"[browser-proxy] from QEMU guest: http://10.0.2.2:{addr[1]}", flush=True)
    print(f"[browser-proxy] test: curl -X POST http://localhost:{addr[1]}/render -d '{{\"url\":\"https://example.com\"}}' -o /tmp/x.bgra", flush=True)
    httpd.serve_forever()


if __name__ == "__main__":
    main()

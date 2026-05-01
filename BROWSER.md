# Bat_OS Browser

A native browser engine running inside the Bat_OS kernel. Not a port of
Chromium — kernel-level Rust code that fetches, parses, lays out, and
paints HTML/CSS/JS, with security policies enforced *below* the
renderer (so a compromised page can't lift them).

This document is the doorway. Read `DESIGN_BROWSER.md` for the
architecture, `docs/SESSION_JOURNAL.md` for the build log
(STUMP #65–#108 are the renderer's history).

---

## What works today

**Network**
- HTTP/1.0 GET + POST (form-urlencoded body)
- HTTPS GET + POST via TLS 1.3 (X25519, AES-128-GCM)
- DNS via QEMU user-mode resolver (10.0.2.3)
- Cookie jar — Set-Cookie ingest + Cookie request header on subsequent
  fetches to the same host
- Same-origin policy enforcement on `<link>`, `<img>`, JS `fetch()`

**Parsing**
- HTML5-ish parser: tags, attributes, void elements, common entities
- UTF-8 decode at parse time with ASCII fallback for the entire Latin-1
  supplement, General Punctuation, currency, ©®™ etc.
- CSS parser: tag selectors, classes, IDs, descendant combinators,
  inline `style=`, `<style>` blocks, `<link rel=stylesheet>`
- JavaScript: bytecode VM with NaN-boxed values, ~100 opcodes, native
  function bridge to DOM and built-ins

**Layout & paint**
- Block + inline flow with margin collapsing
- `<table>` with column-aligned two-pass layout
- Flexbox (row + column, justify-content, align-items, gap)
- `position: static / relative / absolute / fixed / sticky`
- Word-wrap with TT-measured per-word widths (no mid-word breaks)
- Box-shadow with Gaussian-ish soft falloff
- Italic via per-row pixel shear, monospace via fixed-advance override
- TrueType glyph rendering (Verdana subset, ASCII coverage)
- PNG `<img>` decoding + alpha blending

**Interactivity** (Sprint 1)
- Hit-testing the live layout tree
- `click_xy=<x>,<y>` shell arg + Ctrl+E in interactive mode
- Click on `<a href>` → fetch + re-layout + paint in place
- Click on submit button → walk form → urlencode body → POST
- Form fill: type into focused `<input>` / `<textarea>` mutates `value`
  attribute, re-layouts on every keystroke
- virtio-tablet driver + virtio-keyboard for live demo
- Live virtio-gpu output via `make render-live`

**Security** (Sprint 2)
- TLS verification mode: `lockdown` / `research` / `open`
  (default lockdown — only pinned hosts handshake)
- Same-origin policy with per-pair allowlist
- Global JS execute toggle (`js-mode off` for sensitive contexts)
- Append-only audit log of every fetch / script / form-submit / mode
  flip (1024-entry ring, BatFS flush available)
- Per-cave reset on switch wipes cookies, localStorage, JS heap, DOM,
  TLS sessions, TCP state — 14+ subsystems hooked

**Web platform** (Sprint 3)
- HTTP cookies (jar + Set-Cookie + Cookie header)
- `localStorage` (getItem / setItem / removeItem / clear)
- Synchronous `fetch_sync(url)` from JS — same SOP/TLS/audit pipeline
  as the Rust-side fetcher
- UTF-8 decode + ASCII fallback at parse time

---

## What does NOT work yet

- JPEG / GIF / WebP decoders (PNG only)
- Real Unicode font fallback — non-Latin scripts (Greek, CJK, Arabic)
  render as `?` placeholders
- HTTP/2, WebSocket
- Promises / async-await / event loop in JS
- Service workers, IndexedDB, WebRTC, WebGL
- Full per-origin BatCaves (SOP enforces the same policy in fewer LoC)
- TLS for HTTPS hosts that hard-require non-X25519 key shares or
  reject our minimal ClientHello fingerprint (HN, httpbin, …)
- `<script src=...>` external script loading (only inline `<script>`
  tags are picked up)

---

## Demos

The simplest path: run the renderer headless and read the output as a
PNG screenshot.

```sh
make render URL=file:///bin/showcase.html      # local file
make render URL=http://example.com/             # live HTTP
make render URL=https://example.com/            # live HTTPS (TLS 1.3)
```

PNGs land in `logs/qemu-tests/render-<timestamp>.png`. Pages taller
than 1900 px paginate into `*.p1.png`, `*.p2.png`, …

### Live (windowed) demo

```sh
make render-live URL=http://10.0.2.2:8765/
```

Opens a QEMU window with virtio-gpu, virtio-keyboard, and
virtio-tablet attached. Once the kernel boots:

1. Type `batman` Enter at the bat-logo screen.
2. Type `render <URL> live=1` Enter at the shell.
3. Use the keyboard cursor:
   - `Ctrl+W` / `Ctrl+A` / `Ctrl+S` / `Ctrl+D` — move cursor
   - `Ctrl+E` — click at cursor
   - `Ctrl+G` — recenter cursor
   - typing — into focused `<input>` after a Ctrl+E
   - `ESC` — exit interactive loop

Why the keyboard cursor? QEMU's cocoa display backend on macOS doesn't
deliver mouse motion to the virtio-tablet device. Cursor-via-keys is
the workaround. Linux + GTK or SDL works the normal way.

### Form POST round-trip

A test echo server lives in `/tmp/bat_os_http_test/echo_server.py`:

```sh
python3 /tmp/bat_os_http_test/echo_server.py &
make render-live URL=http://10.0.2.2:8765/
# bat-logo: batman + Enter
# shell:    render http://10.0.2.2:8765/ live=1 + Enter
# Ctrl+G, Ctrl+S to email field, Ctrl+E, type, repeat for password,
# Ctrl+S to Submit, Ctrl+E.
```

The form POSTs, the server echoes back the urlencoded body, the
response page renders in place.

---

## Shell commands

In the kernel shell (after `make render-live` and auth):

| Command | What it does |
|---|---|
| `render <url> [live=1] [click_xy=x,y] [type=id=value] [click=id]` | render a page |
| `dump-dom <url>` | dump the parsed DOM tree |
| `tls-mode [lockdown\|research\|open]` | TLS verification policy |
| `js-mode [on\|off]` | global JS execute toggle |
| `audit [N\|all]` | dump the last N audit-log entries |
| `audit-flush` | serialize the ring → `/audit.log` in BatFS |
| `origin` | current main origin + SOP allowlist |
| `origin-allow <main-host> <other-host>` | add cross-origin pair |
| `origin-mode [strict\|permissive]` | flip SOP enforcement |
| `cookies` | list active cookies (host + name; values redacted) |
| `cookies clear` | wipe the jar |

---

## Threat model

The browser is engineered for **government / private-client grade
secure operator workflows** — not for a public-internet best-effort
browse:

- **Lockdown by default.** TLS pinning is on; only allowlisted hosts
  handshake. Operator opts into `tls-mode research` to browse
  arbitrary hosts.
- **No accidental exfiltration.** Cross-origin sub-resource fetches
  (`<img>`, `<link>`, JS `fetch()`) are blocked unless the operator
  has explicitly allowlisted the (main, other) host pair. Hostile
  JS in origin A can't ping attacker.com.
- **Audit log is non-optional.** Every fetch / script / form / mode
  flip lands in the ring, regardless of whether the operator looked.
  Privacy: form bodies / cookie values / passphrases logged by byte
  count only, never contents.
- **JS is a kill switch, not a sandbox.** `js-mode off` disables the
  VM entirely for sensitive contexts. Reading a classified policy
  doc with embedded JS off → zero exfil surface.
- **Cave switch is hard reset.** Cookies, localStorage, JS heap,
  TCP state, TLS sessions, DOM all wiped on `cave::enter` so a
  logged-out cave doesn't inherit the previous tenant's session.

What this is NOT designed for:
- Untrusted-page rendering as a security boundary on its own.
  That's what per-origin BatCaves would add (deferred — see journal).
- Defeating a kernel-resident attacker. Once you're at ring 0, you
  read the audit ring directly. That's a different problem.

---

## How the codebase is laid out

```
src/
  browser/
    dom.rs              fixed-size DOM (4096 nodes max, attrs as flat arrays)
    html/parser.rs      HTML5-ish parser + UTF-8 decode + entity decode
    css/                tokenizer, parser, selector matcher
    layout/mod.rs       block + inline + flex + table layout
    paint/mod.rs        TrueType + PNG paint into a u32 framebuffer
    js/
      vm.rs             bytecode VM (NaN-boxed values, ~100 opcodes)
      compiler.rs       AST → bytecode
      dom_api.rs        document.getElementById, setAttribute, …
      storage.rs        localStorage backing store
      ...
    media/img_pool.rs   PNG decoder + ARGB cache
  net/
    fetch.rs            fetch_url / fetch_post_url (HTTP + HTTPS, cookies)
    cookies.rs          host+name jar
    tls.rs              TLS 1.3 client (X25519 + AES-128-GCM)
    tls_pinning.rs      lockdown/research/open mode + PINS list
  security/
    audit.rs            append-only ring (#103)
    origin.rs           same-origin policy (#104)
  ui/
    shell.rs            cmd_render + interactive_loop + click_xy + cursor
    apps/browser.rs     BatBrowser app (text-mode, separate from cmd_render)
  drivers/virtio/
    keyboard.rs         virtio-keyboard
    tablet.rs           virtio-tablet (#98)
    gpu.rs              virtio-gpu framebuffer
```

---

## Status

22 stumps shipped over the renderer push (#86 through #108). Detailed
log in `docs/SESSION_JOURNAL.md` — every stump has a date, a symptom,
a diagnosis, and a commit. Read backwards from the end to understand
how the current state was reached.

Branch: `feat/js-engine-browser-posix`.

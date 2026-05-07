# BatBrowser Engine — Full Rendering Engine Plan

> **Scope of this doc.** This describes the **native** BatBrowser engine
> in `src/browser/` (~16K Rust, zero deps, bare metal). It is one of
> three browser paths now coexisting on Bat_OS:
>
> 1. **BatBrowser (native)** — this doc. Pure-Rust engine. Engineering pride.
> 2. **Ladybird port** — `port/ladybird` branch. BSD-licensed independent
>    browser (LibWeb + Skia + LibJS). As of iter 28, real DOM → Layout →
>    Skia paint → `/batos/fb0` works. Engineering achievement.
> 3. **stream-client** — `cmd_web` shell command. Mac-side headless
>    Chromium via Playwright (`scripts/browser_proxy.py`); Bat_OS receives
>    BGRA. Pragmatic daily driver.
>
> Which becomes "the" Bat_OS browser is an open strategic question
> (see `docs/SESSION_JOURNAL.md` 2026-05-07 entry). This doc is the
> roadmap for path 1 only.

## The Mission
Build a real web rendering engine from scratch inside Bat_OS.
Pure Rust, zero dependencies, bare metal. Every byte auditable.

## Current State (as of 2026-05-07, port/ladybird branch)

The native engine has reached **roughly Level 4** (Levels 1–3 done,
Level 4 substantially done). Reality vs the original Level 0 plan:

- HTTP/HTTPS fetching (TLS 1.3 — X25519/AES-256-GCM, hybrid PQ
  X25519+ML-KEM-768 implemented but disabled by default) ✅
- HTML parser → DOM tree (`src/browser/html/parser.rs`, ~855 LOC) ✅
- CSS tokenizer + selector matching + cascade (`src/browser/css/`,
  ~1,070 LOC) ✅
- Block + inline + flex layout (`src/browser/layout/`, ~660 LOC) ✅
- Paint to bitmap (`src/browser/paint/`, ~500 LOC) ✅
- TrueType font rasterizer (`src/ui/truetype.rs`, `src/ui/font.rs`) ✅
- PNG + JPEG + gzip decoders (`src/browser/media/`, ~1,360 LOC) ✅
- JS engine — lexer + parser + bytecode compiler + VM (`src/browser/js/`,
  ~5,800 LOC: vm.rs 1,900, compiler.rs 1,260, parser.rs 1,013,
  builtins.rs 970, dom_api.rs 673) ✅
- DOM API from JS (querySelector, classList, textContent, addEventListener) ✅
- localStorage / sessionStorage ✅
- Chrome-style UI (tabs, nav bar, bookmarks, status bar) ✅
- Live HTTPS Wikipedia render (verified 2026-04-30) ✅

---

## Level 1 — Styled Text Renderer ✅ DONE
**Lines: ~300 | Time: 1 session**

What it does:
- Different colors per HTML element (h1=white, p=gray, a=blue)
- Heading sizes (h1 pixel-doubled, h2 bold, h3 normal+bright)
- Fake bold (draw text twice with 1px offset)
- Paragraph spacing (blank line between <p> blocks)
- List bullets (• before <li> items)
- Horizontal rules (<hr> → line across page)
- Blockquote indentation
- <code> and <pre> with different background color

What it looks like:
- Wikipedia articles are readable with clear heading hierarchy
- Blog posts look like actual blog posts
- Documentation sites are usable

---

## Level 2 — DOM Tree + Basic CSS ✅ DONE
**Lines: ~5,000 | Time: 2-3 sessions**

### 2a: HTML Parser → DOM Tree
- Tokenizer: converts HTML bytes into tokens (start tag, end tag, text, comment)
- Tree builder: tokens → tree of nodes (Element, Text, Comment)
- Node structure: tag name, attributes (HashMap-like), children list, parent pointer
- Handle self-closing tags (<br>, <img>, <hr>, <meta>)
- Handle malformed HTML (unclosed tags, nested errors)

### 2b: CSS Parser
- Parse <style> blocks and style="" attributes
- Property parser: color, background-color, font-size, font-weight,
  margin, padding, display, text-align, border, width, height
- Selector parser: element selectors (h1, p, a), class selectors (.foo),
  ID selectors (#bar), descendant selectors (div p)
- Cascade: inline styles > ID > class > element > inherited
- Specificity calculation

### 2c: Layout Engine (Block + Inline)
- Box model: content + padding + border + margin
- Block layout: elements stack vertically, full width
- Inline layout: elements flow left-to-right, wrap at container edge
- Line breaking: split text at word boundaries
- Display property: block, inline, none
- Width/height: auto, fixed px, percentage
- Margin collapse (top/bottom margins merge)

### 2d: Paint
- Walk the layout tree, draw each box
- Background colors
- Border drawing (solid, with color)
- Text rendering within boxes
- Z-ordering (later elements paint over earlier ones)

What it looks like:
- Pages have proper structure (not just a wall of text)
- Colored backgrounds, borders visible
- Text wraps correctly within containers
- Looks like a 2005 website

---

## Level 3 — Images + Fonts ✅ DONE
**Lines: ~15,000 | Time: 2-3 weeks**

### 3a: PNG Decoder
- DEFLATE decompression (zlib)
  - Huffman coding
  - LZ77 sliding window
- PNG chunk parsing (IHDR, IDAT, IEND, PLTE)
- Pixel reconstruction (filtering: None, Sub, Up, Average, Paeth)
- Color types: grayscale, RGB, RGBA, palette
- Render decoded pixels to framebuffer
- Scale images to fit container

### 3b: JPEG Decoder
- Huffman decoding
- Inverse DCT (8x8 blocks)
- YCbCr → RGB conversion
- Chroma subsampling (4:2:0, 4:2:2, 4:4:4)

### 3c: TrueType Font Renderer
- Parse .ttf file (table directory, cmap, glyf, hmtx, head)
- Character → glyph mapping (cmap table)
- Glyph outline extraction (contours, on/off curve points)
- Bezier curve rasterization (quadratic B-splines)
- Horizontal metrics (advance width, left side bearing)
- Variable-width text layout (proportional fonts)
- Font sizes via scaling

### 3d: <img> Element Support
- Parse src attribute
- Fetch image via HTTP/HTTPS
- Decode PNG/JPEG
- Render in layout at specified width/height
- Alt text fallback

What it looks like:
- Images appear inline with text
- Proper proportional fonts (not monospace)
- Pages look like real websites from 2010
- Can read most content sites

---

## Level 4 — JavaScript Engine 🟡 SUBSTANTIALLY DONE
**Lines: ~50,000-100,000 | Time: 2-4 months**

> **Reality check:** the native JS engine landed in ~5,800 LOC, far
> below the original estimate, because we went **bytecode VM**
> (lexer → AST → compiler → opcodes → VM) rather than tree-walking.
> Standard library coverage is partial; AsyncAwait, generators, ES6
> modules are stubbed/missing.

### 4a: Lexer
- Tokenize JavaScript source into tokens
- Keywords, identifiers, numbers, strings, operators, punctuation
- Regular expression literals
- Template literals
- Automatic semicolon insertion

### 4b: Parser → AST
- Recursive descent parser
- Expression parsing (precedence climbing)
- Statement parsing (if, for, while, switch, try/catch)
- Function declarations and expressions
- Arrow functions
- Object/array literals
- Destructuring
- Classes (ES6)
- Modules (import/export)

### 4c: Interpreter
- Tree-walking interpreter (simplest approach)
- Variable scoping (lexical, closures)
- Prototype-based object system
- Built-in types: Number, String, Boolean, Array, Object, Function
- Operators: arithmetic, comparison, logical, bitwise
- Control flow: if/else, loops, switch, exceptions
- this binding
- Garbage collection (mark-and-sweep)

### 4d: Standard Library
- Math object
- String methods (split, replace, indexOf, etc.)
- Array methods (map, filter, reduce, forEach, etc.)
- JSON.parse / JSON.stringify
- Date object
- RegExp (basic)
- console.log
- setTimeout / setInterval

### 4e: DOM API
- document.getElementById()
- document.querySelector() / querySelectorAll()
- element.innerHTML / textContent
- element.style property
- element.classList
- element.setAttribute() / getAttribute()
- document.createElement()
- element.appendChild() / removeChild()
- Event system: addEventListener, event propagation
- Event types: click, input, submit, keydown, load
- window.location
- window.alert / confirm / prompt
- XMLHttpRequest / fetch API (basic)

### 4f: Integration
- <script> tag execution (inline and src)
- DOM manipulation triggers re-layout and re-paint
- Event loop (single-threaded, async via callbacks)
- Script loading order

What it looks like:
- Interactive forms work
- Dropdown menus, accordions, tabs
- Basic SPAs that manipulate the DOM
- Login forms, search boxes functional

---

## Level 5 — Modern Web Platform ⏳ PLANNED
**Lines: ~1,000,000+ | Time: 1-2 years**

> **Strategic note:** Level 5 is where the native engine cost gets
> existential. The Ladybird port (path 2) gives us a Level-5-ish
> renderer for "free" by reusing LibWeb. The strategic question is
> whether the native engine continues past Level 4 or stops at "good
> enough for static + simple JS sites" while Ladybird carries the
> modern-web load.

### 5a: CSS Advanced Layout
- Flexbox (flex-direction, justify-content, align-items, flex-grow/shrink)
- CSS Grid (grid-template-columns/rows, grid-area, gap)
- Positioned elements (relative, absolute, fixed, sticky)
- Float and clear
- Overflow (scroll, hidden, auto)
- Media queries (@media screen and (max-width: ...))
- CSS transitions and animations (@keyframes)
- Transform (translate, rotate, scale)
- Opacity and visibility
- Box shadows, border-radius
- Gradients (linear, radial)
- Pseudo-elements (::before, ::after)
- Pseudo-classes (:hover, :focus, :nth-child)

### 5b: Advanced Text
- International text (Unicode, BiDi, RTL)
- Font loading (@font-face, WOFF/WOFF2)
- Text decoration (underline, strikethrough)
- Text shadow
- Word spacing, letter spacing
- Hyphenation
- Vertical text

### 5c: Canvas API
- 2D drawing context
- Path operations (moveTo, lineTo, arc, bezierCurveTo)
- Fill and stroke
- Gradients and patterns
- Image drawing (drawImage)
- Text drawing
- Pixel manipulation (getImageData, putImageData)
- Compositing operations

### 5d: WebGL (GPU Rendering)
- OpenGL ES 2.0 subset
- Shader compilation (vertex + fragment)
- Buffer objects (VBO, IBO)
- Texture mapping
- 3D transformations
- Framebuffer operations

### 5e: Media
- <video> and <audio> elements
- Media codecs (H.264, VP9, Opus, AAC)
- MediaSource Extensions
- Web Audio API

### 5f: Networking Advanced
- HTTP/2 (multiplexing, header compression, server push)
- WebSocket (RFC 6455)
- Server-Sent Events
- CORS
- Cookie management
- Cache API
- Service Workers

### 5g: Storage
- localStorage / sessionStorage
- IndexedDB
- Cache API
- Cookies

### 5h: Web APIs
- Geolocation API
- Notifications API
- Clipboard API
- Drag and Drop
- File API
- FormData
- URL API
- History API (pushState, popstate)
- Intersection Observer
- Resize Observer
- Mutation Observer
- Web Workers
- Performance API

### 5i: Security
- Content Security Policy (CSP)
- Subresource Integrity (SRI)
- Sandboxed iframes
- Same-origin policy
- CORS validation
- XSS protection
- Mixed content blocking

---

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  BatBrowser UI                   │
│  Tab Bar │ Nav Bar │ Bookmarks │ DevTools        │
├─────────────────────────────────────────────────┤
│                Page Compositor                   │
│  Layers │ Scrolling │ Hit Testing │ Painting    │
├─────────────────────────────────────────────────┤
│              Layout Engine                       │
│  Box Model │ Block │ Inline │ Flex │ Grid        │
├──────────────┬──────────────────────────────────┤
│  DOM Tree    │  CSS Engine                       │
│  Nodes       │  Selectors │ Cascade │ Computed  │
│  Attributes  │  Inheritance │ Specificity       │
├──────────────┴──────────────────────────────────┤
│              HTML Parser                         │
│  Tokenizer │ Tree Builder │ Error Recovery       │
├─────────────────────────────────────────────────┤
│              JavaScript Engine                   │
│  Lexer │ Parser │ AST │ Interpreter │ GC        │
│  DOM API │ Events │ Standard Library             │
├─────────────────────────────────────────────────┤
│              Resource Loader                     │
│  HTTP │ HTTPS/TLS │ Cache │ Cookies │ CORS      │
├─────────────────────────────────────────────────┤
│              Media Decoders                      │
│  PNG │ JPEG │ GIF │ SVG │ Video │ Audio         │
├─────────────────────────────────────────────────┤
│              Font Engine                         │
│  TrueType │ OpenType │ Shaping │ Rasterizer     │
├─────────────────────────────────────────────────┤
│              Bat_OS Platform                     │
│  VirtIO GPU │ TCP/IP │ VFS │ Frame Allocator    │
└─────────────────────────────────────────────────┘
```

## Development Order

1. Level 1 (styled text) → immediate improvement
2. Level 2a (HTML parser) → foundation for everything
3. Level 2c (block layout) → pages have structure
4. Level 2b (CSS parser) → pages have style
5. Level 2d (paint) → visual output
6. Level 3c (fonts) → proportional text
7. Level 3a (PNG) → images work
8. Level 4a-c (JS interpreter) → interactivity
9. Level 4d-e (DOM API) → real web apps
10. Level 5a (CSS layout) → modern sites render

## File Structure

```
src/browser/
  mod.rs              — module root
  html/
    tokenizer.rs      — HTML tokenizer
    parser.rs         — HTML tree builder
    dom.rs            — DOM node types
    entities.rs       — HTML entity table
  css/
    tokenizer.rs      — CSS tokenizer
    parser.rs         — CSS rule parser
    selectors.rs      — selector matching
    cascade.rs        — specificity + cascade
    properties.rs     — CSS property definitions
    values.rs         — CSS value types (color, length, etc.)
  layout/
    box_model.rs      — box dimensions
    block.rs          — block layout
    inline.rs         — inline/text layout
    flex.rs           — flexbox
    grid.rs           — CSS grid
    position.rs       — absolute/relative/fixed
    text.rs           — line breaking, wrapping
  paint/
    display_list.rs   — paint commands
    renderer.rs       — framebuffer drawing
    compositor.rs     — layer compositing
  js/
    lexer.rs          — JavaScript tokenizer
    parser.rs         — JS parser → AST
    ast.rs            — AST node types
    interpreter.rs    — tree-walking interpreter
    runtime.rs        — built-in objects
    gc.rs             — garbage collector
    dom_api.rs        — DOM bindings
    events.rs         — event system
  media/
    png.rs            — PNG decoder
    jpeg.rs           — JPEG decoder
    font.rs           — TrueType renderer
  net/
    loader.rs         — resource fetching
    cache.rs          — HTTP cache
    cookies.rs        — cookie jar
```

## Decision Log

| # | Decision | Why |
|---|----------|-----|
| 1 | ~~Tree-walking JS interpreter~~ → **Bytecode VM** (lexer→AST→compiler→opcodes→VM) | AST walker was the original plan; bytecode landed simpler in practice and runs faster. Still no JIT. |
| 2 | Block+inline layout first, flex/grid later | 90% of pages work with just block+inline |
| 3 | PNG before JPEG | Simpler codec, more common for web graphics |
| 4 | Monospace font fallback always available | Guaranteed readable even if TTF parsing fails |
| 5 | No WebAssembly | Too complex, not needed for security browser |
| 6 | No WebRTC | Requires SRTP, STUN, ICE — massive networking |
| 7 | Single-threaded | Matches bare-metal OS model, simpler |
